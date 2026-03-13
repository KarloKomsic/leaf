use std::path::PathBuf;

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub path: PathBuf,
}

impl Document {
    // Clean up titles in the names
    fn clean_title(filename: &str) -> String {
        // Replace _ with spaces
        let mut title = filename.replace('_', " ");

        // Replace the .pdf with empty stuff so it makes it cleaner
        title = title.replace(".pdf", "");

        title
    }

    pub fn new(path: PathBuf) -> Self {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        let title = Self::clean_title(&filename);

        Self { title, path }
    }
}
