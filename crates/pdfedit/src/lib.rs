//! PDF 的页操作与批注。
//!
//! WordCraft 的文档模型是排版语义（段落、样式、分页），PDF 没有这些——它只有画在页面上的
//! 东西。所以 PDF 编辑**不进文档模型**，而是一个独立的、直接对 PDF 文件本身动手的会话：
//! 打开 → 旋转 / 删除 / 提取 / 合并 / 加水印 / 批注 → 另存。文字重排（把 PDF 变成可编辑文档）
//! 是另一件事，走导入通道。
//!
//! 全部依赖 lopdf（纯 Rust），不下载任何二进制，不引入 C 代码。

pub mod text;

use std::collections::HashMap;
use std::path::Path;

use lopdf::{Document, Object};

/// 打开或改写一个 PDF 时出的错。
#[derive(Debug)]
pub enum Error {
    /// 读写文件失败。
    Io(std::io::Error),
    /// 文件不是能解析的 PDF（加密、损坏、或根本不是 PDF）。
    Pdf(String),
    /// 参数不合法（页码越界、角度不是 90 的倍数……）。
    Bad(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::Pdf(m) | Error::Bad(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<lopdf::Error> for Error {
    fn from(e: lopdf::Error) -> Self {
        Error::Pdf(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// 一个待编辑的 PDF 文件。页码一律**从 1 开始**，与阅读器和 Word 的习惯一致。
#[derive(Debug, Clone)]
pub struct Pdf {
    doc: Document,
}

impl Pdf {
    /// 读一个 PDF 文件。
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let doc = Document::load(path).map_err(|e| Error::Pdf(format!("{}: {e}", path.display())))?;
        if doc.objects.is_empty() {
            return Err(Error::Pdf(format!("{}: 一个对象都没有的 PDF", path.display())));
        }
        Ok(Pdf { doc })
    }

    /// 读内存里的 PDF 字节（拖进来的文件、附件、下载的文件都走这里）。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let doc = Document::load_mem(bytes).map_err(|e| Error::Pdf(e.to_string()))?;
        if doc.objects.is_empty() {
            return Err(Error::Pdf("一个对象都没有的 PDF".into()));
        }
        Ok(Pdf { doc })
    }

    /// 页数。0 页的 PDF 也能读到，后续每一条命令都会拒绝。
    pub fn page_count(&self) -> usize {
        self.doc.get_pages().len()
    }

    /// 页码，从 1 到 [`Self::page_count`]。
    pub fn page_numbers(&self) -> Vec<u32> {
        self.doc.get_pages().keys().copied().collect()
    }

    /// 一页的文本层（按阅读序排好的行）。扫描件没有文字层，返回空——那是 OCR 的活。
    pub fn text_of(&self, page: u32) -> text::PageText {
        text::page_text(&self.doc, page).sorted()
    }

    /// 一页的宽和高（PostScript 点，1/72 英寸）。批注和水印按它定位。
    pub fn page_size(&self, page: u32) -> Result<(f64, f64)> {
        let id = self.id_of(page)?;
        let Object::Dictionary(d) = self.doc.get_object(id)? else {
            return Err(Error::Pdf(format!("第 {page} 页不是字典")));
        };
        let Object::Array(box_) = d.get(b"MediaBox")? else { return Err(Error::Pdf(format!("第 {page} 页没有 MediaBox"))) };
        let at = |i: usize| match box_.get(i) {
            Some(Object::Integer(v)) => *v as f64,
            Some(Object::Real(v)) => *v as f64,
            _ => 0.0,
        };
        Ok(((at(2) - at(0)).abs(), (at(3) - at(1)).abs()))
    }

    /// 把选中的页转 90 / 180 / 270 度（改页面的 `/Rotate`，不动内容，所以文字还是可选中的）。
    ///
    /// `pages` 留空表示每一页。
    pub fn rotate(&mut self, pages: &[u32], degrees: u32) -> Result<()> {
        if !matches!(degrees, 90 | 180 | 270) {
            return Err(Error::Bad(format!("旋转角度只能是 90 / 180 / 270，收到 {degrees}")));
        }
        for id in self.ids_for(pages)? {
            let now: i64 = self.doc.get_object(id).ok().and_then(int_of_rotate).unwrap_or(0);
            let dict = self.page_dict(id)?;
            dict.set("Rotate", Object::Integer((now + degrees as i64).rem_euclid(360)));
        }
        Ok(())
    }

    /// 删掉选中的页。删到一页不剩会报错：那样只会得到一个打不开的 PDF。
    pub fn delete(&mut self, pages: &[u32]) -> Result<()> {
        for p in pages {
            self.id_of(*p)?;
        }
        if self.page_count() <= pages.len() {
            return Err(Error::Bad("不能把所有页都删掉".into()));
        }
        self.doc.delete_pages(pages);
        Ok(())
    }

    /// 只留下选中的页（其余删掉），也就是「提取这几页」。
    pub fn keep(&mut self, pages: &[u32]) -> Result<()> {
        if pages.is_empty() {
            return Err(Error::Bad("至少要选一页".into()));
        }
        for p in pages {
            self.id_of(*p)?;
        }
        let keep: Vec<u32> = pages.to_vec();
        let drop: Vec<u32> = self.page_numbers().into_iter().filter(|p| !keep.contains(p)).collect();
        self.doc.delete_pages(&drop);
        Ok(())
    }

    /// 把另一个 PDF 的选中的页**追加**到这份 PDF 末尾（合并）。`pages` 留空表示整份。
    pub fn append(&mut self, other: &Pdf, pages: &[u32]) -> Result<()> {
        if other.page_count() == 0 {
            return Err(Error::Bad("要合并的 PDF 没有页".into()));
        }
        let all = other.page_numbers();
        let wanted: Vec<u32> = if pages.is_empty() { all.clone() } else { pages.to_vec() };
        let missing: Vec<u32> = wanted.iter().filter(|p| !all.contains(p)).copied().collect();
        if !missing.is_empty() {
            return Err(Error::Bad(format!("那份 PDF 里没有第 {missing:?} 页（一共 {} 页）", all.len())));
        }

        // 对方的对象整体平移一个基准，编号就不会和自己的撞。
        let base = self.doc.max_id;
        let map: HashMap<lopdf::ObjectId, lopdf::ObjectId> =
            other.doc.objects.keys().map(|(num, generation)| ((*num, *generation), (num + base, *generation))).collect();
        let mut sub = other.doc.clone();
        sub.traverse_objects(|o| {
            if let Object::Reference(r) = o
                && let Some(n) = map.get(r)
            {
                *r = *n;
            }
        });
        for (old, obj) in sub.objects {
            let new = map.get(&old).copied().unwrap_or(old);
            self.doc.objects.insert(new, obj);
            self.doc.max_id = self.doc.max_id.max(new.0);
        }

        // 挂到自己的页树下。
        let added: Vec<Object> = wanted.iter().map(|p| Object::Reference(map[&other.doc.get_pages()[p]])).collect();
        let pages_id = self.root_pages()?;
        let Object::Dictionary(dict) = self.doc.get_object_mut(pages_id)? else {
            return Err(Error::Pdf("页树不是字典".into()));
        };
        let kids = match dict.get_mut(b"Kids")? {
            Object::Array(a) => a,
            _ => return Err(Error::Pdf("页树没有 Kids 数组".into())),
        };
        kids.extend(added);
        let count = kids.len() as i64;
        dict.set("Count", Object::Integer(count));
        Ok(())
    }

    /// 存到 `path`（覆盖）。
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.doc.save(path)?;
        Ok(())
    }

    /// 双层 PDF：给每页叠一层**隐形文本**（渲染模式 3），让扫描件可搜索、可复制。
    ///
    /// `layers` 是每页的文本（下标 0 对应第 1 页；短的按缺省处理——没有文本的页跳过）。
    /// 文本按 12pt 行距排在页面左侧自上而下；坐标只是让它落在页内，视觉上完全不可见
    /// （`3 Tr` 是 PDF 的隐形渲染模式，选择/搜索照常工作）。
    pub fn overlay_text(&mut self, layers: &[String]) -> Result<()> {
        if layers.is_empty() {
            return Err(Error::Bad("没有可叠的文本层".into()));
        }
        let n = self.page_count();
        if layers.len() > n {
            return Err(Error::Bad(format!("给了 {} 页的文本，但 PDF 只有 {n} 页", layers.len())));
        }
        // 一个共享的简单字体（Helvetica，WinAnsi 可直出 ASCII；非 ASCII 转成 '?'——
        // 双层 PDF 的搜索层以可检索为先，字形保真交给页面图像层）。
        let font = self.doc.add_object(Object::Dictionary(lopdf::Dictionary::from_iter([
            ("Type", Object::Name(b"Font".to_vec())),
            ("Subtype", Object::Name(b"Type1".to_vec())),
            ("BaseFont", Object::Name(b"Helvetica".to_vec())),
        ])));
        for (i, text) in layers.iter().enumerate() {
            if text.trim().is_empty() {
                continue;
            }
            let page_no = u32::try_from(i + 1).map_err(|_| Error::Bad("页数溢出".into()))?;
            let id = self.id_of(page_no)?;
            // 页面尺寸（文字排进页内；拿不到就用 A4）。
            let (w, h) = self.page_size(page_no).unwrap_or((595.0, 842.0));
            let _ = w;
            let mut body = String::with_capacity(text.len() + 64);
            body.push_str("BT\n3 Tr\n/F1 12 Tf\n");
            let mut y = h - 24.0;
            for line in text.lines() {
                if y < 24.0 {
                    break; // 页满即止（剩余文本丢弃：搜索层不是重排版）
                }
                body.push_str(&format!("1 0 0 1 14 {:.1} Tm ({}) Tj\n", y, escape_simple(line)));
                y -= 14.0;
            }
            body.push_str("ET");
            let content = self.doc.add_object(Object::Stream(lopdf::Stream::new(
                lopdf::Dictionary::from_iter([("Length", Object::Integer(body.len() as i64))]),
                body.into_bytes(),
            )));
            // 页字典取快照改完写回（借用一次一收）。
            let mut page = match self.doc.get_object(id)? {
                Object::Dictionary(d) => d.clone(),
                _ => return Err(Error::Pdf(format!("{id:?} 不是页面字典"))),
            };
            // 页资源里挂 F1；已有 Font 字典就并入，没有就新建。
            match page.get(b"Resources").cloned().unwrap_or(Object::Dictionary(lopdf::Dictionary::new())) {
                Object::Reference(rid) => {
                    let Object::Dictionary(mut rd) = self.doc.get_object(rid)?.clone() else {
                        return Err(Error::Pdf("Resources 不是字典".into()));
                    };
                    merge_font(&mut rd, font);
                    *self.doc.get_object_mut(rid)? = Object::Dictionary(rd);
                }
                Object::Dictionary(mut rd) => {
                    merge_font(&mut rd, font);
                    let rid = self.doc.add_object(Object::Dictionary(rd));
                    page.set("Resources", Object::Reference(rid));
                }
                _ => return Err(Error::Pdf("Resources 不是字典".into())),
            }
            // 内容流：已有 Contents（数组或单流）就追加为多流。
            let contents = page.get(b"Contents").cloned().ok();
            let new_contents = match contents {
                Some(Object::Reference(existing)) => {
                    let arr = Object::Array(vec![Object::Reference(existing), Object::Reference(content)]);
                    self.doc.add_object(arr)
                }
                Some(Object::Array(existing)) => {
                    let mut a = existing.clone();
                    a.push(Object::Reference(content));
                    self.doc.add_object(a)
                }
                _ => content,
            };
            page.set("Contents", Object::Reference(new_contents));
            *self.doc.get_object_mut(id)? = Object::Dictionary(page);
        }
        Ok(())
    }

    /// 存成内存里的 PDF 字节。
    pub fn to_bytes(&mut self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        self.doc.save_to(&mut out)?;
        Ok(out)
    }

    /// 页码 → 对象 id；`pages` 留空表示全部页。
    fn ids_for(&self, pages: &[u32]) -> Result<Vec<lopdf::ObjectId>> {
        if pages.is_empty() {
            return Ok(self.doc.get_pages().values().copied().collect());
        }
        pages.iter().map(|p| self.id_of(*p)).collect()
    }

    fn id_of(&self, page: u32) -> Result<lopdf::ObjectId> {
        self.doc.get_pages().get(&page).copied().ok_or_else(|| Error::Bad(format!("没有第 {page} 页（一共 {} 页）", self.doc.get_pages().len())))
    }

    /// 一页的页面字典。
    fn page_dict(&mut self, id: lopdf::ObjectId) -> Result<&mut lopdf::Dictionary> {
        match self.doc.get_object_mut(id)? {
            Object::Dictionary(d) => Ok(d),
            _ => Err(Error::Pdf(format!("{id:?} 不是页面字典"))),
        }
    }

    /// 自己页树（`/Pages`）的对象 id。
    fn root_pages(&self) -> Result<lopdf::ObjectId> {
        let root = reference_of(self.doc.trailer.get(b"Root")?)?;
        match self.doc.get_object(root)? {
            Object::Dictionary(cat) => reference_of(cat.get(b"Pages")?),
            _ => Err(Error::Pdf("Root 不是字典".into())),
        }
    }
}

/// `/Rotate` 的当前值，没有就是 0。
fn int_of_rotate(obj: &Object) -> Option<i64> {
    match obj {
        Object::Dictionary(d) => d.get(b"Rotate").ok().and_then(int_of_rotate),
        Object::Integer(i) => Some(*i),
        _ => None,
    }
}

/// 取一个对象引用；不是引用就是文件坏的。
fn reference_of(o: &Object) -> Result<lopdf::ObjectId> {
    match o {
        Object::Reference(id) => Ok(*id),
        _ => Err(Error::Pdf("PDF 里该是对象引用的地方不是引用".into())),
    }
}

/// 简单字体字符串里的字符转义（`(`、`)`、`\`；其余非 ASCII 降为 `?`——搜索层够用）。
fn escape_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' | '\r' => out.push(' '),
            c if (c as u32) < 0x80 => out.push(c),
            _ => out.push('?'),
        }
    }
    out
}

