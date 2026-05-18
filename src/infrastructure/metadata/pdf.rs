// PDF metadata extraction with a fast path (pdfinfo) and a slow but
// reliable fallback (lopdf) for unusual or malformed files.

use std::char::decode_utf16;
use std::path::Path;
use std::process::Command;

use lopdf::{Dictionary, Document as PdfDocument};

use crate::domain::metadata::Metadata;

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    // Try pdfinfo first: takes about 13ms per file versus 3.75s
    // for lopdf, so it is well worth trying the external tool first.
    if let Some(metadata) = extract_via_pdfinfo(path) {
        return Some(metadata);
    }

    // Fall back to lopdf when pdfinfo is not available or fails
    extract_via_lopdf(path)
}

fn extract_via_pdfinfo(path: &Path) -> Option<Metadata> {
    let output = Command::new("pdfinfo").arg(path).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut title = None;
    let mut author = None;

    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("Title:").map(|s| s.trim()) {
            if !value.is_empty() {
                title = Some(value.to_string());
            }
        } else if let Some(value) = line.strip_prefix("Author:").map(|s| s.trim()) {
            if !value.is_empty() {
                author = Some(value.to_string());
            }
        }
    }

    Some(Metadata { title, author })
}

fn extract_via_lopdf(path: &Path) -> Option<Metadata> {
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

    // PDFs encoded in UTF-16BE usually start with the BOM bytes FE FF
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
