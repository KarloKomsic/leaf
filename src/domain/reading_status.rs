use serde::{Deserialize, Serialize};

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
