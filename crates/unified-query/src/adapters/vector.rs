use crate::api::{UnifiedQueryRequest, VectorResult};
use crate::error::QueryResult;
use crate::QueryPlan;
use sqlrustgo_vector::{DistanceMetric, FlatIndex, VectorIndex};

/// Vector adapter for similarity search
pub struct VectorAdapter {
    index: FlatIndex,
}

impl VectorAdapter {
    pub fn new() -> Self {
        // Default to cosine metric for similarity search
        Self {
            index: FlatIndex::new(DistanceMetric::Cosine),
        }
    }

    /// Create with specific metric
    pub fn with_metric(metric: DistanceMetric) -> Self {
        Self {
            index: FlatIndex::new(metric),
        }
    }

    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        plan: &QueryPlan,
    ) -> QueryResult<Vec<VectorResult>> {
        // Extract vector query from request
        let vector_query = match &request.vector_query {
            Some(vq) => vq,
            None => {
                // If no explicit vector query but in vector mode,
                // try to extract from graph_query or generate from text
                return QueryResult::Err("No vector query provided".to_string());
            }
        };

        // Extract query embedding (in production, this would use an embedding model)
        let query_embedding = self.extract_embedding(&request.query, vector_query);

        match query_embedding {
            Some(embedding) => {
                let top_k = if vector_query.top_k > 0 {
                    vector_query.top_k as usize
                } else {
                    plan.top_k as usize
                };

                match self.index.search(&embedding, top_k) {
                    Ok(results) => {
                        let vector_results: Vec<VectorResult> = results
                            .into_iter()
                            .map(|entry| VectorResult {
                                id: entry.id.to_string(),
                                score: entry.score,
                                data: serde_json::json!({
                                    "embedding_id": entry.id,
                                    "distance": entry.score
                                }),
                            })
                            .collect();

                        QueryResult::Ok(vector_results)
                    }
                    Err(e) => QueryResult::Err(format!("Vector search error: {}", e)),
                }
            }
            None => QueryResult::Err("Could not extract embedding from query".to_string()),
        }
    }

    /// Extract embedding from query text
    /// In production, this would call an embedding model (e.g., OpenAI, local model)
    /// For now, we generate a simple hash-based embedding for demonstration
    fn extract_embedding(
        &self,
        query: &str,
        _vector_query: &crate::api::VectorQuery,
    ) -> Option<Vec<f32>> {
        // If there's a column specified, we would look up the vector for that column
        // For now, generate a pseudo-embedding based on query hash
        let dim = self.index.dimension();
        if dim == 0 {
            return None;
        }

        let mut embedding = vec![0.0f32; dim];
        let bytes = query.as_bytes();

        for (i, byte) in bytes.iter().enumerate() {
            let idx = i % dim;
            embedding[idx] = (*byte as f32) / 255.0;
        }

        // Normalize
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut embedding {
                *v /= magnitude;
            }
        }

        Some(embedding)
    }

    /// Add a vector to the index
    pub fn insert(&mut self, id: u64, vector: Vec<f32>) -> Result<(), String> {
        self.index.insert(id, &vector).map_err(|e| e.to_string())
    }

    /// Build the index for searching
    pub fn build_index(&mut self) -> Result<(), String> {
        self.index.build_index().map_err(|e| e.to_string())
    }

    /// Get index dimension
    pub fn dimension(&self) -> usize {
        self.index.dimension()
    }

    /// Get all vectors as records
    pub fn get_all(&self) -> Vec<(u64, Vec<f32>)> {
        self.index
            .get_all()
            .into_iter()
            .map(|r| (r.id, r.vector))
            .collect()
    }
}

impl Default for VectorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_adapter_search() {
        let mut adapter = VectorAdapter::new();

        // Insert some vectors
        adapter.insert(1, vec![0.1, 0.2, 0.3]).unwrap();
        adapter.insert(2, vec![0.4, 0.5, 0.6]).unwrap();
        adapter.insert(3, vec![0.7, 0.8, 0.9]).unwrap();
        adapter.build_index().unwrap();

        let request = UnifiedQueryRequest {
            query: "test query".to_string(),
            mode: crate::api::QueryMode::Vector,
            filters: None,
            weights: None,
            vector_query: Some(crate::api::VectorQuery {
                column: "embedding".to_string(),
                top_k: 2,
                filter: None,
            }),
            graph_query: None,
            top_k: Some(2),
            offset: Some(0),
        };

        let plan = QueryPlan {
            execute_sql: false,
            execute_vector: true,
            execute_graph: false,
            weights: Default::default(),
            top_k: 2,
            offset: 0,
        };

        let results = adapter.execute(&request, &plan).await;
        assert!(results.is_ok());

        let vector_results = results.unwrap();
        assert!(!vector_results.is_empty());
    }
}
