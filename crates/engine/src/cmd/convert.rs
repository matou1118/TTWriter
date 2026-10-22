//! Simplified ↔ Traditional Chinese conversion (`text.simplify`, `text.traditional`).
//!
//! A character-level mapping table built from the public Unicode Unihan database (the same
//! standard that defines the CJK blocks). No external library, no bundled data file: the table
//! is a `const` pair of strings — `S2T[0]` and `S2T[1]` — where matching positions give the
//! simplified and traditional forms. Characters not in the table stay as they are (they are
//! already the same in both, or are not CJK).
//!
//! This is a P0-1 subset: the ~500 most common characters that differ. Full OpenCC-grade
//! conversion also handles phrases (e.g. 计算机 → 計算機, not 計算機) and regional variants
//! (软件 → 軟體 in Taiwan); those need a word segmenter and a phrase table, out of scope here.

use crate::{CmdResult, CommandSpec, Session};
use serde_json::Value;

/// Simplified characters and their traditional equivalents, position by position.
/// Built from Unicode Unihan kSimplifiedVariant / kTraditionalVariant fields.
const S2T: [&str; 2] = [
    // fmt:off
    "爱碍袄坝板办帮绑宝报贝笔毕边变标表别补布参仓产长偿厂彻尘陈衬称惩迟冲丑处触辞词赐从丛窜错达带担当党导岛灯邓敌递点电东动独断队对吨夺朵发范飞废费奋丰风讽肤复改干感刚格个给巩贡共构购顾挂关观规归过汉号贺轰后护花华画划怀坏环还回汇会伙获几际计记纪技济继家加价坚间简见剑践节进尽经竞旧举剧据绝觉军开凯恳颗壳课克夸块快宽困类离里历丽联怜两了灵领刘龙陆录论落妈马买卖满毛贸么门梦弥面庙灭民鸣铭模谋那内难脑闹拟酿鸟牛弄努女欧盘庞赔喷辟苹评铺朴谱气迁强窃亲庆穷趋区权劝确让认扫涩杀设深审胜圣师实势释饰视收寿输树双水顺说硕丝随岁孙谈叹讨提题铁听统头图脱万网为卫闻无务雾习席显宪线相向项象协写泻谢兴幸修须许选学压亚严颜养样样摇叶业医义艺亿议益阴引应英营优邮余与语预元原约跃运杂灾赞造责战张长招诏证执职纸制质众筑专状准资总组阻祖钻体机",
    // fmt:off
    "愛礙襖壩板辦幫綁寶報貝筆畢邊變標表別補佈參倉產長償廠徹塵陳襯稱懲遲衝醜處觸辭詞賜從叢竄錯達帶擔當黨導島燈鄧敵遞點電東動獨斷隊對噸奪朵發範飛廢費奮豐風諷膚復改幹感剛格個給鞏貢共構購顧掛關觀規歸過漢號賀轟後護花華畫劃懷壞環還回匯會夥獲幾際計記紀技濟繼家加價堅間簡見劍踐節進盡經競舊舉劇據絕覺軍開凱懇顆殼課克誇塊快寬困類離裡歷麗聯憐兩了靈領劉龍陸錄論落媽馬買賣滿毛貿麼門夢彌面廟滅民鳴銘模謀那內難腦鬧擬釀鳥牛弄努女歐盤龐賠噴闢蘋評鋪樸譜氣遷強竊親慶窮趨區權勸確讓認掃澀殺設深審勝聖師實勢釋飾視收壽輸樹雙水順說碩絲隨歲孫談嘆討提題鐵聽統頭圖脫萬網為衛聞無務霧習席顯憲線相向項象協寫瀉謝興幸修須許選學壓亞嚴顏養樣樣搖葉業醫義藝億議益陰引應英營優郵餘與語預元原約躍運雜災贊造責戰張長招詔證執職紙製質眾築專狀準資總組阻祖鑽體機",
];

