//! 中文模板包：`file.new --template` 的中文场景内置模板。
//!
//! 全部是**占位文本**（「张三」「××大学」「在此填写」）——模板是版式不是内容，
//! 具体事实由用户填，不猜、不编（与 gongwen 技能同一铁律）。
//!
//! 字体只写名字（渲染层做安装检测与替换）：正文宋体系、标题黑体系——中文 Windows
//! 全自带；Linux 无中文字体时由 fonts crate 的替换表兜底。
//!
//! 模板清单：
//!
//! | id | 模板 | 要点 |
//! |---|---|---|
//! | `resume-cn` | 中文简历 | 姓名居中、求职意向、教育/工作/技能分节 |
//! | `letter-cn` | 中文信纸 | 称呼顶格、正文缩进 2 字、此致敬礼、落款右对齐 |
//! | `timetable` | 课程表 | 6 节次 × 周一到周五表格 + 午休行 |
//! | `minutes` | 会议纪要 | 标题 + 会议信息表 + 议题分节 |
//! | `memo` | 便笺 | 标题 + 日期 + 事项列表 |

use std::sync::Arc;

use wordcraft_doc::props::{Align, CharProps, LineSpacing};
use wordcraft_doc::table::Table;
use wordcraft_doc::{Block, Document, Paragraph, Rgb};

/// 中文正文：首行缩进 2 字、1.5 倍行距（字号继承 Normal）。
fn cn(text: &str) -> Paragraph {
    let mut p = Paragraph::with_text(text, CharProps::default());
    p.props.indent_first_chars = Some(2.0);
    p.props.line_spacing = Some(LineSpacing::Multiple(1.5));
    p
}

/// 黑体小节标题（顶格）。
fn cn_head(text: &str) -> Paragraph {
    let mut p = Paragraph::with_text(text, CharProps { font_ea: Some("黑体".into()), size: Some(16.0), ..Default::default() });
    p.props.line_spacing = Some(LineSpacing::Multiple(1.5));
    p
}

/// 居中大标题（黑体系）。
fn cn_title(text: &str) -> Paragraph {
    let mut p = Paragraph::with_text(text, CharProps { font_ea: Some("黑体".into()), size: Some(26.0), ..Default::default() });
    p.props.align = Some(Align::Center);
    p.props.space_after = Some(12.0);
    p
}

/// 右对齐段（落款）。
fn cn_right(text: &str) -> Paragraph {
    let mut p = Paragraph::with_text(text, CharProps::default());
    p.props.align = Some(Align::Right);
    p
}

/// 顶格段（称呼/敬语行）。
fn cn_flush(text: &str) -> Paragraph {
    let mut p = Paragraph::with_text(text, CharProps::default());
    p.props.line_spacing = Some(LineSpacing::Multiple(1.5));
    p
}

/// 空段。
fn cn_blank() -> Paragraph {
    Paragraph::new()
}

