// Caches extracted title and author per file so we don't have to
// re-read every PDF and EPUB on every launch. The cache is keyed by
// absolute file path, and each entry stores the file's modified time
// and size so we can detect when a file has changed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

}
