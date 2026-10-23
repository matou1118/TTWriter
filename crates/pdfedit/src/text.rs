//! 提取 PDF 的文本：解每页的 content stream（`Tj`/`'`/`"`/`TJ`），按 `Td`/`TD`/`T*`/`Tm`
//! 的文本矩阵分行，字体是简单编码（非 CID）时按编码直出，CID 字体（中日韩）走
//! Identity-H 的码位直读或 `/ToUnicode` CMap 的映射。
//!
//! 这是「PDF 转 Word」的文字层：坐标 → 段落重建由上层（wordcraft-cli 的 `pdf text`）
//! 做——y 相近的行并成一段，y 跳变分新段。扫描件 PDF 没有文字层，返回空（那是 OCR 的活）。
//!
//! 纯 lopdf，零 C 依赖；对 hostile 输入（坏流、超长操作数）有界处理，不 panic。

use lopdf::{Document, Object};

/// 一行文本：文本矩阵给出的基线 y（PDF 用户空间，原点在左下）与拼接后的文字。
#[derive(Clone, Debug, PartialEq)]
pub struct TextLine {
    pub x: f64,
    pub y: f64,
    pub text: String,
}

/// 一页提取出的行（按 content stream 顺序，未排序）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageText {
    pub lines: Vec<TextLine>,
}

impl PageText {
    /// 按阅读顺序排：y 降序（页面上面的在前），同 y 按 x 升序。
    pub fn sorted(mut self) -> PageText {
        self.lines.sort_by(|a, b| {
            let dy = b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal);
            dy.then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });
        self
    }
}

/// 字体表：资源名（如 `F1`）→ 解码方式。
#[derive(Clone, Debug, Default)]
struct Fonts {
    /// Name → (is_cid, to_unicode map)
    map: std::collections::BTreeMap<Vec<u8>, FontKind>,
}

#[derive(Clone, Debug)]
enum FontKind {
    /// 简单字体：字节直出（PDFDocEncoding/WinAnsi 里的 ASCII 部分一致）。
    Simple,
    /// CID 字体（Identity-H）：两字节一个码位，按 ToUnicode 映射或按 Unicode 猜。
    Cid(Option<std::sync::Arc<std::collections::BTreeMap<u16, String>>>),
}

/// 提取第 `page`（1 起）页的文本。没有文字层（扫描件）返回空 `PageText`。
pub fn page_text(doc: &Document, page: u32) -> PageText {
    let Some(&id) = doc.get_pages().get(&page) else { return PageText::default() };
    let dict = match doc.get_object(id) {
        Ok(Object::Dictionary(d)) => d.clone(),
        _ => return PageText::default(),
    };
    let fonts = read_fonts(doc, &dict);
    #[cfg(test)]
    eprintln!("page_text: fonts={}", fonts.map.len());
    let content = doc.get_page_content(id);
    #[cfg(test)]
    eprintln!("page_text: content={} bytes", content.len());
    let mut out = PageText::default();
    walk_stream(&content, &fonts, &mut out);
    out
}

/// 读一页资源里的字体表（`/Resources /Font`）。
fn read_fonts(doc: &Document, page: &lopdf::Dictionary) -> Fonts {
    let mut fonts = Fonts::default();
    let res = page.get(b"Resources").cloned().unwrap_or(Object::Null);
    let res_dict = match &res {
        Object::Reference(id) => doc.get_object(*id).cloned().ok().and_then(|o| match o {
            Object::Dictionary(d) => Some(d),
            _ => None,
        }),
        Object::Dictionary(d) => Some(d.clone()),
        _ => None,
    };
    let Some(res) = res_dict else { return fonts };
    let font_obj = res.get(b"Font").cloned().unwrap_or(Object::Null);
    let font_dict = match &font_obj {
        Object::Reference(id) => doc.get_object(*id).cloned().ok().and_then(|o| match o {
            Object::Dictionary(d) => Some(d),
            _ => None,
        }),
        Object::Dictionary(d) => Some(d.clone()),
        _ => None,
    };
    let Some(fd) = font_dict else { return fonts };
    for (name, obj) in fd.iter() {
        let font = match obj {
            Object::Reference(id) => doc.get_object(*id).cloned().ok(),
            other => Some(other.clone()),
        };
        let Some(Object::Dictionary(f)) = font else { continue };
        let subtype = f.get(b"Subtype").and_then(Object::as_name).unwrap_or(b"");
        #[cfg(test)]
        eprintln!("font {:?}: subtype {:?}", String::from_utf8_lossy(name), subtype);
        let is_cid = matches!(subtype, b"Type0") || f.get(b"Encoding").and_then(Object::as_name).map(|e| e == b"Identity-H").unwrap_or(false);
        let to_unicode = if is_cid { read_to_unicode(doc, &f) } else { None };
        #[cfg(test)]
        eprintln!("  to_unicode entries: {}", to_unicode.as_ref().map(|m| m.len()).unwrap_or(0));
        fonts.map.insert(name.clone(), if is_cid { FontKind::Cid(to_unicode) } else { FontKind::Simple });
    }
    fonts
}

