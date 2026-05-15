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

use crate::evidence::{EvidenceChain, EvidenceChainBuilder, EvidenceMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ImmutableRecord {
    chain: EvidenceChain,
}

impl ImmutableRecord {
    pub fn new(chain_id: &str, description: &str, content: &str) -> Self {
        let metadata = EvidenceMetadata::new("immutable_record");
        let chain = EvidenceChain::new(chain_id.to_string(), description.to_string())
            .add_document("initial", content, metadata);
        Self { chain }
    }

    pub fn from_chain(chain: EvidenceChain) -> Self {
        Self { chain }
    }

    pub fn chain(&self) -> &EvidenceChain {
        &self.chain
    }

    pub fn add_document(&mut self, node_id: &str, content: &str) {
        let metadata = EvidenceMetadata::new("document");
        let new_chain = self.chain.clone().add_document(node_id, content, metadata);
        self.chain = new_chain;
    }

    pub fn add_sql(&mut self, node_id: &str, sql: &str) {
        let metadata = EvidenceMetadata::new("sql");
        let new_chain = self.chain.clone().add_sql(node_id, sql, metadata);
        self.chain = new_chain;
    }

    pub fn add_vector(&mut self, node_id: &str, content: &str, embedding: &[f32]) {
        let metadata = EvidenceMetadata::new("vector");
        let new_chain = self
            .chain
            .clone()
            .add_vector(node_id, content, embedding, metadata);
        self.chain = new_chain;
    }

    pub fn add_graph(&mut self, node_id: &str, query: &str, results: &str) {
        let metadata = EvidenceMetadata::new("graph");
        let new_chain = self
            .chain
            .clone()
            .add_graph(node_id, query, results, metadata);
        self.chain = new_chain;
    }

    pub fn verify(&self) -> bool {
        self.chain.verify()
    }

    pub fn chain_id(&self) -> &str {
        &self.chain.chain_id
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
}

impl Default for ImmutableRecord {
    fn default() -> Self {
        let metadata = EvidenceMetadata::new("immutable_record");
        Self {
            chain: EvidenceChain::new("default".to_string(), "Default record".to_string())
                .add_document("initial", "", metadata),
        }
    }
}

pub struct ImmutableRecordBuilder {
    chain_id: String,
    description: String,
    metadata: HashMap<String, String>,
}

impl ImmutableRecordBuilder {
    pub fn new(chain_id: &str, description: &str) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            description: description.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_creator(self, creator_id: &str) -> Self {
        self.with_metadata("creator_id", creator_id)
    }

    pub fn with_operation(self, operation: &str) -> Self {
        self.with_metadata("operation", operation)
    }

    pub fn build(self, initial_content: &str) -> ImmutableRecord {
        let mut metadata =
            EvidenceMetadata::new("immutable_record").with_context("chain_id", &self.chain_id);

        for (key, value) in &self.metadata {
            metadata = metadata.with_context(key, value);
        }

        let chain = EvidenceChain::new(self.chain_id, self.description).add_document(
            "initial",
            initial_content,
            metadata,
        );

        ImmutableRecord { chain }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub chain_id: String,
    pub is_valid: bool,
    pub node_count: usize,
    pub verification_time: i64,
    pub errors: Vec<String>,
}

impl VerificationReport {
    pub fn new(chain_id: &str, is_valid: bool, node_count: usize) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            is_valid,
            node_count,
            verification_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_record_create() {
        let record = ImmutableRecord::new("test-chain", "Test description", "Initial content");
        assert_eq!(record.chain_id(), "test-chain");
        assert!(!record.is_empty());
        assert!(record.verify());
    }

    #[test]
    fn test_immutable_record_verify() {
        let record = ImmutableRecord::new("verify-test", "Verification test", "Content");
        assert!(record.verify());
    }

    #[test]
    fn test_immutable_record_builder() {
        let record = ImmutableRecordBuilder::new("builder-test", "Builder test")
            .with_creator("user123")
            .with_operation("CREATE")
            .build("Initial content");

        assert_eq!(record.chain_id(), "builder-test");
        assert!(record.verify());
    }

    #[test]
    fn test_verification_report() {
        let mut report = VerificationReport::new("test-chain", true, 5);
        assert!(report.is_valid);
        assert_eq!(report.node_count, 5);
        assert!(report.errors.is_empty());

        report.add_error("Test error");
        assert_eq!(report.errors.len(), 1);
    }
}
