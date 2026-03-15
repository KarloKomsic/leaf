use std::path::PathBuf;

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub author: Option<String>,
    pub path: PathBuf,
}

impl Document {
    // Clean up titles in the names
    fn clean_title(filename: &str) -> String {
        // Replace the .pdf with empty stuff so it makes it cleaner
        let mut title = filename.replace(".pdf", "");

        // Replace _, ., and - with spaces
        title = title.replace('_', " ");
        title = title.replace('.', " ");
        title = title.replace('-', " ");

        // Collapse multiple spaces into one
        title = title.split_whitespace().collect::<Vec<_>>().join(" ");

        title.trim().to_string()
    }

    // Normalize titles by removing stuff like (Z-Library) or (Anna's Archive)
    pub fn normalized_title(&self) -> String {
        let mut title = self.title.to_lowercase();

        // Remove parenthesis and their contents
        while let Some(start) = title.find('(') {
            if let Some(end) = title[start..].find(')') {
                let end = start + end;
                title.replace_range(start..=end, "");
            } else {
                break;
            }
        }

        title = title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c.is_whitespace() {
                    c
                } else {
                    ' '
                }
            })
            .collect();

        title.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    pub fn new(path: PathBuf) -> Self {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        let cleaned = Self::clean_title(&filename);
        let author = Self::extract_author(&cleaned);

        let title = cleaned
            .split('(')
            .next()
            .unwrap_or(&cleaned)
            .trim()
            .to_string();

        Self {
            title,
            author,
            path,
        }
    }

    fn extract_author(title: &str) -> Option<String> {
        if let Some(start) = title.find('(') {
            if let Some(end) = title[start..].find(')') {
                let author = &title[start + 1..start + end];

                if author.len() > 2 && !author.to_lowercase().contains("library") {
                    return Some(author.trim().to_string());
                }
            }
        }

        None
    }
}