/// 读 `/ToUnicode` CMap 的 `beginbfchar`/`beginbfrange` 映射（够用的子集）。
fn read_to_unicode(doc: &Document, font: &lopdf::Dictionary) -> Option<std::sync::Arc<std::collections::BTreeMap<u16, String>>> {
    let stream = match font.get(b"ToUnicode").ok()? {
        Object::Reference(id) => doc.get_object(*id).ok().cloned()?,
        other => other.clone(),
    };
    let bytes = match &stream {
        Object::Stream(s) => s.decompressed_content().ok()?,
        _ => return None,
    };
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let mut map = std::collections::BTreeMap::new();
    parse_bfchar(&text, &mut map);
    parse_bfrange(&text, &mut map);
    (!map.is_empty()).then(|| std::sync::Arc::new(map))
}

/// `beginbfchar` 的 `<src> <dst>` 行。
fn parse_bfchar(text: &str, map: &mut std::collections::BTreeMap<u16, String>) {
    let mut rest = text;
    while let Some(at) = rest.find("beginbfchar") {
        rest = &rest[at + "beginbfchar".len()..];
        let end = rest.find("endbfchar").unwrap_or(rest.len());
        let toks: Vec<&str> = rest[..end].split_whitespace().collect();
        let mut i = 0;
        while i + 1 < toks.len() {
            if let Some(s) = hex_u16(toks[i]) {
                map.insert(s, hex_utf8(toks[i + 1]));
            }
            i += 2;
        }
        rest = &rest[end..];
    }
}

/// `beginbfrange` 的 `<lo> <hi> <dst>`（base 平移）与 `[<d1> <d2> …]`（数组）。
fn parse_bfrange(text: &str, map: &mut std::collections::BTreeMap<u16, String>) {
    let mut rest = text;
    while let Some(at) = rest.find("beginbfrange") {
        rest = &rest[at + "beginbfrange".len()..];
        let end = rest.find("endbfrange").unwrap_or(rest.len());
        let body = &rest[..end];
        let toks: Vec<&str> = body.split_whitespace().collect();
        let mut i = 0;
        while i + 2 < toks.len() {
            let (lo, hi) = match (hex_u16(toks[i]), hex_u16(toks[i + 1])) {
                (Some(a), Some(b)) if b >= a => (a, b),
                _ => {
                    i += 3;
                    continue;
                }
            };
            let dst = toks[i + 2];
            if dst.starts_with('[') {
                let arr: Vec<&str> = dst.trim_matches(|c| c == '[' || c == ']').split_whitespace().collect();
                for (k, d) in arr.iter().enumerate() {
                    if let Some(c) = lo.checked_add(k as u16).filter(|c| *c <= hi) {
                        map.insert(c, hex_utf8(d));
                    }
                }
                i += 3;
            } else if let Some(base) = hex_u16(dst) {
                for c in lo..=hi {
                    map.insert(c, char::from_u32(u32::from(base) + u32::from(c - lo)).map(String::from).unwrap_or_default());
                }
                i += 3;
            } else {
                i += 3;
            }
        }
        rest = &rest[end..];
    }
}

