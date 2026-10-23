//! OCR 引擎桥：把 PDF 页面的位图喂给外部的 tesseract，取回文本。
//!
//! WordCraft 自己不做 OCR（那是一个完整的识别引擎，不是一个周末的活），做的是
//! **最后一公里**：抠图 → 喂给 tesseract → 收文本 → 叠成双层 PDF（`pdf ocr`）。
//!
//! tesseract 是可选项：装了才有 OCR，没装报错并告诉用户怎么装。零新增 Rust 依赖，
//! 没有 C 编译链——引擎在则 OCR 在。
//!
//! 只通过参数消毒后的命令行调用 tesseract（不 shell 拼接），所以语言串里的花活
//! （`&`、`|`、引号）会被直接拒绝，而不是跑进 cmd 里去。

use std::path::{Path, PathBuf};

use wordcraft_pdfedit::images::{Image, Kind};

/// 一页 OCR 出的文本，附图片信息（给日志和错误信息用）。
#[derive(Debug)]
pub struct OcrPage {
    /// 识别出的文本（tesseract 的 `.txt` 原样，去 BOM、统一换行）。
    pub text: String,
    /// 用的是哪张图（`Im0` 这样的名字）。
    pub image_name: String,
    /// 图片尺寸。
    pub w: u32,
    pub h: u32,
}

/// 找 tesseract 可执行文件：显式路径 → PATH → Windows 常见安装位置。
pub fn find_tesseract(explicit: Option<&str>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("--tesseract 给的路径不是文件：{p}"));
    }
    // PATH 里找（Windows 要带 .exe）。
    for name in ["tesseract", "tesseract.exe"] {
        if let Some(found) = in_path(name) {
            return Ok(found);
        }
    }
    // Windows 默认安装位置（UB Mannheim 安装器写的）。
    if cfg!(windows) {
        for base in [r"C:\Program Files\Tesseract-OCR\tesseract.exe", r"C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"] {
            let p = PathBuf::from(base);
            if p.is_file() {
                return Ok(p);
            }
        }
    }
    Err("没找到 tesseract。装一个（Windows：UB-Mannheim 的 Tesseract 安装包；\
         macOS：brew install tesseract；Linux：发行版包管理器），或用 --tesseract 指定路径。"
        .into())
}

/// PATH 的目录里逐个找同名文件。
fn in_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths).find_map(|dir| {
        let cand = dir.join(name);
        cand.is_file().then_some(cand)
    })
}

/// 语言串消毒：tesseract 的 `-l` 值只允许字母数字下划线和加号（`chi_sim+eng`）。
/// 别的一律拒绝——这个值最终要进命令行，不能让 `&`、引号之类的混进去。
pub fn sanitize_lang(lang: &str) -> Result<String, String> {
    let ok = lang.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+');
    if lang.is_empty() || !ok {
        return Err(format!("--lang 只能是语言代码（如 eng、chi_sim+eng），收到：{lang:?}"));
    }
    Ok(lang.to_string())
}

/// 一页里挑**面积最大**的那张图——扫描件通常整页一张图；
/// 真有多个图时最大的那个就是页面本身（小图标是装饰）。
pub fn biggest_image(images: &[Image]) -> Option<&Image> {
    images.iter().max_by_key(|i| i.w.saturating_mul(i.h))
}

/// 把抠出的位图编码成图片文件字节：JPEG 原样（tesseract 直接吃），
/// Raw 像素重编码成 PNG（灰度走 luma8，RGB 走 rgb8）。
pub fn encode_image_bytes(img: &Image) -> Result<Vec<u8>, String> {
    match &img.kind {
        Kind::Jpeg(bytes) => Ok(bytes.clone()),
        Kind::Raw { components, bytes } => {
            // 长度先验：from_raw 对不上长度是 panic，不是 None（never-crash：自己先查）。
            let need = (img.w as usize).checked_mul(img.h as usize).and_then(|n| n.checked_mul(*components as usize));
            if need != Some(bytes.len()) {
                return Err(format!("图片 {} 的像素数对不上 {}×{}", img.name, img.w, img.h));
            }
            let buf = match *components {
                1 => image::GrayImage::from_raw(img.w, img.h, bytes.clone()).map(image::DynamicImage::ImageLuma8),
                _ => image::RgbImage::from_raw(img.w, img.h, bytes.clone()).map(image::DynamicImage::ImageRgb8),
            };
            let Some(buf) = buf else {
                return Err(format!("图片 {} 的像素装不进 {}×{}", img.name, img.w, img.h));
            };
            let mut out = std::io::Cursor::new(Vec::new());
            buf.write_to(&mut out, image::ImageFormat::Png).map_err(|e| format!("图片 {} 编码 PNG 失败：{e}", img.name))?;
            Ok(out.into_inner())
        }
    }
}

