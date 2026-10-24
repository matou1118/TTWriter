//! Read a PowerPoint presentation (`.pptx`, ECMA-376 Part 1 §13 PresentationML) as a document:
//! each slide becomes a page-break-started block — the slide's text shapes as paragraphs (a
//! title placeholder as a heading), its pictures inline, its speaker notes after the slide —
//! so `wordcraft-cli convert deck.pptx out.docx` (or `.pdf`) turns a deck into a readable,
//! printable handout.
//!
//! Written from the public specification, in the family style of [`sheet`]: a lenient streamed
//! XML walk, size caps everywhere, no panic on hostile input. The presentation's slide size
//! becomes the page size. The layout layer (theme colours, shape fills, charts as pictures,
//! SmartArt, media) is out of scope: what a handout needs is the text, the pictures and the
//! notes, in reading order.
//!
//! Editing decks back into `.pptx` is not this crate's job — the family's presentation app
//! (DeckCraft) owns that. This is the read path only.

use wordcraft_doc::SectionProps;

use crate::model::{FBlock, Flow, Img, Inline, Kind, Meta, Para};
use crate::sheet::{Ev, Package, Rel, attr, dir_of, relationships, rels_of, walk};

/// A slide's own notes part, and its pictures, collected under caps.
const MAX_SLIDES: usize = 500;
/// Per-slide text cap, characters (a slide's text is titles and bullets, not a book).
const MAX_SLIDE_CHARS: usize = 20_000;
/// Total picture bytes kept across the deck (media in decks is often huge).
const MAX_MEDIA_BYTES: u64 = 64 << 20;
/// XML depth cap (hostile input).
const MAX_XML_DEPTH: usize = 64;

/// The extension names read here.
pub const EXTENSIONS: &[&str] = &["pptx", "ppsx", "potx", "pptm"];

/// Whether `bytes` holds a PresentationML package: a presentation part the package root
/// relationships point at.
pub fn is_presentation(bytes: &[u8]) -> bool {
    let Ok(mut pkg) = Package::open(bytes) else { return false };
    let Ok(rels) = pkg.read("_rels/.rels") else { return false };
    relationships(&rels, "").iter().any(|r| (r.kind.ends_with("/presentation") || r.target.ends_with("presentation.xml")) && pkg.has(&r.target))
}

/// Read `bytes` as a presentation into a document flow.
pub fn import(bytes: &[u8]) -> Result<Flow, String> {
    let mut pkg = Package::open(bytes)?;
    let pres_path = presentation_path(&mut pkg)?;
    let (slide_w, slide_h) = slide_size(&mut pkg, &pres_path);
    let slides = slide_paths(&mut pkg, &pres_path)?;

    let mut flow = Flow { meta: core_properties(&mut pkg), ..Default::default() };
    for (i, sp) in slides.iter().take(MAX_SLIDES).enumerate() {
        if i > 0 {
            // Each slide starts on its own page: a page-break paragraph before it.
            let mut pb = Para::new(Kind::Normal);
            pb.page_break = true;
            flow.blocks.push(FBlock::Para(pb));
        }
        slide_into(&mut pkg, &mut flow, i, sp)?;
    }
    if flow.blocks.is_empty() {
        return Err("the presentation has no slides with content".into());
    }
    let page = SectionProps { page_w: slide_w, page_h: slide_h, ..Default::default() };
    Ok(flow_with_page(flow, page))
}

// ---------------------------------------------------------------------------------------------
// Package parts

/// `ppt/presentation.xml` (by the package root relationships, else the default place).
fn presentation_path(pkg: &mut Package<'_>) -> Result<String, String> {
    if let Ok(rels) = pkg.read("_rels/.rels") {
        for r in relationships(&rels, "") {
            if (r.kind.ends_with("/presentation") || r.target.ends_with("presentation.xml")) && pkg.has(&r.target) {
                return Ok(r.target);
            }
        }
    }
    if pkg.has("ppt/presentation.xml") {
        return Ok("ppt/presentation.xml".to_string());
    }
    Err("not a PowerPoint presentation package (no presentation part)".into())
}

