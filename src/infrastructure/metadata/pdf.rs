use std::char::decode_utf16;
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
    let bytes = dict.get(key).ok()?.as_str().ok()?;

    // UTF-16BE PDFs usually start with BOM FE FF
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();

        let decoded: String = decode_utf16(utf16).filter_map(Result::ok).collect();

        Some(decoded.trim().to_string())
    } else {
        Some(String::from_utf8_lossy(bytes).trim().to_string())
    }
}
