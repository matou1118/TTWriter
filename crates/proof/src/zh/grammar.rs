//! Chinese P0-1 grammar checks: repeated CJK characters and stray spaces around CJK
//! punctuation. Rule-table driven, zero external dependencies.
//!
//! These are the cheapest, most reliable checks — the same class as the English repeated-word
//! and space-before-punctuation rules. Deeper checks (的/地/得, measure-word agreement…) need
//! a word segmenter and a part-of-speech tagger, which are out of scope for P0-1.

use crate::{Issue, IssueKind, ProofLang};

fn issue(start: usize, end: usize, message: String, suggestions: Vec<String>) -> Issue {
    Issue { start, end, kind: IssueKind::Grammar, message, suggestions, lang: ProofLang::Zh }
}

/// CJK punctuation that should never have a space before it.
const PUNCT_NO_SPACE_BEFORE: &str = "，。、；：！？）】》」』、．";

/// CJK punctuation that should never have a space after it (when followed by a CJK character).
const PUNCT_NO_SPACE_AFTER: &str = "（【《「『";

/// Is `c` a CJK Unified Ideograph (the common case: U+4E00–U+9FFF)?
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'  // CJK Extension A
        | '\u{F900}'..='\u{FAFF}'  // CJK Compatibility Ideographs
    )
}

pub fn check(text: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    repeated_chars(text, &mut issues);
    space_before_punct(text, &mut issues);
    space_after_open_punct(text, &mut issues);
    issues.sort_by_key(|i| (i.start, i.end));
    issues
}

/// Consecutive identical CJK characters: "我我", "是是". A few characters are legitimate
/// reduplication (姐姐, 慢慢, 刚刚), so we skip a small allow-list.
fn repeated_chars(text: &str, v: &mut Vec<Issue>) {
    // Reduplicated characters that are correct Chinese (AA → "very A" or a kinship term).
    const ALLOW: &str = "人个天年少长短高低大小好坏快慢远近早晚刚刚常常轻轻缓缓深深紧紧满满圆圆白白红红黑黑长长短短高高矮矮胖胖瘦瘦大大小小多多少少真真假假是是非非对对错错好好慢慢快快姐姐妹妹哥哥弟弟爸爸妈妈奶奶爷爷婆婆宝宝叔叔舅舅姑姑亭亭处处事事面面条条步步句句字字声声阵阵种种人人家家年年天天日日时时分分秒秒";

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for w in chars.windows(2) {
        let [(i0, c0), (i1, c1)] = w else { continue };
        if c0 == c1 && is_cjk(*c0) {
            // Check the allow-list: is this character followed by itself a valid reduplication?
            let key = format!("{c0}{c1}");
            if ALLOW.contains(key.as_str()) {
                continue;
            }
            // Only flag if the next char is different (avoid triple-flagging "我我我").
            let next = chars.get(w.len()).map(|(_, c)| *c);
            if next == Some(*c0) {
                continue;
            }
            let end = i1 + c1.len_utf8();
            v.push(issue(*i0, end, format!("重复字符：\"{c0}\""), vec![c0.to_string()]));
        }
    }
}

/// A space before CJK punctuation: "你好 ，世界" → the space is wrong.
fn space_before_punct(text: &str, v: &mut Vec<Issue>) {
    for (i, _) in text.match_indices(' ') {
        let rest = text.get(i + 1..).unwrap_or("");
        let Some(p) = rest.chars().next() else { continue };
        if PUNCT_NO_SPACE_BEFORE.contains(p) {
            v.push(issue(i, i + 1 + p.len_utf8(), "标点前不应有空格".into(), vec![p.to_string()]));
        }
    }
}

/// A space after an opening CJK bracket: "（ 你好）" → the space is wrong.
fn space_after_open_punct(text: &str, v: &mut Vec<Issue>) {
    for (i, c) in text.char_indices() {
        if !PUNCT_NO_SPACE_AFTER.contains(c) {
            continue;
        }
        let after_pos = i + c.len_utf8();
        let rest = text.get(after_pos..).unwrap_or("");
        let Some(s) = rest.chars().next() else { continue };
        if s == ' ' {
            v.push(issue(i, after_pos + 1, "括号后不应有空格".into(), vec![]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_cjk_characters_are_flagged() {
        let issues = check("我我去了学校");
        assert_eq!(issues.len(), 1);
        assert_eq!(&issues[0].message, "重复字符：\"我\"");
    }

    #[test]
    fn legitimate_reduplication_is_allowed() {
        let issues = check("姐姐慢慢走");
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn space_before_cjk_punct_is_flagged() {
        let issues = check("你好 ，世界");
        assert_eq!(issues.len(), 1);
        assert_eq!(&issues[0].message, "标点前不应有空格");
    }

    #[test]
    fn space_after_opening_bracket_is_flagged() {
        let issues = check("（ 你好）");
        assert_eq!(issues.len(), 1);
        assert_eq!(&issues[0].message, "括号后不应有空格");
    }

    #[test]
    fn clean_chinese_text_has_no_issues() {
        let issues = check("今天天气很好，我们去公园玩。");
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn english_text_is_quiet() {
        let issues = check("Hello world, this is fine.");
        assert!(issues.is_empty(), "{issues:?}");
    }
}
