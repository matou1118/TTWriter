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

--pages takes 1-based page numbers, ranges and commas (\"1-3,5\"); left out it means every page.
";

/// Options whose value is not a file name.
const VALUED: &[&str] = &["--degrees", "--pages"];

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
            let mut first = true;
            for img_path in imgs {
                let bytes = std::fs::read(img_path).map_err(|e| format!("{img_path}: {e}"))?;
                let decoder = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format().map_err(|e| e.to_string())?;
                let (w, h) = decoder.into_dimensions().map_err(|_| format!("{img_path}: 不是支持的图片"))?;
                // 像素 → 磅（72dpi：1px = 1pt），A4 内钳制防超页。
                let (pw, ph) = ((w as f32).clamp(72.0, 595.0), (h as f32).clamp(72.0, 842.0));
                let key = doc.add_media(bytes, &img_ext(img_path));
                if !first {
                    // 每图一节一页：分节符段落携带新页面尺寸。
                    doc.body.push(wordcraft_doc::para_block({
                        let mut p = wordcraft_doc::Paragraph::new();
                        p.section = Some(Box::new(wordcraft_doc::SectionProps { page_w: pw, page_h: ph, ..Default::default() }));
                        p
                    }));
                } else {
                    // 第一节由首段携带。
                    let mut p = wordcraft_doc::Paragraph::with_text("", wordcraft_doc::CharProps::default());
                    p.section = Some(Box::new(wordcraft_doc::SectionProps { page_w: pw, page_h: ph, ..Default::default() }));
                    doc.body.push(wordcraft_doc::para_block(p));
                    first = false;
                }
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
                p.insert_object(0, img, &wordcraft_doc::CharProps::default()).map_err(|e| e.to_string())?;
                doc.body.push(wordcraft_doc::para_block(p));
            }
            let pdf_bytes = wordcraft_pdf::export(&doc, &wordcraft_pdf::PdfOptions::default()).map_err(|e| e.to_string())?;
            std::fs::write(output, &pdf_bytes).map_err(|e| format!("{output}: {e}"))?;
            eprintln!("wrote {output} ({} pages, {} images)", imgs.len(), imgs.len());
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
    }
}