/// The slide size, in points: `p:sldSz`'s EMU (914400 per inch) over 12700 per point.
fn slide_size(pkg: &mut Package<'_>, path: &str) -> (f32, f32) {
    let xml = pkg.read(path).unwrap_or_default();
    let (mut cx, mut cy) = (0f32, 0f32);
    walk(&xml, |ev| {
        if let Ev::Start(e) = ev
            && e.local_name().as_ref() == b"sldSz"
        {
            cx = attr(e, b"cx").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            cy = attr(e, b"cy").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
        }
    });
    let pt = |emu: f32| (emu / 12_700.0 * 100.0).round() / 100.0;
    // The default when the part says nothing: 16:9 at 13.33×7.5 in.
    if cx > 0.0 && cy > 0.0 { (pt(cx).max(72.0), pt(cy).max(72.0)) } else { (960.0, 540.0) }
}

/// The `r:id` attribute of an element (relationships use the `r:` prefix; the sheet walker's
/// `attr` matches local names only, which `sldId`'s own `id` attribute would shadow).
fn r_id(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    e.attributes().with_checks(false).flatten().take(64).find(|a| a.key.as_ref() == b"r:id").map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

/// The slide parts in presentation order: `p:sldIdLst` entries resolved through the
/// presentation part's relationships.
fn slide_paths(pkg: &mut Package<'_>, pres: &str) -> Result<Vec<String>, String> {
    let xml = pkg.read(pres)?;
    let rels = pkg.read(&rels_of(pres)).map(|r| relationships(&r, dir_of(pres))).unwrap_or_default();
    let mut ids: Vec<String> = Vec::new();
    walk(&xml, |ev| {
        if let Ev::Start(e) = ev
            && e.local_name().as_ref() == b"sldId"
            && ids.len() < MAX_SLIDES
        {
            ids.push(r_id(e).unwrap_or_default());
        }
    });
    let slides = ids
        .into_iter()
        .filter_map(|id| {
            let r: &Rel = rels.iter().find(|r| r.id == id && r.kind.ends_with("/slide"))?;
            Some(r.target.clone())
        })
        .collect::<Vec<_>>();
    if slides.is_empty() {
        return Err("the presentation lists no slides".into());
    }
    Ok(slides)
}

/// `docProps/core.xml`, the document properties a handout keeps.
fn core_properties(pkg: &mut Package<'_>) -> Meta {
    let mut meta = Meta::default();
    let xml = pkg.read("docProps/core.xml").unwrap_or_default();
    let mut stack: Vec<Vec<u8>> = Vec::new();
    walk(&xml, |ev| match ev {
        Ev::Start(e) => stack.push(e.local_name().as_ref().to_vec()),
        Ev::End(_) => {
            stack.pop();
        }
        Ev::Text(t) => {
            let name = stack.last().map(|s| s.as_slice()).unwrap_or_default();
            match name {
                b"title" if meta.title.is_empty() => meta.title = t,
                b"creator" if meta.author.is_empty() => meta.author = t,
                b"subject" if meta.subject.is_empty() => meta.subject = t,
                b"keywords" if meta.keywords.is_empty() => meta.keywords = t,
                b"description" if meta.description.is_empty() => meta.description = t,
                b"created" if meta.created.is_empty() => meta.created = t,
                _ => {}
            }
        }
    });
    meta
}

// ---------------------------------------------------------------------------------------------
// Slides

/// A shape being read: its placeholder type (if any) and the paragraphs found in it.
struct ShapeText {
    /// `p:ph/@type`: `title`, `ctrTitle`, `subTitle`, `body`, or empty (a plain text box).
    ph: String,
    paras: Vec<Para>,
}

/// One slide's text shapes (in document order) and the media keys of its pictures.
fn slide_shapes(pkg: &mut Package<'_>, path: &str) -> Result<(Vec<ShapeText>, Vec<String>), String> {
    let xml = pkg.read(path)?;
    let rels = pkg.read(&rels_of(path)).map(|r| relationships(&r, dir_of(path))).unwrap_or_default();

    let mut shapes: Vec<ShapeText> = Vec::new();
    let mut media: Vec<String> = Vec::new();
    let mut media_bytes: u64 = 0;
    let mut chars: usize = 0;

    // Walk the slide's shape tree, tracking element depth per shape.
    let mut depth = 0usize;
    let mut in_shape = false;
    let mut cur: Option<ShapeText> = None;
    let mut para: Option<Para> = None;
    let mut run_fmt = crate::model::Fmt::default();
    let mut in_text = false;

    walk(&xml, |ev| {
        match ev {
            Ev::Start(e) => {
                depth = depth.saturating_add(1);
                if depth > MAX_XML_DEPTH {
                    return;
                }
                match e.local_name().as_ref() {
                    b"sp" => {
                        // A text shape starts.
                        in_shape = true;
                        cur = Some(ShapeText { ph: String::new(), paras: Vec::new() });
                    }
                    b"ph" if in_shape => {
                        if let Some(c) = cur.as_mut() {
                            c.ph = attr(e, b"type").unwrap_or_default();
                        }
                    }
                    b"p" if in_shape => {
                        para = Some(Para::new(Kind::Normal));
                    }
                    b"pPr" if para.is_some() => {
                        // Paragraph properties: indentation (list level) is the reading-order cue
                        // worth keeping; a levelled paragraph becomes a list item.
                        if let Some(p) = para.as_mut()
                            && let Some(l) = attr(e, b"lvl").and_then(|v| v.parse::<u8>().ok())
                        {
                            p.list = Some(crate::model::ListInfo { ordered: false, level: l.min(8), start: 1 });
                        }
                    }
                    b"rPr" if in_text || para.is_some() => {
                        run_fmt = crate::model::Fmt {
                            bold: attr(e, b"b").is_some_and(|v| v == "1"),
                            italic: attr(e, b"i").is_some_and(|v| v == "1"),
                            ..Default::default()
                        };
                    }
                    b"t" => {
                        in_text = true;
                    }
                    b"blip" => {
                        // A picture: r:embed names the media part through the slide's rels.
                        if let Some(id) = attr(e, b"embed").or_else(|| attr(e, b"link"))
                            && let Some(r) = rels.iter().find(|r| r.id == id && r.kind.ends_with("/image"))
                            && !media.contains(&r.target)
                            && media_bytes < MAX_MEDIA_BYTES
                            && let Ok(bytes) = pkg.read(&r.target)
                        {
                            media_bytes = media_bytes.saturating_add(bytes.len() as u64);
                            media.push(r.target.clone());
                        }
                    }
                    _ => {}
                }
            }
            Ev::End(name) => {
                match name.as_slice() {
                    b"sp" if in_shape => {
                        if let Some(c) = cur.take()
                            && chars < MAX_SLIDE_CHARS
                            && !c.paras.is_empty()
                        {
                            chars = chars.saturating_add(c.paras.iter().map(|p| p.text().len()).sum());
                            shapes.push(c);
                        }
                        in_shape = false;
                    }
                    b"p" if in_shape => {
                        if let Some(p) = para.take()
                            && let Some(c) = cur.as_mut()
                        {
                            c.paras.push(p);
                        }
                    }
                    b"t" => {
                        in_text = false;
                    }
                    _ => {}
                }
                depth = depth.saturating_sub(1);
            }
            Ev::Text(t) => {
                if in_text
                    && chars < MAX_SLIDE_CHARS
                    && let Some(p) = para.as_mut()
                {
                    chars = chars.saturating_add(t.len());
                    p.push_text(&t, &run_fmt);
                }
            }
        }
    });

    Ok((shapes, media))
}

/// A slide's notes part path (through the slide's relationships).
fn notes_path(pkg: &mut Package<'_>, slide: &str) -> Option<String> {
    let rels = pkg.read(&rels_of(slide)).map(|r| relationships(&r, dir_of(slide))).unwrap_or_default();
    let r = rels.iter().find(|r| r.kind.ends_with("/notesSlide"))?;
    pkg.has(&r.target).then(|| r.target.clone())
}

/// Read one slide into `flow`.
fn slide_into(pkg: &mut Package<'_>, flow: &mut Flow, index: usize, slide: &str) -> Result<(), String> {
    let (shapes, media) = slide_shapes(pkg, slide)?;

    // The slide number as a small header, then the title (if any) as a heading.
    let mut title = None;
    let mut body: Vec<&ShapeText> = Vec::new();
    for sh in &shapes {
        if matches!(sh.ph.as_str(), "title" | "ctrTitle") {
            title = Some(sh);
        } else {
            body.push(sh);
        }
    }

    let mut head = Para::new(Kind::Heading(6));
    head.push_text(&format!("Slide {}", index + 1), &crate::model::Fmt::default());
    flow.blocks.push(FBlock::Para(head));

    if let Some(t) = title {
        let mut h = Para::new(if t.ph == "ctrTitle" { Kind::Title } else { Kind::Heading(1) });
        for p in &t.paras {
            if p.is_empty() {
                continue;
            }
            h.kind = if t.ph == "ctrTitle" { Kind::Title } else { Kind::Heading(1) };
            let mut q = Para::new(h.kind);
            q.inlines = p.inlines.clone();
            flow.blocks.push(FBlock::Para(q));
        }
    }

    for sh in body {
        for p in &sh.paras {
            if p.is_empty() {
                continue;
            }
            let mut q = Para::new(Kind::Normal);
            q.inlines = p.inlines.clone();
            q.list = p.list;
            flow.blocks.push(FBlock::Para(q));
        }
    }

    // The slide's pictures, after its text, each on its own paragraph.
    for m in &media {
        if let Some((bytes, ext)) = read_media(pkg, m) {
            let img = Img { data: bytes.into(), ext, w: 0.0, h: 0.0, alt: String::new() };
            let mut p = Para::new(Kind::Normal);
            p.inlines.push(Inline::Image(img));
            flow.blocks.push(FBlock::Para(p));
        }
    }

    // Speaker notes, in a quote style after the slide.
    if let Some(n) = notes_path(pkg, slide) {
        let (notes_shapes, _) = slide_shapes(pkg, &n)?;
        for sh in &notes_shapes {
            for p in &sh.paras {
                if p.is_empty() {
                    continue;
                }
                let mut q = Para::new(Kind::Quote);
                q.inlines = p.inlines.clone();
                flow.blocks.push(FBlock::Para(q));
            }
        }
    }
    Ok(())
}

/// Read a media part's bytes and guess its extension from the part name.
fn read_media(pkg: &mut Package<'_>, target: &str) -> Option<(Vec<u8>, String)> {
    let bytes = pkg.read(target).ok()?;
    let ext = target.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_else(|| "png".into());
    let ext = match ext.as_str() {
        "jpg" | "jpeg" => "jpeg".to_string(),
        "tif" | "tiff" => "tiff".to_string(),
        e if ["png", "gif", "bmp", "webp", "svg"].contains(&e) => e.to_string(),
        _ => "png".to_string(),
    };
    Some((bytes, ext))
}

/// Attach a page size to a flow (importers that give no page size use the caller's).
fn flow_with_page(mut flow: Flow, page: SectionProps) -> Flow {
    if let Some(last) = flow.blocks.last_mut() {
        if let FBlock::Para(p) = last {
            p.section = Some(Box::new(page));
        }
    } else {
        let mut p = Para::new(Kind::Normal);
        p.section = Some(Box::new(page));
        flow.blocks.push(FBlock::Para(p));
    }
    flow
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small PNG made with the image crate.
    fn tiny_png() -> Vec<u8> {
        let img = image::RgbaImage::from_fn(8, 4, |x, _| image::Rgba([(x * 30) as u8, 80, 160, 255]));
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png).unwrap();
        buf
    }

    /// A tiny presentation: two slides — a title slide and one with bullets, a picture and notes.
    fn pptx() -> Vec<u8> {
        let pres = br#"<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldSz cx="12192000" cy="6858000"/><p:sldIdLst><p:sldId id="256" r:id="rId1"/><p:sldId id="257" r:id="rId2"/></p:sldIdLst></p:presentation>"#;
        let pres_rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide2.xml"/></Relationships>"#;
        let root_rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/></Relationships>"#;
        let slide1 = br#"<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="ctrTitle"/></p:nvPr></p:nvSpPr><p:txBody><a:bodyPr/><a:p><a:r><a:rPr lang="en-US"/><a:t>Quarterly Review</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#;
        let slide2 = br#"<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="3" name="Content"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:txBody><a:bodyPr/><a:p><a:pPr lvl="0"/><a:r><a:rPr lang="en-US"/><a:t>Revenue up</a:t></a:r></a:p><a:p><a:pPr lvl="1"/><a:r><a:t>in every region</a:t></a:r></a:p></p:txBody></p:sp><p:pic><p:nvPicPr><p:cNvPr id="4" name="Chart"/><p:cNvPicPr/></p:nvPicPr><p:blipFill><a:blip r:embed="rId3"/></p:blipFill><p:spPr/></p:pic></p:spTree></p:cSld></p:sld>"#;
        let slide2_rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#;
        let notes1 = br#"<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="5" name="Notes"/><p:cNvSpPr/><p:nvPr><p:ph type="body" idx="1"/></p:nvPr></p:nvSpPr><p:txBody><a:bodyPr/><a:p><a:r><a:t>Say the numbers slowly.</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:notes>"#;
        let png = tiny_png();

        let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let add = |z: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>, name: &str, data: &[u8]| {
            z.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            std::io::Write::write_all(z, data).unwrap();
        };
        add(&mut z, "_rels/.rels", root_rels);
        add(&mut z, "ppt/presentation.xml", pres);
        add(&mut z, "ppt/_rels/presentation.xml.rels", pres_rels);
        add(&mut z, "ppt/slides/slide1.xml", slide1);
        add(&mut z, "ppt/slides/slide2.xml", slide2);
        add(&mut z, "ppt/slides/_rels/slide2.xml.rels", slide2_rels);
        add(&mut z, "ppt/media/image1.png", &png);
        add(&mut z, "ppt/notesSlides/notesSlide1.xml", notes1);
        z.finish().unwrap().into_inner()
    }

    #[test]
    fn reads_titles_bullets_pictures_and_notes() {
        let flow = import(&pptx()).unwrap();
        let text = flow
            .blocks
            .iter()
            .filter_map(|b| match b {
                FBlock::Para(p) => Some(p.text()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Quarterly Review"), "the title");
        assert!(text.contains("Revenue up"), "a bullet");
        assert!(text.contains("in every region"), "a levelled bullet");
        assert!(text.contains("Say the numbers slowly."), "the notes");
        assert!(text.contains("Slide 1") && text.contains("Slide 2"), "slide markers");

        // A picture came through as an image inline.
        assert!(flow.blocks.iter().any(|b| matches!(b, FBlock::Para(p) if p.inlines.iter().any(|i| matches!(i, Inline::Image(_))))), "the picture");

        // The second slide starts on a new page.
        let brk = flow
            .blocks
            .iter()
            .filter_map(|b| match b {
                FBlock::Para(p) => Some(p.page_break),
                _ => None,
            })
            .filter(|&b| b)
            .count();
        assert_eq!(brk, 1, "one page break between two slides");
    }

    #[test]
    fn detects_a_presentation_package() {
        assert!(is_presentation(&pptx()));
        assert!(!is_presentation(b"not a zip at all"));
    }

    #[test]
    fn a_bad_package_is_an_error_not_a_panic() {
        for junk in [b"".as_slice(), b"PK\x03\x04".as_slice(), b"PK\x03\x04junkjunk".as_slice()] {
            assert!(import(junk).is_err());
            assert!(!is_presentation(junk));
        }
    }
}
