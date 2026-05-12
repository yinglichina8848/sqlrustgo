//! Full-Text Search (FTS) module for SQLRustGo
//!
//! Provides inverted index-based full-text search capabilities with
//! support for multiple tokenizers (English, Chinese, multilingual).

mod inverted_index;
mod tokenizer;

pub use inverted_index::InvertedIndex;
pub use tokenizer::{Tokenizer, SimpleTokenizer, ChineseTokenizer, MultiLanguageTokenizer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("Hello World, this is a test!");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.contains(&"this".to_string()));
    }

    #[test]
    fn test_chinese_tokenizer() {
        let tokenizer = ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("你好世界，这是一个测试");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_multilanguage_tokenizer() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("Hello 你好 World 世界");
        assert!(!tokens.is_empty());
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
    }

    #[test]
    fn test_inverted_index_add_and_search() {
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
    fn test_inverted_index_chinese() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "你好世界");
        index.add_document(2, "Hello 世界");

        let results = index.search("你好");
        assert!(!results.is_empty());
        assert!(results.contains(&1));
    }

    #[test]
    fn test_inverted_index_fuzzy() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "testing");
        index.add_document(2, "testng");

        let results = index.fuzzy_search("testing", 1);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }
}
