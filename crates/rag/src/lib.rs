pub mod inverted_index;
pub mod tokenizer;

pub use inverted_index::*;
pub use tokenizer::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: u64,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

impl Document {
    pub fn new(id: u64, content: String) -> Self {
        Self {
            id,
            content,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

pub struct RAGPipeline {
    index: InvertedIndex,
    documents: HashMap<u64, Document>,
}

impl RAGPipeline {
    pub fn new() -> Self {
        Self {
            index: InvertedIndex::new(),
            documents: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, doc: Document) {
        self.documents.insert(doc.id, doc.clone());
        self.index.add_document(doc.id, &doc.content);
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<(u64, String)> {
        let doc_ids = self.index.search_with_limit(query, top_k);
        doc_ids
            .into_iter()
            .filter_map(|id| self.documents.get(&id).map(|doc| (id, doc.content.clone())))
            .collect()
    }

    pub fn retrieve(&self, query: &str, top_k: usize) -> Vec<String> {
        self.search(query, top_k)
            .into_iter()
            .map(|(_, content)| content)
            .collect()
    }

    pub fn doc_count(&self) -> u64 {
        self.index.doc_count()
    }
}

impl Default for RAGPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_pipeline() {
        let mut pipeline = RAGPipeline::new();

        pipeline.add_document(Document::new(
            1,
            "Rust is a systems programming language".to_string(),
        ));
        pipeline.add_document(Document::new(
            2,
            "Python is great for data science".to_string(),
        ));
        pipeline.add_document(Document::new(
            3,
            "Rust provides memory safety without GC".to_string(),
        ));

        let results = pipeline.search("Rust memory safety", 2);
        assert!(!results.is_empty());
        assert!(results.iter().any(|(id, _)| *id == 1 || *id == 3));
    }
}
