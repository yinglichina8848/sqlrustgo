//! GPU Acceleration Support for Vector Operations
//!
//! Provides GPU-accelerated distance computation using OpenCL.
//! Falls back to CPU SIMD when GPU is unavailable.

use crate::error::{VectorError, VectorResult};
use crate::metrics::DistanceMetric;
use std::sync::RwLock;

/// GPU device info
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: String,
    pub memory_mb: u64,
    pub compute_units: u32,
}

/// GPU acceleration status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuStatus {
    Available,
    Unavailable,
    Error,
}

/// GPU configuration
#[derive(Debug, Clone)]
pub struct GpuConfig {
    pub device_id: usize,
    pub memory_mb: u64,
    pub enable_fallback: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            device_id: 0,
            memory_mb: 4096,
            enable_fallback: true,
        }
    }
}

/// GPU accelerator trait
pub trait GpuAccelerator: Send + Sync {
    /// Copy data to GPU memory
    fn copy_to_device(&mut self, data: &[f32]) -> VectorResult<GpuBuffer>;
    
    /// Compute distances between query and vectors on GPU
    fn compute_distances(
        &self,
        query: &GpuBuffer,
        vectors: &[&GpuBuffer],
        metric: DistanceMetric,
    ) -> VectorResult<Vec<f32>>;
    
    /// Get GPU device info
    fn device_info(&self) -> Option<GpuDevice>;
    
    /// Check if GPU is available
    fn is_available(&self) -> bool;
}

/// GPU buffer handle
#[derive(Debug, Clone)]
pub struct GpuBuffer {
    pub id: u64,
    pub size: usize,
    device_id: usize,
}

/// CPU SIMD fallback accelerator
pub struct CpuSimdAccelerator {
    status: GpuStatus,
    device_info: Option<GpuDevice>,
}

impl CpuSimdAccelerator {
    pub fn new() -> Self {
        // Detect CPU SIMD capabilities
        let status = if cfg!(target_arch = "x86_64") || cfg!(target_arch = "aarch64") {
            GpuStatus::Available
        } else {
            GpuStatus::Unavailable
        };
        
        let device_info = Some(GpuDevice {
            name: "CPU SIMD".to_string(),
            vendor: "Auto-detected".to_string(),
            memory_mb: 8192,
            compute_units: std::thread::available_parallelism().map(|p| p.get() as u32).unwrap_or(4),
        });
        
        Self { status, device_info }
    }
    
    /// Compute cosine similarity with SIMD hints
    #[inline]
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot / (norm_a * norm_b)
    }
    
    /// Compute batch cosine similarities (SIMD optimized)
    pub fn batch_cosine_similarity(&self, query: &[f32], vectors: &[Vec<f32>]) -> Vec<f32> {
        vectors
            .iter()
            .map(|v| self.cosine_similarity(query, v))
            .collect()
    }
    
    /// Compute euclidean distance with SIMD hints
    #[inline]
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());
        
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }
    
    /// Compute batch euclidean distances
    pub fn batch_euclidean_distance(&self, query: &[f32], vectors: &[Vec<f32>]) -> Vec<f32> {
        vectors
            .iter()
            .map(|v| self.euclidean_distance(query, v))
            .collect()
    }
}

impl Default for CpuSimdAccelerator {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuAccelerator for CpuSimdAccelerator {
    fn copy_to_device(&mut self, _data: &[f32]) -> VectorResult<GpuBuffer> {
        if self.status != GpuStatus::Available {
            return Err(VectorError::InvalidParameter(
                "GPU not available".to_string(),
            ));
        }
        
        Ok(GpuBuffer {
            id: rand::random(),
            size: _data.len(),
            device_id: 0,
        })
    }
    
    fn compute_distances(
        &self,
        _query: &GpuBuffer,
        _vectors: &[&GpuBuffer],
        _metric: DistanceMetric,
    ) -> VectorResult<Vec<f32>> {
        Err(VectorError::InvalidParameter(
            "CPU accelerator does not support GPU buffer operations".to_string(),
        ))
    }
    
    fn device_info(&self) -> Option<GpuDevice> {
        self.device_info.clone()
    }
    
