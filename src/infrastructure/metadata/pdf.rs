use std::path::Path;

use lopdf::{Dictionary, Document as PdfDocument};

pub fn extract_title_author(path: &Path) -> Option<(String, Option<String>)> {
    let pdf = PdfDocument::load(path).ok()?;

    let info_ref = pdf.trailer.get(b"Info").ok()?;
    let info_obj = pdf.get_object(info_ref.as_reference().ok()?).ok()?;

    let dict = info_obj.as_dict().ok()?;

    let title = extract_string(dict, b"Title")?;
    let author = extract_string(dict, b"Author");

    Some((title, author))
}

fn extract_string(dict: &Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key)
        .ok()
        .and_then(|value| value.as_str().ok())
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
}
