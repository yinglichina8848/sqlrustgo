use crate::tokenizer::MultiLanguageTokenizer;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
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

    pub fn add_document(&mut self, doc_id: u64, text: &str) {
        let tokens = self.tokenizer.tokenize(text);
        for token in tokens {
            self.index
                .entry(token)
                .or_insert_with(HashSet::new)
                .insert(doc_id);
        }
        self.doc_count += 1;
    }

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

    pub fn search_with_limit(&self, query: &str, limit: usize) -> Vec<u64> {
        let mut results = self.search(query);
        results.truncate(limit);
        results
    }

    pub fn fuzzy_search(&self, query: &str, max_distance: usize) -> Vec<u64> {
        let tokens = self.tokenizer.tokenize(query);
        let mut results: HashMap<u64, usize> = HashMap::new();

        for (term, _) in &self.index {
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

    pub fn doc_count(&self) -> u64 {
        self.doc_count
    }

    pub fn term_count(&self) -> usize {
        self.index.len()
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; b.len() + 1]; a.len() + 1];

    for i in 0..=a.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b.len() {
        matrix[0][j] = j;
    }

    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost,
            );
        }
    }

    matrix[a.len()][b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_search() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "Hello World");
        index.add_document(2, "Hello World Hello");
        index.add_document(3, "World of Warcraft");

        let results = index.search("Hello");
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(!results.contains(&3));
    }

    #[test]
    fn test_phrase_search() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "The quick brown fox");
        index.add_document(2, "The slow brown dog");

        let results = index.search("quick fox");
        assert!(results.contains(&1));
    }

    #[test]
    fn test_fuzzy_search() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "testing");
        index.add_document(2, "testng");

        let results = index.fuzzy_search("testing", 1);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }
}
