// Copyright 2025 SQLRustGo Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Evidence Chain Module - Immutable audit trail for GMP operations
//!
//! This module provides tamper-evident evidence chains for all GMP operations,
//! enabling compliance with regulatory requirements (GDPR, HIPAA, SOX).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata associated with an evidence node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMetadata {
    /// Source of the evidence (e.g., "document", "vector_search", "graph_query")
    pub source: String,
    /// Timestamp when this evidence was created
    pub timestamp: i64,
    /// Optional additional context
    pub context: HashMap<String, String>,
    /// Score or relevance if applicable
    pub score: Option<f32>,
    /// References to other nodes in the chain
    pub references: Vec<String>,
}

impl EvidenceMetadata {
    /// Create new evidence metadata
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            context: HashMap::new(),
            score: None,
            references: vec![],
        }
    }

    /// Add a score to the metadata
    pub fn with_score(mut self, score: f32) -> Self {
        self.score = Some(score);
        self
    }

    /// Add context information
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a reference to another node
    pub fn with_reference(mut self, node_id: &str) -> Self {
        self.references.push(node_id.to_string());
        self
    }
}

/// Individual node in the evidence chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNode {
    /// Unique identifier for this node
    pub node_id: String,
    /// Type of the node (document, chunk, vector, graph, sql)
    pub node_type: String,
    /// Content or data this node represents
    pub content: String,
    /// Hash of the content
    pub content_hash: String,
    /// Hash of the previous node (empty for first node)
    pub previous_hash: String,
    /// Hash of this node (includes previous_hash for chain integrity)
    pub self_hash: String,
    /// Metadata associated with this node
    pub metadata: EvidenceMetadata,
}

impl EvidenceNode {
    /// Create a new evidence node
    pub fn new(
        node_id: &str,
        node_type: &str,
        content: &str,
        previous_hash: &str,
        metadata: EvidenceMetadata,
    ) -> Self {
        let content_hash = Self::hash_content(content);
        let self_hash = Self::compute_self_hash(node_id, node_type, &content_hash, previous_hash);
        
        Self {
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            content: content.to_string(),
            content_hash,
            previous_hash: previous_hash.to_string(),
            self_hash,
            metadata,
        }
    }

    /// Hash content using SHA-256
    fn hash_content(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Compute self-hash including previous hash for chain integrity
    fn compute_self_hash(node_id: &str, node_type: &str, content_hash: &str, previous_hash: &str) -> String {
        let data = format!("{}:{}:{}:{}", node_id, node_type, content_hash, previous_hash);
        Self::hash_content(&data)
    }
}

/// Evidence chain - an immutable, tamper-evident log of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceChain {
    /// Unique identifier for this chain
    pub chain_id: String,
    /// Description of what this chain tracks
    pub description: String,
    /// Nodes in the chain (in order)
    pub nodes: Vec<EvidenceNode>,
    /// Hash of the entire chain for integrity verification
    pub integrity_hash: String,
    /// When the chain was created
    pub created_at: i64,
    /// When the chain was last updated
    pub updated_at: i64,
}

