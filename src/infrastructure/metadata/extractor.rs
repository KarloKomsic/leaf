use std::path::Path;

use crate::domain::{document::Document, metadata::Metadata};

// Routes to the correct extractor based on file extension. Called from
// scanner.rs when metadata isn't already in the cache.
pub fn enrich_document(document: &mut Document) -> bool {
    if let Some(metadata) = extract_metadata(&document.path) {
        let had_title = metadata.title.is_some();
        let had_author = metadata.author.is_some();

        apply_metadata(document, metadata);

        had_title || had_author
    } else {
        false
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

// Skip junk that PDF generators often leave in the metadata fields
fn is_good_metadata(text: &str) -> bool {
    let lower = text.to_lowercase();

    !lower.is_empty()
        && !lower.contains("pdfdrive")
        && !lower.contains("z-library")
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
