#[derive(Debug)]
pub enum LibraryState {
    NotConfigured,      // First launch, or removed directory via settings
    Invalid,            // Not valid library path
    Ready(LibraryPath), // Valid library path
}

#[derive(Debug, Clone)]
pub struct LibraryPath(String);

// Implementing methods for LibraryPath
impl LibraryPath {
    pub fn new(path: String) -> Self {
        Self(path) // creates a new instance of LibraryPath
    }

    pub fn as_str(&self) -> &str {
        &self.0 // returns reference to string inside struct
    }
}

// Implementing methods for LibraryState
impl LibraryState {
    // Checking library state
    pub fn is_ready(&self) -> bool {
        // If it is ready, return true
        // NOTE: (_) = wildcard, tells Rust whether the variant is actually true, ignoring
        // everything inside it
        // Otherwise, false
        matches!(self, LibraryState::Ready(_))
    }
}
