//! Distance metric implementations

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        DistanceMetric::Cosine
    }
}

impl DistanceMetric {
    pub fn name(&self) -> &'static str {
        match self {
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::Euclidean => "euclidean",
            DistanceMetric::DotProduct => "dotproduct",
            DistanceMetric::Manhattan => "manhattan",
        }
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    if a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => 1.0 - cosine_similarity(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => -dot_product(a, b),
        DistanceMetric::Manhattan => manhattan_distance(a, b),
    }
}

pub fn compute_similarity(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => cosine_similarity(a, b),
        DistanceMetric::Euclidean => 1.0 / (1.0 + euclidean_distance(a, b)),
        DistanceMetric::DotProduct => dot_product(a, b),
        DistanceMetric::Manhattan => 1.0 / (1.0 + manhattan_distance(a, b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_same() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((manhattan_distance(&a, &b) - 7.0).abs() < 0.001);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((dot_product(&a, &b) - 32.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        assert!((cosine_similarity(&a, &b) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance_same_point() {
        let a = vec![1.0, 2.0, 3.0];
        assert!(euclidean_distance(&a, &a).abs() < 0.001);
    }

    #[test]
    fn test_euclidean_distance_3d() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_manhattan_distance_same_point() {
        let a = vec![1.0, 2.0, 3.0];
        assert!(manhattan_distance(&a, &a).abs() < 0.001);
    }

    #[test]
    fn test_compute_distance_all_metrics() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((compute_distance(&a, &b, DistanceMetric::Cosine) - 1.0).abs() < 0.001);
        assert!((compute_distance(&a, &b, DistanceMetric::Euclidean) - std::f32::consts::SQRT_2).abs() < 0.001);
    }

    #[test]
    fn test_compute_similarity_all_metrics() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!((compute_similarity(&a, &b, DistanceMetric::Cosine) - 1.0).abs() < 0.001);
        assert!((compute_similarity(&a, &b, DistanceMetric::DotProduct) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_metric_name() {
        assert_eq!(DistanceMetric::Cosine.name(), "cosine");
        assert_eq!(DistanceMetric::Euclidean.name(), "euclidean");
        assert_eq!(DistanceMetric::DotProduct.name(), "dotproduct");
        assert_eq!(DistanceMetric::Manhattan.name(), "manhattan");
    }
}
