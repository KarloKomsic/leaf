use crate::domain::{
    self,
    document::{self, Document},
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
}

impl Library {
    pub fn new(mut documents: Vec<Document>) -> Self {
        let mut seen = HashSet::new();

        // Remove duplicates
        documents.retain(|doc| {
            let normalized = doc.normalized_title();
            seen.insert(normalized)
        });

        // Take two documents
        // Compare their titles
        // Sort alphabetically
        // Ignore capitalization
        documents.sort_by_key(|doc| doc.title.to_lowercase());

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

    pub fn random(&self) -> Option<&Document> {
        let mut rng = thread_rng();
        self.documents.choose(&mut rng)
    }
}
