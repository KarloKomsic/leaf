use serde::{Deserialize, Serialize};

// Raw metadata extracted from a book file's internals.
// Both fields can be empty because not every PDF or EPUB
// bothers to include them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
}
