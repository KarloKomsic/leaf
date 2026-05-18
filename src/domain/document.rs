use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub author: Option<String>,
    pub path: PathBuf,
}

impl Document {
    // Strips parenthetical noise, punctuation, and lowercases for search matching
    pub fn normalized_title(&self) -> String {
        let mut title = self.title.to_lowercase();

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

    // Fallback when no metadata is available — just use the filename
    pub fn from_path(path: PathBuf) -> Self {
        let filename = path.file_stem().unwrap_or_default().to_string_lossy();

        let title = filename
            .replace('_', " ")
            .replace('.', " ")
            .replace('-', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            title,
            author: None,
            path,
        }
    }

    pub fn apply_metadata(&mut self, title: Option<String>, author: Option<String>) {
        if let Some(title) = title {
            self.title = title;
        }

        if let Some(author) = author {
            self.author = Some(author);
        }
    }
}
