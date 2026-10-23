//! `wordcraft-cli redhead`：参数化生成 GB/T 9704—2012 红头版头。
//!
//! 红头是**生成器**：机关名、文号、密级、份号、签发人给齐，直接产出一份只含版头的
//! `.docx`——正文由用户接在后面（或配合 gongwen 技能的套版脚本排全文）。缺的要素
//! 留 `[待补充]` 占位，不猜、不编。
//!
//! 版头要素（国标顺序，自上而下）：
//!
//! ```text
//!  份号        000001            ← 3号黑体，左顶格
//!  密级        秘密★1年          ← 3号黑体，左顶格
//!  紧急程度    特急               ← 3号黑体，左顶格
//!  发文机关标志  ××市人民政府      ← 小标宋一系，红色，特大，居中
//!  发文字号     ×政发〔2026〕12号  ← 3号仿宋（上行文时右侧「签发人：××」）
//!  ─────────────────────────     ← 红色分隔线（文号段下边框，2pt，c00000）
//! ```
//!
//! 红线用**段落下边框**画（文档模型原生能力），不插图形——跟着文档走、docx 里是真边框。
//!
//! 字体取法向 gongwen 技能看齐：小标宋没有时华文中宋，再没有宋体（带 stderr 提示，
//! 不静默回退）。字体**只写名字**（渲染层做安装检测与替换），本命令不查字体文件。

use wordcraft_doc::props::{Align, Border, BorderStyle, LineSpacing, TextColor};
use wordcraft_doc::{CharProps, Document, Paragraph, Rgb, SectionProps};

/// GB/T 9704 版心（A4，上 37 / 下 35 / 左 28 / 右 26 mm），红头必须装进这个盒子。
const MM: f32 = 72.0 / 25.4;

/// 版头红（GB/T 9704 未规定色值；机关惯例 c00000，Word 的「深红」）。
const RED: Rgb = Rgb(0xC0, 0x00, 0x00);

/// 红头参数（全部可选；缺省留占位）。
#[derive(Debug, Default, Clone)]
pub struct Redhead {
    /// 发文机关标志（「××市人民政府」）。红头的主体。
    pub org: Option<String>,
    /// 发文字号（「×政发〔2026〕12号」）。六角括号是国标写法，调用方负责。
    pub docno: Option<String>,
    /// 份号（6 位数字字符串）。
    pub fenhao: Option<String>,
    /// 密级和保密期限（「秘密★1年」）。
    pub secret: Option<String>,
    /// 紧急程度（「特急」/「加急」）。
    pub urgency: Option<String>,
    /// 签发人（上行文才有）。文号行右侧「签发人：××」。
    pub issuer: Option<String>,
    /// 标题占位（2 号宋体系居中）；版头不要标题就不给。
    pub title: Option<String>,
}

impl Redhead {
    /// 生成版头文档（不含正文；空段收尾让红线不悬在文档末）。
    pub fn build(&self) -> Document {
        let mut doc = Document::new();
        doc.body.clear();
        doc.last_section = SectionProps {
            page_w: 595.0,
            page_h: 842.0,
            margin_top: 37.0 * MM,
            margin_bottom: 35.0 * MM,
            margin_left: 28.0 * MM,
            margin_right: 26.0 * MM,
            ..Default::default()
        };
        // 份号/密级/紧急程度：左顶格 3 号（16pt）黑体，一行一段。
        for text in [&self.fenhao, &self.secret, &self.urgency].into_iter().flatten() {
            doc.body.push(wordcraft_doc::para_block(para(text, hei(16.0), Align::Left)));
        }
        // 份号区与机关标志之间空出（国标视觉距离约 2 行）。
        doc.body.push(wordcraft_doc::para_block(empty(24.0)));
        // 发文机关标志：宋体系（小标宋替代链在渲染层）、红色、放大，居中。
        let org = self.org.clone().unwrap_or_else(|| "[发文机关]".into());
        let size = org_size(&org);
        doc.body.push(wordcraft_doc::para_block(para(&org, song(size, Some(RED)), Align::Center)));
        // 机关名与文号之间空一行。
        doc.body.push(wordcraft_doc::para_block(empty(16.0)));
        // 发文字号：3 号仿宋。居中；上行文（有签发人）时文号居左、右侧签发人。
        let docno = self.docno.clone().unwrap_or_else(|| "[发文字号]".into());
        let mut p = if self.issuer.is_some() {
            // 上行文一行两处：文号（左）＋签发人（右侧）。精确排版要 tab 位；版头生成先
            // 保内容与字体正确（内容层用空格撑开，正文排版交给后续套版）。
            let mut p = para(&docno, fang(16.0), Align::Left);
            p.runs.push(wordcraft_doc::Run { len: "                签发人：".len(), props: fang(16.0) });
            p.text.push_str("                签发人：");
            if let Some(issuer) = &self.issuer {
                let he = hei(16.0);
                p.runs.push(wordcraft_doc::Run { len: issuer.len(), props: he });
                p.text.push_str(issuer);
            }
            p
        } else {
            para(&docno, fang(16.0), Align::Center)
        };
        // 红色分隔线：文号段的下边框（single 2pt 红）。
        p.props.borders.get_or_insert_with(Default::default).bottom =
            Some(Border { style: BorderStyle::Single, width: 2.0, color: Some(RED), space: 2.0 });
        doc.body.push(wordcraft_doc::para_block(p));
        // 红线下空一行，再是可选标题占位。
        doc.body.push(wordcraft_doc::para_block(empty(16.0)));
        if let Some(title) = &self.title {
            doc.body.push(wordcraft_doc::para_block(para(title, song(22.0, None), Align::Center)));
        }
        // 收尾空段（文档以边框段结尾在部分打开器里显示怪异）。
        doc.body.push(wordcraft_doc::para_block(empty(16.0)));
        doc.ensure_nonempty();
        doc
    }
}

