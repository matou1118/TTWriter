//! `wordcraft-cli pdf …`：PDF 的页操作（信息、旋转、删除、提取、合并）。
//!
//! 这些命令改的是 PDF 文件本身，不碰 WordCraft 的文档模型——PDF 没有排版语义，
//! 把它变成可编辑文档是「导入」，是另一条通道。

use serde_json::json;
use wordcraft_pdfedit::Pdf;

use crate::Failure;

const USAGE: &str = "\
wordcraft-cli pdf — PDF page operations (no document model involved)

USAGE:
  wordcraft-cli pdf info <file>                        pages and each page's size (JSON)
  wordcraft-cli pdf rotate <in> <out> [--pages 1-3,5] --degrees 90|180|270
                                                       turn the selected pages (all by default)
  wordcraft-cli pdf delete <in> <out> [--pages 2,4]    drop the selected pages (at least one stays)
  wordcraft-cli pdf extract <in> <out> [--pages 1,3]   keep only the selected pages
  wordcraft-cli pdf merge <out> <in1> <in2> [in…]      concatenate the files in order
  wordcraft-cli pdf images <out> <img1> [img2…]       each picture becomes one PDF page
                                                       (the page is the picture's own size,
                                                       clamped to A4; jpg/png/gif/bmp/webp/tiff)
  wordcraft-cli pdf text <in.pdf> [out.docx]          the PDF's text layer rebuilt as a Word
                                                       document (paragraphs by line position;
                                                       a scan with no text layer is refused)
  wordcraft-cli pdf ocr <in.pdf> <out.pdf> --text TEXT  overlay an invisible, searchable text
                                     or --text-file F    layer (a double-deck PDF: the page
                                     or --engine          image plus searchable text; pour OCR
                                       tesseract          output in through --text-file, or
                                     [--lang eng]         let tesseract do the OCR itself:
                                     [--tesseract PATH]   it has to be installed — the error
                                                         says how)

