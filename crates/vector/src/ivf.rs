//! IVF (Inverted File) index implementation with k-means clustering

use crate::error::{VectorError, VectorResult};
use crate::metrics::{compute_similarity, DistanceMetric};
use crate::traits::{IndexEntry, VectorIndex};

#[derive(Debug, Clone)]
struct ClusterCenter {
    id: u64,
    vector: Vec<f32>,
}

#[derive(Debug, Clone)]
struct Cluster {
    center: ClusterCenter,
    vector_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct IvfIndex {
    dimension: usize,
    metric: DistanceMetric,
    nlist: usize,
    vectors: Vec<(u64, Vec<f32>)>,
    clusters: Vec<Cluster>,
    built: bool,
}

impl IvfIndex {
    pub fn new(metric: DistanceMetric, nlist: usize) -> Self {
        Self {
            dimension: 0,
            metric,
            nlist,
            vectors: Vec::new(),
            clusters: Vec::new(),
            built: false,
        }
    }

    pub fn with_nlist(nlist: usize, metric: DistanceMetric) -> Self {
        Self::new(metric, nlist)
    }

    fn kmeans_init(vectors: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        if vectors.is_empty() || k == 0 {
            return Vec::new();
        }
        let k = k.min(vectors.len());
        let mut centers: Vec<Vec<f32>> = Vec::with_capacity(k);
        centers.push(vectors[0].clone());

        for _ in 1..k {
            let mut distances: Vec<f32> = Vec::with_capacity(vectors.len());
            for v in vectors {
                let min_dist = centers
                    .iter()
                    .map(|c| Self::euclidean(v, c))
                    .fold(f32::MAX, f32::min);
                distances.push(min_dist);
            }
            let total: f32 = distances.iter().sum();
            let mut threshold = rand::random::<f32>() * total;
            let mut selected = 0;
            for (i, d) in distances.iter().enumerate() {
                threshold -= d;
                if threshold <= 0.0 {
                    selected = i;
                    break;
                }
            }
            centers.push(vectors[selected].clone());
        }
        centers
    }

    fn euclidean(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    fn kmeans_assign(vectors: &[Vec<f32>], centers: &[Vec<f32>]) -> Vec<usize> {
        vectors
            .iter()
            .map(|v| {
                centers
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, Self::euclidean(v, c)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap()
                    .0
            })
            .collect()
    }

    fn kmeans_update(vectors: &[Vec<f32>], assignments: &[usize], k: usize) -> Vec<Vec<f32>> {
        let dim = vectors.first().map(|v| v.len()).unwrap_or(0);
        let mut new_centers: Vec<Vec<f32>> = vec![vec![0.0; dim]; k];

        let mut counts: Vec<usize> = vec![0; k];
        for (v, &cluster) in vectors.iter().zip(assignments.iter()) {
            counts[cluster] += 1;
            for (i, val) in v.iter().enumerate() {
                new_centers[cluster][i] += val;
            }
        }

        for (i, center) in new_centers.iter_mut().enumerate() {
            if counts[i] > 0 {
                for val in center.iter_mut() {
                    *val /= counts[i] as f32;
                }
            }
        }
        new_centers
    }

    fn kmeans(vectors: &[Vec<f32>], k: usize, max_iter: usize) -> Vec<Vec<f32>> {
        if vectors.is_empty() || k == 0 {
            return Vec::new();
        }
        let k = k.min(vectors.len());
        let mut centers = Self::kmeans_init(vectors, k);

        for _ in 0..max_iter {
            let assignments = Self::kmeans_assign(vectors, &centers);
            let new_centers = Self::kmeans_update(vectors, &assignments, k);

            let diff: f32 = centers
                .iter()
                .zip(new_centers.iter())
                .map(|(a, b)| Self::euclidean(a, b))
                .sum();

            centers = new_centers;
            if diff < 1e-4 {
                break;
            }
        }
        centers
    }

    fn build_clusters(&mut self) -> VectorResult<()> {
        if self.vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        let nlist = self.nlist.min(self.vectors.len());
        let vectors_only: Vec<_> = self.vectors.iter().map(|(_, v)| v.clone()).collect();
        let centers = Self::kmeans(&vectors_only, nlist, 50);

        self.clusters = centers
            .into_iter()
            .enumerate()
            .map(|(i, vector)| Cluster {
                center: ClusterCenter {
                    id: i as u64,
                    vector,
                },
                vector_ids: Vec::new(),
            })
            .collect();

        let assignments = Self::kmeans_assign(
            &vectors_only,
            &self
                .clusters
                .iter()
                .map(|c| c.center.vector.clone())
                .collect::<Vec<_>>(),
        );

        for ((id, _), &cluster) in self.vectors.iter().zip(assignments.iter()) {
            if let Some(c) = self.clusters.get_mut(cluster) {
                c.vector_ids.push(*id);
            }
        }

        self.built = true;
        Ok(())
    }
}

impl VectorIndex for IvfIndex {
    fn insert(&mut self, id: u64, vector: &[f32]) -> VectorResult<()> {
        if self.dimension == 0 {
            self.dimension = vector.len();
        } else if vector.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            });
        }

