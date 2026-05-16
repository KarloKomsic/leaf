use crate::domain::document::Document;

use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_library(path: &Path) -> Vec<Document> {
    let mut documents = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();

            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "pdf" {
                    documents.push(Document::from_path(path));
                }
            }
        }
    }

    documents
}