--pages takes 1-based page numbers, ranges and commas (\"1-3,5\"); left out it means every page.
";

/// Options whose value is not a file name.
const VALUED: &[&str] = &["--degrees", "--pages", "--text", "--text-file", "--engine", "--lang", "--tesseract"];

pub fn run(args: &[String]) -> Result<(), Failure> {
    let (cmd, rest) = args.split_first().ok_or_else(|| Failure::from(USAGE))?;
    match cmd.as_str() {
        "info" => {
            let file = positional(rest, VALUED).into_iter().next().ok_or_else(crate::missing_argument)?;
            let pdf = Pdf::open(&file).map_err(|e| e.to_string())?;
            let sizes: Vec<serde_json::Value> = pdf
                .page_numbers()
                .into_iter()
                .map(|n| {
                    let (w, h) = pdf.page_size(n).unwrap_or((0.0, 0.0));
                    json!({"page": n, "widthPt": w, "heightPt": h})
                })
                .collect();
            let info = serde_json::to_string_pretty(&json!({"file": file, "pages": pdf.page_count(), "sizes": sizes})).unwrap_or_default();
            println!("{info}");
            Ok(())
        }
        "rotate" | "delete" | "extract" => {
            let files = positional(rest, VALUED);
            let (Some(input), Some(output)) = (files.first(), files.get(1)) else { return Err(crate::missing_argument()) };
            let mut pdf = Pdf::open(input).map_err(|e| e.to_string())?;
            let pages = parse_pages(option(rest, "--pages"))?;
            match cmd.as_str() {
                "rotate" => {
                    let degrees: u32 = option(rest, "--degrees")
                        .ok_or_else(|| Failure::from("pdf rotate 需要 --degrees 90|180|270"))?
                        .trim()
                        .parse()
                        .map_err(|_| Failure::from("pdf rotate 的 --degrees 只能是 90 / 180 / 270"))?;
                    pdf.rotate(&pages, degrees).map_err(|e| e.to_string())?
                }
                "delete" => pdf.delete(&pages).map_err(|e| e.to_string())?,
                _ => pdf.keep(&pages).map_err(|e| e.to_string())?,
            }
            pdf.save(output).map_err(|e| e.to_string())?;
            eprintln!("wrote {output} ({} pages)", pdf.page_count());
            Ok(())
        }
        "merge" => {
            let files = positional(rest, VALUED);
            if files.len() < 3 {
                return Err(Failure::from("pdf merge 需要 <out> <in1> <in2> 至少三个参数"));
            }
            let output = &files[0];
            let mut into = Pdf::open(&files[1]).map_err(|e| e.to_string())?;
            for input in &files[2..] {
                let pdf = Pdf::open(input).map_err(|e| e.to_string())?;
                into.append(&pdf, &[]).map_err(|e| e.to_string())?;
            }
            into.save(output).map_err(|e| e.to_string())?;
            eprintln!("wrote {output} ({} pages)", into.page_count());
            Ok(())
        }
        // 图片转 PDF：每张图一页，页面尺寸即图片尺寸（像素→磅，72dpi）。
        // 走文档模型（每页一个 InlineObject::Image + 分节），复用 wordcraft-pdf 的导出。
        "images" => {
            let files = positional(rest, VALUED);
            if files.len() < 2 {
                return Err(Failure::from("pdf images 需要 <out> <img1> [img2…]"));
            }
            let (output, imgs) = (&files[0], &files[1..]);
            let mut doc = wordcraft_doc::Document::new();
            // 初始空段属于默认节：留着占一行。
            doc.body.clear();
            // 单节文档：全部图片排在 body 级 sectPr（`last_section`）的节里。
            // 图片段各自带节会得到「段节 + 尾节」两节两页——那是错的。
            let mut first_img = true;
            let mut last: wordcraft_doc::SectionProps = wordcraft_doc::SectionProps {
                margin_top: 0.0,
                margin_bottom: 0.0,
                margin_left: 0.0,
                margin_right: 0.0,
                header: 0.0,
                footer: 0.0,
                ..Default::default()
            };
            for img_path in imgs {
                let bytes = std::fs::read(img_path).map_err(|e| format!("{img_path}: {e}"))?;
                let decoder = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format().map_err(|e| e.to_string())?;
                let (w, h) = decoder.into_dimensions().map_err(|_| format!("{img_path}: 不是支持的图片"))?;
                // 像素 → 磅（72dpi：1px = 1pt），A4 内钳制防超页。
                let (pw, ph) = ((w as f32).clamp(72.0, 595.0), (h as f32).clamp(72.0, 842.0));
                if first_img {
                    // 首图定节：页尺寸即图片尺寸（边距 0）。
                    last.page_w = pw;
                    last.page_h = ph;
                    first_img = false;
                }
                let key = doc.add_media(bytes, &img_ext(img_path));
                let img = wordcraft_doc::InlineObject::Image {
                    media: key,
                    w: pw,
                    h: ph,
                    alt: img_path.clone(),
                    float: Default::default(),
                    crop: [0.0; 4],
                    ole: None,
                };
                let mut p = wordcraft_doc::Paragraph::new();
                if !doc.body.is_empty() {
                    // 后续图片：分页符（同节内换页）。
                    p.props.page_break_before = Some(true);
                }
                // 行距 Exact 0：图片行的高度就是图片本身。
                p.props.line_spacing = Some(wordcraft_doc::props::LineSpacing::Exactly(0.0));
                p.insert_object(0, img, &wordcraft_doc::CharProps::default()).map_err(|e| e.to_string())?;
                doc.body.push(wordcraft_doc::para_block(p));
            }
            doc.last_section = last;
            let pdf_bytes = wordcraft_pdf::export(&doc, &wordcraft_pdf::PdfOptions::default()).map_err(|e| e.to_string())?;
            std::fs::write(output, &pdf_bytes).map_err(|e| format!("{output}: {e}"))?;
            eprintln!("wrote {output} ({} pages, {} images)", imgs.len(), imgs.len());
            Ok(())
        }
        // PDF→Word 的文字层：提取带坐标的行（y 跳变分新段），重建为 docx 段落。
        "text" => {
            let files = positional(rest, VALUED);
            let Some(input) = files.first().cloned() else {
                return Err(Failure::from("pdf text 需要 <in.pdf> [out.docx]（缺省 .docx 同名）"));
            };
            let output = files
                .get(1)
                .cloned()
                .unwrap_or_else(|| input.rsplit_once('.').map(|(b, _)| format!("{b}.docx")).unwrap_or_else(|| "out.docx".into()));
            let pdf = Pdf::open(&input).map_err(|e| e.to_string())?;
            let mut doc = wordcraft_doc::Document::new();
            let mut first = true;
            let mut total_lines = 0usize;
            for page in pdf.page_numbers() {
                let text = pdf.text_of(page);
                // y 相近（行距内）的行并一段；y 跳变分新段。
                let mut para: Vec<&str> = Vec::new();
                let mut last_y: Option<f64> = None;
                let flush = |para: &mut Vec<&str>,
                             doc: &mut wordcraft_doc::Document,
                             first: &mut bool,
                             page: u32,
                             text: &wordcraft_pdfedit::text::PageText| {
                    if para.is_empty() {
                        return;
                    }
                    let joined = para.join("\n");
                    if !*first {
                        // 每页分页符
                        let mut brk = wordcraft_doc::Paragraph::new();
                        brk.props.page_break_before = Some(true);
                        doc.body.push(wordcraft_doc::para_block(brk));
                    }
                    *first = false;
                    doc.body.push(wordcraft_doc::para_block(wordcraft_doc::Paragraph::with_text(&joined, wordcraft_doc::CharProps::default())));
                    let _ = (page, text);
                };
                for line in &text.lines {
                    let breaks = last_y.is_some_and(|ly| (ly - line.y).abs() > 20.0);
                    if breaks {
                        flush(&mut para, &mut doc, &mut first, page, &text);
                        para.clear();
                    }
                    para.push(&line.text);
                    last_y = Some(line.y);
                    total_lines += 1;
                }
                flush(&mut para, &mut doc, &mut first, page, &text);
            }
            if total_lines == 0 {
                return Err(Failure::from(format!("{input}：没有文字层（扫描件需要 OCR），转换中止")));
            }
            doc.ensure_nonempty();
            let bytes = wordcraft_docx::write(&doc).map_err(|e| e.to_string())?;
            std::fs::write(&output, bytes).map_err(|e| format!("{output}: {e}"))?;
            eprintln!("wrote {output} ({} pages, {total_lines} text lines)", pdf.page_count());
            Ok(())
        }
        // 双层 PDF：给（扫描）PDF 叠一层隐形可搜索文本。文本来源：
        //   --text "…"           全部页同一文本
        //   --text-file 文件      每页一段（空行分页），或整文件一层
        //   --engine tesseract   自动跑 OCR（装了 tesseract 才有；--lang eng/chi_sim+eng，
        //                         --tesseract 指定可执行文件路径）
        // 配合外部 OCR 工具产出文本后灌入也行（我们做最后一公里）。
        "ocr" => {
            let files = positional(rest, VALUED);
            if files.len() < 2 {
                return Err(Failure::from("pdf ocr 需要 <in.pdf> <out.pdf> --text … / --text-file … / --engine tesseract"));
            }
            let (input, output) = (files[0].clone(), files[1].clone());
            let single = option(rest, "--text");
            let text_file = option(rest, "--text-file");
            let engine = option(rest, "--engine");
            let (single, text_file) = match (single, text_file, engine.as_deref()) {
                (Some(t), None, None) => (Some(t), None),
                (None, Some(f), None) => (None, Some(f)),
                (None, None, Some("tesseract")) => (None, None),
                (None, None, Some(other)) => return Err(Failure::from(format!("pdf ocr 不认识引擎 {other:?}（现在只有 tesseract）"))),
                (None, None, None) => return Err(Failure::from("pdf ocr 需要 --text \"…\"、--text-file 文件，或 --engine tesseract")),
                _ => return Err(Failure::from("--text / --text-file / --engine 只能给一个")),
            };
            let mut pdf = Pdf::open(&input).map_err(|e| e.to_string())?;
            let layers: Vec<String> = if let Some(t) = single {
                vec![t]
            } else if let Some(f) = text_file {
                // 文本文件：空行（两个连续换行）分页；没有分页符就整文件当第 1 页。
                let content = std::fs::read_to_string(&f).map_err(|e| e.to_string())?;
                let parts: Vec<String> = content.split("\n\n\n").map(|s| s.to_string()).collect();
                if parts.len() > 1 { parts } else { vec![content] }
            } else {
                // 引擎桥：逐页抠图 → tesseract → 文本。
                let lang = crate::ocrbridge::sanitize_lang(option(rest, "--lang").as_deref().unwrap_or("eng"))?;
                let exe = crate::ocrbridge::find_tesseract(option(rest, "--tesseract").as_deref())?;
                let dir = std::env::temp_dir().join(format!("wordcraft-ocr-{}", std::process::id()));
                std::fs::create_dir_all(&dir).map_err(|e| format!("建临时目录失败：{e}"))?;
                let result = (|| -> Result<Vec<String>, Failure> {
                    let mut layers = Vec::new();
                    for page in pdf.page_numbers() {
                        let imgs = pdf.images_of(page);
                        let layer = match crate::ocrbridge::biggest_image(&imgs.images) {
                            Some(img) => {
                                let bytes = crate::ocrbridge::encode_image_bytes(img).map_err(Failure::from)?;
                                let out = crate::ocrbridge::ocr_image(&exe, &lang, img, &bytes, &dir, page).map_err(Failure::from)?;
                                eprintln!("page {page}: {}×{} ({}) → {} chars", out.w, out.h, out.image_name, out.text.chars().count());
                                out.text
                            }
                            None => String::new(),
                        };
                        layers.push(layer);
                    }
                    Ok(layers)
                })();
                let _ = std::fs::remove_dir_all(&dir);
                let mut layers = result?;
                if layers.iter().all(|l| l.trim().is_empty()) {
                    return Err(Failure::from(format!("{input}：一页都没认出字（没有可 OCR 的页面图片？）——纯图片 PDF 每页要有嵌入位图")));
                }
                // 尾部空页的层去掉：overlay_text 按下标对页，全空层没意义。
                while layers.last().is_some_and(|l| l.trim().is_empty()) {
                    layers.pop();
                }
                layers
            };
            pdf.overlay_text(&layers).map_err(|e| e.to_string())?;
            pdf.save(&output).map_err(|e| e.to_string())?;
            eprintln!("wrote {output} ({} pages, {} text layers)", pdf.page_count(), layers.len());
            Ok(())
        }
        _ => Err(Failure::from(USAGE)),
    }
}

/// 路径的图片扩展名（小写；默认 png）。
fn img_ext(path: &str) -> String {
    path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_else(|| "png".into())
}

/// `--name value` 的值。
fn option(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

/// 不是选项、也不是某个选项的值的那些参数。
fn positional(args: &[String], valued: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a.starts_with("--") {
            skip = valued.contains(&a.as_str());
            continue;
        }
        out.push(a.clone());
    }
    out
}

/// `"1-3,5"` → `[1, 2, 3, 5]`；空表示「每一页」。
fn parse_pages(spec: Option<String>) -> Result<Vec<u32>, Failure> {
    let Some(spec) = spec.filter(|s| !s.trim().is_empty()) else { return Ok(Vec::new()) };
    let mut pages = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (from, to) = match part.split_once('-') {
            Some((a, b)) => (page(a)?, page(b)?),
            None => (page(part)?, page(part)?),
        };
        if from > to || to - from > 10_000 {
            return Err(Failure::from(format!("pdf: 页码范围不对：{part}")));
        }
        pages.extend(from..=to);
    }
    Ok(pages)
}

