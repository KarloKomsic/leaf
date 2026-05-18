use serde::{Deserialize, Serialize};

// A book can be in one of three states: not touched yet, in progress,
// or finished. These are kept in a separate file from the metadata
// cache so re-scanning the library folder doesn't reset your place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReadingStatus {
    Unread,
    CurrentlyReading,
    Completed,
}

impl Default for ReadingStatus {
    fn default() -> Self {
        Self::Unread
    }
}

// Each book gets an entry that stores its current status and the
// last time it was opened (as a unix timestamp). The timestamp is
// mainly useful for sorting by recently opened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStatusEntry {
    pub status: ReadingStatus,
    pub last_opened: Option<u64>,
}

impl ReadingStatusEntry {
    pub fn new() -> Self {
        Self {
            status: ReadingStatus::Unread,
            last_opened: None,
        }
    }
}

impl Default for ReadingStatusEntry {
    fn default() -> Self {
        Self::new()
    }
}
