use crate::domain::document::Document;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strsim::levenshtein;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
    index: HashMap<String, Vec<usize>>,
    prefix_index: HashMap<String, Vec<usize>>,
}

#[derive(Debug)]
pub struct SearchResult<'a> {
    pub document: &'a Document,
    pub score: usize,
    pub match_type: MatchType,
}

#[derive(Debug)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Acronym,
    Subsequence,
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

    fn subsequence_score(query: &str, text: &str) -> usize {
        let mut score = 0;
        let mut query_chars = query.chars();

        let mut current = query_chars.next();

        for c in text.chars() {
            if let Some(q) = current {
                if q == c {
                    score += 1;
                    current = query_chars.next();
                }
            } else {
                break;
            }
        }

        score
    }

    fn acronym(text: &str) -> String {
        let stop_words = ["and", "the", "of", "to", "a", "in"];

        text.split_whitespace()
            .filter(|word| !stop_words.contains(word))
            .filter_map(|word| word.chars().next())
            .collect::<String>()
            .to_lowercase()
    }

    // Searching function
    pub fn search(&self, query: &str) -> Vec<SearchResult<'_>> {
        let query_lower = query.to_lowercase();

        let compact_query: String = query_lower.chars().filter(|c| !c.is_whitespace()).collect();

        let mut acronym_results = Vec::new();

        for doc in &self.documents {
            let title = doc.normalized_title().to_lowercase();
            let author = doc.author.clone().unwrap_or_default().to_lowercase();

            let title_acronym = Self::acronym(&title);
            let author_acronym = Self::acronym(&author);

            if title_acronym == compact_query || author_acronym == compact_query {
                acronym_results.push(SearchResult {
                    document: doc,
                    score: 200,
                    match_type: MatchType::Acronym,
                });
            }
        }

        if !acronym_results.is_empty() {
            return acronym_results;
        }

        if compact_query.len() <= 2 {
            return Vec::new();
        }

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
                let mut scored: Vec<SearchResult<'_>> = indices
                    .iter()
                    .map(|&i| {
                        let doc = &self.documents[i];
                        let score = self.score_document(doc, query);

                        let match_type = if doc.normalized_title().to_lowercase() == query_lower {
                            MatchType::Exact
                        } else {
                            MatchType::Fuzzy
                        };

                        SearchResult {
                            document: doc,
                            score,
                            match_type,
                        }
                    })
                    .collect();

                scored.sort_by(|a, b| b.score.cmp(&a.score));

                scored
            }

            None => {
                let query_lower: String = query
                    .to_lowercase()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();

                let mut scored = Vec::new();

                for doc in &self.documents {
                    let title = doc.normalized_title().to_lowercase();
                    let author = doc.author.clone().unwrap_or_default().to_lowercase();
                    let combined = format!("{} {}", title, author);

                    let compact_combined: String =
                        combined.chars().filter(|c| !c.is_whitespace()).collect();

                    let subseq_score = Self::subsequence_score(&compact_query, &compact_combined);

                    if subseq_score < 2 {
                        continue;
                    }

                    let similarity = subseq_score as f32 / compact_query.len() as f32;
                    if similarity >= 0.95 {
                        let score = self.score_document(doc, query) + subseq_score;

                        scored.push(SearchResult {
                            document: doc,
                            score,
                            match_type: MatchType::Subsequence,
                        });
                    }
                }

                scored.sort_by(|a, b| b.score.cmp(&a.score));

                scored
            }
        }
    }

    // Determines points for document to base search results on
    fn score_document(&self, doc: &Document, query: &str) -> usize {
        let mut score = 0;

        let title = doc.normalized_title().to_lowercase();
        let author = doc.author.clone().unwrap_or_default().to_lowercase();

        let query_lower = query.to_lowercase();

        if title == query_lower {
            score += 100;
        }

        if title.starts_with(&query_lower) {
            score += 50;
        }

        if title.contains(&query_lower) {
            score += 25;
        }

        if author.contains(&query_lower) {
            score += 10;
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
                let entry = index.entry(word.to_string()).or_insert_with(Vec::new);

                if !entry.contains(&i) {
                    entry.push(i);
                }
            }
        }

        index
    }

    fn build_prefix_index(documents: &[Document]) -> HashMap<String, Vec<usize>> {
        let mut index = HashMap::new();

        let stop_words = ["the", "and", "of", "to", "a", "in", "for", "on"];

        for (i, doc) in documents.iter().enumerate() {
            let combined = doc.normalized_title();

            for word in combined.to_lowercase().split_whitespace() {
                if word.len() < 3 || stop_words.contains(&word) {
                    continue;
                }

                for (byte_index, _) in word.char_indices() {
                    if byte_index == 0 {
                        continue;
                    }

                    let prefix = &word[..byte_index];

                    let entry = index.entry(prefix.to_string()).or_insert_with(Vec::new);

                    if !entry.contains(&i) {
                        entry.push(i);
                    }
                }

                let entry = index.entry(word.to_string()).or_insert_with(Vec::new);

                if !entry.contains(&i) {
                    entry.push(i);
                }
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
            let max_distance = match query.len() {
                0..=4 => 1,
                5..=8 => 2,
                _ => 3,
            };

            if (word.len() as isize - query.len() as isize).abs() > max_distance as isize {
                continue;
            }

            let distance = levenshtein(query, word);

            if distance < best_distance && distance <= max_distance {
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
