// Caches extracted title and author per file so we don't have to
// re-read every PDF and EPUB on every launch. The cache is keyed by
// absolute file path, and each entry stores the file's modified time
// and size so we can detect when a file has changed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::metadata::Metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    pub metadata: Metadata,
    pub modified: u64,
    pub size: u64,
}

// Old-format numeric keys (from an earlier version that used
// {size}-{modified} hashes) are silently dropped during loading.
#[derive(Debug, Default)]
pub struct MetadataCache {
    entries: HashMap<String, CachedEntry>,
}

impl MetadataCache {
    pub fn load(cache_path: &Path) -> Self {
        let contents = fs::read_to_string(cache_path);

        if let Ok(json) = contents {
            if let Ok(entries) = serde_json::from_str::<HashMap<String, CachedEntry>>(&json) {
                let entries: HashMap<String, CachedEntry> = entries
                    .into_iter()
                    .filter(|(key, _)| {
                        let has_path = key.contains('/');
                        if !has_path {
                            println!("  Dropping stale cache entry (old format): {}", key);
                        }
                        has_path
                    })
                    .collect();

                if entries.is_empty() {
                    return Self::default();
                }

                return Self { entries };
            }
        }

        Self::default()
    }

    pub fn save(&self, cache_path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(cache_path, json);
        }
    }

    pub fn get(&self, path: &Path) -> Option<&CachedEntry> {
        let key = path.to_string_lossy().to_string();
        self.entries.get(&key)
    }

    pub fn insert(&mut self, path: PathBuf, entry: CachedEntry) {
        let key = path.to_string_lossy().to_string();
        self.entries.insert(key, entry);
    }

    pub fn remove(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        self.entries.remove(&key);
    }

    /// Checks whether the cached entry for `path` still matches the
    /// file on disk by comparing stored modified time and size against
    /// the current file metadata. Returns false if the file is gone
    /// or has changed since the cache was written.
    pub fn is_fresh(&self, path: &Path) -> bool {
        let key = path.to_string_lossy().to_string();
        let entry = match self.entries.get(&key) {
            Some(e) => e,
            None => return false,
        };

        let meta = match fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        let current_modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let current_size = meta.len();

        entry.modified == current_modified && entry.size == current_size
    }

    /// Removes cache entries for files that are not in `seen_paths`.
    /// Call this after a full scan to drop orphaned entries (deleted
    /// or moved books).
    pub fn prune(&mut self, seen_paths: &HashSet<PathBuf>) {
        self.entries.retain(|key, _| {
            let path = Path::new(key);
            let keep = seen_paths.contains(path);
            if !keep {
                println!("  Pruning orphaned cache entry: {}", key);
            }
            keep
        });
    }
}
