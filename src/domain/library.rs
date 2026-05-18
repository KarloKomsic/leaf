// Tracks whether the user has set up a library directory yet and
// whether the saved path still points to a real folder on disk.
#[derive(Debug)]
pub enum LibraryState {
    NotConfigured,
    Invalid,
    Ready(LibraryPath),
}

// Wraps a directory path string so we can give it semantic meaning
// instead of passing raw Strings around.
#[derive(Debug, Clone)]
pub struct LibraryPath(String);

impl LibraryPath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}