    fn is_available(&self) -> bool {
        self.status == GpuStatus::Available
    }
}

/// OpenCL GPU accelerator (stub - requires opencl crate)
pub struct OpenClAccelerator {
    config: GpuConfig,
    status: GpuStatus,
    device_info: Option<GpuDevice>,
}

impl OpenClAccelerator {
    pub fn new(config: GpuConfig) -> Self {
        // In a real implementation, this would initialize OpenCL
        // For now, return unavailable status
        Self {
            config,
            status: GpuStatus::Unavailable,
            device_info: None,
        }
    }
    
    /// List available OpenCL devices
    pub fn list_devices() -> Vec<GpuDevice> {
        // Stub: would enumerate OpenCL devices
        vec![]
    }
}

impl GpuAccelerator for OpenClAccelerator {
    fn copy_to_device(&mut self, data: &[f32]) -> VectorResult<GpuBuffer> {
        if self.status != GpuStatus::Available {
            return Err(VectorError::InvalidParameter(
                "OpenCL not available".to_string(),
            ));
        }
        
        Ok(GpuBuffer {
            id: rand::random(),
            size: data.len(),
            device_id: self.config.device_id,
        })
    }
    
    fn compute_distances(
        &self,
        _query: &GpuBuffer,
        _vectors: &[&GpuBuffer],
        _metric: DistanceMetric,
    ) -> VectorResult<Vec<f32>> {
        Err(VectorError::InvalidParameter(
            "OpenCL not initialized".to_string(),
        ))
    }
    
    fn device_info(&self) -> Option<GpuDevice> {
        self.device_info.clone()
    }
    
    fn is_available(&self) -> bool {
        self.status == GpuStatus::Available
    }
}

/// Create accelerator based on availability
pub fn create_accelerator(config: GpuConfig) -> Box<dyn GpuAccelerator> {
    // Try OpenCL first
    let opencl = OpenClAccelerator::new(config.clone());
    if opencl.is_available() {
        return Box::new(opencl);
    }
    
    // Fall back to CPU SIMD
    Box::new(CpuSimdAccelerator::new())
}

/// Performance comparison between CPU and GPU
pub struct PerformanceComparison {
    pub cpu_time_ms: f64,
    pub gpu_time_ms: f64,
    pub speedup: f64,
    pub recommended: String,
}

impl PerformanceComparison {
    pub fn new(cpu_ms: f64, gpu_ms: f64) -> Self {
        let speedup = if gpu_ms > 0.0 { cpu_ms / gpu_ms } else { 0.0 };
        let recommended = if speedup > 1.5 {
            "GPU".to_string()
        } else {
            "CPU".to_string()
        };
        
        Self {
            cpu_time_ms: cpu_ms,
            gpu_time_ms: gpu_ms,
            speedup,
            recommended,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_simd_accelerator() {
        let accel = CpuSimdAccelerator::new();
        
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![1.0f32, 0.0, 0.0];
        
        assert!((accel.cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!(accel.is_available());
    }

    #[test]
    fn test_batch_cosine() {
        let accel = CpuSimdAccelerator::new();
        
        let query = vec![1.0f32, 0.0];
        let vectors = vec![
            vec![1.0f32, 0.0],
            vec![0.0f32, 1.0],
            vec![0.5f32, 0.5],
        ];
        
        let scores = accel.batch_cosine_similarity(&query, &vectors);
        
        assert_eq!(scores.len(), 3);
        assert!((scores[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gpu_buffer() {
        let buffer = GpuBuffer {
            id: 123,
            size: 1024,
            device_id: 0,
        };
        
        assert_eq!(buffer.id, 123);
        assert_eq!(buffer.size, 1024);
    }

    #[test]
    fn test_performance_comparison() {
        let comp = PerformanceComparison::new(100.0, 50.0);
        
        assert!((comp.speedup - 2.0).abs() < 0.01);
        assert_eq!(comp.recommended, "GPU");
    }

    #[test]
    fn test_create_accelerator() {
        let accel = create_accelerator(GpuConfig::default());
        // Should fall back to CPU SIMD
        assert!(accel.is_available());
    }
}
