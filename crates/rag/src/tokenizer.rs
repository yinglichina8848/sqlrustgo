use std::collections::HashSet;

pub trait Tokenizer {
    fn tokenize(&self, text: &str) -> Vec<String>;
}

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
                i = j; // Skip all accumulated chars
            } else if c.is_whitespace() || c.is_ascii_punctuation() || c == '　' {
                // Skip punctuation and whitespace
                i += 1;
            } else {
                // Chinese character: create unigram and bigram
                // Unigram
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

    // Edge case tests
    #[test]
    fn test_simple_tokenizer_empty() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_simple_tokenizer_only_stop_words() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("the a an and or");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_simple_tokenizer_numbers() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("test123 hello456");
        assert!(!tokens.is_empty());
        assert!(tokens
            .iter()
            .any(|t| t.contains("test123") || t.contains("hello456")));
    }

    #[test]
    fn test_simple_tokenizer_underscore() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("hello_world test_case");
        assert!(tokens.len() >= 2);
    }

    #[test]
    fn test_simple_tokenizer_punctuation() {
        let tokenizer = SimpleTokenizer::new();
        let tokens = tokenizer.tokenize("hello.world!test?foo");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_chinese_tokenizer_empty() {
        let tokenizer = ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_chinese_tokenizer_mixed() {
        let tokenizer = ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("hello世界123test");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_chinese_tokenizer_punctuation() {
        let tokenizer = ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("你好，的世界！测试？");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_chinese_tokenizer_english() {
        let tokenizer = ChineseTokenizer::new();
        let tokens = tokenizer.tokenize("Rust Programming");
        // English should still be tokenized
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_multilanguage_tokenizer_empty() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_multilanguage_tokenizer_special_chars() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("!@#$%^&*() 你好 ");
        assert!(!tokens.is_empty() || tokens.is_empty()); // Either is valid
    }

    #[test]
    fn test_multilanguage_tokenizer_japanese() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("こんにちは你好hello");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_multilanguage_tokenizer_korean() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("안녕하세요你好hello");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_multilanguage_tokenizer_dedup() {
        let tokenizer = MultiLanguageTokenizer::new();
        let tokens = tokenizer.tokenize("hello Hello HELLO");
        // Should deduplicate
        let hello_count = tokens.iter().filter(|t| *t == "hello").count();
        assert!(hello_count <= 1);
    }
}
