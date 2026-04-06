//! Explicit SIMD Implementation for Vector Operations
//!
//! Uses explicit SIMD intrinsics (AVX2/AVX-512) when available.
//! Falls back to scalar code on unsupported platforms.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// SIMD vector size for f32 (AVX2 = 8, AVX-512 = 16)
pub const SIMD_LANES: usize = 8; // Default to AVX2 size

/// Detect maximum SIMD lane count supported
pub fn detect_simd_lanes() -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return 16;
        } else if is_x86_feature_detected!("avx2") {
            return 8;
        }
    }
    4 // Scalar fallback
}

/// Compute dot product using SIMD
#[inline]
pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    
    let len = a.len();
    let mut sum = 0.0_f32;
    let mut i = 0usize;
    
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let mut acc = _mm256_setzero_ps();
            
            while i + 8 <= len {
                let a_vec = _mm256_loadu_ps(a.as_ptr().add(i));
                let b_vec = _mm256_loadu_ps(b.as_ptr().add(i));
                acc = _mm256_add_ps(acc, _mm256_mul_ps(a_vec, b_vec));
                i += 8;
            }
            
            // Horizontal sum of AVX register
            let sum128: __m128 = _mm256_castps256_ps128(acc);
            let high = _mm256_extractf128_ps(acc, 1);
            let sum256 = _mm_add_ps(sum128, high);
            let sum2 = _mm_hadd_ps(sum256, sum256);
            let sum_final = _mm_hadd_ps(sum2, sum2);
            sum = _mm_cvtss_f32(sum_final);
        }
    } else {
        // Scalar fallback
        while i < len {
            sum += a[i] * b[i];
            i += 1;
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        while i < len {
            sum += a[i] * b[i];
            i += 1;
        }
    }
    
    // Handle remaining elements
    while i < len {
        sum += a[i] * b[i];
        i += 1;
    }
    
    sum
}

/// Compute euclidean distance using SIMD
#[inline]
pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    
    let len = a.len();
    let mut sum = 0.0_f32;
    let mut i = 0usize;
    
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let mut acc = _mm256_setzero_ps();
            
            while i + 8 <= len {
                let a_vec = _mm256_loadu_ps(a.as_ptr().add(i));
                let b_vec = _mm256_loadu_ps(b.as_ptr().add(i));
                let diff = _mm256_sub_ps(a_vec, b_vec);
                acc = _mm256_add_ps(acc, _mm256_mul_ps(diff, diff));
                i += 8;
            }
            
            // Horizontal sum
            let sum128: __m128 = _mm256_castps256_ps128(acc);
            let high = _mm256_extractf128_ps(acc, 1);
            let sum256 = _mm_add_ps(sum128, high);
            let sum2 = _mm_hadd_ps(sum256, sum256);
            let sum_final = _mm_hadd_ps(sum2, sum2);
            sum = _mm_cvtss_f32(sum_final);
        }
    } else {
        while i < len {
            let diff = a[i] - b[i];
            sum += diff * diff;
            i += 1;
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        while i < len {
            let diff = a[i] - b[i];
            sum += diff * diff;
            i += 1;
        }
    }
    
    while i < len {
        let diff = a[i] - b[i];
        sum += diff * diff;
        i += 1;
    }
    
    sum.sqrt()
}

/// Compute cosine similarity using SIMD
#[inline]
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product_simd(a, b);
    let norm_a = euclidean_distance_simd(a, a).sqrt();
    let norm_b = euclidean_distance_simd(b, b).sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    dot / (norm_a * norm_b)
}

/// Compute manhattan distance using SIMD
#[inline]
pub fn manhattan_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    
    let len = a.len();
    let mut sum = 0.0_f32;
    let mut i = 0usize;
    
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        unsafe {
            let mut acc = _mm256_setzero_ps();
            
            while i + 8 <= len {
                let a_vec = _mm256_loadu_ps(a.as_ptr().add(i));
                let b_vec = _mm256_loadu_ps(b.as_ptr().add(i));
                let diff = _mm256_sub_ps(a_vec, b_vec);
                let abs_diff = _mm256_abs_ps(diff);
                acc = _mm256_add_ps(acc, abs_diff);
                i += 8;
            }
            
            // Horizontal sum
            let sum128: __m128 = _mm256_castps256_ps128(acc);
            let high = _mm256_extractf128_ps(acc, 1);
            let sum256 = _mm_add_ps(sum128, high);
            let sum2 = _mm_hadd_ps(sum256, sum256);
            let sum_final = _mm_hadd_ps(sum2, sum2);
            sum = _mm_cvtss_f32(sum_final);
        }
    } else {
        while i < len {
            sum += (a[i] - b[i]).abs();
            i += 1;
        }
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        while i < len {
            sum += (a[i] - b[i]).abs();
            i += 1;
        }
    }
    
    while i < len {
        sum += (a[i] - b[i]).abs();
        i += 1;
    }
    
    sum
}

/// Compute similarity score based on metric using SIMD
#[inline]
pub fn compute_similarity_simd(a: &[f32], b: &[f32], metric: crate::metrics::DistanceMetric) -> f32 {
    match metric {
        crate::metrics::DistanceMetric::Cosine => cosine_similarity_simd(a, b),
        crate::metrics::DistanceMetric::Euclidean => 1.0 / (1.0 + euclidean_distance_simd(a, b)),
        crate::metrics::DistanceMetric::DotProduct => dot_product_simd(a, b),
        crate::metrics::DistanceMetric::Manhattan => 1.0 / (1.0 + manhattan_distance_simd(a, b)),
    }
}

/// Batch compute distances from query to multiple vectors
pub fn batch_compute_distances<V: AsRef<[f32]>>(
    query: &[f32],
    vectors: &[V],
    metric: crate::metrics::DistanceMetric,
) -> Vec<f32> {
    vectors.iter().map(|v| compute_similarity_simd(query, v.as_ref(), metric)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = vec![8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        
        let simd_result = dot_product_simd(&a, &b);
        let scalar_result: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        
        assert!((simd_result - scalar_result).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_simd() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![5.0, 6.0, 7.0, 8.0];
        
        let simd_result = euclidean_distance_simd(&a, &b);
        let scalar_result = ((a[0]-b[0]).powi(2) + (a[1]-b[1]).powi(2) + 
                             (a[2]-b[2]).powi(2) + (a[3]-b[3]).powi(2)).sqrt();
        
        assert!((simd_result - scalar_result).abs() < 1e-6);
    }

    #[test]
    fn test_detect_simd_lanes() {
        let lanes = detect_simd_lanes();
        assert!(lanes >= 1);
        println!("Detected SIMD lanes: {}", lanes);
    }
}
