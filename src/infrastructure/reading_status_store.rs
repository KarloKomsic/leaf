// Persists reading progress (unread, in progress, completed) in its
// own JSON file so re-scanning the library or clearing the metadata
// cache does not reset the user's place.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::domain::reading_status::{ReadingStatus, ReadingStatusEntry};

#[derive(Debug, Default)]
pub struct ReadingStatusStore {
    entries: HashMap<String, ReadingStatusEntry>,
}

impl ReadingStatusStore {
    // Timestamps use seconds-since-epoch rather than any human-readable
    // format because they're only compared (e.g. "was this opened more
    // recently than that?") and never displayed to the user directly,
    // which we can use to give the current time which we can store
    // as last opened or completed.
    fn now() -> Option<u64> {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
    }

    pub fn get(&self, path: &Path) -> ReadingStatus {
        let key = path.to_string_lossy().to_string();
        self.entries
            .get(&key)
            .map(|e| e.status.clone())
            .unwrap_or(ReadingStatus::Unread)
    }

    pub fn mark_started(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        self.entries.insert(
            key,
            ReadingStatusEntry {
                status: ReadingStatus::CurrentlyReading,
                last_opened: Self::now(),
            },
        );
    }

    pub fn mark_completed(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        self.entries.insert(
            key,
            ReadingStatusEntry {
                status: ReadingStatus::Completed,
                last_opened: Self::now(),
            },
        );
    }

    pub fn mark_uncompleted(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        self.entries.insert(
            key,
            ReadingStatusEntry {
                status: ReadingStatus::Unread,
                last_opened: None,
            },
        );
    }

    pub fn load(path: &Path) -> Self {
        let contents = fs::read_to_string(path);
        if let Ok(json) = contents {
            if let Ok(entries) = serde_json::from_str(&json) {
                return Self { entries };
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(path, json);
        }
    }
}
