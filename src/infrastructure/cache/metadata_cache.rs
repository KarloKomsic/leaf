use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
        let contents = fs::read_to_string(cache_path);

        if let Ok(json) = contents {
            if let Ok(entries) = serde_json::from_str(&json) {
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
