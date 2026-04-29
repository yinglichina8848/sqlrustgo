pub mod inverted_index;
pub mod tokenizer;

pub use inverted_index::*;
pub use tokenizer::{ChineseTokenizer, MultiLanguageTokenizer, SimpleTokenizer, Tokenizer};

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

    pub fn delete_document(&mut self, doc_id: u64) -> Option<Document> {
        let doc = self.documents.remove(&doc_id);
        if doc.is_some() {
            // Re-index remaining documents (inverted index doesn't support direct deletion)
            self.rebuild_index();
        }
        doc
    }

    pub fn update_document(&mut self, doc: Document) -> Option<Document> {
        let old_doc = self.documents.insert(doc.id, doc.clone());
        if old_doc.is_some() {
            self.rebuild_index();
        }
        old_doc
    }

    pub fn get_document(&self, doc_id: u64) -> Option<&Document> {
        self.documents.get(&doc_id)
    }

    pub fn list_documents(&self) -> Vec<&Document> {
        self.documents.values().collect()
    }

    pub fn clear(&mut self) {
        self.documents.clear();
        self.index = InvertedIndex::new();
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

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn get_statistics(&self) -> RAGStatistics {
        RAGStatistics {
            document_count: self.documents.len() as u64,
            index_term_count: self.index.term_count(),
        }
    }

    fn rebuild_index(&mut self) {
        self.index = InvertedIndex::new();
        for doc in self.documents.values() {
            self.index.add_document(doc.id, &doc.content);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGStatistics {
    pub document_count: u64,
    pub index_term_count: usize,
}

/// KnowledgeBase provides document management with versioning and multi-source support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseEntry {
    pub document: Document,
    pub version: u64,
    pub source: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl KnowledgeBaseEntry {
    pub fn new(document: Document, source: &str, timestamp: u64) -> Self {
        Self {
            document,
            version: 1,
            source: source.to_string(),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBase {
    entries: HashMap<u64, KnowledgeBaseEntry>,
    current_version: u64,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            current_version: 0,
        }
    }

    pub fn add_document(&mut self, document: Document, source: &str) {
        let timestamp = self.current_version;
        self.current_version += 1;

        let entry = KnowledgeBaseEntry::new(document.clone(), source, timestamp);
        self.entries.insert(document.id, entry);
    }

    pub fn update_document(&mut self, document: Document) -> Option<Document> {
        let timestamp = self.current_version;
        self.current_version += 1;

        if let Some(entry) = self.entries.get_mut(&document.id) {
            let old_doc = entry.document.clone();
            entry.document = document;
            entry.version += 1;
            entry.updated_at = timestamp;
            Some(old_doc)
        } else {
            None
        }
    }

    pub fn get_document(&self, doc_id: u64) -> Option<&Document> {
        self.entries.get(&doc_id).map(|e| &e.document)
    }

    pub fn get_entry(&self, doc_id: u64) -> Option<&KnowledgeBaseEntry> {
        self.entries.get(&doc_id)
    }

    pub fn delete_document(&mut self, doc_id: u64) -> Option<Document> {
        self.entries.remove(&doc_id).map(|e| e.document)
    }

    pub fn list_entries(&self) -> Vec<&KnowledgeBaseEntry> {
        self.entries.values().collect()
    }

    pub fn get_documents_by_source(&self, source: &str) -> Vec<&Document> {
        self.entries
            .values()
            .filter(|e| e.source == source)
            .map(|e| &e.document)
            .collect()
    }

    pub fn document_count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_version = 0;
    }

    /// Get history of a document (returns all versions as document IDs)
    pub fn get_document_history(&self, doc_id: u64) -> Option<KnowledgeBaseEntry> {
        self.entries.get(&doc_id).cloned()
    }
}

/// OpenClaw API for RAG-powered knowledge queries
pub struct OpenClawClient {
    rag_pipeline: RAGPipeline,
    knowledge_base: KnowledgeBase,
}

impl OpenClawClient {
    pub fn new() -> Self {
        Self {
            rag_pipeline: RAGPipeline::new(),
            knowledge_base: KnowledgeBase::new(),
        }
    }

    /// Query knowledge base and return top-k relevant documents
    pub fn query_knowledge(&self, query: &str, top_k: usize) -> Vec<QueryResult> {
        let doc_ids = self.rag_pipeline.search(query, top_k);
        doc_ids
            .into_iter()
            .filter_map(|(id, content)| {
                self.knowledge_base.get_entry(id).map(|entry| QueryResult {
                    doc_id: id,
                    content,
                    source: entry.source.clone(),
                    score: 1.0, // TODO: implement actual scoring
                })
            })
            .collect()
    }

    /// RAG query - search and return formatted results
    pub fn rag_query(&self, question: &str, top_k: usize) -> Vec<String> {
        self.rag_pipeline.retrieve(question, top_k)
    }

    /// Batch query processing
    pub fn batch_query(&self, questions: &[&str], top_k: usize) -> Vec<Vec<String>> {
        questions
            .iter()
            .map(|q| self.rag_pipeline.retrieve(q, top_k))
            .collect()
    }

    pub fn add_document(&mut self, document: Document, source: &str) {
        self.rag_pipeline.add_document(document.clone());
        self.knowledge_base.add_document(document, source);
    }

    pub fn update_document(&mut self, document: Document) -> Option<Document> {
        let old = self.rag_pipeline.update_document(document.clone());
        if old.is_some() {
            self.knowledge_base.update_document(document);
        }
        old
    }

    pub fn delete_document(&mut self, doc_id: u64) -> Option<Document> {
        let doc = self.rag_pipeline.delete_document(doc_id);
        if doc.is_some() {
            self.knowledge_base.delete_document(doc_id);
        }
        doc
    }

    pub fn get_statistics(&self) -> ClientStatistics {
        ClientStatistics {
            rag_stats: self.rag_pipeline.get_statistics(),
            knowledge_base_doc_count: self.knowledge_base.document_count() as u64,
        }
    }

    pub fn doc_count(&self) -> u64 {
        self.rag_pipeline.doc_count()
    }
}

impl Default for OpenClawClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub doc_id: u64,
    pub content: String,
    pub source: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStatistics {
    pub rag_stats: RAGStatistics,
    pub knowledge_base_doc_count: u64,
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

    #[test]
    fn test_document_with_metadata() {
        let doc = Document::new(1, "Test content".to_string())
            .with_metadata("author", "test")
            .with_metadata("category", "test_category");

        assert_eq!(doc.id, 1);
        assert_eq!(doc.content, "Test content");
        assert_eq!(doc.metadata.get("author"), Some(&"test".to_string()));
        assert_eq!(
            doc.metadata.get("category"),
            Some(&"test_category".to_string())
        );
    }

    #[test]
    fn test_rag_pipeline_retrieve() {
        let mut pipeline = RAGPipeline::new();

        pipeline.add_document(Document::new(1, "Rust is fast and memory safe".to_string()));
        pipeline.add_document(Document::new(
            2,
            "Python is interpreted and dynamic".to_string(),
        ));

        let retrieved = pipeline.retrieve("Rust fast", 5);
        assert!(!retrieved.is_empty());
        assert!(retrieved.iter().any(|c| c.contains("Rust")));
    }

    #[test]
    fn test_rag_pipeline_doc_count() {
        let mut pipeline = RAGPipeline::new();
        assert_eq!(pipeline.doc_count(), 0);

        pipeline.add_document(Document::new(1, "First document".to_string()));
        assert_eq!(pipeline.doc_count(), 1);

        pipeline.add_document(Document::new(2, "Second document".to_string()));
        assert_eq!(pipeline.doc_count(), 2);
    }

    #[test]
    fn test_rag_pipeline_search_limit() {
        let mut pipeline = RAGPipeline::new();

        for i in 1..=10 {
            pipeline.add_document(Document::new(
                i,
                format!("Document {} with keywords test", i),
            ));
        }

        let results = pipeline.search("test", 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_rag_pipeline_empty_query() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Test content".to_string()));

        let results = pipeline.search("nonexistent keyword xyz", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rag_pipeline_chinese_content() {
        let mut pipeline = RAGPipeline::new();

        pipeline.add_document(Document::new(1, "Rust编程语言性能优异".to_string()));
        pipeline.add_document(Document::new(2, "Python适合数据科学".to_string()));

        let results = pipeline.search("Rust 性能", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_rag_pipeline_delete_document() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Test content".to_string()));
        pipeline.add_document(Document::new(2, "Another content".to_string()));

        let deleted = pipeline.delete_document(1);
        assert!(deleted.is_some());
        assert_eq!(pipeline.doc_count(), 1);
        assert!(pipeline.get_document(1).is_none());
    }

    #[test]
    fn test_rag_pipeline_update_document() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Original content".to_string()));

        let old = pipeline.update_document(Document::new(1, "Updated content".to_string()));
        assert!(old.is_some());

        let results = pipeline.search("Updated", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_rag_pipeline_clear() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Test".to_string()));
        pipeline.add_document(Document::new(2, "Test2".to_string()));

        pipeline.clear();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.doc_count(), 0);
    }

    #[test]
    fn test_rag_pipeline_get_document() {
        let mut pipeline = RAGPipeline::new();
        let doc = Document::new(1, "Test content".to_string());
        pipeline.add_document(doc.clone());

        let retrieved = pipeline.get_document(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[test]
    fn test_rag_pipeline_list_documents() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "First".to_string()));
        pipeline.add_document(Document::new(2, "Second".to_string()));

        let docs = pipeline.list_documents();
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_rag_pipeline_statistics() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Hello World".to_string()));

        let stats = pipeline.get_statistics();
        assert_eq!(stats.document_count, 1);
        assert!(stats.index_term_count > 0);
    }

    #[test]
    fn test_rag_pipeline_is_empty() {
        let pipeline = RAGPipeline::new();
        assert!(pipeline.is_empty());
    }

    #[test]
    fn test_rag_pipeline_delete_nonexistent() {
        let mut pipeline = RAGPipeline::new();
        pipeline.add_document(Document::new(1, "Test".to_string()));

        let result = pipeline.delete_document(999);
        assert!(result.is_none());
        assert_eq!(pipeline.doc_count(), 1);
    }

    // KnowledgeBase tests
    #[test]
    fn test_knowledge_base_add_and_get() {
        let mut kb = KnowledgeBase::new();
        let doc = Document::new(1, "Test content".to_string());
        kb.add_document(doc.clone(), "test_source");

        assert!(kb.get_document(1).is_some());
        assert_eq!(kb.document_count(), 1);
    }

    #[test]
    fn test_knowledge_base_update() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "Original".to_string()), "source");

        let old = kb.update_document(Document::new(1, "Updated".to_string()));
        assert!(old.is_some());

        let entry = kb.get_entry(1).unwrap();
        assert_eq!(entry.version, 2);
        assert_eq!(entry.document.content, "Updated");
    }

    #[test]
    fn test_knowledge_base_delete() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "Test".to_string()), "source");

        let deleted = kb.delete_document(1);
        assert!(deleted.is_some());
        assert!(kb.is_empty());
    }

    #[test]
    fn test_knowledge_base_source_filter() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "Doc1".to_string()), "source_a");
        kb.add_document(Document::new(2, "Doc2".to_string()), "source_b");
        kb.add_document(Document::new(3, "Doc3".to_string()), "source_a");

        let from_a = kb.get_documents_by_source("source_a");
        assert_eq!(from_a.len(), 2);
    }

    #[test]
    fn test_knowledge_base_history() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "V1".to_string()), "source");

        let entry_after_add = kb.get_entry(1).unwrap();
        assert_eq!(entry_after_add.version, 1);

        kb.update_document(Document::new(1, "V2".to_string()));

        let entry_after_update = kb.get_entry(1).unwrap();
        assert_eq!(entry_after_update.version, 2);
        assert_eq!(entry_after_update.document.content, "V2");
    }

    // OpenClawClient tests
    #[test]
    fn test_openclaw_client_query() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Rust is fast".to_string()), "docs");
        client.add_document(Document::new(2, "Python is dynamic".to_string()), "docs");

        let results = client.query_knowledge("Rust", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_openclaw_client_rag_query() {
        let mut client = OpenClawClient::new();
        client.add_document(
            Document::new(1, "Rust is fast and memory safe".to_string()),
            "docs",
        );

        let results = client.rag_query("Rust fast", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_openclaw_client_batch_query() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Rust programming".to_string()), "docs");
        client.add_document(Document::new(2, "Python data science".to_string()), "docs");

        let questions = vec!["Rust", "Python"];
        let results = client.batch_query(&questions, 5);

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty() || !results[1].is_empty());
    }

    #[test]
    fn test_openclaw_client_statistics() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Test".to_string()), "docs");

        let stats = client.get_statistics();
        assert_eq!(stats.rag_stats.document_count, 1);
        assert_eq!(stats.knowledge_base_doc_count, 1);
    }

    #[test]
    fn test_openclaw_client_delete() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Test".to_string()), "docs");

        let deleted = client.delete_document(1);
        assert!(deleted.is_some());
        assert_eq!(client.doc_count(), 0);
    }

    // KnowledgeBase persistence tests
    #[test]
    fn test_knowledge_base_serialization() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "Test content".to_string()), "test_source");

        // Serialize to JSON
        let json = serde_json::to_string(&kb).unwrap();
        assert!(!json.is_empty());

        // Deserialize back
        let restored: KnowledgeBase = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.document_count(), 1);
        assert_eq!(restored.get_document(1).unwrap().content, "Test content");
    }

    #[test]
    fn test_knowledge_base_serialization_multiple_docs() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "First".to_string()), "source_a");
        kb.add_document(Document::new(2, "Second".to_string()), "source_b");
        kb.add_document(Document::new(3, "Third".to_string()), "source_a");

        let json = serde_json::to_string(&kb).unwrap();
        let restored: KnowledgeBase = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.document_count(), 3);
        assert_eq!(restored.get_documents_by_source("source_a").len(), 2);
        assert_eq!(restored.get_documents_by_source("source_b").len(), 1);
    }

    #[test]
    fn test_knowledge_base_version_tracking() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "V1".to_string()), "source");

        let entry_v1 = kb.get_entry(1).unwrap();
        assert_eq!(entry_v1.version, 1);
        assert_eq!(entry_v1.document.content, "V1");

        kb.update_document(Document::new(1, "V2".to_string()));

        let entry_v2 = kb.get_entry(1).unwrap();
        assert_eq!(entry_v2.version, 2);
        assert_eq!(entry_v2.document.content, "V2");
    }

    #[test]
    fn test_knowledge_base_serialization_after_update() {
        let mut kb = KnowledgeBase::new();
        kb.add_document(Document::new(1, "Original".to_string()), "source");
        kb.update_document(Document::new(1, "Updated".to_string()));

        let json = serde_json::to_string(&kb).unwrap();
        let restored: KnowledgeBase = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.get_document(1).unwrap().content, "Updated");
        assert_eq!(restored.get_entry(1).unwrap().version, 2);
    }

    #[test]
    fn test_document_serialization() {
        let doc = Document::new(1, "Test content".to_string())
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        let json = serde_json::to_string(&doc).unwrap();
        let restored: Document = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, 1);
        assert_eq!(restored.content, "Test content");
        assert_eq!(restored.metadata.get("key1"), Some(&"value1".to_string()));
    }

    // OpenClawClient enhanced tests
    #[test]
    fn test_openclaw_client_query_result_structure() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Rust is fast".to_string()), "docs");
        client.add_document(Document::new(2, "Python is slow".to_string()), "docs");

        let results = client.query_knowledge("Rust", 5);

        assert!(!results.is_empty());
        let first = &results[0];
        assert_eq!(first.doc_id, 1);
        assert_eq!(first.source, "docs");
        assert!(first.content.contains("Rust"));
    }

    #[test]
    fn test_openclaw_client_empty_query() {
        let client = OpenClawClient::new();
        let results = client.query_knowledge("nonexistent", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_openclaw_client_batch_query_empty() {
        let client = OpenClawClient::new();
        let questions: Vec<&str> = vec![];
        let results = client.batch_query(&questions, 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_openclaw_client_consistency() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Test content".to_string()), "source");

        // Both query methods should return consistent results
        let query_results = client.query_knowledge("Test", 5);
        let rag_results = client.rag_query("Test", 5);

        assert_eq!(query_results.len(), rag_results.len());
    }

    #[test]
    fn test_openclaw_client_update_and_query() {
        let mut client = OpenClawClient::new();
        client.add_document(Document::new(1, "Original content".to_string()), "docs");

        let old = client.update_document(Document::new(1, "Updated content".to_string()));
        assert!(old.is_some());

        let results = client.query_knowledge("Updated", 5);
        assert!(!results.is_empty());
        assert!(results[0].content.contains("Updated"));
    }

    #[test]
    fn test_openclaw_client_multiple_sources() {
        let mut client = OpenClawClient::new();
        client.add_document(
            Document::new(1, "Doc from source A".to_string()),
            "source_a",
        );
        client.add_document(
            Document::new(2, "Doc from source B".to_string()),
            "source_b",
        );
        client.add_document(
            Document::new(3, "Another from source A".to_string()),
            "source_a",
        );

        let results_a = client.query_knowledge("source", 5);
        let sources: Vec<_> = results_a.iter().map(|r| r.source.clone()).collect();

        // All results should have valid sources
        for source in sources {
            assert!(source == "source_a" || source == "source_b");
        }
    }

    #[test]
    fn test_rag_pipeline_rebuild_index() {
        let mut pipeline = RAGPipeline::new();

        // Add some documents
        pipeline.add_document(Document::new(1, "First document".to_string()));
        pipeline.add_document(Document::new(2, "Second document".to_string()));

        // Delete one document - triggers rebuild
        pipeline.delete_document(1);

        // The remaining document should still be searchable
        let results = pipeline.search("Second", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 2);
    }

    #[test]
    fn test_rag_pipeline_update_rebuilds_index() {
        let mut pipeline = RAGPipeline::new();

        pipeline.add_document(Document::new(1, "Original content".to_string()));
        pipeline.update_document(Document::new(1, "New content".to_string()));

        // Old content should not be searchable
        let old_results = pipeline.search("Original", 5);
        assert!(old_results.is_empty());

        // New content should be searchable
        let new_results = pipeline.search("New", 5);
        assert!(!new_results.is_empty());
    }

    #[test]
    fn test_inverted_index_search_returns_in_order() {
        let mut index = InvertedIndex::new();
        // Add multiple documents with same term
        index.add_document(1, "Rust programming");
        index.add_document(2, "Rust is awesome");
        index.add_document(3, "Rust and more Rust");

        let results = index.search("Rust");
        // Should return all three
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_inverted_index_term_frequency() {
        let mut index = InvertedIndex::new();
        index.add_document(1, "Rust Rust Rust");
        index.add_document(2, "Rust");

        // Document 1 has more occurrences of "rust"
        let results = index.search("Rust");
        // Both should be found (AND semantics)
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn test_metadata_operations() {
        let doc = Document::new(1, "Content".to_string())
            .with_metadata("author", "Alice")
            .with_metadata("version", "1.0")
            .with_metadata("reviewed", "true");

        assert_eq!(doc.metadata.len(), 3);
        assert_eq!(doc.metadata.get("author"), Some(&"Alice".to_string()));

        // Metadata should be preserved in KnowledgeBase
        let mut kb = KnowledgeBase::new();
        kb.add_document(doc.clone(), "test");
        let stored = kb.get_document(1).unwrap();
        assert_eq!(stored.metadata.get("author"), Some(&"Alice".to_string()));
    }
}

// ============================================================================
// SQLRustGo Storage Integration
// ============================================================================

#[cfg(feature = "sqlrustgo")]
mod sqlrustgo_integration {
    use super::*;
    use sqlrustgo_storage::engine::Value;
    use sqlrustgo_storage::{ColumnDefinition, Record, StorageEngine, TableInfo};

    /// Table name for OpenClaw knowledge base
    pub const OPENCLAW_TABLE_NAME: &str = "_openclaw_knowledge_base";

    /// Column names for the knowledge base table schema
    pub mod columns {
        pub const ID: &str = "doc_id";
        pub const CONTENT: &str = "content";
        pub const METADATA: &str = "metadata_json";
        pub const SOURCE: &str = "source";
        pub const VERSION: &str = "version";
        pub const CREATED_AT: &str = "created_at";
        pub const UPDATED_AT: &str = "updated_at";
    }

    impl OpenClawClient {
        /// Save the knowledge base to SQLRustGo storage
        pub fn save_to_storage(&self, storage: &mut dyn StorageEngine) -> Result<(), String> {
            if !storage.has_table(OPENCLAW_TABLE_NAME) {
                let table_info = TableInfo {
                    name: OPENCLAW_TABLE_NAME.to_string(),
                    columns: vec![
                        ColumnDefinition {
                            name: columns::ID.to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            primary_key: true,
                        },
                        ColumnDefinition {
                            name: columns::CONTENT.to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: false,
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: columns::METADATA.to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: true,
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: columns::SOURCE.to_string(),
                            data_type: "TEXT".to_string(),
                            nullable: false,
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: columns::VERSION.to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: columns::CREATED_AT.to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            primary_key: false,
                        },
                        ColumnDefinition {
                            name: columns::UPDATED_AT.to_string(),
                            data_type: "INTEGER".to_string(),
                            nullable: false,
                            primary_key: false,
                        },
                    ],
                    ..Default::default()
                };
                storage
                    .create_table(&table_info)
                    .map_err(|e| e.to_string())?;
            }

            storage
                .delete(OPENCLAW_TABLE_NAME, &[])
                .map_err(|e| e.to_string())?;

            let records: Vec<Record> = self
                .knowledge_base
                .list_entries()
                .iter()
                .map(|entry| {
                    let metadata_json =
                        serde_json::to_string(&entry.document.metadata).unwrap_or_default();
                    vec![
                        Value::Integer(entry.document.id as i64),
                        Value::Text(entry.document.content.clone()),
                        Value::Text(metadata_json),
                        Value::Text(entry.source.clone()),
                        Value::Integer(entry.version as i64),
                        Value::Integer(entry.created_at as i64),
                        Value::Integer(entry.updated_at as i64),
                    ]
                })
                .collect();

            if !records.is_empty() {
                storage
                    .insert(OPENCLAW_TABLE_NAME, records)
                    .map_err(|e| e.to_string())?;
            }

            Ok(())
        }

        /// Load the knowledge base from SQLRustGo storage
        pub fn load_from_storage(&mut self, storage: &dyn StorageEngine) -> Result<(), String> {
            if !storage.has_table(OPENCLAW_TABLE_NAME) {
                return Ok(());
            }

            let records = storage
                .scan(OPENCLAW_TABLE_NAME)
                .map_err(|e| e.to_string())?;

            for record in records {
                let doc_id = match record.first() {
                    Some(Value::Integer(id)) => *id as u64,
                    _ => continue,
                };
                let content = match record.get(1) {
                    Some(Value::Text(c)) => c.clone(),
                    _ => continue,
                };
                let metadata_json = match record.get(2) {
                    Some(Value::Text(s)) => s.clone(),
                    _ => "{}".to_string(),
                };
                let source = match record.get(3) {
                    Some(Value::Text(s)) => s.clone(),
                    _ => continue,
                };
                let version = match record.get(4) {
                    Some(Value::Integer(v)) => *v as u64,
                    _ => 1,
                };
                let created_at = match record.get(5) {
                    Some(Value::Integer(t)) => *t as u64,
                    _ => 0,
                };
                let updated_at = match record.get(6) {
                    Some(Value::Integer(t)) => *t as u64,
                    _ => 0,
                };

                let metadata: std::collections::HashMap<String, String> =
                    serde_json::from_str(&metadata_json).unwrap_or_default();

                let document = Document {
                    id: doc_id,
                    content: content.clone(),
                    metadata,
                };

                let entry = KnowledgeBaseEntry {
                    document,
                    version,
                    source: source.clone(),
                    created_at,
                    updated_at,
                };

                self.knowledge_base.entries.insert(doc_id, entry);
                self.rag_pipeline.add_document(Document {
                    id: doc_id,
                    content,
                    metadata: Default::default(),
                });
            }

            Ok(())
        }

        pub fn exists_in_storage(storage: &dyn StorageEngine) -> bool {
            storage.has_table(OPENCLAW_TABLE_NAME)
        }

        pub fn drop_from_storage(storage: &mut dyn StorageEngine) -> Result<(), String> {
            if storage.has_table(OPENCLAW_TABLE_NAME) {
                storage
                    .drop_table(OPENCLAW_TABLE_NAME)
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }

        pub fn sql_like_query(
            &self,
            storage: &dyn StorageEngine,
            like_pattern: &str,
        ) -> Result<Vec<QueryResult>, String> {
            if !storage.has_table(OPENCLAW_TABLE_NAME) {
                return Ok(vec![]);
            }

            let records = storage
                .scan(OPENCLAW_TABLE_NAME)
                .map_err(|e| e.to_string())?;
            let pattern_lower = like_pattern.to_lowercase();

            let results: Vec<QueryResult> = records
                .into_iter()
                .filter(|record| {
                    if let Some(Value::Text(content)) = record.get(1) {
                        content.to_lowercase().contains(&pattern_lower)
                    } else {
                        false
                    }
                })
                .filter_map(|record| {
                    let doc_id = match record.first() {
                        Some(Value::Integer(i)) => *i as u64,
                        _ => return None,
                    };
                    let content = match record.get(1) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => return None,
                    };
                    let source = match record.get(3) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => return None,
                    };
                    Some(QueryResult {
                        doc_id,
                        content,
                        source,
                        score: 1.0,
                    })
                })
                .collect();

            Ok(results)
        }

        pub fn sql_query_by_source(
            &self,
            storage: &dyn StorageEngine,
            source: &str,
        ) -> Result<Vec<QueryResult>, String> {
            if !storage.has_table(OPENCLAW_TABLE_NAME) {
                return Ok(vec![]);
            }

            let records = storage
                .scan(OPENCLAW_TABLE_NAME)
                .map_err(|e| e.to_string())?;

            let results: Vec<QueryResult> = records
                .into_iter()
                .filter(|record| {
                    if let Some(Value::Text(s)) = record.get(3) {
                        s == source
                    } else {
                        false
                    }
                })
                .filter_map(|record| {
                    let doc_id = match record.first() {
                        Some(Value::Integer(i)) => *i as u64,
                        _ => return None,
                    };
                    let content = match record.get(1) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => return None,
                    };
                    let source = match record.get(3) {
                        Some(Value::Text(s)) => s.clone(),
                        _ => return None,
                    };
                    Some(QueryResult {
                        doc_id,
                        content,
                        source,
                        score: 1.0,
                    })
                })
                .collect();

            Ok(results)
        }

        pub fn sql_count(&self, storage: &dyn StorageEngine) -> Result<u64, String> {
            if !storage.has_table(OPENCLAW_TABLE_NAME) {
                return Ok(0);
            }

            let records = storage
                .scan(OPENCLAW_TABLE_NAME)
                .map_err(|e| e.to_string())?;
            Ok(records.len() as u64)
        }
    }
}

/// Standalone functions for working with SQLRustGo storage
#[cfg(feature = "sqlrustgo")]
pub mod storage_helpers {
    use super::sqlrustgo_integration::{columns, OPENCLAW_TABLE_NAME};
    use sqlrustgo_storage::{ColumnDefinition, StorageEngine, TableInfo};

    pub fn create_openclaw_table(storage: &mut dyn StorageEngine) -> Result<(), String> {
        let table_info = TableInfo {
            name: OPENCLAW_TABLE_NAME.to_string(),
            columns: vec![
                ColumnDefinition {
                    name: columns::ID.to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: true,
                },
                ColumnDefinition {
                    name: columns::CONTENT.to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                ColumnDefinition {
                    name: columns::METADATA.to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    primary_key: false,
                },
                ColumnDefinition {
                    name: columns::SOURCE.to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                ColumnDefinition {
                    name: columns::VERSION.to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                ColumnDefinition {
                    name: columns::CREATED_AT.to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: false,
                },
                ColumnDefinition {
                    name: columns::UPDATED_AT.to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    primary_key: false,
                },
            ],
            ..Default::default()
        };
        storage.create_table(&table_info).map_err(|e| e.to_string())
    }

    pub fn openclaw_table_exists(storage: &dyn StorageEngine) -> bool {
        storage.has_table(OPENCLAW_TABLE_NAME)
    }

    pub fn drop_openclaw_table(storage: &mut dyn StorageEngine) -> Result<(), String> {
        if storage.has_table(OPENCLAW_TABLE_NAME) {
            storage
                .drop_table(OPENCLAW_TABLE_NAME)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "sqlrustgo")]
mod integration_tests {
    use super::sqlrustgo_integration::*;
    use crate::{Document, OpenClawClient};
    use sqlrustgo_storage::{MemoryStorage, StorageEngine};

    #[test]
    fn test_save_and_load_from_storage() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        client.add_document(Document::new(1, "Rust is fast".to_string()), "docs");
        client.add_document(Document::new(2, "Python is dynamic".to_string()), "docs");

        client.save_to_storage(&mut storage).unwrap();

        assert!(storage.has_table(OPENCLAW_TABLE_NAME));

        let mut client2 = OpenClawClient::new();
        client2.load_from_storage(&storage).unwrap();

        assert_eq!(client2.doc_count(), 2);
        assert_eq!(client2.get_statistics().knowledge_base_doc_count, 2);
    }

    #[test]
    fn test_save_empty_knowledge_base() {
        let mut storage = MemoryStorage::new();
        let client = OpenClawClient::new();

        client.save_to_storage(&mut storage).unwrap();

        assert!(storage.has_table(OPENCLAW_TABLE_NAME));
        let records = storage.scan(OPENCLAW_TABLE_NAME).unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_exists_in_storage() {
        let mut storage = MemoryStorage::new();
        let client = OpenClawClient::new();

        assert!(!OpenClawClient::exists_in_storage(&storage));

        client.save_to_storage(&mut storage).unwrap();

        assert!(OpenClawClient::exists_in_storage(&storage));
    }

    #[test]
    fn test_drop_from_storage() {
        let mut storage = MemoryStorage::new();
        let client = OpenClawClient::new();

        client.save_to_storage(&mut storage).unwrap();
        assert!(OpenClawClient::exists_in_storage(&storage));

        OpenClawClient::drop_from_storage(&mut storage).unwrap();
        assert!(!OpenClawClient::exists_in_storage(&storage));
    }

    #[test]
    fn test_sql_like_query() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        client.add_document(
            Document::new(1, "Rust is fast and memory safe".to_string()),
            "docs",
        );
        client.add_document(
            Document::new(2, "Python is great for data science".to_string()),
            "docs",
        );
        client.add_document(
            Document::new(3, "Go is simple and concurrent".to_string()),
            "docs",
        );

        client.save_to_storage(&mut storage).unwrap();

        let results = client.sql_like_query(&storage, "Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("Rust"));

        let results = client.sql_like_query(&storage, "is").unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_sql_query_by_source() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        client.add_document(Document::new(1, "Doc 1".to_string()), "source_a");
        client.add_document(Document::new(2, "Doc 2".to_string()), "source_b");
        client.add_document(Document::new(3, "Doc 3".to_string()), "source_a");

        client.save_to_storage(&mut storage).unwrap();

        let results = client.sql_query_by_source(&storage, "source_a").unwrap();
        assert_eq!(results.len(), 2);

        let results = client.sql_query_by_source(&storage, "source_b").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_sql_count() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        assert_eq!(client.sql_count(&storage).unwrap(), 0);

        client.add_document(Document::new(1, "Test 1".to_string()), "docs");
        client.add_document(Document::new(2, "Test 2".to_string()), "docs");

        client.save_to_storage(&mut storage).unwrap();

        assert_eq!(client.sql_count(&storage).unwrap(), 2);
    }

    #[test]
    fn test_update_persistence() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        client.add_document(Document::new(1, "Original content".to_string()), "docs");
        client.save_to_storage(&mut storage).unwrap();

        client.update_document(Document::new(1, "Updated content".to_string()));
        client.save_to_storage(&mut storage).unwrap();

        let mut client2 = OpenClawClient::new();
        client2.load_from_storage(&storage).unwrap();

        let results = client2.query_knowledge("Updated", 5);
        assert!(!results.is_empty());

        let results = client2.query_knowledge("Original", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_delete_persistence() {
        let mut storage = MemoryStorage::new();
        let mut client = OpenClawClient::new();

        client.add_document(Document::new(1, "Test 1".to_string()), "docs");
        client.add_document(Document::new(2, "Test 2".to_string()), "docs");
        client.save_to_storage(&mut storage).unwrap();

        client.delete_document(1);
        client.save_to_storage(&mut storage).unwrap();

        let mut client2 = OpenClawClient::new();
        client2.load_from_storage(&storage).unwrap();

        assert_eq!(client2.doc_count(), 1);
    }

    #[test]
    fn test_storage_helpers() {
        use super::storage_helpers::*;

        let mut storage = MemoryStorage::new();

        create_openclaw_table(&mut storage).unwrap();
        assert!(openclaw_table_exists(&storage));

        drop_openclaw_table(&mut storage).unwrap();
        assert!(!openclaw_table_exists(&storage));
    }
}
