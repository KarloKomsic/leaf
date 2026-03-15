use crate::domain::document::Document;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strsim::levenshtein;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
    index: HashMap<String, Vec<usize>>,
}

impl Library {
    pub fn new(documents: Vec<Document>) -> Self {
        let index = Self::build_index(&documents);

        Self { documents, index }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    // Searching function
    pub fn search(&self, query: &str) -> Vec<&Document> {
        // Parse word
        let words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        // Have a vector that may or may not have any values
        let mut results: Option<Vec<usize>> = None;

        // Go thru each word in query
        for word in words {
            let indices = if let Some(indices) = self.index.get(&word) {
                indices.clone()
            } else if let Some(similar) = self.find_similar_word(&word) {
                self.index.get(&similar).cloned().unwrap_or_default()
            } else {
                return Vec::new();
            };

            results = Some(match results {
                None => indices,
                Some(prev) => prev.into_iter().filter(|i| indices.contains(i)).collect(),
            });
        }

        // Check if result matches
        match results {
            Some(indices) => {
                let mut scored: Vec<(&Document, usize)> = indices
                    .iter()
                    .map(|&i| {
                        let doc = &self.documents[i];
                        let score = self.score_document(doc, query);
                        (doc, score)
                    })
                    .collect();

                scored.sort_by(|a, b| b.1.cmp(&a.1));

                scored.into_iter().map(|(doc, _)| doc).collect()
            }

            None => Vec::new(),
        }
    }

    // Determines points for document to base search results on
    fn score_document(&self, doc: &Document, query: &str) -> usize {
        let mut score = 0;

        let title = doc.normalized_title();
        let author = doc.author.clone().unwrap_or_default().to_lowercase();

        for word in query.to_lowercase().split_whitespace() {
            if title.contains(word) {
                score += 3;
            }

            if author.contains(word) {
                score += 2;
            }
        }

        score
    }

    // Function for randomizing book choice
    pub fn random(&self) -> Option<&Document> {
        let mut rng = thread_rng();
        self.documents.choose(&mut rng)
    }

    // Indexing for hash maps
    fn build_index(documents: &[Document]) -> HashMap<String, Vec<usize>> {
        let mut index = HashMap::new();

        for (i, doc) in documents.iter().enumerate() {
            let combined = format!(
                "{} {}",
                doc.normalized_title(),
                doc.author.as_deref().unwrap_or("").to_lowercase()
            );

            for word in combined.split_whitespace() {
                index
                    .entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        index
    }

    // Fuzzy finding for similar word
    // NOTE: This function uses the Levenshtein distance to calculate similarities:
    // https://en.wikipedia.org/wiki/Levenshtein_distance
    // TLDR: checks if distance between result and search query is less than or equal to certain
    // metric
    // If it is, we accept it
    fn find_similar_word(&self, query: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for word in self.index.keys() {
            const MAX_DISTANCE: usize = 2;
            if (word.len() as isize - query.len() as isize).abs() > 2 {
                continue;
            }

            let distance = levenshtein(query, word);

            if distance < best_distance && distance <= MAX_DISTANCE {
                best_distance = distance;
                best_match = Some(word.clone());
            }
        }

        best_match
    }
}
