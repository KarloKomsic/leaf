use std::path::PathBuf;

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub path: PathBuf,
}
