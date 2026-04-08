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
            self.index.entry(token).or_default().insert(doc_id);
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

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; b_len + 1]; a_len + 1];

    #[allow(clippy::needless_range_loop)]
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    #[allow(clippy::needless_range_loop)]
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..a_len + 1 {
        for j in 1..b_len + 1 {
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

    #[test]
    fn test_search_with_limit() {
        let mut index = InvertedIndex::new();
        for i in 1..=10 {
            index.add_document(i, "matching content");
        }
        index.add_document(99, "non-matching");

        let results = index.search_with_limit("matching", 5);
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_doc_count() {
        let mut index = InvertedIndex::new();
        assert_eq!(index.doc_count(), 0);

        index.add_document(1, "First");
        assert_eq!(index.doc_count(), 1);

        index.add_document(2, "Second");
        assert_eq!(index.doc_count(), 2);
    }

    #[test]
    fn test_term_count() {
        let mut index = InvertedIndex::new();
        assert_eq!(index.term_count(), 0);

        index.add_document(1, "Hello World");
        assert!(index.term_count() > 0);
    }

    #[test]
    fn test_empty_query() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "Hello World");

        let results = index.search("");
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_match_query() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "Hello World");
        index.add_document(2, "Hello Again");

        let results = index.search("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_chinese_search() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "你好世界");
        index.add_document(2, "Hello 世界");

        let results = index.search("你好");
        assert!(
            !results.is_empty(),
            "Search should return results for '你好'"
        );
        assert!(results.contains(&1), "Should find doc 1");
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "testing");

        let results = index.fuzzy_search("nonexistentword", 2);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_distance_2() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "testing");
        index.add_document(2, "tosting");

        let results = index.fuzzy_search("testing", 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_and_query() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "Rust is fast");
        index.add_document(2, "Rust is safe");
        index.add_document(3, "Python is dynamic");

        let results = index.search("Rust fast");
        assert!(results.contains(&1));
        assert!(!results.contains(&2));
        assert!(!results.contains(&3));
    }

    #[test]
    fn test_chinese_tokenization() {
        use crate::tokenizer::ChineseTokenizer;
        let tokenizer = ChineseTokenizer::new();
        let doc_tokens = tokenizer.tokenize("你好世界");
        assert!(!doc_tokens.is_empty(), "Doc tokens should not be empty");
        assert!(doc_tokens.contains(&"你".to_string()));
        assert!(doc_tokens.contains(&"你好".to_string()));
    }
}