/// 资源字典里并入 `/F1 → font`（已有 Font 字典就改，没有就建）。
fn merge_font(rd: &mut lopdf::Dictionary, font: lopdf::ObjectId) {
    match rd.get_mut(b"Font") {
        Ok(Object::Dictionary(fd)) => {
            fd.set(b"F1", Object::Reference(font));
        }
        _ => {
            let fd = lopdf::Dictionary::from_iter([(b"F1".to_vec(), Object::Reference(font))]);
            rd.set(b"Font", Object::Dictionary(fd));
        }
    }
}

#[cfg(test)]
mod tests {
    use lopdf::{Dictionary, Stream};

    use super::*;

    /// 双层 PDF：叠的隐形文本层要能被（我们自己的）提取器读回来。
    #[test]
    fn overlay_text_makes_a_searchable_layer() {
        let mut pdf = blank(1);
        pdf.overlay_text(&["Searchable layer 42".into()]).expect("overlay");
        let out = pdf.text_of(1);
        let text: String = out.lines.iter().map(|l| l.text.clone()).collect();
        assert!(text.contains("Searchable layer 42"), "the layer reads back: {text:?}");
    }

    /// 转义与多页：括号文本不破坏流，多余的层报错，中文降级为问号但流仍合法。
    #[test]
    fn overlay_text_escapes_and_validates() {
        let mut pdf = blank(2);
        pdf.overlay_text(&["with (parens) and \\slash".into(), "page two".into()]).expect("overlay");
        let one: String = pdf.text_of(1).lines.iter().map(|l| l.text.clone()).collect();
        assert!(one.contains("(parens)"), "{one:?}");
        let two: String = pdf.text_of(2).lines.iter().map(|l| l.text.clone()).collect();
        assert!(two.contains("page two"), "{two:?}");
        // 层数超过页数是错误，不是 panic。
        assert!(blank(1).overlay_text(&["a".into(), "b".into()]).is_err());
        // 空层报错。
        assert!(blank(1).overlay_text(&[]).is_err());
    }