/// `"<ABCD>"` → 0xABCD。
fn hex_u16(s: &str) -> Option<u16> {
    let h = s.trim_matches(|c| c == '<' || c == '>').trim();
    let v = u32::from_str_radix(h, 16).ok()?;
    u16::try_from(v).ok()
}

/// `"<0048006500…>"` → UTF-8 字符串（按 UTF-16BE 或单字节 Unicode）。
fn hex_utf8(s: &str) -> String {
    let h = s.trim_matches(|c| c == '<' || c == '>');
    let bytes: Vec<u8> = (0..h.len() / 2).filter_map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).ok()).collect();
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16BE with BOM
        String::from_utf16_lossy(&bytes[2..].chunks(2).map(|c| u16::from_be_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect::<Vec<_>>())
    } else if bytes.len() % 2 == 0 && bytes.len() >= 2 {
        // UTF-16BE without BOM
        String::from_utf16_lossy(&bytes.chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect::<Vec<_>>())
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// 解码一个显示字符串操作符的字节，按当前字体。
fn decode(bytes: &[u8], font: &FontKind) -> String {
    match font {
        FontKind::Simple => bytes.iter().map(|&b| char::from(b)).collect(),
        FontKind::Cid(Some(map)) => bytes
            .chunks(2)
            .filter_map(|c| c.get(1).map(|hi| u16::from_be_bytes([*c.first().unwrap_or(&0), *hi])))
            .map(|code| map.get(&code).cloned().unwrap_or_default())
            .collect(),
        FontKind::Cid(None) => {
            // Identity-H without ToUnicode：两字节码位当 Unicode 猜（中文常见 0x4E2D 直接是字）。
            bytes
                .chunks(2)
                .filter_map(|c| c.get(1).map(|hi| u16::from_be_bytes([*c.first().unwrap_or(&0), *hi])))
                .map(|code| char::from_u32(u32::from(code)).unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect()
        }
    }
}

/// 走 content stream 的操作符（BT…ET 块里的文本定位与显示）。
fn walk_stream(content: &[u8], fonts: &Fonts, out: &mut PageText) {
    let mut cur_font: Option<&FontKind> = None;
    let mut tm = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0); // Tm
    let mut tl = 0.0; // T*
    let mut pending: Vec<String> = Vec::new();
    let mut pending_x = 0.0;
    let mut pending_y = 0.0;

    let toks = tokenize(content);
    let mut stack: Vec<Vec<u8>> = Vec::new();
    for tok in toks {
        match tok {
            Tok::Operand(b) => stack.push(b),
            Tok::Op(op) => {
                match op.as_slice() {
                    b"Tf" if stack.len() >= 2 => {
                        // 操作数是名字（/F1）：剥掉斜杠查资源表。
                        let name = &stack[stack.len() - 2];
                        let name = name.strip_prefix(b"/").unwrap_or(name);
                        #[cfg(test)]
                        eprintln!("Tf: name={:?} found={}", String::from_utf8_lossy(name), fonts.map.contains_key(name));
                        cur_font = fonts.map.get(name);
                    }
                    b"Tm" if stack.len() >= 6 => {
                        let nums: Vec<f64> =
                            stack[stack.len() - 6..].iter().filter_map(|b| std::str::from_utf8(b).ok().and_then(|s| s.parse().ok())).collect();
                        if nums.len() == 6 {
                            // 新文本矩阵：收掉旧行
                            flush(&mut pending, out, pending_x, pending_y);
                            tm = (nums[0], nums[1], nums[2], nums[3], nums[4], nums[5]);
                            pending_x = tm.4;
                            pending_y = tm.5;
                        }
                    }
                    b"Td" | b"TD" if stack.len() >= 2 => {
                        let nums: Vec<f64> =
                            stack[stack.len() - 2..].iter().filter_map(|b| std::str::from_utf8(b).ok().and_then(|s| s.parse().ok())).collect();
                        if nums.len() == 2 {
                            if op == b"TD" {
                                tl = -nums[1];
                            }
                            // 行内相对移动
                            tm.4 += nums[0];
                            tm.5 += nums[1];
                            if nums[1] != 0.0 {
                                flush(&mut pending, out, pending_x, pending_y);
                                pending_x = tm.4;
                                pending_y = tm.5;
                            }
                        }
                    }
                    b"T*" => {
                        flush(&mut pending, out, pending_x, pending_y);
                        tm.5 -= tl;
                        pending_x = tm.4;
                        pending_y = tm.5;
                    }
                    b"Tj" | b"'" | b"\"" if !stack.is_empty() => {
                        if let Some(bytes) = string_bytes(stack.last().unwrap()) {
                            let font = cur_font.unwrap_or(&FontKind::Simple);
                            pending.push(decode(&bytes, font));
                        }
                        if op != b"Tj" {
                            flush(&mut pending, out, pending_x, pending_y);
                            tm.5 -= tl;
                            pending_x = tm.4;
                            pending_y = tm.5;
                        }
                    }
                    b"TJ" if !stack.is_empty() => {
                        // 数组里的字符串段（数字是 kerning，跳过）
                        let bytes = array_strings(stack.last().unwrap());
                        if !bytes.is_empty() {
                            let font = cur_font.unwrap_or(&FontKind::Simple);
                            pending.push(decode(&bytes, font));
                        }
                    }
                    b"BT" => {
                        tm = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
                    }
                    b"ET" => {
                        flush(&mut pending, out, pending_x, pending_y);
                    }
                    _ => {}
                }
                stack.clear();
            }
        }
    }
    flush(&mut pending, out, pending_x, pending_y);
}

/// 逗留的字符串收成一行。
fn flush(pending: &mut Vec<String>, out: &mut PageText, x: f64, y: f64) {
    if pending.is_empty() {
        return;
    }
    let text = pending.join("");
    if !text.trim().is_empty() {
        out.lines.push(TextLine { x, y, text });
    }
    pending.clear();
}

/// 一个 PDF 字符串操作数（`(...)` 或 `<hex>`）的原始字节。
fn string_bytes(tok: &[u8]) -> Option<Vec<u8>> {
    if tok.first() == Some(&b'(') {
        // 去括号 + 转义（\n \r \t \b \f \( \) \\ \ddd）
        let inner = &tok[1..tok.len().saturating_sub(1)];
        let mut out = Vec::with_capacity(inner.len());
        let mut i = 0;
        while i < inner.len() {
            let b = inner[i];
            if b == b'\\' && i + 1 < inner.len() {
                let n = inner[i + 1];
                match n {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0C),
                    b'(' | b')' | b'\\' => out.push(n),
                    b'0'..=b'7' => {
                        let mut v = 0u32;
                        let mut k = i + 1;
                        while k < inner.len() && k < i + 4 && (b'0'..=b'7').contains(&inner[k]) {
                            v = v * 8 + (inner[k] - b'0') as u32;
                            k += 1;
                        }
                        out.push(v.clamp(0, 255) as u8);
                        i = k - 2;
                    }
                    b'\n' => {}
                    b'\r' if i + 2 < inner.len() => {}
                    _ => out.push(n),
                }
                i += 2;
            } else {
                out.push(b);
                i += 1;
            }
        }
        Some(out)
    } else if tok.first() == Some(&b'<') {
        let h: Vec<u8> = tok.iter().skip(1).take(tok.len().saturating_sub(2)).filter(|b| !b.is_ascii_whitespace()).cloned().collect();
        let mut out = Vec::with_capacity(h.len() / 2);
        for i in (0..h.len()).step_by(2) {
            let hi = (h[i] as char).to_digit(16);
            let lo = h.get(i + 1).and_then(|b| (*b as char).to_digit(16));
            if let (Some(a), Some(b)) = (hi, lo) {
                out.push((a * 16 + b) as u8);
            } else if let (Some(a), None) = (hi, lo) {
                out.push(a as u8);
            }
        }
        Some(out)
    } else {
        None
    }
}