        self.vectors.push((id, vector.to_vec()));
        self.built = false;
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorResult<Vec<IndexEntry>> {
        if !self.built {
            return Err(VectorError::IndexNotBuilt);
        }

        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            });
        }

        let mut cluster_scores: Vec<_> = self
            .clusters
            .iter()
            .map(|c| {
                let score = compute_similarity(query, &c.center.vector, self.metric);
                (c.center.id, score)
            })
            .collect();

        cluster_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let nprobe = (self.clusters.len() as f32 * 0.1).ceil() as usize;
        let selected_ids: Vec<_> = cluster_scores
            .iter()
            .take(nprobe.max(1))
            .map(|(id, _)| *id)
            .collect();

        let id_to_vector: std::collections::HashMap<u64, &Vec<f32>> =
            self.vectors.iter().map(|(id, v)| (*id, v)).collect();

        let mut candidates: Vec<_> = Vec::new();

        for &cluster_id in &selected_ids {
            if let Some(cluster) = self.clusters.iter().find(|c| c.center.id == cluster_id) {
                for &vid in &cluster.vector_ids {
                    if let Some(v) = id_to_vector.get(&vid) {
                        let score = compute_similarity(query, v, self.metric);
                        candidates.push(IndexEntry::new(vid, score));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(candidates.into_iter().take(k).collect())
    }

    fn build_index(&mut self) -> VectorResult<()> {
        self.build_clusters()
    }

    fn delete(&mut self, _id: u64) -> VectorResult<()> {
        self.built = false;
        Ok(())
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn metric(&self) -> DistanceMetric {
        self.metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ivf_index() {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 2);
        for i in 0..100 {
            let v = vec![i as f32, i as f32];
            index.insert(i, &v).unwrap();
        }
        index.build_index().unwrap();

        let results = index.search(&[50.0, 50.0], 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_ivf_index_dimension_mismatch() {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 2);
        index.insert(1, &[1.0, 0.0]).unwrap();
        let result = index.insert(2, &[1.0, 0.0, 0.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_ivf_index_search_before_build() {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 2);
        index.insert(1, &[1.0, 0.0]).unwrap();
        let result = index.search(&[1.0, 0.0], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_ivf_index_with_nlist() {
        let mut index = IvfIndex::with_nlist(5, DistanceMetric::Cosine);
        for i in 0..50 {
            let v = vec![i as f32, (50 - i) as f32];
            index.insert(i, &v).unwrap();
        }
        index.build_index().unwrap();
        assert_eq!(index.len(), 50);
    }

    #[test]
    fn test_ivf_index_empty() {
        let index = IvfIndex::new(DistanceMetric::Euclidean, 2);
        assert!(index.is_empty());
    }

    #[test]
    fn test_ivf_index_query_dimension_mismatch() {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 2);
        index.insert(1, &[1.0, 0.0]).unwrap();
        index.build_index().unwrap();
        let result = index.search(&[1.0, 0.0, 0.0], 5);
        assert!(result.is_err());
    }
}