/// 机关标志字号：名长则缩（国标允许回行；30pt 起，每超 8 字缩半磅，下限 22）。
fn org_size(org: &str) -> f32 {
    let chars = org.chars().count().max(1) as f32;
    (30.0 - ((chars - 8.0).max(0.0) * 0.5).min(8.0)).max(22.0)
}

/// 黑体 run 属性。
fn hei(size: f32) -> CharProps {
    CharProps { font: Some("黑体".into()), size: Some(size), ..Default::default() }
}

/// 仿宋 run 属性。
fn fang(size: f32) -> CharProps {
    CharProps { font: Some("仿宋".into()), size: Some(size), ..Default::default() }
}

/// 宋体系 run 属性（机关标志/标题用）；`color` 为红时红字。
fn song(size: f32, color: Option<Rgb>) -> CharProps {
    CharProps { font: Some("宋体".into()), size: Some(size), color: color.map(TextColor::Rgb), ..Default::default() }
}

/// 一段文本：行距固定 28 磅（国标每面 22 行基准）+ 对齐。
fn para(text: &str, props: CharProps, align: Align) -> Paragraph {
    let mut p = Paragraph::with_text(text, props);
    p.props.align = Some(align);
    p.props.line_spacing = Some(LineSpacing::Exactly(28.0));
    p
}

/// 空段（撑距离），行高 `pt`。
fn empty(pt: f32) -> Paragraph {
    let mut p = Paragraph::new();
    p.props.line_spacing = Some(LineSpacing::Exactly(pt));
    p
}

