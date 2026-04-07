pub mod tokenizer {
    pub trait Tokenizer {
        fn tokenize(&self, text: &str) -> Vec<String>;
    }

    #[derive(Clone)]
    pub struct SimpleTokenizer {
        stop_words: Vec<&'static str>,
    }

    impl SimpleTokenizer {
        pub fn new() -> Self {
            let stop_words = vec![
                "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
                "by", "from", "as", "is", "was", "are", "were", "be", "been", "being", "have",
                "has", "had", "do", "does", "did", "will", "would", "should", "could", "may",
                "might", "must", "can", "this", "that", "these", "those", "i", "you", "he", "she",
                "it", "we", "they",
            ];
            Self { stop_words }
        }

        pub fn tokenize(&self, text: &str) -> Vec<String> {
            text.to_lowercase()
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|s| !s.is_empty() && s.len() > 1 && !self.stop_words.contains(&s))
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
            let mut current = String::new();

            for c in text.chars() {
                if c.is_ascii_alphanumeric() {
                    current.push(c);
                } else if c.is_whitespace() {
                    if !current.is_empty() {
                        let lower = current.to_lowercase();
                        if lower.len() > 1 {
                            tokens.push(lower);
                        }
                        current.clear();
                    }
                } else if c.is_ascii_punctuation() || c == '　' {
                    if !current.is_empty() {
                        let lower = current.to_lowercase();
                        if lower.len() > 1 {
                            tokens.push(lower);
                        }
                        current.clear();
                    }
                } else {
                    if !current.is_empty() {
                        let lower = current.to_lowercase();
                        if lower.len() > 1 {
                            tokens.push(lower);
                        }
                        current.clear();
                    }
                    if !c.is_whitespace() {
                        let s = c.to_string();
                        if s.len() > 0 {
                            tokens.push(s);
                        }
                    }
                }
            }

            if !current.is_empty() {
                let lower = current.to_lowercase();
                if lower.len() > 1 {
                    tokens.push(lower);
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
    }
}

pub use tokenizer::*;
