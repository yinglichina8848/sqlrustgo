use async_trait::async_trait;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub dimension: usize,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
            dimension: 768,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub dimension: usize,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "text-embedding-3-small".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            dimension: 1536,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EmbeddingProviderConfig {
    Ollama(OllamaConfig),
    OpenAI(OpenAIConfig),
    Hash(HashConfig),
}

#[derive(Debug, Clone)]
pub struct HashConfig {
    pub dimension: usize,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self { dimension: 256 }
    }
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn name(&self) -> &str;
    fn dimension(&self) -> usize;
    fn is_local(&self) -> bool;
    async fn generate(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn Error + Send + Sync>>;
    fn supports_batch(&self) -> bool {
        true
    }
}

pub mod ollama {
    use super::*;
    use reqwest::Client;

    pub struct OllamaProvider {
        config: OllamaConfig,
        client: Client,
    }

    impl OllamaProvider {
        pub fn new(config: OllamaConfig) -> Self {
            Self {
                config,
                client: Client::new(),
            }
        }

        pub fn from_env() -> Self {
            Self::new(OllamaConfig::default())
        }
    }

    #[async_trait]
    impl EmbeddingProvider for OllamaProvider {
        fn name(&self) -> &str {
            "ollama"
        }

        fn dimension(&self) -> usize {
            self.config.dimension
        }

        fn is_local(&self) -> bool {
            true
        }

        async fn generate(
            &self,
            texts: Vec<String>,
        ) -> Result<Vec<Vec<f32>>, Box<dyn Error + Send + Sync>> {
            let url = format!("{}/api/embeddings", self.config.base_url);
            let mut results = Vec::with_capacity(texts.len());

            for text in texts {
                let body = serde_json::json!({
                    "model": self.config.model,
                    "prompt": text,
                });

                let response = self.client.post(&url).json(&body).send().await?;

                if !response.status().is_success() {
                    return Err(format!("Ollama API error: {}", response.status()).into());
                }

                let json: serde_json::Value = response.json().await?;
                let embedding = json["embedding"]
                    .as_array()
                    .ok_or("Invalid embedding response")?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect();

                results.push(embedding);
            }

            Ok(results)
        }
    }
}

pub mod openai {
    use super::*;
    use reqwest::Client;

    pub struct OpenAIProvider {
        config: OpenAIConfig,
        client: Client,
    }

    impl OpenAIProvider {
        pub fn new(config: OpenAIConfig) -> Self {
            Self {
                config,
                client: Client::new(),
            }
        }

        pub fn from_env() -> Option<Self> {
            let api_key = std::env::var("OPENAI_API_KEY").ok()?;
            Some(Self::new(OpenAIConfig {
                api_key,
                ..Default::default()
            }))
        }
    }

    #[async_trait]
    impl EmbeddingProvider for OpenAIProvider {
        fn name(&self) -> &str {
            "openai"
        }

        fn dimension(&self) -> usize {
            self.config.dimension
        }

        fn is_local(&self) -> bool {
            false
        }

        async fn generate(
            &self,
            texts: Vec<String>,
        ) -> Result<Vec<Vec<f32>>, Box<dyn Error + Send + Sync>> {
            let url = format!("{}/embeddings", self.config.base_url);

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .json(&serde_json::json!({
                    "model": self.config.model,
                    "input": texts,
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(format!("OpenAI API error: {}", response.status()).into());
            }

            let json: serde_json::Value = response.json().await?;
            let embeddings = json["data"]
                .as_array()
                .ok_or("Invalid embedding response")?
                .iter()
                .map(|v| {
                    v["embedding"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|x| x.as_f64().unwrap_or(0.0) as f32)
                        .collect()
                })
                .collect();

            Ok(embeddings)
        }
    }
}

pub mod hash {
    use super::*;
    use crate::embedding::{EmbeddingModel, HashEmbeddingModel};

    pub struct HashEmbeddingProvider {
        model: HashEmbeddingModel,
    }

    impl HashEmbeddingProvider {
        pub fn new(config: HashConfig) -> Self {
            Self {
                model: HashEmbeddingModel::new(config.dimension),
            }
        }
    }

    #[async_trait]
    impl EmbeddingProvider for HashEmbeddingProvider {
        fn name(&self) -> &str {
            "hash"
        }

        fn dimension(&self) -> usize {
            self.model.dimension()
        }

        fn is_local(&self) -> bool {
            true
        }

        async fn generate(
            &self,
            texts: Vec<String>,
        ) -> Result<Vec<Vec<f32>>, Box<dyn Error + Send + Sync>> {
            Ok(texts
                .iter()
                .map(|t| self.model.generate_embedding(t))
                .collect())
        }

        fn supports_batch(&self) -> bool {
            true
        }
    }
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: EmbeddingProviderConfig) -> Box<dyn EmbeddingProvider> {
        match config {
            EmbeddingProviderConfig::Ollama(cfg) => Box::new(ollama::OllamaProvider::new(cfg)),
            EmbeddingProviderConfig::OpenAI(cfg) => Box::new(openai::OpenAIProvider::new(cfg)),
            EmbeddingProviderConfig::Hash(cfg) => Box::new(hash::HashEmbeddingProvider::new(cfg)),
        }
    }

    pub fn create_ollama() -> Box<dyn EmbeddingProvider> {
        Box::new(ollama::OllamaProvider::from_env())
    }

    pub fn create_openai() -> Option<Box<dyn EmbeddingProvider>> {
        openai::OpenAIProvider::from_env().map(|p| Box::new(p) as Box<dyn EmbeddingProvider>)
    }

    pub fn create_hash() -> Box<dyn EmbeddingProvider> {
        Box::new(hash::HashEmbeddingProvider::new(HashConfig::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_provider_basic() {
        let provider = ProviderFactory::create_hash();

        assert_eq!(provider.name(), "hash");
        assert_eq!(provider.dimension(), 256);
        assert!(provider.is_local());

        let embeddings = provider
            .generate(vec!["hello world".to_string()])
            .await
            .unwrap();
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].len(), 256);
    }

    #[tokio::test]
    async fn test_hash_provider_deterministic() {
        let provider = ProviderFactory::create_hash();

        let emb1 = provider
            .generate(vec!["test text".to_string()])
            .await
            .unwrap();
        let emb2 = provider
            .generate(vec!["test text".to_string()])
            .await
            .unwrap();

        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_hash_provider_batch() {
        let provider = ProviderFactory::create_hash();

        let embeddings = provider
            .generate(vec![
                "first text".to_string(),
                "second text".to_string(),
                "third text".to_string(),
            ])
            .await
            .unwrap();

        assert_eq!(embeddings.len(), 3);
        for emb in &embeddings {
            assert_eq!(emb.len(), 256);
        }
    }

    #[test]
    fn test_provider_factory_create() {
        let hash_provider =
            ProviderFactory::create(EmbeddingProviderConfig::Hash(HashConfig { dimension: 128 }));
        assert_eq!(hash_provider.dimension(), 128);
    }

    #[test]
    fn test_ollama_config_default() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.model, "nomic-embed-text");
        assert_eq!(config.dimension, 768);
    }

    #[test]
    fn test_openai_config_default() {
        let config = OpenAIConfig::default();
        assert_eq!(config.model, "text-embedding-3-small");
        assert_eq!(config.dimension, 1536);
    }

    #[test]
    fn test_hash_config_default() {
        let config = HashConfig::default();
        assert_eq!(config.dimension, 256);
    }
}