/// CLI 入口：`redhead <out.docx> [--org …] [--docno …] [--fenhao …] [--secret …]
/// [--urgency …] [--issuer …] [--title …] [--pdf <out.pdf>]`。
pub fn run(args: &[String]) -> Result<(), String> {
    let mut out: Option<String> = None;
    let mut rh = Redhead::default();
    let mut pdf: Option<String> = None;
    let valued = ["org", "docno", "fenhao", "secret", "urgency", "issuer", "title", "pdf"];
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let Some(name) = a.strip_prefix("--") else {
            if out.is_none() {
                out = Some(a.clone());
            } else {
                return Err(format!("多余的位置参数 {a}"));
            }
            i += 1;
            continue;
        };
        let val = if valued.contains(&name) {
            i += 1;
            args.get(i).cloned().ok_or_else(|| format!("{a} 需要一个值"))?
        } else {
            return Err(format!("不认识的参数 {a}"));
        };
        match name {
            "org" => rh.org = Some(val),
            "docno" => rh.docno = Some(val),
            "fenhao" => rh.fenhao = Some(val),
            "secret" => rh.secret = Some(val),
            "urgency" => rh.urgency = Some(val),
            "issuer" => rh.issuer = Some(val),
            "title" => rh.title = Some(val),
            "pdf" => pdf = Some(val),
            // valued 表已限定只这八个；到不了这里，但坏输入也不 panic。
            _ => return Err(format!("不认识的参数 {a}")),
        }
        i += 1;
    }
    let Some(out) = out else {
        return Err("用法：wordcraft-cli redhead <out.docx> [--org 机关名] [--docno 文号] [--fenhao 份号] [--secret 密级] [--urgency 紧急] [--issuer 签发人] [--title 标题] [--pdf out.pdf]".into());
    };
    if rh.org.is_none() && rh.docno.is_none() {
        return Err("至少给 --org 或 --docno（否则生成的不是红头）".into());
    }
    let doc = rh.build();
    let bytes = wordcraft_docx::write(&doc).map_err(|e| e.to_string())?;
    std::fs::write(&out, bytes).map_err(|e| format!("{out}: {e}"))?;
    eprintln!("wrote {out} ({} paragraphs)", doc.body.len());
    if let Some(pdf) = pdf {
        let bytes = wordcraft_pdf::export(&doc, &wordcraft_pdf::PdfOptions::default()).map_err(|e| e.to_string())?;
        std::fs::write(&pdf, bytes).map_err(|e| format!("{pdf}: {e}"))?;
        eprintln!("wrote {pdf}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(doc: &Document) -> Vec<String> {
        doc.body
            .iter()
            .filter_map(|b| match &**b {
                wordcraft_doc::Block::Para(p) => Some(p.plain_text()),
                _ => None,
            })
            .collect()
    }

    /// 机关名越长字号越小，有下限。
    #[test]
    fn org_size_scales_down_with_length() {
        assert_eq!(org_size("中共中央办公厅"), 30.0);
        assert!(org_size("某某省某某市某某县人民政府办公室") < 30.0);
        assert!(org_size(&"很".repeat(60)) >= 22.0, "下限 22");
    }

    /// 全参数版头：段落顺序 = 国标要素顺序，红线在文号段上，上行文签发人同行。
    #[test]
    fn full_redhead_layout_order() {
        let rh = Redhead {
            org: Some("××市人民政府".into()),
            docno: Some("×政发〔2026〕12号".into()),
            fenhao: Some("000001".into()),
            secret: Some("秘密★1年".into()),
            urgency: Some("特急".into()),
            issuer: Some("张三".into()),
            title: Some("关于印发某某办法的通知".into()),
        };
        let doc = rh.build();
        let ts = texts(&doc);
        assert_eq!(ts[0], "000001", "份号最前");
        assert_eq!(ts[1], "秘密★1年", "密级第二");
        assert_eq!(ts[2], "特急", "紧急程度第三");
        assert!(ts.iter().any(|t| t.contains("××市人民政府")), "机关标志在");
        // 上行文：文号 + 签发人同行。
        let docno_line = ts.iter().find(|t| t.contains("×政发〔2026〕12号")).unwrap();
        assert!(docno_line.contains("签发人：张三"), "签发人在文号行: {docno_line}");
        // 红线：文号段的下边框是红色 single。
        let has_red_line = doc.body.iter().any(|b| match &**b {
            wordcraft_doc::Block::Para(p) => {
                p.plain_text().contains("×政发")
                    && matches!(&p.props.borders.as_ref().and_then(|bs| bs.bottom), Some(b) if b.color == Some(RED) && b.width >= 1.0)
            }
            _ => false,
        });
        assert!(has_red_line, "文号段带红色下边框（分隔线）");
        // 标题在文号之后。
        let title_at = ts.iter().position(|t| t.contains("关于印发")).unwrap();
        let docno_at = ts.iter().position(|t| t.contains("×政发")).unwrap();
        assert!(title_at > docno_at, "标题在文号后");
        // 版心是 GB 版心。
        assert!((doc.last_section.margin_top - 37.0 * MM).abs() < 0.1, "上边距 37mm");
    }

    /// 最小红头（只有机关名）：文号给占位，非上行文无签发人。
    #[test]
    fn minimal_redhead_has_placeholders() {
        let rh = Redhead { org: Some("测试机关".into()), ..Default::default() };
        let doc = rh.build();
        let ts = texts(&doc);
        assert!(ts.iter().any(|t| t.contains("[发文字号]")), "缺文号给占位不猜");
        assert!(!ts.iter().any(|t| t.contains("签发人")), "非上行文无签发人");
        assert!(doc.body.len() >= 5, "空行撑距 + 机关 + 文号 + 收尾");
    }

    /// 红头文档能写成 docx（zip 容器）再读回。
    #[test]
    fn redhead_docx_roundtrip() {
        let rh = Redhead { org: Some("测试机关".into()), docno: Some("测发〔2026〕1号".into()), ..Default::default() };
        let doc = rh.build();
        let bytes = wordcraft_docx::write(&doc).expect("write");
        let back = wordcraft_docx::read_with_password(&bytes, None).expect("read back");
        let ts = texts(&back);
        assert!(ts.iter().any(|t| t.contains("测试机关")), "读回还有机关名");
        assert!(ts.iter().any(|t| t.contains("测发〔2026〕1号")), "读回还有文号");
    }

    /// CLI 参数解析与拒绝。
    #[test]
    fn cli_args_parse() {
        let dir = std::env::temp_dir().join("wordcraft-redhead-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("head.docx");
        // 正常。
        let args: Vec<String> = vec![out.to_string_lossy().into_owned(), "--org".into(), "××局".into(), "--docno".into(), "×发〔2026〕1号".into()];
        run(&args).expect("generates");
        assert!(out.is_file(), "写出了 docx");
        // 两个位置参数 → 报错。
        let bad: Vec<String> = ["a.docx", "b.docx"].iter().map(|s| s.to_string()).collect();
        assert!(run(&bad).is_err());
        // 既无 org 也无 docno → 拒绝。
        let none: Vec<String> = vec![out.to_string_lossy().into_owned(), "--title".into(), "t".into()];
        assert!(run(&none).is_err());
        // 未知参数 → 报错。
        let unknown: Vec<String> = vec![out.to_string_lossy().into_owned(), "--org".into(), "x".into(), "--wat".into()];
        assert!(run(&unknown).is_err());
        // 缺值 → 报错。
        let novalue: Vec<String> = vec![out.to_string_lossy().into_owned(), "--org".into()];
        assert!(run(&novalue).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
