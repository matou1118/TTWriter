//! The East Asian language of a run: OOXML `w:lang/@w:eastAsia`. Word writes the three
//! language attributes of a run together — `w:val` (Latin text), `w:eastAsia` (Chinese,
//! Japanese and Korean text) and `w:bidi` (complex script) — and reads them back per script.
//! Without the East Asian one, a Chinese run in a document Word saved is indistinguishable
//! from a Latin one, and Word proofing, fonts and line breaking follow the wrong rule.
//!
//! Fixtures are hand-written from ECMA-376 Part 1 §17.3.2.17 (lang).

use std::io::{Read, Write};

use wordcraft_doc::Document;

fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        zw.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        zw.write_all(bytes).unwrap();
    }
    zw.finish().unwrap().into_inner()
}

/// A package whose body is `body`.
fn docx(body: &str) -> Vec<u8> {
    let root = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );
    zip(&[("_rels/.rels", root.as_bytes()), ("word/document.xml", doc.as_bytes())])
}

fn read(body: &str) -> Document {
    wordcraft_docx::read(&docx(body)).unwrap()
}

#[test]
fn reads_the_east_asian_language_of_a_run() {
    // What Word writes for a document with Chinese, Latin and complex-script text.
    let body = r#"
<w:p><w:r><w:rPr><w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/></w:rPr><w:t>混合文本</w:t></w:r></w:p>"#;
    let d = read(body);
    let p = d.body.iter().filter_map(|b| b.as_para()).next().unwrap();
    let r = &p.runs[0].props;
    assert_eq!(r.lang.as_deref(), Some("en-US"));
    assert_eq!(r.lang_ea.as_deref(), Some("zh-CN"), "the East Asian language is read");
    assert_eq!(r.lang_bidi.as_deref(), Some("ar-SA"));
}

#[test]
fn a_run_without_an_east_asian_language_has_none() {
    let body = r#"<w:p><w:r><w:rPr><w:lang w:val="en-US"/></w:rPr><w:t>Latin</w:t></w:r></w:p>"#;
    let d = read(body);
    let p = d.body.iter().filter_map(|b| b.as_para()).next().unwrap();
    assert_eq!(p.runs[0].props.lang_ea, None);
}

#[test]
fn the_east_asian_language_survives_a_save() {
    // The bug this file covers: `w:lang/@w:eastAsia` was read and then dropped, so a
    // document Word saved lost its Chinese language on the way through.
    let body = r#"<w:p><w:r><w:rPr><w:lang w:val="en-US" w:eastAsia="zh-CN"/></w:rPr><w:t>中文</w:t></w:r></w:p>"#;
    let d = read(body);
    let out = wordcraft_docx::write(&d).expect("write");
    let mut zf = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut xml = String::new();
    zf.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
    assert!(xml.contains(r#"w:eastAsia="zh-CN""#), "the East Asian language is written back: {xml}");
}

#[test]
fn all_three_languages_are_written_together() {
    let body = r#"<w:p><w:r><w:rPr><w:lang w:val="en-US" w:eastAsia="zh-CN" w:bidi="ar-SA"/></w:rPr><w:t>x</w:t></w:r></w:p>"#;
    let d = read(body);
    let out = wordcraft_docx::write(&d).expect("write");
    let mut zf = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut xml = String::new();
    zf.by_name("word/document.xml").unwrap().read_to_string(&mut xml).unwrap();
    let tag = xml.split("<w:lang").nth(1).unwrap().split('>').next().unwrap();
    assert!(tag.contains(r#"w:val="en-US""#), "{tag}");
    assert!(tag.contains(r#"w:eastAsia="zh-CN""#), "{tag}");
    assert!(tag.contains(r#"w:bidi="ar-SA""#), "{tag}");
}
