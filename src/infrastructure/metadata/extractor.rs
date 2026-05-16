use std::path::Path;

use crate::domain::document::Document;

pub fn extract_document(path: &Path) -> Option<Document> {
    let extension = path.extension()?.to_string_lossy().to_lowercase();

    match extension.as_str() {
        "pdf" => extract_pdf(path),

        _ => None,
    }
}

fn extract_pdf(path: &Path) -> Option<Document> {
    println!("Scanning PDF: {:?}", path);

    if let Some((title, author)) = crate::infrastructure::metadata::pdf::extract_title_author(path)
    {
        println!("Metadata success: {}", title);

        return Some(Document {
            title,
            author,
            path: path.to_path_buf(),
        });
    }

    println!("Fallback filename parsing");
}
