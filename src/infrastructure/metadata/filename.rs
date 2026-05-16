use std::path::Path;

use crate::domain::document::Document;

pub fn extract(path: &Path) -> Option<Document> {
    let filename = path.file_name()?.to_string_lossy().to_string();

    let cleaned = clean_title(&filename);

    let author = extract_author(&cleaned);

    let title = cleaned
        .split('(')
        .next()
        .unwrap_or(&cleaned)
        .trim()
        .to_string();

    Some(Document {
        title,
        author,
        path: path.to_path_buf(),
    })
}

fn clean_title(filename: &str) -> String {
    let mut title = filename.replace(".pdf", "");

    title = title.replace('_', " ");
    title = title.replace('.', " ");
    title = title.replace('-', " ");

    title = title.split_whitespace().collect::<Vec<_>>().join(" ");

    title.trim().to_string()
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
