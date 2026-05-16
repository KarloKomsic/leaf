use crate::domain::document::Document;
use crate::infrastructure::metadata::extractor::extract_document;

use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_library(path: &Path) -> Vec<Document> {
    let mut documents = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();

            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "pdf" {
                    if let Some(document) = extract_document(&path) {
                        documents.push(document);
                    }
                }
            }
        }
    }

    documents
}
