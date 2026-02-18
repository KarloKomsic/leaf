#[derive(Debug)]
pub enum LibraryState {
    NotConfigured,
    Invalid,
    Ready,
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
