use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::domain::reading_status::{ReadingStatus, ReadingStatusEntry};

// Separate persistence layer so we can wipe the metadata cache without
// losing the user's reading progress.
#[derive(Debug, Default)]
pub struct ReadingStatusStore {
    entries: HashMap<String, ReadingStatusEntry>,
}

impl ReadingStatusStore {
    pub fn get(&self, path: &Path) -> ReadingStatus {
        let key = path.to_string_lossy().to_string();
        self.entries
            .get(&key)
            .map(|e| e.status.clone())
            .unwrap_or(ReadingStatus::Unread)
    }

    pub fn mark_started(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());

        self.entries.insert(
            key,
            ReadingStatusEntry {
                status: ReadingStatus::CurrentlyReading,
                last_opened: now,
            },
        );
    }

    pub fn mark_completed(&mut self, path: &Path) {
        let key = path.to_string_lossy().to_string();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());

        self.entries.insert(
            key,
            ReadingStatusEntry {
                status: ReadingStatus::Completed,
                last_opened: now,
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
