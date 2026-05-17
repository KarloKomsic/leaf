use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::domain::metadata::Metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    pub metadata: Metadata,

    // Used to detect file changes
    pub modified: u64,
    pub size: u64,
}

#[derive(Debug, Default)]
pub struct MetadataCache {
    entries: HashMap<String, CachedEntry>,
}

impl MetadataCache {
    pub fn load(cache_path: &Path) -> Self {
        println!("Loading cache from: {:?}", cache_path);

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

                println!("  Loaded {} valid cache entries", entries.len());

                return Self { entries };
            }
        }

        Self::default()
    }

    pub fn save(&self, cache_path: &Path) {
        println!("Saving cache to: {:?}", cache_path);

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

    pub fn is_valid(&self, path: &Path) -> bool {
        let cached = match self.get(path) {
            Some(entry) => entry,
            None => return false,
        };

        let metadata = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return false,
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        let size = metadata.len();

        cached.modified == modified && cached.size == size
    }
}
