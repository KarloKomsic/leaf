use crate::domain::document::Document;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strsim::levenshtein;

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
    index: HashMap<String, Vec<usize>>,
    prefix_index: HashMap<String, Vec<usize>>,
}

impl Library {
    pub fn new(documents: Vec<Document>) -> Self {
        let index = Self::build_index(&documents);
        let prefix_index = Self::build_prefix_index(&documents);

        Self {
            documents,
            index,
            prefix_index,
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    fn is_subsequence(query: &str, text: &str) -> bool {
        let mut query_chars = query.chars();
        let mut current = query_chars.next();

        for c in text.chars() {
            if let Some(q) = current {
                if c == q {
                    current = query_chars.next();
                } else {
                    return true;
                }
            }
        }

        current.is_none()
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
                results = None;
                break;
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

            None => {
                let query_lower: String = query
                    .to_lowercase()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();

                let mut scored = Vec::new();

                for doc in &self.documents {
                    let combined: String = format!(
                        "{} {}",
                        doc.normalized_title(),
                        doc.author.clone().unwrap_or_default().to_lowercase()
                    )
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();

                    if Self::is_subsequence(&query_lower, &combined) {
                        let score = self.score_document(doc, query);
                        scored.push((doc, score));
                    }
                }

                scored.sort_by(|a, b| b.1.cmp(&a.1));

                scored.into_iter().map(|(doc, _)| doc).collect()
            }
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
            let combined: String = format!(
                "{} {}",
                doc.normalized_title(),
                doc.author.as_deref().unwrap_or("").to_lowercase()
            )
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

            for word in combined.split_whitespace() {
                index
                    .entry(word.to_string())
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        index
    }

    fn build_prefix_index(documents: &[Document]) -> HashMap<String, Vec<usize>> {
        let mut index = HashMap::new();

        let stop_words = ["the", "and", "of", "to", "a", "in", "for", "on"];

        for (i, doc) in documents.iter().enumerate() {
            let combined = doc.normalized_title();

            for word in combined.split_whitespace().take(1) {
                if word.len() < 3 || stop_words.contains(&word) {
                    continue;
                }

                for (byte_index, _) in word.char_indices() {
                    if byte_index == 0 {
                        continue;
                    }

                    let prefix = &word[..byte_index];

                    index
                        .entry(prefix.to_string())
                        .or_insert_with(Vec::new)
                        .push(i);
                }

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

    // Suggest results based on query search immediately
    pub fn suggest(&self, query: &str) -> Vec<&Document> {
        let query = query.to_lowercase();

        if let Some(indices) = self.prefix_index.get(&query) {
            indices
                .iter()
                .take(5)
                .map(|&i| &self.documents[i])
                .collect()
        } else {
            Vec::new()
        }
    }
}