/// 表格：文本矩阵 → 带表头行的网格表（首行黑体居中）。
fn grid(rows: &[&[&str]], width: f32) -> Block {
    let mut t = Table::new(rows.len(), rows[0].len(), width);
    for (r, row) in rows.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                let props = if r == 0 { CharProps { font_ea: Some("黑体".into()), ..Default::default() } } else { CharProps::default() };
                let mut p = Paragraph::with_text(txt, props);
                p.props.align = Some(Align::Center);
                cell.blocks = vec![Arc::new(Block::Para(p))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    Block::Table(t)
}

/// **中文简历**：姓名居中 + 求职意向 + 分节（教育/工作/技能/自我评价）。
pub fn resume_cn() -> Document {
    let mut d = Document::new();
    let b: Vec<Block> = vec![
        Block::Para(cn_title("张三")),
        Block::Para({
            let mut p = Paragraph::with_text("求职意向：××岗位", CharProps::default());
            p.props.align = Some(Align::Center);
            p
        }),
        Block::Para({
            let mut p = Paragraph::with_text("电话：138××××0000　·　邮箱：zhangsan@example.com", CharProps::default());
            p.props.align = Some(Align::Center);
            p
        }),
        Block::Para(cn_blank()),
        Block::Para(cn_head("教育背景")),
        Block::Para(cn("××大学　××专业　本科　20××年9月—20××年6月")),
        Block::Para(cn("主修课程：××、××、××（在此填写）")),
        Block::Para(cn_head("工作经历")),
        Block::Para(cn("××公司　××岗位　20××年×月—至今")),
        Block::Para(cn("负责××（在此填写工作内容与业绩，建议量化：提升××%）")),
        Block::Para(cn("××公司　××岗位　20××年×月—20××年×月")),
        Block::Para(cn("参与××（在此填写）")),
        Block::Para(cn_head("专业技能")),
        Block::Para(cn("语言：普通话（母语）、英语（××级）")),
        Block::Para(cn("软件/工具：××、××（在此填写熟练程度）")),
        Block::Para(cn_head("自我评价")),
        Block::Para(cn("（两三句，写在××方面的特点；空话删掉。）")),
    ];
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// **中文信纸**：传统书信版式。
pub fn letter_cn() -> Document {
    let mut d = Document::new();
    let b: Vec<Block> = vec![
        Block::Para(cn_flush("李四老师：")),
        Block::Para(cn("您好！")),
        Block::Para(cn("（正文第一段：先说为什么写信。首行已缩进两字。）")),
        Block::Para(cn("（正文第二段：具体的事，一段一个意思。）")),
        Block::Para(cn("此致")),
        Block::Para(cn_flush("敬礼！")),
        Block::Para(cn_blank()),
        Block::Para(cn_right("张三")),
        Block::Para(cn_right("20××年×月×日")),
    ];
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// **课程表**：上午 4 节 + 午休 + 下午 2 节，周一到周五。
pub fn timetable() -> Document {
    let mut d = Document::new();
    let rows: Vec<&[&str]> = vec![
        &["节次", "周一", "周二", "周三", "周四", "周五"],
        &["第1节 8:00", "语文", "数学", "英语", "物理", "化学"],
        &["第2节 9:00", "数学", "语文", "物理", "化学", "英语"],
        &["第3节 10:00", "英语", "化学", "数学", "语文", "体育"],
        &["第4节 11:00", "物理", "体育", "语文", "数学", "数学"],
        &["午休 12:00", "", "", "", "", ""],
        &["第5节 14:00", "历史", "地理", "政治", "生物", "自习"],
        &["第6节 15:00", "地理", "历史", "生物", "自习", "班会"],
    ];
    let b: Vec<Block> = vec![
        Block::Para(cn_title("××班课程表")),
        Block::Para({
            let mut p = Paragraph::with_text("20××—20××学年 第×学期", CharProps::default());
            p.props.align = Some(Align::Center);
            p
        }),
        Block::Para(cn_blank()),
        grid(&rows, 468.0),
        Block::Para(cn_blank()),
        Block::Para(cn_flush("（课程名替换为本班的；午休行可删。）")),
    ];
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// **会议纪要**：标题 + 信息表 + 议题分节。
pub fn minutes() -> Document {
    let mut d = Document::new();
    let rows: Vec<&[&str]> = vec![
        &["会议时间", "20××年×月×日 ××:××—××:××"],
        &["会议地点", "××会议室"],
        &["主持人", "×××"],
        &["记录人", "×××"],
        &["出席人员", "×××、×××、×××（共×人）"],
        &["缺席人员", "无（或：×××，原因）"],
    ];
    let b: Vec<Block> = vec![
        Block::Para(cn_title("××会议纪要")),
        Block::Para(cn_blank()),
        grid(&rows, 468.0),
        Block::Para(cn_blank()),
        Block::Para(cn_head("一、会议议题")),
        Block::Para(cn("（列出会议讨论的事项。）")),
        Block::Para(cn_head("二、讨论情况")),
        Block::Para(cn("（按议题分条记录各方观点与分歧。）")),
        Block::Para(cn_head("三、会议决定")),
        Block::Para(cn("（形成的决定，逐条编号；明确责任人与时限。）")),
        Block::Para(cn_head("四、后续安排")),
        Block::Para(cn("（下次会议时间或待办跟踪方式。）")),
    ];
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

/// **便笺**：标题 + 日期 + 事项。
pub fn memo() -> Document {
    let mut d = Document::new();
    let mut b: Vec<Block> = vec![
        Block::Para(cn_title("便笺")),
        Block::Para(cn_right("20××年×月×日")),
        Block::Para(cn_blank()),
        Block::Para(cn_head("事项")),
        Block::Para(cn("一、（在此填写）")),
        Block::Para(cn("二、（在此填写）")),
        Block::Para(cn("三、（在此填写）")),
    ];
    // 底部一条分隔线（段落下边框），随手记的感觉。
    let mut rule = Paragraph::new();
    use wordcraft_doc::props::{Border, BorderStyle};
    rule.props.borders.get_or_insert_with(Default::default).bottom =
        Some(Border { style: BorderStyle::Single, width: 1.0, color: Some(Rgb(0x99, 0x99, 0x99)), space: 4.0 });
    b.push(Block::Para(rule));
    d.body = b.into_iter().map(Arc::new).collect();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 每个模板都能布局出页（文档模型合法），且是中文占位文本。
    #[test]
    fn cn_templates_layout() {
        for (name, d) in
            [("resume-cn", resume_cn()), ("letter-cn", letter_cn()), ("timetable", timetable()), ("minutes", minutes()), ("memo", memo())]
        {
            let l = wordcraft_layout::layout(&d, &mut wordcraft_layout::LayoutCache::new(), &Default::default());
            assert!(!l.pages.is_empty(), "{name} 布局出页");
        }
    }

    /// 课程表是 6 列 8 行的表格，带表头行。
    #[test]
    fn timetable_is_a_grid() {
        let d = timetable();
        let tables: Vec<&Table> = d
            .body
            .iter()
            .filter_map(|b| match &**b {
                Block::Table(t) => Some(t),
                _ => None,
            })
            .collect();
        assert_eq!(tables.len(), 1, "一个表格");
        let t = tables[0];
        assert_eq!(t.rows.len(), 8, "8 行（表头+6 节+午休）");
        assert_eq!(t.rows[0].cells.len(), 6, "6 列（节次+周一到周五）");
        assert!(t.rows[0].props.header, "表头行");
    }

    /// 信纸版式：称呼顶格、正文缩进、此致敬礼、落款右对齐。
    #[test]
    fn letter_cn_layout_semantics() {
        let d = letter_cn();
        let paras: Vec<&Paragraph> = d
            .body
            .iter()
            .filter_map(|b| match &**b {
                Block::Para(p) => Some(p),
                _ => None,
            })
            .collect();
        // 称呼顶格（无首行缩进）。
        assert_eq!(paras[0].plain_text(), "李四老师：");
        assert!(paras[0].props.indent_first_chars.is_none(), "称呼顶格");
        // 正文缩进 2 字。
        assert_eq!(paras[2].props.indent_first_chars, Some(2.0), "正文缩进");
        // 敬语顶格。
        let jingli = paras.iter().find(|p| p.plain_text() == "敬礼！").unwrap();
        assert!(jingli.props.indent_first_chars.is_none(), "敬礼顶格");
        // 落款右对齐。
        let sign = paras.iter().find(|p| p.plain_text() == "张三").unwrap();
        assert_eq!(sign.props.align, Some(Align::Right), "落款右对齐");
    }

    /// 纪要有信息表，简历有四个分节标题。
    #[test]
    fn minutes_has_info_table_resume_has_sections() {
        let m = minutes();
        assert!(m.body.iter().any(|b| matches!(&**b, Block::Table(_))), "纪要有信息表");
        let r = resume_cn();
        let heads: Vec<String> = r
            .body
            .iter()
            .filter_map(|b| match &**b {
                Block::Para(p) if p.runs.iter().any(|r| r.props.font_ea.as_deref() == Some("黑体")) => Some(p.plain_text()),
                _ => None,
            })
            .collect();
        for want in ["教育背景", "工作经历", "专业技能", "自我评价"] {
            assert!(heads.iter().any(|h| h.contains(want)), "简历分节 {want}");
        }
    }
}