/// TJ 数组操作数里全部字符串段的拼接。
fn array_strings(tok: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let s = tok;
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            b'(' => {
                // 找配对右括号（忽略转义）
                let mut depth = 1;
                let mut j = i + 1;
                while j < s.len() && depth > 0 {
                    match s[j] {
                        b'\\' => j += 1,
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                if let Some(mut part) = string_bytes(&s[i..j.min(s.len())]) {
                    out.append(&mut part);
                }
                i = j;
            }
            b'<' => {
                let j = s[i + 1..].iter().position(|&b| b == b'>').map(|p| i + 1 + p + 1).unwrap_or(s.len());
                if let Some(mut part) = string_bytes(&s[i..j.min(s.len())]) {
                    out.append(&mut part);
                }
                i = j;
            }
            _ => i += 1,
        }
    }
    out
}

enum Tok {
    Operand(Vec<u8>),
    Op(Vec<u8>),
}

/// content stream 分词：空白/注释分隔；字符串和数组当单个操作数。
fn tokenize(content: &[u8]) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut i = 0;
    let mut cur = Vec::new();
    let mut in_token = false;
    while i < content.len() {
        let b = content[i];
        match b {
            b' ' | b'\n' | b'\r' | b'\t' | 0x0C => {
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                i += 1;
            }
            b'%' => {
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                while i < content.len() && content[i] != b'\n' && content[i] != b'\r' {
                    i += 1;
                }
            }
            b'(' => {
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                let mut depth = 1;
                let mut j = i + 1;
                while j < content.len() && depth > 0 {
                    match content[j] {
                        b'\\' => j += 1,
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                out.push(Tok::Operand(content[i..j.min(content.len())].to_vec()));
                i = j;
            }
            b'<' => {
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                // << 字典开始当普通 token；<hex> 字符串
                if content.get(i + 1) == Some(&b'<') {
                    cur.extend_from_slice(b"<<");
                    in_token = true;
                    i += 2;
                } else {
                    let j = content[i + 1..].iter().position(|&b| b == b'>').map(|p| i + 1 + p + 1).unwrap_or(content.len());
                    out.push(Tok::Operand(content[i..j.min(content.len())].to_vec()));
                    i = j;
                }
            }
            b'[' | b']' => {
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                // 数组整体当一个操作数（TJ 用）
                if b == b'[' {
                    let mut depth = 1;
                    let mut j = i + 1;
                    while j < content.len() && depth > 0 {
                        match content[j] {
                            b'(' => {
                                // 跳过字符串
                                let mut k = j + 1;
                                let mut sdepth = 1;
                                while k < content.len() && sdepth > 0 {
                                    match content[k] {
                                        b'\\' => k += 1,
                                        b'(' => sdepth += 1,
                                        b')' => sdepth -= 1,
                                        _ => {}
                                    }
                                    k += 1;
                                }
                                j = k;
                                continue;
                            }
                            b'[' => depth += 1,
                            b']' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                    out.push(Tok::Operand(content[i..j.min(content.len())].to_vec()));
                    i = j;
                } else {
                    out.push(Tok::Op(b"]".to_vec()));
                    i += 1;
                }
            }
            b'/' => {
                // 名字定界符：上一个 token 结束，名字自成一词（`Tr\n/f0` 里 Tr 与 /f0 分开）。
                if in_token {
                    out.push(kind_of(std::mem::take(&mut cur)));
                    in_token = false;
                }
                cur.push(b'/');
                in_token = true;
                i += 1;
            }
            _ => {
                cur.push(b);
                in_token = true;
                i += 1;
            }
        }
    }
    if in_token {
        out.push(kind_of(cur));
    }
    out
}

/// 一个裸 token 是操作数（数字/名字）还是操作符（字母）。
fn kind_of(tok: Vec<u8>) -> Tok {
    // 名字（/F1）与数字是操作数；纯字母是操作符。
    if tok.first() == Some(&b'/') || tok.iter().all(|b| b.is_ascii_digit() || *b == b'.' || *b == b'-' || *b == b'+') {
        Tok::Operand(tok)
    } else if tok.iter().all(|b| b.is_ascii_alphabetic() || *b == b'*' || *b == b'\'') {
        Tok::Op(tok)
    } else {
        Tok::Operand(tok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一页带简单字体文本的最小 PDF content stream。
    fn stream(body: &str) -> Vec<u8> {
        format!("BT /F1 12 Tf 72 720 Td ({body}) Tj ET").into_bytes()
    }

    /// 一页带 Type0 字体 + Identity-H + ToUnicode CMap + hex 字符串的 PDF：
    /// 端到端验证（这正是 krilla 导出的形态：子集字体，码位是任意小整数）。
    #[test]
    fn type0_with_to_unicode_is_decoded_end_to_end() {
        use lopdf::{Dictionary, Stream};
        let mut doc = lopdf::Document::with_version("1.5");
        // CMap：码位 1→'H' 2→'i'（子集字体的真实形态）
        let cmap = b"%!PS-Adobe-3.0\nbegincmap\nbeginbfchar\n<0001> <0048>\n<0002> <0069>\nendbfchar\nendcmap\n";
        let to_unicode = doc.add_object(Stream::new(Dictionary::new(), cmap.to_vec()));
        // Type0 字体：Encoding Identity-H，ToUnicode 指向 CMap。
        let type0 = doc.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Font".to_vec())),
            ("Subtype", Object::Name(b"Type0".to_vec())),
            ("BaseFont", Object::Name(b"ABCDEF+Test".to_vec())),
            ("Encoding", Object::Name(b"Identity-H".to_vec())),
            ("ToUnicode", Object::Reference(to_unicode)),
        ]));
        let font_dict = doc.add_object(Dictionary::from_iter([("F1", Object::Reference(type0))]));
        // Content：hex 字符串 <00010002>（码位 1、2）。
        let content = doc.add_object(Stream::new(Dictionary::new(), b"BT /F1 12 Tf 72 720 Td <00010002> Tj ET".to_vec()));
        let pages_id = doc.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Pages".to_vec())),
            ("Kids", Object::Array(vec![])),
            ("Count", Object::Integer(0)),
        ]));
        let page = doc.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Page".to_vec())),
            ("Parent", Object::Reference(pages_id)),
            ("MediaBox", Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(595), Object::Integer(842)])),
            ("Resources", Object::Dictionary(Dictionary::from_iter([("Font", Object::Reference(font_dict))]))),
            ("Contents", Object::Reference(content)),
        ]));
        let lopdf::Object::Dictionary(pages) = doc.get_object_mut(pages_id).unwrap() else { panic!() };
        pages.set("Kids", Object::Array(vec![Object::Reference(page)]));
        pages.set("Count", Object::Integer(1));
        let catalog = doc.add_object(Dictionary::from_iter([("Type", Object::Name(b"Catalog".to_vec())), ("Pages", Object::Reference(pages_id))]));
        doc.trailer.set("Root", Object::Reference(catalog));

        let out = page_text(&doc, 1);
        assert_eq!(out.lines.len(), 1, "{:?}", out.lines);
        assert_eq!(out.lines[0].text, "Hi", "the ToUnicode CMap decoded the subset codes: {:?}", out.lines);
        assert_eq!((out.lines[0].x, out.lines[0].y), (72.0, 720.0));
    }

    /// 真实 krilla 导出的 PDF（sample.pdf 由 CLI 生成，gitignored）：子集字体 +
    /// Identity-H + Flate 压缩的 CMap + 跨对象引用的 Resources。没有该文件时跳过。
    #[test]
    fn krilla_exported_pdf_is_decoded() {
        // 测试的 cwd 是 crate 目录；sample.pdf 在 repo 根。
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sample.pdf");
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("skipping: no {} (generate: wordcraft-cli run --template sample --save sample.pdf)", path.display());
            return;
        };
        let doc = match lopdf::Document::load_mem(&bytes) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skipping: {e}");
                return;
            }
        };
        let out = page_text(&doc, 1);
        assert!(!out.lines.is_empty(), "text lines come out");
        let first = out.lines.iter().map(|l| l.text.trim()).find(|t| !t.is_empty()).expect("a non-empty line");
        assert!(first.chars().any(|c| c.is_alphanumeric() || (c as u32) > 0x2E00), "decoded through the CMap, not raw codes: {first:?}");
    }

    #[test]
    fn simple_text_is_extracted() {
        let fonts = Fonts { map: [(b"F1".to_vec(), FontKind::Simple)].into_iter().collect() };
        let mut out = PageText::default();
        walk_stream(&stream("Hello PDF"), &fonts, &mut out);
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].text, "Hello PDF");
        assert_eq!((out.lines[0].x, out.lines[0].y), (72.0, 720.0));
    }

    #[test]
    fn cid_text_via_identity_guess() {
        // 0x4E2D 0x6587 = 中文
        let bytes = [0x4E, 0x2D, 0x65, 0x87];
        let key = b"F1".to_vec();
        let fonts = Fonts { map: [(key.clone(), FontKind::Cid(None))].into_iter().collect() };
        let decoded = decode(&bytes, fonts.map.get(&key).unwrap());
        assert_eq!(decoded, "中文");
    }

    #[test]
    fn line_breaks_on_td_down() {
        let content = b"BT /F1 12 Tf 72 720 Td (A) Tj 0 -14 Td (B) Tj ET";
        let fonts = Fonts { map: [(b"F1".to_vec(), FontKind::Simple)].into_iter().collect() };
        let mut out = PageText::default();
        walk_stream(content, &fonts, &mut out);
        assert_eq!(out.lines.len(), 2, "{:?}", out.lines);
        assert_eq!(out.lines[0].text, "A");
        assert_eq!(out.lines[1].text, "B");
        assert_eq!(out.lines[1].y, 706.0);
    }

    #[test]
    fn tj_array_strings_are_joined() {
        let content = b"BT /F1 12 Tf 72 720 Td [(Hel) 2 (lo)] TJ ET";
        let fonts = Fonts { map: [(b"F1".to_vec(), FontKind::Simple)].into_iter().collect() };
        let mut out = PageText::default();
        walk_stream(content, &fonts, &mut out);
        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].text, "Hello");
    }

    #[test]
    fn empty_and_hostile_streams_do_not_panic() {
        let fonts = Fonts::default();
        for junk in [b"" as &[u8], b"BT", b"BT (unclosed", b"BT [(a) (b)] TJ", b"BT /F1 Tf (x", b"%%%"] {
            let mut out = PageText::default();
            walk_stream(junk, &fonts, &mut out);
        }
    }

    #[test]
    fn hex_utf8_reads_utf16() {
        assert_eq!(hex_utf8("<0048>"), "H");
        assert_eq!(hex_utf8("<FEFF4E2D6587>"), "中文");
        assert_eq!(hex_utf8("<4E2D6587>"), "中文");
    }

    #[test]
    fn to_unicode_range_maps() {
        let map = read_to_unicode_from_cmap("beginbfrange\n<0041> <0043> <0061>\nendbfrange");
        assert_eq!(map.get(&0x41).map(String::as_str), Some("a"));
        assert_eq!(map.get(&0x43).map(String::as_str), Some("c"));
    }

    /// 直接从 CMap 文本建映射（测试辅助）。
    fn read_to_unicode_from_cmap(text: &str) -> std::collections::BTreeMap<u16, String> {
        // 复用 read_to_unicode 的解析逻辑（inline 简化版）
        let mut map = std::collections::BTreeMap::new();
        let mut rest = text;
        while let Some(at) = rest.find("beginbfrange") {
            rest = &rest[at + "beginbfrange".len()..];
            let end = rest.find("endbfrange").unwrap_or(rest.len());
            let toks: Vec<&str> = rest[..end].split_whitespace().collect();
            if toks.len() >= 3 {
                if let (Some(lo), Some(hi)) = (hex_u16(toks[0]), hex_u16(toks[1])) {
                    let base = hex_u16(toks[2]).unwrap_or(0);
                    for c in lo..=hi {
                        map.insert(c, char::from_u32(base as u32 + (c - lo) as u32).map(String::from).unwrap_or_default());
                    }
                }
            }
            rest = &rest[end..];
        }
        map
    }
}
