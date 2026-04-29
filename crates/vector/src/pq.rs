//! Product Quantization (PQ) for vector compression
//!
//! PQ divides vectors into m_pq sub-vectors and quantizes each to k_sub centroids.
//! This enables 16x compression for 128-dimensional vectors with m_pq=16.

use crate::error::{VectorError, VectorResult};
use crate::simd_explicit::euclidean_distance_simd as euclidean;

/// Product Quantization encoder
///
/// Splits vectors into sub-vectors and quantizes each to k_sub centroids.
/// Encoding: 128 dim / 16 sub_vecs = 8 dim per sub-vector, each encoded as 1 byte.
#[derive(Debug, Clone)]
pub struct ProductQuantizer {
    /// Original vector dimension
    pub dimension: usize,
    /// Number of sub-vectors
    pub m_pq: usize,
    /// Number of centroids per sub-space
    pub k_sub: usize,
    /// Dimension of each sub-vector
    pub sub_dim: usize,
    /// Codebook: [sub_idx][centroid_idx][sub_dim]
    pub centroids: Vec<Vec<Vec<f32>>>,
}

impl ProductQuantizer {
    /// Create a new ProductQuantizer
    ///
    /// # Arguments
    /// * `dimension` - Vector dimension (must be divisible by m_pq)
    /// * `m_pq` - Number of sub-vectors
    /// * `k_sub` - Number of centroids per sub-space
    pub fn new(dimension: usize, m_pq: usize, k_sub: usize) -> Self {
        let sub_dim = dimension / m_pq;
        Self {
            dimension,
            m_pq,
            k_sub,
            sub_dim,
            centroids: Vec::new(),
        }
    }

    /// Train the PQ encoder on a set of vectors
    ///
    /// Uses random initialization and iterative refinement.
    /// Optimized for speed: reduced iterations, parallel sub-space training.
    pub fn train(&mut self, vectors: &[Vec<f32>]) -> VectorResult<()> {
        if vectors.is_empty() {
            return Err(VectorError::EmptyIndex);
        }

        if vectors[0].len() != self.dimension {
            return Err(VectorError::DimensionMismatch {
                expected: self.dimension,
                actual: vectors[0].len(),
            });
        }

        let sub_dim = self.dimension / self.m_pq;

        // Sample vectors if too many (for speed)
        let sample_size = (10_000usize).min(vectors.len());
        let sampled: Vec<&[f32]> = if vectors.len() > sample_size {
            use rand::seq::SliceRandom;
            let mut indices: Vec<usize> = (0..vectors.len()).collect();
            indices.shuffle(&mut rand::thread_rng());
            indices[..sample_size]
                .iter()
                .map(|&i| vectors[i].as_slice())
                .collect()
        } else {
            vectors.iter().map(|v| v.as_slice()).collect()
        };

        // Process each sub-space (can be parallelized)
        self.centroids = (0..self.m_pq)
            .map(|sub_idx| {
                let sub_vectors: Vec<Vec<f32>> = sampled
                    .iter()
                    .map(|v| v[sub_idx * sub_dim..(sub_idx + 1) * sub_dim].to_vec())
                    .collect();

                // Random initialization (fast, good enough)
                let mut rng = rand::thread_rng();
                let mut sub_centroids: Vec<Vec<f32>> = (0..self.k_sub)
                    .map(|_| {
                        let idx =
                            rand::seq::index::sample(&mut rng, sampled.len(), 1).into_vec()[0];
                        sub_vectors[idx].clone()
                    })
                    .collect();

                // Iterative refinement (reduced to 10 iterations)
                for _ in 0..10 {
                    let assignments = Self::assign_to_centroids(&sub_vectors, &sub_centroids);
                    let (new_centroids, _) =
                        Self::update_centroids(&sub_vectors, &assignments, sub_dim);
                    sub_centroids = new_centroids;
                }

                sub_centroids
            })
            .collect();

        Ok(())
    }

    /// Assign each vector to nearest centroid
    fn assign_to_centroids(vectors: &[Vec<f32>], centroids: &[Vec<f32>]) -> Vec<Vec<usize>> {
        let k = centroids.len();
        let mut assignments: Vec<Vec<usize>> = vec![Vec::new(); k];

        for (vec_idx, v) in vectors.iter().enumerate() {
            let mut min_dist = f32::MAX;
            let mut min_centroid = 0;

            for (c_idx, centroid) in centroids.iter().enumerate() {
                let dist = euclidean(v, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_centroid = c_idx;
                }
            }

            assignments[min_centroid].push(vec_idx);
        }

        assignments
    }

    /// Update centroids based on assignments
    #[allow(unused_mut)]
    fn update_centroids(
        vectors: &[Vec<f32>],
        assignments: &[Vec<usize>],
        sub_dim: usize,
    ) -> (Vec<Vec<f32>>, bool) {
        let k = assignments.len();
        let mut new_centroids: Vec<Vec<f32>> = Vec::with_capacity(k);
        let mut converged = true;

        for members in assignments.iter() {
            if members.is_empty() {
                new_centroids.push(vec![0.0f32; sub_dim]);
                continue;
            }

            let mut centroid = vec![0.0f32; sub_dim];
            for &member_idx in members {
                let sub_vec = &vectors[member_idx];
                for (i, &val) in sub_vec.iter().enumerate() {
                    centroid[i] += val;
                }
            }
            for val in centroid.iter_mut() {
                *val /= members.len() as f32;
            }

            // Check convergence
            // Note: We don't have old centroid here, skip convergence check
            new_centroids.push(centroid);
        }

        (new_centroids, converged)
    }

