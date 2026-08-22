// Walks the library directory tree and builds a list of Document
// objects. Metadata is pulled from the cache if available, otherwise
// extracted from the file and saved for next time.

use crate::domain::document::Document;
use crate::domain::metadata::Metadata;
use crate::infrastructure::cache::metadata_cache::{CachedEntry, MetadataCache};
use crate::infrastructure::metadata::extractor::enrich_document;

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

const SUPPORTED_EXTENSIONS: &[&str] = &["pdf", "epub"];

// Uses an explicit stack instead of recursion so we don't risk
// blowing the stack on deeply nested directory trees.
pub fn scan_library(path: &Path, cache: &mut MetadataCache) -> Vec<Document> {
    let start = Instant::now();
    let mut documents = Vec::new();
    let mut seen_paths = HashSet::new();

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

                seen_paths.insert(entry_path.clone());
                let mut document = Document::from_path(entry_path.clone());

                if cache.is_fresh(&entry_path) {
                    // Cache hit and file hasn't changed — use cached metadata
                    let metadata = cache.get(&entry_path).unwrap();
                    document.apply_metadata(
                        metadata.metadata.title.clone(),
                        metadata.metadata.author.clone(),
                    );
                } else {
                    // Cache miss or file changed — (re-)extract metadata
                    enrich_document(&mut document);

                    let file_metadata = match fs::metadata(&entry_path) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("  Skipping {}: {}", entry_path.display(), e);
                            continue;
                        }
                    };

                    let modified = match file_metadata.modified() {
                        Ok(t) => t
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                        Err(_) => 0,
                    };

                    cache.insert(
                        entry_path.clone(),
                        CachedEntry {
                            metadata: Metadata {
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

    // Drop cache entries for files that no longer exist
    cache.prune(&seen_paths);

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
