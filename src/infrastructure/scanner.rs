use crate::domain::document::Document;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_library(path: &Path) -> Vec<Document> {
    let mut documents = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "pdf" {
                    let title = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    documents.push(Document { title, path });
                }
            }
        }
    }

    documents
}
