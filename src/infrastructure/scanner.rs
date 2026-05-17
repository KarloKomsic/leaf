use crate::domain::document::Document;
use crate::infrastructure::cache::metadata_cache::MetadataCache;
use crate::infrastructure::metadata::extractor::enrich_document;

use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_library(path: &Path, cache: &mut MetadataCache) -> Vec<Document> {
    let mut documents = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();

            if !is_pdf(&path) {
                continue;
            }

            let mut document = Document::from_path(path.clone());

            if let Some(metadata) = cache.get(&path) {
                document.apply_metadata(metadata.title.clone(), metadata.author.clone());
            } else {
                enrich_document(&mut document);

                cache.insert(
                    path.clone(),
                    crate::domain::metadata::Metadata {
                        title: Some(document.title.clone()),
                        author: document.author.clone(),
                    },
                );
            }

            documents.push(document);
        }
    }

    documents
}

fn is_pdf(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase() == "pdf")
        .unwrap_or(false)
}
