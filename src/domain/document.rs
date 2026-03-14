use std::path::PathBuf;

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub path: PathBuf,
}

impl Document {
    // Clean up titles in the names
    fn clean_title(filename: &str) -> String {
        // Replace the .pdf with empty stuff so it makes it cleaner
        let mut title = filename.replace(".pdf", "");
        // Replace _ with spaces
        title = title.replace('_', " ");
        // Replace "." with space
        title = title.replace('.', " ");
        // Replace "-" with space
        title = title.replace('-', " ");

        title
    }

    // Normalize titles by removing stuff like (Z-Library) or (Anna's Archive)
    pub fn normalized_title(&self) -> String {
        let mut title = self.title.to_lowercase();

        // Remove parenthesis and their contents
        while let Some(start) = title.find('(') {
            if let Some(end) = title[start..].find(')') {
                title.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }

        title.trim().to_string()
    }

    pub fn new(path: PathBuf) -> Self {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        let title = Self::clean_title(&filename);

        Self { title, path }
    }
}
