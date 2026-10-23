//! 提取一页里嵌入的位图（给 OCR 桥喂图用）。
//!
//! 扫描件 PDF 的每页通常是一整张图片（`/XObject /Subtype /Image`）。这里把它们抠出来：
//! - **DCTDecode**（JPEG）：`Stream.content` 就是 JPEG 文件字节，原样交出；
//! - **FlateDecode / 未压缩**：解出来是**原始像素**（不是图片文件），连同宽高、颜色分量
//!   一起交出，由调用方重编码成 PNG。
//!
//! 颜色空间只认设备灰度（1 分量）和 RGB（3 分量）；CMYK / 索引 / 掩码图不支持——
//! OCR 不需要它们，遇到就跳过而不是失败。

/// 一张抠出来的位图。
#[derive(Debug, Clone)]
pub struct Image {
    /// XObject 名（`/Im0` 这样的，去掉了斜杠）。
    pub name: String,
    /// 图片种类：JPEG 文件字节，或待重编码的原始像素。
    pub kind: Kind,
    /// 像素宽（`/Width`）。
    pub w: u32,
    /// 像素高（`/Height`）。
    pub h: u32,
}

/// 位图的载荷形态。
#[derive(Debug, Clone)]
pub enum Kind {
    /// 一整个 JPEG 文件（DCTDecode），写盘就能用。
    Jpeg(Vec<u8>),
    /// 未编码的像素行（FlateDecode 后或未压缩），每像素 `components` 个字节，
    /// 从左上角开始逐行排列。没有行填充。
    Raw {
        /// 每像素的字节数（1=灰度，3=RGB）。
        components: u8,
        /// 像素数据。
        bytes: Vec<u8>,
    },
}

/// 一页的位图列表（资源序）。
#[derive(Debug, Clone, Default)]
pub struct PageImages {
    /// 图片，按资源字典里的顺序。
    pub images: Vec<Image>,
}

/// 一页里抠出的图太多就别抠了——正常页面撑死几张，几百张是坏文件。
pub const MAX_IMAGES_PER_PAGE: usize = 64;
/// 单张图的单个流最多这么大（JPEG 原样字节也照此上限）。
pub const MAX_STREAM_BYTES: usize = 64 * 1024 * 1024;

/// 读一页的嵌入位图。页不存在或没有图 → 空列表（不是错误：扫描件才需要它）。
pub fn page_images(doc: &lopdf::Document, page: u32) -> PageImages {
    let mut out = PageImages::default();
    let Some(&id) = doc.get_pages().get(&page) else { return out };
    let dict = match doc.get_object(id) {
        Ok(lopdf::Object::Dictionary(d)) => d.clone(),
        _ => return out,
    };
    // Resources 可以是间接引用，也可以内联在页字典里。
    let res = match dict.get(b"Resources").cloned().unwrap_or(lopdf::Object::Null) {
        lopdf::Object::Reference(rid) => doc.get_object(rid).cloned().ok().and_then(|o| match o {
            lopdf::Object::Dictionary(d) => Some(d),
            _ => None,
        }),
        lopdf::Object::Dictionary(d) => Some(d),
        _ => None,
    };
    let Some(res) = res else { return out };
    let xobjs = match res.get(b"XObject").cloned().unwrap_or(lopdf::Object::Null) {
        lopdf::Object::Reference(rid) => doc.get_object(rid).cloned().ok().and_then(|o| match o {
            lopdf::Object::Dictionary(d) => Some(d),
            _ => None,
        }),
        lopdf::Object::Dictionary(d) => Some(d),
        _ => None,
    };
    let Some(xobjs) = xobjs else { return out };
    for (name, obj) in xobjs.iter() {
        if out.images.len() >= MAX_IMAGES_PER_PAGE {
            break;
        }
        // 资源字典的 key 一般不带斜杠（`Im0`），偶尔带（`/Im0`）——都收。
        let bare = name.strip_prefix(b"/").unwrap_or(name);
        let name = String::from_utf8_lossy(bare).into_owned();
        let stream = match obj {
            lopdf::Object::Reference(sid) => match doc.get_object(*sid) {
                Ok(lopdf::Object::Stream(s)) => s.clone(),
                _ => continue,
            },
            lopdf::Object::Stream(s) => s.clone(),
            _ => continue,
        };
        if let Some(img) = image_of(&name, &stream) {
            out.images.push(img);
        }
    }
    out
}

/// 一个图片 XObject 流 → [`Image`]；不是图、或颜色空间不支持 → `None`。
fn image_of(name: &str, stream: &lopdf::Stream) -> Option<Image> {
    let d = &stream.dict;
    let subtype = d.get(b"Subtype").ok().and_then(|o| o.as_name().ok()).unwrap_or(b"");
    if subtype != b"Image" {
        return None;
    }
    let w: u32 = number(d.get(b"Width").ok()?)? as u32;
    let h: u32 = number(d.get(b"Height").ok()?)? as u32;
    if w == 0 || h == 0 || w > 30_000 || h > 30_000 {
        return None;
    }
    let filter = stream.filters().unwrap_or_default();
    let kind = match filter.first().map(|f| &f[..]) {
        // JPEG：流字节就是 JPEG 文件，原样交出。
        Some(b"DCTDecode") => Kind::Jpeg(stream.content.clone()),
        Some(b"JPXDecode") => return None, // JPEG2000：跳过（罕见且编码器不在依赖里）
        // 原始像素（Flate 后或未压缩）。
        Some(b"FlateDecode") | None => {
            let bytes = match stream.decompressed_content() {
                Ok(b) => b,
                Err(_) => return None,
            };
            let components = components_of(d)?;
            let want = (w as usize).checked_mul(h as usize)?.checked_mul(components as usize)?;
            if bytes.len() != want {
                return None; // 尺寸对不上（有行填充或奇偶 packing）——放弃而不是猜
            }
            Kind::Raw { components, bytes }
        }
        _ => return None, // LZW / CCITT / RunLength：罕见，跳过
    };
    if stream.content.len() > MAX_STREAM_BYTES {
        return None;
    }
    Some(Image { name: name.to_string(), kind, w, h })
}

/// `/ColorSpace` → 每像素字节数；只认设备灰度和 RGB。
fn components_of(d: &lopdf::Dictionary) -> Option<u8> {
    match d.get(b"ColorSpace").ok()? {
        lopdf::Object::Name(n) => match n.as_slice() {
            b"DeviceGray" | b"G" | b"CalGray" => Some(1),
            b"DeviceRGB" | b"RGB" | b"CalRGB" => Some(3),
            _ => None,
        },
        // [/ICCBased N]：看 ICC 流的 /N。
        lopdf::Object::Array(a) => {
            if a.first().and_then(|o| o.as_name().ok()) == Some(b"ICCBased") {
                match a.get(1) {
                    Some(lopdf::Object::Integer(n)) if *n == 1 || *n == 3 => Some(*n as u8),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// `/Width`、`/Height` 这种数字项（Integer 或 Real 都收，钳到非负）。
fn number(obj: &lopdf::Object) -> Option<i64> {
    match obj {
        lopdf::Object::Integer(i) => Some(*i),
        lopdf::Object::Real(r) => Some(r.round() as i64),
        _ => None,
    }
}
