#[derive(Debug)]
pub enum LibraryState {
    NotConfigured,
    Invalid,
    Ready(LibraryPath),
}

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

#[allow(dead_code)]
impl LibraryState {
    pub fn is_ready(&self) -> bool {
        matches!(self, LibraryState::Ready(_))
    }
}
