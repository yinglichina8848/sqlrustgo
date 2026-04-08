use serde::{Deserialize, Serialize};
use sqlrustgo_rag::{Document, RAGPipeline};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub timestamp: u64,
    pub tags: Vec<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub importance: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    Conversation,
    Query,
    Result,
    Schema,
    Policy,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMemoryRequest {
    pub content: String,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub importance: Option<f32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMemoryResponse {
    pub id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMemoryRequest {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub since: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMemoryResponse {
    pub memories: Vec<MemoryEntry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryRequest {
    pub query: String,
    pub agent_id: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryResponse {
    pub results: Vec<MemoryEntry>,
    pub total: usize,
    pub scores: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearMemoryRequest {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub older_than: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearMemoryResponse {
    pub cleared: usize,
    pub success: bool,
}

pub struct MemoryService {
    memories: HashMap<String, MemoryEntry>,
    agent_memories: HashMap<String, Vec<String>>,
    session_memories: HashMap<String, Vec<String>>,
    tag_memories: HashMap<String, Vec<String>>,
    rag_pipeline: RAGPipeline,
}

impl MemoryService {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            agent_memories: HashMap::new(),
            session_memories: HashMap::new(),
            tag_memories: HashMap::new(),
            rag_pipeline: RAGPipeline::new(),
        }
    }

    pub fn save_memory(&mut self, request: SaveMemoryRequest) -> SaveMemoryResponse {
        let memory_type = request.memory_type.unwrap_or(MemoryType::Custom);
        let tags = request.tags.unwrap_or_default();
        let importance = request.importance.unwrap_or(0.5);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = format!("mem_{}_{}", timestamp, generate_random_suffix());

        let entry = MemoryEntry {
            id: id.clone(),
            content: request.content.clone(),
            memory_type,
            timestamp,
            tags: tags.clone(),
            agent_id: request.agent_id.clone(),
            session_id: request.session_id.clone(),
            importance,
            metadata: request.metadata.unwrap_or_default(),
        };

        self.memories.insert(id.clone(), entry);

        // Add to RAG pipeline for full-text search
        let doc_id = generate_doc_id_from_mem_id(&id);
        self.rag_pipeline
            .add_document(Document::new(doc_id, request.content.clone()));

        if let Some(ref agent_id) = request.agent_id {
            self.agent_memories
                .entry(agent_id.clone())
                .or_default()
                .push(id.clone());
        }

        if let Some(ref session_id) = request.session_id {
            self.session_memories
                .entry(session_id.clone())
                .or_default()
                .push(id.clone());
        }

        for tag in &tags {
            self.tag_memories
                .entry(tag.clone())
                .or_default()
                .push(id.clone());
        }

        SaveMemoryResponse {
            id,
            success: true,
            message: "Memory saved successfully".to_string(),
        }
    }

    pub fn load_memory(&self, request: LoadMemoryRequest) -> LoadMemoryResponse {
        let mut candidates: Vec<&MemoryEntry> = self.memories.values().collect();

        if let Some(ref agent_id) = request.agent_id {
            if let Some(memory_ids) = self.agent_memories.get(agent_id) {
                candidates.retain(|m| memory_ids.contains(&m.id));
            } else {
                return LoadMemoryResponse {
                    memories: vec![],
                    total: 0,
                };
            }
        }

        if let Some(ref session_id) = request.session_id {
            candidates.retain(|m| m.session_id.as_ref() == Some(session_id));
        }

        if let Some(ref memory_type) = request.memory_type {
            candidates.retain(|m| m.memory_type == *memory_type);
        }

        if let Some(tags) = &request.tags {
            for tag in tags {
                if let Some(memory_ids) = self.tag_memories.get(tag) {
                    candidates.retain(|m| memory_ids.contains(&m.id));
                }
            }
        }

        if let Some(since) = request.since {
            candidates.retain(|m| m.timestamp >= since);
        }

        candidates.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let limit = request.limit.unwrap_or(100);
        candidates.truncate(limit);

        let total = candidates.len();
        LoadMemoryResponse {
            memories: candidates.into_iter().cloned().collect(),
            total,
        }
    }

    pub fn search_memory(&self, request: SearchMemoryRequest) -> SearchMemoryResponse {
        // Get agent-filtered candidates first
        let candidates: Vec<&MemoryEntry> = if let Some(ref agent_id) = request.agent_id {
            if let Some(memory_ids) = self.agent_memories.get(agent_id) {
                self.memories
                    .values()
                    .filter(|m| memory_ids.contains(&m.id))
                    .collect()
            } else {
                return SearchMemoryResponse {
                    results: vec![],
                    total: 0,
                    scores: vec![],
                };
            }
        } else {
            self.memories.values().collect()
        };

        // Apply memory_type filter
        let candidates: Vec<&MemoryEntry> = if let Some(ref memory_type) = request.memory_type {
            candidates
                .into_iter()
                .filter(|m| m.memory_type == *memory_type)
                .collect()
        } else {
            candidates
        };

        // Use RAG pipeline for full-text search with Chinese support
        let limit = request.limit.unwrap_or(10);
        let rag_results = self.rag_pipeline.search(&request.query, limit * 2);

        // Map RAG doc_ids back to memory entries and filter by candidates
        let mut scored: Vec<(&MemoryEntry, f32)> = Vec::new();
        for (doc_id, _content) in rag_results {
            // Find memory entry by doc_id
            if let Some(memory) = candidates.iter().find(|m| {
                let mem_doc_id = generate_doc_id_from_mem_id(&m.id);
                mem_doc_id == doc_id
            }) {
                // RAG returned this document, it matches the query
                // Use importance as additional scoring factor
                let score = 1.0 + memory.importance;
                scored.push((memory, score));
            }
        }

        // If RAG returned no results, fall back to keyword matching for remaining candidates
        if scored.is_empty() {
            let query_lower = request.query.to_lowercase();
            let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

            for m in &candidates {
                let mut score = 0.0;
                let content_lower = m.content.to_lowercase();
                let mut has_match = false;

                for term in &query_terms {
                    if content_lower.contains(term) {
                        has_match = true;
                        score += 1.0;
                        if content_lower.starts_with(term) {
                            score += 0.5;
                        }
                    }
                }

                if content_lower.contains(&query_lower) {
                    score += 2.0;
                }

                if has_match {
                    score += m.importance;
                    scored.push((m, score));
                }
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        SearchMemoryResponse {
            results: scored.iter().map(|(m, _)| (*m).clone()).collect(),
            total: scored.len(),
            scores: scored.iter().map(|(_, s)| *s).collect(),
        }
    }

    pub fn clear_memory(&mut self, request: ClearMemoryRequest) -> ClearMemoryResponse {
        let mut to_remove: Vec<String> = Vec::new();

        if let Some(ref agent_id) = request.agent_id {
            if let Some(memory_ids) = self.agent_memories.get(agent_id) {
                to_remove.extend(memory_ids.clone());
            }
        }

        if let Some(ref session_id) = request.session_id {
            if let Some(memory_ids) = self.session_memories.get(session_id) {
                for id in memory_ids {
                    if !to_remove.contains(id) {
                        to_remove.push(id.clone());
                    }
                }
            }
        }

        if let Some(older_than) = request.older_than {
            for (id, memory) in &self.memories {
                if memory.timestamp < older_than && !to_remove.contains(id) {
                    to_remove.push(id.clone());
                }
            }
        }

        let cleared = to_remove.len();
        for id in &to_remove {
            if let Some(memory) = self.memories.remove(id) {
                // Remove from RAG pipeline
                let doc_id = generate_doc_id_from_mem_id(id);
                self.rag_pipeline.delete_document(doc_id);

                if let Some(ref agent_id) = memory.agent_id {
                    if let Some(ids) = self.agent_memories.get_mut(agent_id) {
                        ids.retain(|i| i != id);
                    }
                }
                if let Some(ref session_id) = memory.session_id {
                    if let Some(ids) = self.session_memories.get_mut(session_id) {
                        ids.retain(|i| i != id);
                    }
                }
                for tag in &memory.tags {
                    if let Some(ids) = self.tag_memories.get_mut(tag) {
                        ids.retain(|i| i != id);
                    }
                }
            }
        }

        ClearMemoryResponse {
            cleared,
            success: true,
        }
    }

    pub fn get_stats(&self) -> MemoryStats {
        let mut type_counts = HashMap::new();
        for memory in self.memories.values() {
            let type_str = format!("{:?}", memory.memory_type).to_lowercase();
            *type_counts.entry(type_str).or_insert(0) += 1;
        }

        MemoryStats {
            total_memories: self.memories.len(),
            by_type: type_counts,
            unique_agents: self.agent_memories.len(),
            unique_sessions: self.session_memories.len(),
            unique_tags: self.tag_memories.len(),
        }
    }

    pub fn get_memory(&self, id: &str) -> Option<MemoryEntry> {
        self.memories.get(id).cloned()
    }

    pub fn delete_memory(&mut self, id: &str) -> bool {
        if let Some(memory) = self.memories.remove(id) {
            // Remove from RAG pipeline
            let doc_id = generate_doc_id_from_mem_id(id);
            self.rag_pipeline.delete_document(doc_id);

            if let Some(ref agent_id) = memory.agent_id {
                if let Some(ids) = self.agent_memories.get_mut(agent_id) {
                    ids.retain(|i| i != id);
                }
            }
            if let Some(ref session_id) = memory.session_id {
                if let Some(ids) = self.session_memories.get_mut(session_id) {
                    ids.retain(|i| i != id);
                }
            }
            for tag in &memory.tags {
                if let Some(ids) = self.tag_memories.get_mut(tag) {
                    ids.retain(|i| i != id);
                }
            }
            true
        } else {
            false
        }
    }
}

impl Default for MemoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub by_type: HashMap<String, usize>,
    pub unique_agents: usize,
    pub unique_sessions: usize,
    pub unique_tags: usize,
}

fn generate_random_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", duration.as_secs(), duration.subsec_nanos())
}

/// Generate a u64 doc_id from a memory id string for RAG pipeline
fn generate_doc_id_from_mem_id(mem_id: &str) -> u64 {
    // Use a hash of the memory id to generate a consistent u64
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    mem_id.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_memory() {
        let mut service = MemoryService::new();
        let request = SaveMemoryRequest {
            content: "Test memory".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: Some(vec!["test".to_string()]),
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: Some(0.8),
            metadata: None,
        };

        let response = service.save_memory(request);
        assert!(response.success);
        assert!(!response.id.is_empty());
    }

    #[test]
    fn test_load_memory() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "Memory 1".to_string(),
            memory_type: Some(MemoryType::Query),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let response = service.load_memory(LoadMemoryRequest {
            agent_id: Some("agent1".to_string()),
            session_id: None,
            memory_type: None,
            tags: None,
            limit: None,
            since: None,
        });

        assert_eq!(response.total, 1);
        assert_eq!(response.memories[0].content, "Memory 1");
    }

    #[test]
    fn test_search_memory() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "How to query users".to_string(),
            memory_type: Some(MemoryType::Query),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        service.save_memory(SaveMemoryRequest {
            content: "Product information".to_string(),
            memory_type: Some(MemoryType::Result),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let response = service.search_memory(SearchMemoryRequest {
            query: "query".to_string(),
            agent_id: Some("agent1".to_string()),
            memory_type: None,
            limit: Some(10),
        });

        assert_eq!(response.total, 1);
        assert!(response.results[0].content.contains("query"));
    }

    #[test]
    fn test_clear_memory() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "Old memory".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let response = service.clear_memory(ClearMemoryRequest {
            agent_id: Some("agent1".to_string()),
            session_id: None,
            memory_type: None,
            older_than: None,
        });

        assert_eq!(response.cleared, 1);
        assert!(response.success);
    }

    #[test]
    fn test_memory_stats() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "Memory 1".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        service.save_memory(SaveMemoryRequest {
            content: "Memory 2".to_string(),
            memory_type: Some(MemoryType::Query),
            tags: None,
            agent_id: Some("agent2".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        let stats = service.get_stats();
        assert_eq!(stats.total_memories, 2);
        assert_eq!(stats.unique_agents, 2);
    }

    #[test]
    fn test_delete_memory() {
        let mut service = MemoryService::new();

        let response = service.save_memory(SaveMemoryRequest {
            content: "To be deleted".to_string(),
            memory_type: None,
            tags: None,
            agent_id: None,
            session_id: None,
            importance: None,
            metadata: None,
        });

        let deleted = service.delete_memory(&response.id);
        assert!(deleted);
        assert!(service.get_memory(&response.id).is_none());
    }

    #[test]
    fn test_load_with_session_id() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "Session 1 memory".to_string(),
            memory_type: None,
            tags: None,
            agent_id: None,
            session_id: Some("session1".to_string()),
            importance: None,
            metadata: None,
        });

        service.save_memory(SaveMemoryRequest {
            content: "Session 2 memory".to_string(),
            memory_type: None,
            tags: None,
            agent_id: None,
            session_id: Some("session2".to_string()),
            importance: None,
            metadata: None,
        });

        let response = service.load_memory(LoadMemoryRequest {
            agent_id: None,
            session_id: Some("session1".to_string()),
            memory_type: None,
            tags: None,
            limit: None,
            since: None,
        });

        assert_eq!(response.total, 1);
        assert!(response.memories[0].content.contains("Session 1"));
    }

    #[test]
    fn test_search_memory_chinese() {
        let mut service = MemoryService::new();

        // Save Chinese memories
        service.save_memory(SaveMemoryRequest {
            content: "Rust编程语言性能优异".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: Some(0.8),
            metadata: None,
        });

        service.save_memory(SaveMemoryRequest {
            content: "Python适合数据科学和机器学习".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: Some(0.7),
            metadata: None,
        });

        // Search with Chinese query
        let response = service.search_memory(SearchMemoryRequest {
            query: "Rust 性能".to_string(),
            agent_id: Some("agent1".to_string()),
            memory_type: None,
            limit: Some(10),
        });

        // Should find the Rust memory
        assert!(response.total >= 1, "Should find at least one result");
        assert!(response.results.iter().any(|m| m.content.contains("Rust")));
    }

    #[test]
    fn test_search_memory_multilingual() {
        let mut service = MemoryService::new();

        service.save_memory(SaveMemoryRequest {
            content: "Hello world".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        service.save_memory(SaveMemoryRequest {
            content: "你好世界".to_string(),
            memory_type: Some(MemoryType::Conversation),
            tags: None,
            agent_id: Some("agent1".to_string()),
            session_id: None,
            importance: None,
            metadata: None,
        });

        // Search Chinese
        let response_cn = service.search_memory(SearchMemoryRequest {
            query: "你好".to_string(),
            agent_id: Some("agent1".to_string()),
            memory_type: None,
            limit: Some(10),
        });
        assert!(response_cn.total >= 1);
        assert!(response_cn
            .results
            .iter()
            .any(|m| m.content.contains("你好")));

        // Search English
        let response_en = service.search_memory(SearchMemoryRequest {
            query: "Hello".to_string(),
            agent_id: Some("agent1".to_string()),
            memory_type: None,
            limit: Some(10),
        });
        assert!(response_en.total >= 1);
        assert!(response_en
            .results
            .iter()
            .any(|m| m.content.contains("Hello")));
    }
}