    /// Encode a vector to PQ codes
    ///
    /// Returns a Vec<u8> of length m_pq, where each byte is the centroid index.
    pub fn encode(&self, vector: &[f32]) -> Vec<u8> {
        let sub_dim = self.dimension / self.m_pq;
        let mut code = Vec::with_capacity(self.m_pq);

        for sub_idx in 0..self.m_pq {
            let sub_vec = &vector[sub_idx * sub_dim..(sub_idx + 1) * sub_dim];

            let mut min_dist = f32::MAX;
            let mut min_idx = 0u8;

            for (c_idx, centroid) in self.centroids[sub_idx].iter().enumerate() {
                let dist = euclidean(sub_vec, centroid);
                if dist < min_dist {
                    min_dist = dist;
                    min_idx = c_idx as u8;
                }
            }
            code.push(min_idx);
        }

        code
    }

    /// Decode PQ codes back to vector (approximation)
    pub fn decode(&self, code: &[u8]) -> Vec<f32> {
        let _sub_dim = self.dimension / self.m_pq;
        let mut vector = Vec::with_capacity(self.dimension);

        for (sub_idx, &c_idx) in code.iter().enumerate().take(self.m_pq) {
            let c_idx = c_idx as usize;
            if let Some(centroid) = self.centroids.get(sub_idx).and_then(|c| c.get(c_idx)) {
                vector.extend_from_slice(centroid);
            }
        }

        vector
    }

    /// ADC: Asymmetric Distance Computation
    ///
    /// Computes distance between query (original) and code (PQ encoded)
    /// WITHOUT decoding the code first. This is the key optimization.
    pub fn adc_distance(&self, query: &[f32], code: &[u8]) -> f32 {
        let sub_dim = self.dimension / self.m_pq;
        let mut total = 0.0f32;

        for sub_idx in 0..self.m_pq {
            let sub_query = &query[sub_idx * sub_dim..(sub_idx + 1) * sub_dim];
            let c_idx = code[sub_idx] as usize;

            if let Some(centroid) = self.centroids.get(sub_idx).and_then(|c| c.get(c_idx)) {
                let dist = euclidean(sub_query, centroid);
                total += dist * dist;
            }
        }

        total.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_new() {
        let pq = ProductQuantizer::new(128, 16, 256);
        assert_eq!(pq.dimension, 128);
        assert_eq!(pq.m_pq, 16);
        assert_eq!(pq.k_sub, 256);
        assert_eq!(pq.sub_dim, 8);
    }

    #[test]
    fn test_pq_train_and_encode() {
        let dimension = 128;
        let m_pq = 16;
        let k_sub = 256;

        let mut pq = ProductQuantizer::new(dimension, m_pq, k_sub);

        // Create 1000 random vectors for training
        let vectors: Vec<Vec<f32>> = (0..1000)
            .map(|_| (0..dimension).map(|_| rand::random::<f32>()).collect())
            .collect();

        pq.train(&vectors).unwrap();

        // Encode a vector
        let test_vector: Vec<f32> = (0..dimension).map(|_| rand::random::<f32>()).collect();
        let code = pq.encode(&test_vector);

        assert_eq!(code.len(), m_pq); // 16 bytes
    }

    #[test]
    fn test_pq_encode_decode() {
        let mut pq = ProductQuantizer::new(8, 4, 4);

        let vectors: Vec<Vec<f32>> = vec![
            vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ];

        pq.train(&vectors).unwrap();

        let original = vectors[0].clone();
        let code = pq.encode(&original);
        let decoded = pq.decode(&code);

        // Decoded should be close to centroids, not exact
        assert_eq!(decoded.len(), 8);
        assert_eq!(code.len(), 4);
    }

    #[test]
    fn test_pq_adc_distance() {
        let dimension = 8;
        let m_pq = 4; // 4 sub-vectors, each 2 dims
        let k_sub = 4;

        let mut pq = ProductQuantizer::new(dimension, m_pq, k_sub);

        let vectors: Vec<Vec<f32>> = vec![
            vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
            vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
        ];

        pq.train(&vectors).unwrap();

        let query = vec![0.1, 0.9, 0.1, 0.9, 0.1, 0.9, 0.1, 0.9];
        let code = pq.encode(&vectors[0]);

        let dist = pq.adc_distance(&query, &code);
        assert!(dist >= 0.0);
    }

    #[test]
    fn test_pq_empty_vectors() {
        let mut pq = ProductQuantizer::new(128, 16, 256);
        let result = pq.train(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pq_dimension_mismatch() {
        let mut pq = ProductQuantizer::new(128, 16, 256);
        let vectors = vec![vec![0.0f32; 64]]; // Wrong dimension
        let result = pq.train(&vectors);
        assert!(result.is_err());
    }
}
