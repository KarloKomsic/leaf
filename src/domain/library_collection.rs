use crate::domain::document::Document;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use strsim::levenshtein;

// Builds a search index over title+author so we can fuzzy-match even
// with typos or acronyms (e.g. "tlotr" → "The Lord of the Rings").
const STOP_WORDS: &[&str] = &["and", "the", "of", "to", "a", "in", "for", "on"];

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
}

impl Library {
    pub fn new(documents: Vec<Document>) -> Self {
        Self {
            index: Self::build_index(&documents),
            prefix_index: Self::build_prefix_index(&documents),
            documents,
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult<'_>> {
        let normalized_query = Self::normalize(query);

        if normalized_query.len() <= 2 {
            return Vec::new();
        }

        // 1. Acronym search (highest priority)
        let acronym_results = self.search_acronyms(&normalized_query);

        if !acronym_results.is_empty() {
            return acronym_results;
        }

        // 2. Indexed search
        let indexed_results = self.search_indexed(query);

        if !indexed_results.is_empty() {
            return indexed_results;
        }

        // 3. Fallback search
        self.search_fallback(query)
    }

    pub fn suggest(&self, query: &str) -> Vec<&Document> {
        let normalized_query = query.to_lowercase();

        self.prefix_index
            .get(&normalized_query)
            .map(|indices| {
                indices
                    .iter()
                    .take(5)
                    .map(|&i| &self.documents[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn random(&self) -> Option<&Document> {
        let mut rng = thread_rng();
        self.documents.choose(&mut rng)
    }

    // =========================================================
    // SEARCH IMPLEMENTATIONS
    // =========================================================

    fn search_acronyms(&self, query: &str) -> Vec<SearchResult<'_>> {
        let mut results = Vec::new();

        for doc in &self.documents {
            let title = doc.normalized_title().to_lowercase();
            let author = doc.author.clone().unwrap_or_default().to_lowercase();

            let title_acronym = Self::acronym(&title);
            let author_acronym = Self::acronym(&author);

            if title_acronym == query || author_acronym == query {
                results.push(SearchResult {
                    document: doc,
                    score: 200,
                    match_type: MatchType::Acronym,
                });
            }
        }

        results
    }

    fn search_indexed(&self, query: &str) -> Vec<SearchResult<'_>> {
        let words = Self::query_words(query);

        let mut matched_indices: Option<HashSet<usize>> = None;

        for word in words {
            let current_indices = self.indices_for_word(&word);

            if current_indices.is_empty() {
                return Vec::new();
            }

            let current_set: HashSet<usize> = current_indices.into_iter().collect();

            matched_indices = Some(match matched_indices {
                None => current_set,
                Some(previous) => previous.intersection(&current_set).copied().collect(),
            });
        }

        let Some(indices) = matched_indices else {
            return Vec::new();
        };

        let mut results: Vec<SearchResult<'_>> = indices
            .into_iter()
            .map(|index| self.create_search_result(index, query))
            .collect();

        Self::sort_results(&mut results);

        results
    }

    fn search_fallback(&self, query: &str) -> Vec<SearchResult<'_>> {
        let mut results = Vec::new();

        for (index, _) in self.documents.iter().enumerate() {
            let result = self.create_search_result(index, query);

            if result.score > 0 {
                results.push(result);
            }
        }

        Self::sort_results(&mut results);

        results
    }

    // =========================================================
    // SEARCH HELPERS
    // =========================================================

    fn create_search_result(&self, index: usize, query: &str) -> SearchResult<'_> {
        let document = &self.documents[index];

        let score = self.score_document(document, query);

        let match_type = if document.normalized_title().to_lowercase() == query.to_lowercase() {
            MatchType::Exact
        } else {
            MatchType::Fuzzy
        };

        SearchResult {
            document,
            score,
            match_type,
        }
    }

    fn indices_for_word(&self, word: &str) -> Vec<usize> {
        if let Some(indices) = self.index.get(word) {
            return indices.clone();
        }

        if let Some(similar_word) = self.find_similar_word(word) {
            return self.index.get(&similar_word).cloned().unwrap_or_default();
        }

        Vec::new()
    }

    fn query_words(query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    fn sort_results(results: &mut Vec<SearchResult<'_>>) {
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.document.title.cmp(&b.document.title))
        });
    }

    // =========================================================
    // SCORING
    // =========================================================

    fn score_document(&self, doc: &Document, query: &str) -> usize {
        let mut score = 0;

        let title = doc.normalized_title().to_lowercase();
        let author = doc.author.clone().unwrap_or_default().to_lowercase();

        let query = query.to_lowercase();

        if title == query {
            score += 100;
        }

        if title.starts_with(&query) {
            score += 50;
        }

        if title.split_whitespace().any(|word| word == query) {
            score += 40;
        }

        if title.contains(&query) {
            score += 25;
        }

        if author.contains(&query) {
            score += 10;
        }

        score
    }

    // =========================================================
    // TEXT UTILITIES
    // =========================================================

    fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }

    fn acronym(text: &str) -> String {
        text.split_whitespace()
            .filter(|word| !STOP_WORDS.contains(word))
            .filter_map(|word| word.chars().next())
            .collect::<String>()
            .to_lowercase()
    }

    // =========================================================
    // INDEX BUILDING
    // =========================================================

    fn build_index(documents: &[Document]) -> HashMap<String, Vec<usize>> {
        let mut index = HashMap::new();

        for (i, doc) in documents.iter().enumerate() {
            let combined_text = format!(
                "{} {}",
                doc.normalized_title(),
                doc.author.as_deref().unwrap_or("").to_lowercase()
            );

            for word in combined_text.split_whitespace() {
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

        for (i, doc) in documents.iter().enumerate() {
            for word in doc.normalized_title().to_lowercase().split_whitespace() {
                if word.len() < 3 || STOP_WORDS.contains(&word) {
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

    // =========================================================
    // FUZZY SEARCH
    // =========================================================

    fn find_similar_word(&self, query: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        let max_distance = Self::max_levenshtein_distance(query);

        for word in self.index.keys() {
            let length_difference = (word.len() as isize - query.len() as isize).abs();

            if length_difference > max_distance as isize {
                continue;
            }

            let distance = levenshtein(query, word);

            if distance <= max_distance && distance < best_distance {
                best_distance = distance;
                best_match = Some(word.clone());
            }
        }

        best_match
    }

    // Levenshtein distance lets us catch typos like "atomik" → "atomic".
    // Tolerance scales with query length so short words aren't over-fuzzed.
    fn max_levenshtein_distance(query: &str) -> usize {
        match query.len() {
            0..=4 => 1,
            5..=8 => 2,
            _ => 3,
        }
    }
}
