//! Inverted Index implementation for full-text search
//!
//! Provides an in-memory inverted index for fast text search.

use std::collections::{HashMap, HashSet};

use super::tokenizer::MultiLanguageTokenizer;

/// Inverted index for full-text search
///
/// Maps tokens to document IDs, enabling fast search operations.
pub struct InvertedIndex {
    index: HashMap<String, HashSet<u64>>,
    tokenizer: MultiLanguageTokenizer,
    doc_count: u64,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            tokenizer: MultiLanguageTokenizer::new(),
            doc_count: 0,
        }
    }

    /// Add a document to the index
    pub fn add_document(&mut self, doc_id: u64, text: &str) {
        let tokens = self.tokenizer.tokenize(text);
        for token in tokens {
            self.index.entry(token).or_default().insert(doc_id);
        }
        self.doc_count += 1;
    }

    /// Remove a document from the index
    pub fn remove_document(&mut self, doc_id: u64, text: &str) {
        let tokens = self.tokenizer.tokenize(text);
        for token in tokens {
            if let Some(doc_ids) = self.index.get_mut(&token) {
                doc_ids.remove(&doc_id);
                if doc_ids.is_empty() {
                    self.index.remove(&token);
                }
            }
        }
        self.doc_count = self.doc_count.saturating_sub(1);
    }

    /// Search for documents matching all query tokens (AND search)
    pub fn search(&self, query: &str) -> Vec<u64> {
        let tokens = self.tokenizer.tokenize(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut result: Option<HashSet<u64>> = None;

        for token in &tokens {
            if let Some(doc_ids) = self.index.get(token) {
                match &mut result {
                    None => result = Some(doc_ids.clone()),
                    Some(set) => {
                        *set = set.intersection(doc_ids).copied().collect();
                    }
                }
            } else {
                return Vec::new();
            }
        }

        result
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }

    /// Search with result limit
    pub fn search_with_limit(&self, query: &str, limit: usize) -> Vec<u64> {
        let mut results = self.search(query);
        results.truncate(limit);
        results
    }

    /// Fuzzy search with maximum edit distance
    pub fn fuzzy_search(&self, query: &str, max_distance: usize) -> Vec<u64> {
        let tokens = self.tokenizer.tokenize(query);
        let mut results: HashMap<u64, usize> = HashMap::new();

        for term in self.index.keys() {
            for token in &tokens {
                let dist = levenshtein_distance(token, term);
                if dist <= max_distance {
                    if let Some(doc_ids) = self.index.get(term) {
                        for doc_id in doc_ids {
                            *results.entry(*doc_id).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut sorted: Vec<_> = results.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().map(|(doc_id, _)| doc_id).collect()
    }

    /// Get total number of indexed documents
    pub fn doc_count(&self) -> u64 {
        self.doc_count
    }

    /// Get total number of unique terms
    pub fn term_count(&self) -> usize {
        self.index.len()
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Levenshtein edit distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}