fn page(s: &str) -> Result<u32, Failure> {
    let n: u32 = s.trim().parse().map_err(|_| Failure::from(format!("pdf: 页码不是数字：{s}")))?;
    if n == 0 {
        return Err(Failure::from("pdf: 页码从 1 开始".to_string()));
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_are_numbers_ranges_and_commas() {
        assert_eq!(parse_pages(Some("1-3,5".into())).unwrap(), vec![1, 2, 3, 5]);
        assert_eq!(parse_pages(Some("2".into())).unwrap(), vec![2]);
        assert_eq!(parse_pages(None).unwrap(), Vec::<u32>::new(), "不给就是每一页");
        assert_eq!(parse_pages(Some(" ".into())).unwrap(), Vec::<u32>::new());
        assert!(parse_pages(Some("0".into())).is_err(), "页码从 1 开始");
        assert!(parse_pages(Some("a".into())).is_err(), "页码要数字");
        assert!(parse_pages(Some("5-2".into())).is_err(), "范围要从小到大");
    }

    #[test]
    fn an_option_value_is_not_a_file() {
        let args: Vec<String> = ["in.pdf", "--pages", "1,2", "--degrees", "90"].iter().map(|s| s.to_string()).collect();
        assert_eq!(positional(&args, VALUED), vec!["in.pdf".to_string()]);
        assert_eq!(option(&args, "--pages").as_deref(), Some("1,2"));
        assert_eq!(option(&args, "--degrees").as_deref(), Some("90"));
        assert_eq!(option(&args, "--nope"), None);
        // ocr 桥的选项也认：文件位置不被选项值吃掉。
        let args: Vec<String> = ["in.pdf", "out.pdf", "--engine", "tesseract", "--lang", "chi_sim+eng"].iter().map(|s| s.to_string()).collect();
        assert_eq!(positional(&args, VALUED), vec!["in.pdf".to_string(), "out.pdf".to_string()]);
        assert_eq!(option(&args, "--engine").as_deref(), Some("tesseract"));
        assert_eq!(option(&args, "--lang").as_deref(), Some("chi_sim+eng"));
    }

    /// `pdf images` 一图一页（回归：曾经每张图多出一页空白/尾页——空首段 + 段级节 + 尾节
    /// 三重叠加，1 图能出 3 页）。
    #[test]
    fn pdf_images_makes_one_page_per_picture() {
        let dir = std::env::temp_dir().join("wordcraft-pdfcmd-images");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mk = |name: &str, w: u32, h: u32| {
            let path = dir.join(name);
            let img = image::GrayImage::from_fn(w, h, |_, _| image::Luma([200u8]));
            img.save_with_format(&path, image::ImageFormat::Png).unwrap();
            path
        };
        let a = mk("a.png", 200, 100);
        let b = mk("b.png", 80, 160);
        let args: Vec<String> = vec![
            "images".to_string(),
            dir.join("out.pdf").to_string_lossy().into_owned(),
            a.to_string_lossy().into_owned(),
            b.to_string_lossy().into_owned(),
        ];
        run(&args).expect("pdf images");
        let pdf = wordcraft_pdfedit::Pdf::open(dir.join("out.pdf")).unwrap();
        assert_eq!(pdf.page_count(), 2, "两图两页，一图一页");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
