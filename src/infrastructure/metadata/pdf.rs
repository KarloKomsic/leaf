use std::path::Path;

use lopdf::{Dictionary, Document as PdfDocument};

use crate::domain::metadata::Metadata;

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    let pdf = PdfDocument::load(path).ok()?;

    let info_ref = pdf.trailer.get(b"Info").ok()?;
    let info_obj = pdf.get_object(info_ref.as_reference().ok()?).ok()?;

    let dict = info_obj.as_dict().ok()?;

    let title = extract_string(dict, b"Title");
    let author = extract_string(dict, b"Author");

    Some(Metadata { title, author })
}

fn extract_string(dict: &Dictionary, key: &[u8]) -> Option<String> {
    dict.get(key)
        .ok()
        .and_then(|obj| obj.as_str().ok())
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
}
