use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub author: Option<String>,
    pub path: PathBuf,
}

impl Document {
    pub fn from_path(path: PathBuf) -> Self {
        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let cleaned = Self::clean_filename(&filename);

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

    fn clean_filename(filename: &str) -> String {
        let mut title = filename.to_string();

        title = title.replace('_', " ");
        title = title.replace('.', " ");
        title = title.replace('-', " ");

        title
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn extract_author(text: &str) -> Option<String> {
        if let Some(start) = text.find('(') {
            if let Some(end) = text[start..].find(')') {
                let end = start + end;

                let author = text[start + 1..end].trim();

                if author.len() > 2 {
                    return Some(author.to_string());
                }
            }
        }

        None
    }

    // Optional metadata enrichment
    pub fn apply_metadata(&mut self, title: Option<String>, author: Option<String>) {
        if let Some(title) = title {
            if !title.trim().is_empty() {
                self.title = title;
            }
        }

        if let Some(author) = author {
            if !author.trim().is_empty() {
                self.author = Some(author);
            }
        }
    }

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
}
