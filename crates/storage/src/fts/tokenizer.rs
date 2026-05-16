//! Tokenizer implementations for full-text search
//!
//! Provides tokenization for English, Chinese, and multilingual text.

use std::collections::HashSet;

/// Tokenizer trait for text tokenization
pub trait Tokenizer: Send + Sync {
    /// Tokenize text into a list of tokens
    fn tokenize(&self, text: &str) -> Vec<String>;
}

/// Simple tokenizer for English text with stop words
#[derive(Clone)]
pub struct SimpleTokenizer {
    stop_words: HashSet<&'static str>,
}

impl SimpleTokenizer {
    pub fn new() -> Self {
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may", "might", "must",
            "can", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we", "they",
        ];
        Self {
            stop_words: stop_words.into_iter().collect(),
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty() && s.len() > 1 && !self.stop_words.contains(s))
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        Self::tokenize(self, text)
    }
}

/// Chinese tokenizer with N-gram (bigram + trigram) support
/// Optimized for full-text search - produces fewer but more meaningful tokens
#[derive(Clone)]
pub struct ChineseTokenizer {
    /// Minimum token length to keep
    min_token_len: usize,
}

impl ChineseTokenizer {
    pub fn new() -> Self {
        Self { min_token_len: 2 }
    }

    /// Configure minimum token length
    pub fn with_min_token_len(mut self, min_len: usize) -> Self {
        self.min_token_len = min_len;
        self
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        // Collect segments of continuous non-ASCII characters
        let mut i = 0;
        while i < len {
            let c = chars[i];

            if c.is_ascii_alphanumeric() {
                // Accumulate ASCII sequences as single token
                let mut j = i;
                while j < len && chars[j].is_ascii_alphanumeric() {
                    j += 1;
                }
                let word: String = chars[i..j].iter().collect();
                let lower = word.to_lowercase();
                if lower.len() >= self.min_token_len {
                    tokens.push(lower);
                }
                i = j;
            } else if c.is_whitespace() || c.is_ascii_punctuation() || c == '　' {
                // Skip punctuation and whitespace
                i += 1;
            } else {
                // Chinese/Unicode character - use N-gram approach
                // Collect consecutive Chinese characters
                let start = i;
                let mut end = i + 1;
                while end < len
                    && !chars[end].is_ascii_alphanumeric()
                    && !chars[end].is_whitespace()
                    && chars[end] != '　'
                    && !chars[end].is_ascii_punctuation()
                {
                    end += 1;
                }

                let chinese_segment: String = chars[start..end].iter().collect();
                let seg_len = chinese_segment.chars().count();

                if seg_len >= 2 {
                    // For 2+ character segments: generate bigrams and trigrams
                    // This reduces noise compared to unigram + bigram
                    let chars_in_seg: Vec<char> = chinese_segment.chars().collect();

                    // Generate bigrams
                    for j in 0..seg_len - 1 {
                        let bigram: String = chars_in_seg[j..=j + 1].iter().collect();
                        tokens.push(bigram);
                    }

                    // Generate trigrams for segments >= 3
                    if seg_len >= 3 {
                        for j in 0..seg_len - 2 {
                            let trigram: String = chars_in_seg[j..=j + 2].iter().collect();
                            tokens.push(trigram);
                        }
                    }
                } else if seg_len == 1 {
                    // Single Chinese character - keep as unigram for search flexibility
                    tokens.push(c.to_string());
                }

                i = end;
            }
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        tokens.retain(|t| seen.insert(t.clone()));
        tokens
    }
}

impl Default for ChineseTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for ChineseTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        Self::tokenize(self, text)
    }
}

/// Multi-language tokenizer combining Simple and Chinese tokenizers
#[derive(Clone)]
pub struct MultiLanguageTokenizer {
    simple: SimpleTokenizer,
    chinese: ChineseTokenizer,
}

impl MultiLanguageTokenizer {
    pub fn new() -> Self {
        Self {
            simple: SimpleTokenizer::new(),
            chinese: ChineseTokenizer::new(),
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let simple_tokens = self.simple.tokenize(text);
        let chinese_tokens = self.chinese.tokenize(text);

        let mut all: Vec<String> = Vec::new();
        all.extend(simple_tokens);
        all.extend(chinese_tokens);
        all.sort();
        all.dedup();
        all
    }
}

impl Default for MultiLanguageTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for MultiLanguageTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        Self::tokenize(self, text)
    }
}
