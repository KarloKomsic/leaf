use crate::domain::{
    self,
    document::{self, Document},
};

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
}

impl Library {
    pub fn new(mut documents: Vec<Document>) -> Self {
        // Take two documents
        // Compare their titles
        // Sort alphabetically
        // Ignore capitalization
        documents.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

        // Returns the vector/list of documents
        Self { documents }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn search(&self, query: &str) -> Vec<&Document> {
        let query = query.to_lowercase();

        self.documents
            .iter() // Iterate through docs
            .filter(|doc| doc.title.to_lowercase().contains(&query)) // Filter titles with query
            .collect() // Collect results
    }
}