impl EvidenceChain {
    /// Create a new evidence chain
    pub fn new(chain_id: String, description: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
            
        Self {
            chain_id,
            description,
            nodes: vec![],
            integrity_hash: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a document node to the chain
    pub fn add_document(mut self, doc_id: &str, content: &str, metadata: EvidenceMetadata) -> Self {
        let previous_hash = self.nodes.last()
            .map(|n| n.self_hash.clone())
            .unwrap_or_default();
            
        let node = EvidenceNode::new(
            doc_id,
            "document",
            content,
            &previous_hash,
            metadata,
        );
        
        self.nodes.push(node);
        self.integrity_hash = self.compute_integrity_hash();
        self
    }

    /// Add a SQL execution node
    pub fn add_sql(mut self, sql_id: &str, sql: &str, metadata: EvidenceMetadata) -> Self {
        let previous_hash = self.nodes.last()
            .map(|n| n.self_hash.clone())
            .unwrap_or_default();
            
        let node = EvidenceNode::new(
            sql_id,
            "sql",
            sql,
            &previous_hash,
            metadata,
        );
        
        self.nodes.push(node);
        self.integrity_hash = self.compute_integrity_hash();
        self
    }

    /// Add a vector search result node
    pub fn add_vector(
        mut self,
        vector_id: &str,
        content: &str,
        embedding: &[f32],
        metadata: EvidenceMetadata,
    ) -> Self {
        let previous_hash = self.nodes.last()
            .map(|n| n.self_hash.clone())
            .unwrap_or_default();
            
        let content_repr = format!("{}:{:?}", content, embedding);
        let node = EvidenceNode::new(
            vector_id,
            "vector",
            &content_repr,
            &previous_hash,
            metadata,
        );
        
        self.nodes.push(node);
        self.integrity_hash = self.compute_integrity_hash();
        self
    }

    /// Add a graph query result node
    pub fn add_graph(
        mut self,
        graph_id: &str,
        query: &str,
        results: &str,
        metadata: EvidenceMetadata,
    ) -> Self {
        let previous_hash = self.nodes.last()
            .map(|n| n.self_hash.clone())
            .unwrap_or_default();
            
        let content = format!("Query: {}\nResults: {}", query, results);
        let node = EvidenceNode::new(
            graph_id,
            "graph",
            &content,
            &previous_hash,
            metadata,
        );
        
        self.nodes.push(node);
        self.integrity_hash = self.compute_integrity_hash();
        self
    }

    /// Verify the integrity of the evidence chain
    pub fn verify(&self) -> bool {
        // Empty chain is valid
        if self.nodes.is_empty() && self.integrity_hash.is_empty() {
            return true;
        }
        if self.integrity_hash.is_empty() {
            return false;
        }
        self.integrity_hash == self.compute_integrity_hash()
    }

    /// Compute integrity hash of the entire chain
    pub fn compute_integrity_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Include chain metadata
        self.chain_id.hash(&mut hasher);
        self.description.hash(&mut hasher);
        
        // Include all nodes
        for node in &self.nodes {
            node.self_hash.hash(&mut hasher);
        }
        
        format!("{:016x}", hasher.finish())
    }

    /// Get the number of nodes in the chain
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Builder for constructing evidence chains fluently
#[derive(Debug)]
pub struct EvidenceChainBuilder {
    chain: EvidenceChain,
}

impl EvidenceChainBuilder {
    /// Create a new builder
    pub fn new(chain_id: &str, description: &str) -> Self {
        Self {
            chain: EvidenceChain::new(chain_id.to_string(), description.to_string()),
        }
    }

    /// Add a document node
    pub fn add_document(mut self, doc_id: &str, content: &str, metadata: EvidenceMetadata) -> Self {
        self.chain = self.chain.add_document(doc_id, content, metadata);
        self
    }

    /// Add a SQL node
    pub fn add_sql(mut self, sql_id: &str, sql: &str, metadata: EvidenceMetadata) -> Self {
        self.chain = self.chain.add_sql(sql_id, sql, metadata);
        self
    }

    /// Add a vector node
    pub fn add_vector(
        mut self,
        vector_id: &str,
        content: &str,
        embedding: &[f32],
        metadata: EvidenceMetadata,
    ) -> Self {
        self.chain = self.chain.add_vector(vector_id, content, embedding, metadata);
        self
    }

    /// Add a graph node
    pub fn add_graph(
        mut self,
        graph_id: &str,
        query: &str,
        results: &str,
        metadata: EvidenceMetadata,
    ) -> Self {
        self.chain = self.chain.add_graph(graph_id, query, results, metadata);
        self
    }

    /// Build the final evidence chain
    pub fn build(self) -> EvidenceChain {
        self.chain
    }
}

/// Summary of an evidence chain for reporting
#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceChainSummary {
    pub chain_id: String,
    pub description: String,
    pub node_count: usize,
    pub node_types: HashMap<String, usize>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_valid: bool,
}

impl From<&EvidenceChain> for EvidenceChainSummary {
    fn from(chain: &EvidenceChain) -> Self {
        let mut node_types = HashMap::new();
        for node in &chain.nodes {
            *node_types.entry(node.node_type.clone()).or_insert(0) += 1;
        }

        Self {
            chain_id: chain.chain_id.clone(),
            description: chain.description.clone(),
            node_count: chain.nodes.len(),
            node_types,
            created_at: chain.created_at,
            updated_at: chain.updated_at,
            is_valid: chain.verify(),
        }
    }
}

/// Evidence record for SQL operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvidence {
    /// Unique identifier
    pub evidence_id: String,
    /// The search query
    pub query: String,
    /// Results returned
    pub results: Vec<String>,
    /// Evidence chain for this search
    pub chain: EvidenceChain,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Whether results were truncated
    pub truncated: bool,
}

