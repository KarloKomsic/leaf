use std::path::PathBuf;

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub author: Option<String>,
    pub path: PathBuf,
}

impl Document {
    // Normalize titles for searching
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
