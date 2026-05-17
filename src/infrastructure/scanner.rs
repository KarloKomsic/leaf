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

            if let Some(entry) = cache.get(&path) {
                document
                    .apply_metadata(entry.metadata.title.clone(), entry.metadata.author.clone());
            } else {
                let file_metadata = fs::metadata(&path).unwrap();

                let modified = file_metadata
                    .modified()
                    .unwrap()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                cache.insert(
                    path.clone(),
                    crate::infrastructure::cache::metadata_cache::CachedEntry {
                        metadata: crate::domain::metadata::Metadata {
                            title: Some(document.title.clone()),
                            author: document.author.clone(),
                        },

                        // Helps us know if the file changed later
                        modified,
                        size: file_metadata.len(),
                    },
                )
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
