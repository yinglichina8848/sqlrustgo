use crate::api::{FusionScore, Weights};
use crate::error::QueryResult;
use crate::executor::ParallelQueryResults;

pub struct ResultFusion;

pub struct FusionResult {
    pub scores: Vec<FusionScore>,
    pub total: usize,
}

impl ResultFusion {
    pub fn new() -> Self {
        Self
    }

    pub fn fuse(
        &self,
        results: ParallelQueryResults,
        weights: &Weights,
        top_k: u32,
    ) -> FusionResult {
        let mut all_scores: Vec<FusionScore> = Vec::new();

        if let Some(QueryResult::Ok(rows)) = results.sql_results {
            for (idx, _row) in rows.iter().enumerate() {
                let sql_score = 1.0 - (idx as f32 * 0.01).min(0.5);
                all_scores.push(FusionScore {
                    id: format!("sql_{}", idx),
                    score: weights.sql * sql_score,
                    source: vec!["sql".to_string()],
                });
            }
        }

        if let Some(QueryResult::Ok(results)) = results.vector_results {
            for result in results {
                all_scores.push(FusionScore {
                    id: result.id.clone(),
                    score: weights.vector * result.score,
                    source: vec!["vector".to_string()],
                });
            }
        }

        if let Some(QueryResult::Ok(results)) = results.graph_results {
            for result in results {
                let path_id = result.path.join("_");
                all_scores.push(FusionScore {
                    id: path_id,
                    score: weights.graph * result.score,
                    source: vec!["graph".to_string()],
                });
            }
        }

        all_scores.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut seen: std::collections::HashMap<String, FusionScore> =
            std::collections::HashMap::new();
        for score in &all_scores {
            seen.entry(score.id.clone())
                .and_modify(|existing| {
                    if score.score > existing.score {
                        *existing = score.clone();
                    }
                })
                .or_insert_with(|| score.clone());
        }

        let mut final_scores: Vec<FusionScore> = seen.into_values().collect();
        final_scores.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total = final_scores.len();
        final_scores.truncate(top_k as usize);

        FusionResult {
            total,
            scores: final_scores,
        }
    }
}

impl Default for ResultFusion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_sum_to_one() {
        let weights = Weights::default();
        let sum = weights.sql + weights.vector + weights.graph;
        assert!((sum - 1.0).abs() < 0.001);
    }
}
