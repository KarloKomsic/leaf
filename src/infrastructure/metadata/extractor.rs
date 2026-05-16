use std::path::Path;

use crate::domain::document::Document;

pub fn extract_document(path: &Path) -> Option<Document> {
    crate::infrastructure::metadata::filename::extract(path)
}