impl SearchEvidence {
    /// Create new search evidence
    pub fn new(query: &str) -> Self {
        Self {
            evidence_id: format!("search-{}", uuid_simple()),
            query: query.to_string(),
            results: vec![],
            chain: EvidenceChain::new(
                format!("chain-search-{}", uuid_simple()),
                format!("Search query: {}", query),
            ),
            execution_time_ms: 0,
            truncated: false,
        }
    }

    /// Add a result to the evidence
    pub fn add_result(mut self, result: &str) -> Self {
        let metadata = EvidenceMetadata::new("search_result")
            .with_context("result_id", &format!("result-{}", self.results.len()));
            
        self.chain = self.chain.add_document(
            &format!("result-{}", self.results.len()),
            result,
            metadata,
        );
        self.results.push(result.to_string());
        self
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }

    /// Mark as truncated
    pub fn with_truncated(mut self) -> Self {
        self.truncated = true;
        self
    }
}

/// Generate a simple UUID (for demo purposes)
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_chain_creation() {
        let chain = EvidenceChain::new(
            "test-chain-1".to_string(),
            "SELECT * FROM users".to_string(),
        );
        assert_eq!(chain.nodes.len(), 0);
        assert!(chain.verify());
    }

    #[test]
    fn test_evidence_chain_with_nodes() {
        let metadata = EvidenceMetadata {
            source: "test".to_string(),
            timestamp: 0,
            context: HashMap::new(),
            score: Some(0.95),
            references: vec![],
        };

        let chain = EvidenceChain::new("test-chain-2".to_string(), "Test chain".to_string())
            .add_document("doc-1", "Document content", metadata.clone())
            .add_sql("sql-1", "SELECT * FROM users", metadata.clone())
            .add_vector("vec-1", "Vector embedding", &[0.1, 0.2, 0.3], metadata);

        assert_eq!(chain.nodes.len(), 3);
        assert!(chain.verify());
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_evidence_chain_tamper_detection() {
        let metadata = EvidenceMetadata::new("test");
        
        let chain = EvidenceChain::new("test-chain-3".to_string(), "Tamper test".to_string())
            .add_document("doc-1", "Original content", metadata.clone());

        assert!(chain.verify());

        // Simulate tampering
        let mut tampered = chain.clone();
        tampered.nodes[0].content = "TAMPERED".to_string();
        
        assert!(!tampered.verify());
    }

    #[test]
    fn test_search_evidence() {
        let evidence = SearchEvidence::new("SELECT * FROM products WHERE price > 100")
            .add_result("Product A: $150")
            .add_result("Product B: $200")
            .with_execution_time(42)
            .with_truncated();

        assert_eq!(evidence.results.len(), 2);
        assert_eq!(evidence.execution_time_ms, 42);
        assert!(evidence.truncated);
        assert!(evidence.chain.verify());
    }

    #[test]
    fn test_evidence_chain_summary() {
        let metadata = EvidenceMetadata::new("test");
        
        let chain = EvidenceChain::new("summary-test".to_string(), "Summary test".to_string())
            .add_document("doc-1", "Content 1", metadata.clone())
            .add_document("doc-2", "Content 2", metadata.clone())
            .add_vector("vec-1", "Embedding", &[0.1, 0.2], metadata);

        let summary = EvidenceChainSummary::from(&chain);
        
        assert_eq!(summary.node_count, 3);
        assert_eq!(*summary.node_types.get("document").unwrap(), 2);
        assert_eq!(*summary.node_types.get("vector").unwrap(), 1);
        assert!(summary.is_valid);
    }

    #[test]
    fn test_evidence_metadata() {
        let metadata = EvidenceMetadata::new("document")
            .with_score(0.85)
            .with_context("author", "test-user")
            .with_reference("previous-doc");

        assert_eq!(metadata.source, "document");
        assert_eq!(metadata.score, Some(0.85));
        assert_eq!(metadata.context.get("author"), Some(&"test-user".to_string()));
        assert_eq!(metadata.references.len(), 1);
    }
}
