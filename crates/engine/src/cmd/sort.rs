//! Pinyin sort (`sort.pinyin`): sort selected paragraphs by their pinyin reading.
//!
//! This is a convenience command that forces Chinese pinyin order regardless of the document's
//! language setting. The existing `para.sort` command already sorts by pinyin when the document
//! language is `zh`; this command is for mixed-language documents where a Chinese list needs
//! pinyin order without changing the document language.

use serde_json::Value;
use wordcraft_doc::Pos;
use wordcraft_doc::pinyin::pinyin_sort_key;

use super::sel_result;
use crate::{CmdError, CmdResult, CommandSpec, Session, p};

/// `sort.pinyin`: sort the selected paragraphs by pinyin.
fn sort_pinyin(s: &mut Session, v: &Value) -> CmdResult {
    let desc = p::bool(v, "descending").unwrap_or(false);
    let (a, b) = s.sel.ordered();
    if a.path.parent() != b.path.parent() {
        return Err(CmdError::Failed("select paragraphs in one place to sort".into()));
    }
    let (i0, i1) = (a.path.last(), b.path.last());
    let bl = s.doc.container_mut(a.story, &a.path)?;
    let Some(slice) = bl.get_mut(i0..=i1) else { return Err(CmdError::Failed("bad range".into())) };
    if slice.iter().any(|b| !matches!(**b, wordcraft_doc::Block::Para(_))) {
        return Err(CmdError::Failed("sorting tables isn't supported here; use Table › Sort".into()));
    }
    let mut keyed: Vec<_> = slice.iter().map(|b| (pinyin_sort_key(&b.as_para().map(|p| p.plain_text()).unwrap_or_default()), b.clone())).collect();
    keyed.sort_by(|(x, _), (y, _)| {
        let o = x.cmp(y);
        if desc { o.reverse() } else { o }
    });
    for (dst, (_, blk)) in slice.iter_mut().zip(keyed) {
        *dst = blk;
    }
    s.sel = crate::Selection { anchor: Pos { off: 0, ..a }, focus: b };
    sel_result(s)
}

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec::new("sort.pinyin", "Sort by Pinyin", "Home › Paragraph", sort_pinyin).params(r#"{"descending"?: bool}"#)]
}
