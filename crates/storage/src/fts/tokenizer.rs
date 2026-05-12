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

/// Chinese tokenizer with unigram and bigram support
#[derive(Clone)]
pub struct ChineseTokenizer;

impl ChineseTokenizer {
    pub fn new() -> Self {
        Self
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
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
                if lower.len() > 1 {
                    tokens.push(lower);
                }
                i = j;
            } else if c.is_whitespace() || c.is_ascii_punctuation() || c == '　' {
                // Skip punctuation and whitespace
                i += 1;
            } else {
                // Chinese character: create unigram and bigram
                tokens.push(c.to_string());
                // Bigram with next Chinese character
                if i + 1 < len
                    && !chars[i + 1].is_ascii_alphanumeric()
                    && !chars[i + 1].is_whitespace()
                    && chars[i + 1] != '　'
                {
                    let mut bigram = c.to_string();
                    bigram.push(chars[i + 1]);
                    tokens.push(bigram);
                }
                i += 1;
            }
        }

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
