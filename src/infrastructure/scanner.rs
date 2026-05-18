// Walks the library directory tree and builds a list of Document
// objects. Metadata is pulled from the cache if available, otherwise
// extracted from the file and saved for next time.

use crate::domain::document::Document;
use crate::infrastructure::cache::metadata_cache::MetadataCache;
use crate::infrastructure::metadata::extractor::enrich_document;

use std::fs;
use std::path::Path;
use std::time::Instant;

const SUPPORTED_EXTENSIONS: &[&str] = &["pdf", "epub"];

// Uses an explicit stack instead of recursion so we don't risk
// blowing the stack on deeply nested directory trees.
pub fn scan_library(path: &Path, cache: &mut MetadataCache) -> Vec<Document> {
    let start = Instant::now();
    let mut documents = Vec::new();

    let mut dirs = vec![path.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    dirs.push(entry_path);
                    continue;
                }

                if !is_supported(&entry_path) {
                    continue;
                }

                let mut document = Document::from_path(entry_path.clone());

                if let Some(metadata) = cache.get(&entry_path) {
                    document.apply_metadata(
                        metadata.metadata.title.clone(),
                        metadata.metadata.author.clone(),
                    );
                } else {
                    enrich_document(&mut document);

                    let file_metadata = fs::metadata(&entry_path).unwrap();

                    let modified = file_metadata
                        .modified()
                        .unwrap()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    cache.insert(
                        entry_path.clone(),
                        crate::infrastructure::cache::metadata_cache::CachedEntry {
                            metadata: crate::domain::metadata::Metadata {
                                title: Some(document.title.clone()),
                                author: document.author.clone(),
                            },
                            modified,
                            size: file_metadata.len(),
                        },
                    );
                }

                documents.push(document);
            }
        }
    }

    println!("Scan completed in {:?}", start.elapsed());

    documents
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .map(|ext| {
            let ext = ext.to_string_lossy().to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}
