//! QMD Bridge core trait and implementation

use crate::config::QmdConfig;
use crate::error::{QmdBridgeError, QmdResult};
use crate::hybrid::{HybridQuery, HybridResult, HybridSearchResultItem};
use crate::types::*;

/// Core trait for QMD Bridge operations
pub trait QmdBridge: Send + Sync {
    /// Synchronize data to QMD
    fn sync_to_qmd(&self, data: &QmdData) -> QmdResult<()>;

    /// Synchronize batch data to QMD
    fn sync_batch_to_qmd(&self, data: &[QmdData]) -> QmdResult<()>;

    /// Search from QMD
    fn search_from_qmd(&self, query: &QmdQuery) -> QmdResult<Vec<SearchResult>>;

    /// Hybrid search combining vector + graph + text
    fn hybrid_search(&self, query: &HybridQuery) -> QmdResult<HybridResult>;

    /// Get synchronization status
    fn sync_status(&self) -> QmdResult<SyncStatus>;
}

/// In-memory implementation of QmdBridge for testing
pub struct QmdBridgeImpl {
    config: QmdConfig,
    /// In-memory storage for testing
    storage: std::sync::Mutex<Vec<QmdData>>,
}

impl QmdBridgeImpl {
    /// Create a new QMD Bridge implementation
    pub fn new(config: QmdConfig) -> Self {
        Self {
            config,
            storage: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(QmdConfig::default())
    }

    /// Get configuration
    pub fn config(&self) -> &QmdConfig {
        &self.config
    }
}

impl Default for QmdBridgeImpl {
    fn default() -> Self {
        Self::new(QmdConfig::default())
    }
}

impl QmdBridge for QmdBridgeImpl {
    fn sync_to_qmd(&self, data: &QmdData) -> QmdResult<()> {
        let mut storage = self.storage.lock().map_err(|e| {
            QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;
        storage.push(data.clone());
        tracing::info!(id = %data.id, "Synced data to QMD");
        Ok(())
    }

    fn sync_batch_to_qmd(&self, data: &[QmdData]) -> QmdResult<()> {
        let mut storage = self.storage.lock().map_err(|e| {
            QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;
        for item in data {
            tracing::debug!(id = %item.id, "Syncing batch item");
            storage.push(item.clone());
        }
        tracing::info!(count = data.len(), "Synced batch to QMD");
        Ok(())
    }

    fn search_from_qmd(&self, query: &QmdQuery) -> QmdResult<Vec<SearchResult>> {
        let storage = self.storage.lock().map_err(|e| {
            QmdBridgeError::Search(format!("Failed to acquire lock: {}", e))
        })?;

        let mut results = Vec::new();
        for data in storage.iter() {
            // Apply filters
            let passes_filter = query.filters.is_empty()
                || {
                    query.filters.iter().all(|f| {
                        data.metadata
                            .get(&f.field)
                            .map_or(false, |v| match f.operator {
                                FilterOperator::Eq => v == &f.value,
                                FilterOperator::Contains => v.contains(&f.value),
                                _ => false,
                            })
                    })
                };

            if passes_filter {
                results.push(SearchResult {
                    id: data.id.clone(),
                    score: 1.0, // Placeholder
                    data: data.clone(),
                });
            }
        }

        // Apply limit
        results.truncate(query.limit);

        tracing::info!(
            query_type = ?query.query_type,
            results = results.len(),
            "Search completed"
        );
        Ok(results)
    }

    fn hybrid_search(&self, query: &HybridQuery) -> QmdResult<HybridResult> {
        // Perform vector search if query vector is provided
        let vector_results = if let Some(ref vec) = query.vector {
            let storage = self.storage.lock().map_err(|e| {
                QmdBridgeError::Search(format!("Failed to acquire lock: {}", e))
            })?;

            let mut results = Vec::new();
            for data in storage.iter() {
                if let Some(ref stored_vec) = data.vector {
                    if stored_vec.len() == vec.len() {
                        // Calculate cosine similarity as placeholder
                        let score = cosine_similarity(vec, stored_vec);
                        results.push(SearchResult {
                            id: data.id.clone(),
                            score,
                            data: data.clone(),
                        });
                    }
                }
            }

            // Sort by score descending
            results.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(query.limit);

            results
        } else {
            Vec::new()
        };

        // Graph results placeholder (would use graph module)
        let graph_results: Vec<SearchResult> = Vec::new();

        // Rerank results (placeholder - would use cross-encoder)
        let mut reranked_results = vector_results.clone();
        reranked_results.extend(graph_results.clone());
        reranked_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        reranked_results.truncate(query.limit);

        let scores: Vec<f32> = reranked_results.iter().map(|r| r.score).collect();

        // Convert SearchResult to HybridSearchResultItem
        let reranked: Vec<HybridSearchResultItem> = reranked_results
            .into_iter()
            .map(|r| HybridSearchResultItem {
                id: r.id,
                score: r.score,
                vector_score: None,
                graph_score: None,
                text_score: None,
                data: r.data,
            })
            .collect();

        Ok(HybridResult {
            vector_results,
            graph_results,
            text_results: vec![],
            results: reranked,
        })
    }

    fn sync_status(&self) -> QmdResult<SyncStatus> {
        let storage = self.storage.lock().map_err(|e| {
            QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        Ok(SyncStatus {
            last_sync: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            items_synced: storage.len() as u64,
            state: SyncState::Completed,
            error: None,
        })
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_sync() {
        let bridge = QmdBridgeImpl::default();
        let data = QmdData {
            id: "test1".to_string(),
            data_type: QmdDataType::Vector,
            vector: Some(vec![0.1, 0.2, 0.3]),
            graph: None,
            text: None,
            metadata: std::collections::HashMap::new(),
            timestamp: 0,
        };

        let result = bridge.sync_to_qmd(&data);
        assert!(result.is_ok());

        let status = bridge.sync_status().unwrap();
        assert_eq!(status.items_synced, 1);
    }

    #[test]
    fn test_bridge_search() {
        let bridge = QmdBridgeImpl::default();
        let data = QmdData {
            id: "test1".to_string(),
            data_type: QmdDataType::Vector,
            vector: Some(vec![0.1, 0.2, 0.3]),
            graph: None,
            text: None,
            metadata: std::collections::HashMap::new(),
            timestamp: 0,
        };
        bridge.sync_to_qmd(&data).unwrap();

        let query = QmdQuery {
            query_type: QueryType::Knn,
            vector: Some(vec![0.1, 0.2, 0.3]),
            graph_pattern: None,
            text: None,
            filters: vec![],
            limit: 10,
            threshold: None,
        };

        let results = bridge.search_from_qmd(&query).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
