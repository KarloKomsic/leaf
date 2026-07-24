// Picks the right metadata extractor based on file extension and
// applies the result to the document. Returns whether any new
// metadata was actually found.

use std::path::Path;

use crate::domain::{document::Document, metadata::Metadata};

pub fn enrich_document(document: &mut Document) {
    if let Some(metadata) = extract_metadata(&document.path) {
        apply_metadata(document, metadata);
    }
}

fn extract_metadata(path: &Path) -> Option<Metadata> {
    let extension = path.extension()?.to_string_lossy().to_lowercase();

    match extension.as_str() {
        "pdf" => crate::infrastructure::metadata::pdf::extract_metadata(path),
        "epub" => crate::infrastructure::metadata::epub::extract_metadata(path),

        _ => None,
    }
}

fn apply_metadata(document: &mut Document, metadata: Metadata) {
    apply_title(document, metadata.title);
    apply_author(document, metadata.author);
}

fn apply_title(document: &mut Document, title: Option<String>) {
    if let Some(title) = clean_and_validate(title) {
        document.title = title;
    }
}

fn apply_author(document: &mut Document, author: Option<String>) {
    if let Some(author) = clean_and_validate(author) {
        document.author = Some(author);
    }
}

fn clean_and_validate(text: Option<String>) -> Option<String> {
    let cleaned = clean_metadata(text?.trim());

    if is_good_metadata(&cleaned) {
        Some(cleaned)
    } else {
        None
    }
}

// Rejects placeholder values that PDF generators often leave behind,
// like "pdfdrive" or "z-library" in the author field.
fn is_good_metadata(text: &str) -> bool {
    let lower = text.to_lowercase();

    !lower.is_empty()
        && !lower.contains("pdfdrive")
        && !lower.contains("unknown")
        && !lower.ends_with(".pdf")
        && !lower.ends_with(".epub")
        && text.len() > 3
}

fn clean_metadata(text: &str) -> String {
    text.trim_matches('\u{feff}')
        .trim_matches('\0')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_good_metadata_rejects_pdfdrive() {
        assert!(!is_good_metadata("pdfdrive"));
    }

    #[test]
    fn is_good_metadata_rejects_unknown() {
        assert!(!is_good_metadata("Unknown"));
    }

    #[test]
    fn is_good_metadata_rejects_too_short() {
        assert!(!is_good_metadata("ab"));
    }

    #[test]
    fn is_good_metadata_rejects_filename_extension() {
        assert!(!is_good_metadata("document.pdf"));
    }

    #[test]
    fn is_good_metadata_accepts_valid_title() {
        assert!(is_good_metadata("The Great Gatsby"));
    }

    #[test]
    fn clean_metadata_removes_bom() {
        assert_eq!(clean_metadata("\u{feff}Title"), "Title");
    }

    #[test]
    fn clean_metadata_removes_null_bytes() {
        assert_eq!(clean_metadata("Title\0\0"), "Title");
    }

    #[test]
    fn clean_metadata_trims_whitespace() {
        assert_eq!(clean_metadata("  Title  "), "Title");
    }

    #[test]
    fn clean_metadata_handles_bom_and_nulls_together() {
        assert_eq!(clean_metadata("\u{feff}\0\0 Title \0\0"), "Title");
    }
}