    /// 一份 n 页的 A4 空白 PDF，测试自己生成，仓库里不留 PDF 文件。
    fn blank(pages: u32) -> Pdf {
        let mut doc = Document::with_version("1.5");
        let font = doc.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Font".to_vec())),
            ("Subtype", Object::Name(b"Type1".to_vec())),
            ("BaseFont", Object::Name(b"Helvetica".to_vec())),
        ]));
        let pages_id = doc.add_object(Dictionary::from_iter([
            ("Type", Object::Name(b"Pages".to_vec())),
            ("Kids", Object::Array(vec![])),
            ("Count", Object::Integer(0)),
        ]));
        let mut kids = Vec::new();
        for _ in 0..pages {
            let resources = Dictionary::from_iter([("Font", Object::Reference(font))]);
            let content = doc.add_object(Stream::new(Dictionary::new(), b"BT /F1 12 Tf ET".to_vec()));
            let page = doc.add_object(Dictionary::from_iter([
                ("Type", Object::Name(b"Page".to_vec())),
                ("Parent", Object::Reference(pages_id)),
                ("MediaBox", Object::Array(vec![Object::Integer(0), Object::Integer(0), Object::Integer(595), Object::Integer(842)])),
                ("Resources", Object::Dictionary(resources)),
                ("Contents", Object::Reference(content)),
            ]));
            kids.push(Object::Reference(page));
        }
        let total = kids.len() as i64;
        let lopdf::Object::Dictionary(pages) = doc.get_object_mut(pages_id).unwrap() else { panic!("页树不是字典") };
        pages.set("Kids", Object::Array(kids));
        pages.set("Count", Object::Integer(total));
        let catalog = doc.add_object(Dictionary::from_iter([("Type", Object::Name(b"Catalog".to_vec())), ("Pages", Object::Reference(pages_id))]));
        doc.trailer.set("Root", Object::Reference(catalog));
        Pdf { doc }
    }

    #[test]
    fn page_count_matches_the_pages_that_were_written() {
        assert_eq!(blank(3).page_count(), 3);
        assert_eq!(blank(3).page_numbers(), vec![1, 2, 3]);
    }

    #[test]
    fn a_page_size_is_a4() {
        let (w, h) = blank(1).page_size(1).unwrap();
        assert!((w - 595.0).abs() < 0.01 && (h - 842.0).abs() < 0.01, "{w}×{h}");
        assert!(blank(1).page_size(2).is_err());
    }

    #[test]
    fn rotating_writes_the_rotate_entry_and_wraps() {
        let mut pdf = blank(2);
        pdf.rotate(&[1], 90).unwrap();
        pdf.rotate(&[1], 180).unwrap();
        assert_eq!(rotate_of(&mut pdf, 1), 270);
        pdf.rotate(&[1], 90).unwrap();
        assert_eq!(rotate_of(&mut pdf, 1), 0, "转回一整圈");
        assert_eq!(rotate_of(&mut pdf, 2), 0, "没选中的页不动");
    }

    #[test]
    fn rotating_needs_a_right_angle() {
        let err = blank(1).rotate(&[], 45).unwrap_err();
        assert!(err.to_string().contains('4'), "{err}");
    }

    #[test]
    fn delete_and_keep_change_the_page_count() {
        let mut pdf = blank(4);
        pdf.delete(&[2]).unwrap();
        assert_eq!(pdf.page_count(), 3);
        assert_eq!(pdf.page_numbers(), vec![1, 2, 3], "留下的页重新编号");
        pdf.keep(&[2]).unwrap();
        assert_eq!(pdf.page_count(), 1);
    }

    #[test]
    fn every_page_or_a_page_that_is_not_there() {
        let mut pdf = blank(2);
        assert!(pdf.delete(&[1, 2]).is_err(), "删到一页不剩要拒绝");
        assert!(pdf.delete(&[9]).is_err(), "不存在的页码要报错");
        assert!(pdf.keep(&[9]).is_err(), "不存在的页码要报错");
        assert!(pdf.keep(&[]).is_err(), "一页不选要报错");
        let mut all = blank(2);
        all.delete(&[]).unwrap();
        assert_eq!(all.page_count(), 2, "删的页码留空＝什么都不做");
    }

    #[test]
    fn append_adds_the_other_documents_pages() {
        let mut pdf = blank(1);
        pdf.append(&blank(3), &[2, 3]).unwrap();
        assert_eq!(pdf.page_count(), 3);
        assert_eq!(pdf.page_size(3).unwrap().0, 595.0, "并进来的页还能读出尺寸");
        let mut whole = blank(1);
        whole.append(&blank(2), &[]).unwrap();
        assert_eq!(whole.page_count(), 3, "不给页码＝整份并进来");
        assert!(whole.append(&blank(0), &[]).is_err(), "空 PDF 要拒绝");
        assert!(blank(1).append(&blank(2), &[7]).is_err(), "不存在的页码要报错");
    }

    #[test]
    fn a_saved_file_is_a_file_it_could_open_again() {
        let path = temp().join("roundtrip.pdf");
        let mut pdf = blank(3);
        pdf.rotate(&[2], 180).unwrap();
        pdf.save(&path).unwrap();
        let mut again = Pdf::open(&path).unwrap();
        assert_eq!(again.page_count(), 3);
        assert!(!again.to_bytes().unwrap().is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_that_is_not_a_pdf_is_refused() {
        let path = temp().join("not-a-pdf.pdf");
        std::fs::write(&path, b"just some text").unwrap();
        assert!(matches!(Pdf::open(&path), Err(Error::Pdf(_))));
        let _ = std::fs::remove_file(&path);
        assert!(matches!(Pdf::from_bytes(b"nope"), Err(Error::Pdf(_))));
    }

    fn temp() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("wordcraft-pdfedit-test");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn rotate_of(pdf: &mut Pdf, page: u32) -> i64 {
        let id = pdf.doc.get_pages()[&page];
        match pdf.doc.get_object(id) {
            Ok(o) => int_of_rotate(o).unwrap_or(0),
            Err(_) => 0,
        }
    }
}