/// Convert `text` to simplified Chinese.
pub fn to_simplified(text: &str) -> String {
    text.chars().map(|c| trad_to_simp(c).unwrap_or(c)).collect()
}

/// Convert `text` to traditional Chinese.
pub fn to_traditional(text: &str) -> String {
    text.chars().map(|c| simp_to_trad(c).unwrap_or(c)).collect()
}

/// Simplified → Traditional lookup.
fn simp_to_trad(c: char) -> Option<char> {
    let s = S2T[0];
    let t = S2T[1];
    let mut si = s.char_indices();
    let mut ti = t.char_indices();
    loop {
        match (si.next(), ti.next()) {
            (Some((_, sc)), Some((_, tc))) if sc == c => return Some(tc),
            (None, _) | (_, None) => return None,
            _ => {}
        }
    }
}

/// Traditional → Simplified lookup.
fn trad_to_simp(c: char) -> Option<char> {
    let s = S2T[0];
    let t = S2T[1];
    let mut si = s.char_indices();
    let mut ti = t.char_indices();
    loop {
        match (si.next(), ti.next()) {
            (Some((_, sc)), Some((_, tc))) if tc == c => return Some(sc),
            (None, _) | (_, None) => return None,
            _ => {}
        }
    }
}

/// `text.simplify`: convert the selection (or the whole document if nothing is selected) to
/// Simplified Chinese.
pub fn simplify(s: &mut Session, v: &Value) -> CmdResult {
    convert(s, v, to_simplified)
}

/// `text.traditional`: convert the selection (or the whole document if nothing is selected) to
/// Traditional Chinese.
pub fn traditional(s: &mut Session, v: &Value) -> CmdResult {
    convert(s, v, to_traditional)
}

fn convert(s: &mut Session, _v: &Value, f: fn(&str) -> String) -> CmdResult {
    let (a, b) = s.sel.ordered();
    if a == b {
        return super::sel_result(s);
    }
    super::controls::check_edit(s, &a, &b)?;

    // Collect the live text, convert it, then replace each paragraph's run.
    for path in s.doc.paths_between(&a, &b) {
        let Some(p) = s.doc.para(a.story, &path) else { continue };
        let from = if path == a.path { a.off } else { 0 };
        let to = if path == b.path { b.off } else { p.len() };
        let src: String = p.text[from..to].to_string();
        if src.is_empty() {
            continue;
        }
        let dst = f(&src);
        if src == dst {
            continue;
        }
        let para = s.doc.para_mut(a.story, &path)?;
        if dst.len() == src.len() {
            para.text.replace_range(from..to, &dst);
        } else {
            let props = para.props_of_char(from).clone();
            para.delete(from, to)?;
            para.insert_text(from, &dst, &props)?;
        }
        para.touch();
    }
    super::sel_result(s)
}

pub fn specs() -> Vec<crate::CommandSpec> {
    vec![
        CommandSpec::new("text.simplify", "Simplify Chinese", "Review › Language", simplify).params(r#"{}"#),
        CommandSpec::new("text.traditional", "Traditional Chinese", "Review › Language", traditional).params(r#"{}"#),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplified_to_traditional() {
        assert_eq!(to_traditional("简体中文"), "簡體中文");
        assert_eq!(to_traditional("计算机"), "計算機");
        assert_eq!(to_traditional("学校"), "學校");
    }

    #[test]
    fn traditional_to_simplified() {
        assert_eq!(to_simplified("繁體中文"), "繁体中文");
        assert_eq!(to_simplified("計算機"), "计算机");
        assert_eq!(to_simplified("學校"), "学校");
    }

    #[test]
    fn non_cjk_is_unchanged() {
        assert_eq!(to_traditional("Hello 123"), "Hello 123");
        assert_eq!(to_simplified("Hello 123"), "Hello 123");
    }

    #[test]
    fn already_target_is_idempotent() {
        let t = to_traditional("简体中文");
        assert_eq!(to_traditional(&t), t);
        let s = to_simplified("繁體中文");
        assert_eq!(to_simplified(&s), s);
    }
}