/// 对一页跑 tesseract：图写临时文件 → `tesseract <img> <base> -l <lang>` → 读 `<base>.txt`。
///
/// `tmp_dir` 由调用方建好（一 PDF 一个），文件名带页码不互相踩。
pub fn ocr_image(exe: &Path, lang: &str, img: &Image, img_bytes: &[u8], tmp_dir: &Path, page: u32) -> Result<OcrPage, String> {
    let stem = format!("p{page}-{}", sanitize_name(&img.name));
    let img_path = tmp_dir.join(format!("{stem}.img"));
    let out_base = tmp_dir.join(&stem);
    // 无论成败都清掉临时文件。
    let result = (|| {
        std::fs::write(&img_path, img_bytes).map_err(|e| format!("写临时图片失败：{e}"))?;
        let output = std::process::Command::new(exe)
            .arg(&img_path)
            .arg(&out_base)
            .arg("-l")
            .arg(lang)
            .output()
            .map_err(|e| format!("启动 tesseract 失败（{}）：{e}", exe.display()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tesseract 认不出第 {page} 页（退出码 {:?}）：{}", output.status.code(), stderr.trim()));
        }
        let txt = std::fs::read_to_string(out_base.with_extension("txt")).map_err(|e| format!("读 tesseract 输出失败：{e}"))?;
        Ok(clean_text(&txt))
    })();
    let _ = std::fs::remove_file(&img_path);
    let _ = std::fs::remove_file(out_base.with_extension("txt"));
    result.map(|text| OcrPage { text, image_name: img.name.clone(), w: img.w, h: img.h })
}

/// tesseract 输出的整理：去 BOM、CRLF → LF、去首尾空白。
fn clean_text(raw: &str) -> String {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    raw.replace("\r\n", "\n").replace('\r', "\n").trim().to_string()
}

/// 图片名进文件名前收窄到安全字符。
fn sanitize_name(name: &str) -> String {
    name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 语言串：合法的形态放行，注入向的一律拒绝。
    #[test]
    fn lang_is_sanitize() {
        assert_eq!(sanitize_lang("eng").unwrap(), "eng");
        assert_eq!(sanitize_lang("chi_sim+eng").unwrap(), "chi_sim+eng");
        assert_eq!(sanitize_lang("osd_1").unwrap(), "osd_1");
        assert!(sanitize_lang("").is_err());
        assert!(sanitize_lang("eng & del").is_err(), "空格和 & 不能进命令行");
        assert!(sanitize_lang("eng;rm").is_err(), "分号不能进命令行");
        assert!(sanitize_lang("en\"g").is_err(), "引号不能进命令行");
        assert!(sanitize_lang("$(boom)").is_err(), "替换不能进命令行");
    }

    /// 大图优先：扫描件的页面图比装饰小图标大。
    #[test]
    fn biggest_image_wins_by_area() {
        let img = |name: &str, w: u32, h: u32| Image { name: name.into(), kind: Kind::Jpeg(vec![1]), w, h };
        let imgs = vec![img("icon", 16, 16), img("page", 1000, 1400), img("logo", 800, 20)];
        assert_eq!(biggest_image(&imgs).unwrap().name, "page");
        assert!(biggest_image(&[]).is_none());
    }

    /// Raw 像素 → PNG 编码（tesseract 吃的形态）。
    #[test]
    fn raw_pixels_encode_to_png() {
        let img = Image { name: "Im0".into(), kind: Kind::Raw { components: 1, bytes: vec![0, 128, 255, 10] }, w: 2, h: 2 };
        let png = encode_image_bytes(&img).unwrap();
        assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G'], "PNG 魔数");
        // 尺寸对不上的像素拒绝（never-crash：不猜）。
        let bad = Image { kind: Kind::Raw { components: 1, bytes: vec![0; 5] }, w: 2, h: 2, ..img.clone() };
        assert!(encode_image_bytes(&bad).is_err());
    }

    /// 端到端：假 tesseract（读啥图都吐同一句）→ 桥接 → 文本层。
    /// 验证：子进程真的被调了、`.txt` 被读回、临时文件被清掉。
    #[test]
    fn ocr_image_runs_the_engine_and_reads_the_txt() {
        let dir = std::env::temp_dir().join("wordcraft-ocrbridge-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = fake_engine(&dir);
        let img = Image { name: "Im0".into(), kind: Kind::Jpeg(vec![0xFF, 0xD8, 0xD9]), w: 3, h: 2 };
        let out = ocr_image(&exe, "eng", &img, &[0xFF, 0xD8, 0xD9], &dir, 1).expect("bridge call");
        assert_eq!(out.text, "Hello from the fake engine");
        assert_eq!(out.image_name, "Im0");
        // 临时文件清干净了。
        assert!(!dir.join("p1-Im0.img").exists(), "输入图片删掉了");
        assert!(!dir.join("p1-Im0.txt").exists(), "输出 txt 删掉了");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 造一个假 tesseract：`fake <img> <base> -l <lang>` → 写 `<base>.txt`（固定一句话）。
    #[cfg(windows)]
    fn fake_engine(dir: &Path) -> PathBuf {
        let script = dir.join("fake-tesseract.ps1");
        std::fs::write(
            &script,
            "param($img, $base, $flag, $lang)\nSet-Content -Path \"$($base).txt\" -Value 'Hello from the fake engine' -NoNewline\n",
        )
        .unwrap();
        let cmd = dir.join("fake-tesseract.cmd");
        std::fs::write(&cmd, format!("@powershell -NoProfile -ExecutionPolicy Bypass -File \"{}\" %*\n", script.display())).unwrap();
        cmd
    }

    #[cfg(unix)]
    fn fake_engine(dir: &Path) -> PathBuf {
        let script = dir.join("fake-tesseract.sh");
        std::fs::write(&script, "#!/bin/sh\n# $1=img $2=base $3=-l $4=lang\necho -n 'Hello from the fake engine' > \"$2.txt\"\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&script).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&script, perm).unwrap();
        script
    }

    /// 引擎退出非零：错误带 stderr，不 panic。
    #[test]
    fn a_failing_engine_is_an_error() {
        let dir = std::env::temp_dir().join("wordcraft-ocrbridge-fail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        #[cfg(windows)]
        let exe = {
            let p = dir.join("fail.cmd");
            std::fs::write(&p, "@echo oops bad language & exit /b 1\n").unwrap();
            p
        };
        #[cfg(unix)]
        let exe = {
            let p = dir.join("fail.sh");
            std::fs::write(&p, "#!/bin/sh\necho 'oops bad language' >&2\nexit 1\n").unwrap();
            use std::os::unix::fs::PermissionsExt;
            let mut perm = std::fs::metadata(&p).unwrap().permissions();
            perm.set_mode(0o755);
            std::fs::set_permissions(&p, perm).unwrap();
            p
        };
        let img = Image { name: "Im0".into(), kind: Kind::Jpeg(vec![1]), w: 2, h: 2 };
        let err = ocr_image(&exe, "eng", &img, &[1], &dir, 1).unwrap_err();
        assert!(err.contains("oops") || err.contains("1"), "错误里有引擎的输出：{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `clean_text`：BOM、CRLF、首尾空白。
    #[test]
    fn text_from_the_engine_is_cleaned() {
        assert_eq!(clean_text("\u{feff}hi\r\nthere\r\n"), "hi\nthere");
        assert_eq!(clean_text("  \n x \n "), "x");
    }
}
