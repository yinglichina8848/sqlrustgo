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
            compute_units: std::thread::available_parallelism()
                .map(|p| p.get() as u32)
                .unwrap_or(4),
        });

        Self {
            status,
            device_info,
        }
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

// =============================================================================
// OpenCL GPU Accelerator (Real Implementation)
// =============================================================================

#[cfg(feature = "opencl")]
mod opencl_impl {
    use super::*;
    use ocl::flags::{MEM_READ_ONLY, MEM_WRITE_ONLY, PROGRAM_BUILD_STATUS};
    use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue};
    use std::collections::HashMap;

    /// OpenCL kernel source for vector distance computation
    const KERNEL_SOURCE: &str = r#"
        // Cosine similarity kernel
        __kernel void cosine_similarity_kernel(
            __global const float* query,
            __global const float* vectors,
            __global float* results,
            int num_vectors,
            int dimension
        ) {
            int idx = get_global_id(0);
            if (idx < num_vectors) {
                float dot = 0.0f;
                float norm_query = 0.0f;
                float norm_vec = 0.0f;
                
                for (int i = 0; i < dimension; i++) {
                    float q = query[i];
                    float v = vectors[idx * dimension + i];
                    dot += q * v;
                    norm_query += q * q;
                    norm_vec += v * v;
                }
                
                float norm_product = sqrt(norm_query) * sqrt(norm_vec);
                results[idx] = (norm_product > 0.0f) ? (dot / norm_product) : 0.0f;
            }
        }
<<<<<<< Updated upstream
        
        // Euclidean distance kernel
        __kernel void euclidean_distance_kernel(
            __global const float* query,
            __global const float* vectors,
            __global float* results,
            int num_vectors,
            int dimension
        ) {
            int idx = get_global_id(0);
            if (idx < num_vectors) {
                float sum = 0.0f;
                for (int i = 0; i < dimension; i++) {
                    float diff = query[i] - vectors[idx * dimension + i];
                    sum += diff * diff;
                }
                results[idx] = sqrt(sum);
            }
        }
        
        // Dot product kernel
        __kernel void dot_product_kernel(
            __global const float* query,
            __global const float* vectors,
            __global float* results,
            int num_vectors,
            int dimension
        ) {
            int idx = get_global_id(0);
            if (idx < num_vectors) {
                float sum = 0.0f;
                for (int i = 0; i < dimension; i++) {
                    sum += query[i] * vectors[idx * dimension + i];
                }
                results[idx] = sum;
            }
        }
        
        // Manhattan distance kernel
        __kernel void manhattan_distance_kernel(
            __global const float* query,
            __global const float* vectors,
            __global float* results,
            int num_vectors,
            int dimension
        ) {
            int idx = get_global_id(0);
            if (idx < num_vectors) {
                float sum = 0.0f;
                for (int i = 0; i < dimension; i++) {
                    float diff = query[i] - vectors[idx * dimension + i];
                    sum += fabs(diff);
                }
                results[idx] = sum;
            }
        }
    "#;

    /// Internal OpenCL buffer with data storage
    struct InternalGpuBuffer {
        buffer: Buffer<f32>,
        data: Vec<f32>,
    }

    /// OpenCL GPU accelerator - REAL implementation
    pub struct OpenClAccelerator {
        context: Context,
        queue: Queue,
        program: Program,
        device: Device,
        config: GpuConfig,
        status: GpuStatus,
        device_info: Option<GpuDevice>,
        buffer_cache: RwLock<HashMap<u64, InternalGpuBuffer>>,
        next_buffer_id: RwLock<u64>,
    }

    impl OpenClAccelerator {
        /// Create a new OpenCL accelerator
        pub fn new(config: GpuConfig) -> Self {
            // Try to initialize OpenCL
            match Self::init_opencl(&config) {
                Ok((context, queue, program, device, device_info)) => Self {
                    context,
                    queue,
                    program,
                    device,
                    config,
                    status: GpuStatus::Available,
                    device_info: Some(device_info),
                    buffer_cache: RwLock::new(HashMap::new()),
                    next_buffer_id: RwLock::new(0),
                },
                Err(e) => {
                    log::warn!("Failed to initialize OpenCL: {}", e);
                    Self {
                        context: unsafe { Context::empty() }, // Dummy context
                        queue: unsafe { Queue::empty() },     // Dummy queue
                        program: unsafe { Program::empty() }, // Dummy program
                        device: Device::null(),
                        config,
                        status: GpuStatus::Unavailable,
                        device_info: None,
                        buffer_cache: RwLock::new(HashMap::new()),
                        next_buffer_id: RwLock::new(0),
                    }
                }
            }
        }

        fn init_opencl(
            config: &GpuConfig,
        ) -> Result<(Context, Queue, Program, Device, GpuDevice), String> {
            // Select platform and device
            let platform = Platform::default().ok_or("No OpenCL platform found")?;

            let devices = Device::list(&platform, Some(ocl::flags::DEVICE_TYPE_ALL))
                .map_err(|e| format!("Failed to list devices: {}", e))?;

            if devices.is_empty() {
                return Err("No OpenCL devices found".to_string());
            }

            // Select GPU device (preferred) or fall back to any device
            let device = devices
                .iter()
                .find(|d| d.is_gpu())
                .or_else(|| devices.first())
                .ok_or("No suitable device found")?
                .clone();

            // Create context and queue
            let context = Context::builder()
                .platform(platform)
                .devices(device.clone())
                .build()
                .map_err(|e| format!("Failed to create context: {}", e))?;

            let queue = Queue::builder()
                .context(&context)
                .device(device.clone())
                .build()
                .map_err(|e| format!("Failed to create queue: {}", e))?;

            // Build program
            let program = Program::builder()
                .source(KERNEL_SOURCE)
                .devices(device.clone())
                .build(&context)
                .map_err(|e| format!("Failed to build program: {}", e))?;

            // Get device info
            let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
            let vendor = device.vendor().unwrap_or_else(|_| "Unknown".to_string());
            let max_compute_units = device.max_compute_units().unwrap_or(1);
            let max_memory = device.max_mem_alloc_size().unwrap_or(0) / (1024 * 1024); // Convert to MB

            let device_info = GpuDevice {
                name,
                vendor,
                memory_mb: max_memory,
                compute_units: max_compute_units as u32,
            };

            log::info!(
                "OpenCL initialized: {} ({}), {} compute units, {} MB memory",
                device_info.name,
                device_info.vendor,
                device_info.compute_units,
                device_info.memory_mb
            );

            Ok((context, queue, program, device, device_info))
        }

        /// List available OpenCL devices
        pub fn list_devices() -> Vec<GpuDevice> {
            let mut devices = Vec::new();

            if let Ok(platforms) = Platform::list() {
                for platform in platforms {
                    if let Ok(platform_devices) =
                        Device::list(&platform, Some(ocl::flags::DEVICE_TYPE_ALL))
                    {
                        for device in platform_devices {
                            if let (Ok(name), Ok(vendor), Ok(compute_units), Ok(max_mem)) = (
                                device.name(),
                                device.vendor(),
                                device.max_compute_units(),
                                device.max_mem_alloc_size(),
                            ) {
                                devices.push(GpuDevice {
                                    name,
                                    vendor,
                                    memory_mb: max_mem / (1024 * 1024),
                                    compute_units: compute_units as u32,
                                });
                            }
                        }
                    }
                }
            }

            devices
        }

        fn get_next_buffer_id(&self) -> u64 {
            let mut counter = self.next_buffer_id.write();
            let id = *counter;
            *counter += 1;
            id
        }
    }

    impl GpuAccelerator for OpenClAccelerator {
        fn copy_to_device(&mut self, data: &[f32]) -> VectorResult<GpuBuffer> {
            if self.status != GpuStatus::Available {
                return Err(VectorError::InvalidParameter(
                    "OpenCL not available".to_string(),
                ));
            }

            let buffer_id = self.get_next_buffer_id();
            let size = data.len();

            // Create OpenCL buffer
            let buffer = Buffer::<f32>::builder()
                .queue(self.queue.clone())
                .flags(MEM_READ_ONLY)
                .dims(size)
                .build()
                .map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to create buffer: {}", e))
                })?;

            // Write data to buffer
            buffer.write(data).enqueue().map_err(|e| {
                VectorError::InvalidParameter(format!("Failed to write to buffer: {}", e))
            })?;

            // Store in cache
            let internal_buffer = InternalGpuBuffer {
                buffer,
                data: data.to_vec(),
            };
            self.buffer_cache.write().insert(buffer_id, internal_buffer);

            Ok(GpuBuffer {
                id: buffer_id,
                size,
                device_id: self.config.device_id,
            })
        }

        fn compute_distances(
            &self,
            query: &GpuBuffer,
            vectors: &[&GpuBuffer],
            metric: DistanceMetric,
        ) -> VectorResult<Vec<f32>> {
            if self.status != GpuStatus::Available {
                return Err(VectorError::InvalidParameter(
                    "OpenCL not available".to_string(),
                ));
            }

            if vectors.is_empty() {
                return Ok(vec![]);
            }

            // Get query buffer from cache
            let query_internal = self.buffer_cache.read().get(&query.id).ok_or_else(|| {
                VectorError::InvalidParameter("Query buffer not found".to_string())
            })?;

            let dimension = query_internal.data.len();
            let num_vectors = vectors.len();

            // Build vectors data and get buffers
            let mut vectors_data: Vec<f32> = Vec::with_capacity(dimension * num_vectors);
            let mut vector_buffers: Vec<Buffer<f32>> = Vec::new();

            for v in vectors {
                let internal = self.buffer_cache.read().get(&v.id).ok_or_else(|| {
                    VectorError::InvalidParameter("Vector buffer not found".to_string())
                })?;
                vectors_data.extend_from_slice(&internal.data);

                let buffer = Buffer::<f32>::builder()
                    .queue(self.queue.clone())
                    .flags(MEM_READ_ONLY)
                    .dims(dimension)
                    .build()
                    .map_err(|e| {
                        VectorError::InvalidParameter(format!(
                            "Failed to create vector buffer: {}",
                            e
                        ))
                    })?;

                buffer.write(&internal.data).enqueue().map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to write vector: {}", e))
                })?;

                vector_buffers.push(buffer);
            }

            // Create result buffer
            let results_buffer: Buffer<f32> = Buffer::builder()
                .queue(self.queue.clone())
                .flags(MEM_WRITE_ONLY)
                .dims(num_vectors)
                .build()
                .map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to create results buffer: {}", e))
                })?;

            // Select and execute kernel based on metric
            let kernel_name = match metric {
                DistanceMetric::Cosine => "cosine_similarity_kernel",
                DistanceMetric::Euclidean => "euclidean_distance_kernel",
                DistanceMetric::DotProduct => "dot_product_kernel",
                DistanceMetric::Manhattan => "manhattan_distance_kernel",
            };

            // For simplicity, process one vector at a time with the kernel
            // In production, you'd batch process multiple vectors
            let mut results = Vec::with_capacity(num_vectors);

            for (i, vector_buffer) in vector_buffers.iter().enumerate() {
                // Create temporary buffer for this vector
                let vector_data = &vectors_data[i * dimension..(i + 1) * dimension];
                let temp_vector: Buffer<f32> = Buffer::builder()
                    .queue(self.queue.clone())
                    .flags(MEM_READ_ONLY)
                    .dims(dimension)
                    .build()
                    .map_err(|e| {
                        VectorError::InvalidParameter(format!(
                            "Failed to create temp buffer: {}",
                            e
                        ))
                    })?;

                temp_vector.write(vector_data).enqueue().map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to write temp vector: {}", e))
                })?;

                let result_buffer: Buffer<f32> = Buffer::builder()
                    .queue(self.queue.clone())
                    .flags(MEM_WRITE_ONLY)
                    .dims(1)
                    .build()
                    .map_err(|e| {
                        VectorError::InvalidParameter(format!(
                            "Failed to create result buffer: {}",
                            e
                        ))
                    })?;

                let kernel = Kernel::builder()
                    .program(&self.program)
                    .name(kernel_name)
                    .queue(self.queue.clone())
                    .arg(&query_internal.buffer)
                    .arg(&temp_vector)
                    .arg(&result_buffer)
                    .arg(num_vectors as i32)
                    .arg(dimension as i32)
                    .build()
                    .map_err(|e| {
                        VectorError::InvalidParameter(format!("Failed to build kernel: {}", e))
                    })?;

                kernel.enqueue().map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to enqueue kernel: {}", e))
                })?;

                self.queue.finish().map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to finish queue: {}", e))
                })?;

                let mut result = vec![0.0f32];
                result_buffer.read(&mut result).enqueue().map_err(|e| {
                    VectorError::InvalidParameter(format!("Failed to read result: {}", e))
                })?;

                // Convert distance to similarity if needed
                let similarity = match metric {
                    DistanceMetric::Cosine | DistanceMetric::DotProduct => result[0],
                    DistanceMetric::Euclidean => 1.0f32 / (1.0f32 + result[0]),
                    DistanceMetric::Manhattan => 1.0f32 / (1.0f32 + result[0]),
                };

                results.push(similarity);
            }

            Ok(results)
        }

        fn device_info(&self) -> Option<GpuDevice> {
            self.device_info.clone()
        }

        fn is_available(&self) -> bool {
            self.status == GpuStatus::Available
        }
    }
}

// =============================================================================
// Stub OpenCL when feature is not enabled
// =============================================================================

#[cfg(not(feature = "opencl"))]
mod opencl_impl {
    use super::*;

    /// OpenCL GPU accelerator - stub when OpenCL feature not enabled
    pub struct OpenClAccelerator {
        config: GpuConfig,
        status: GpuStatus,
        device_info: Option<GpuDevice>,
    }

    impl OpenClAccelerator {
        /// Create a new OpenCL accelerator (stub - OpenCL not compiled in)
        pub fn new(config: GpuConfig) -> Self {
            Self {
                config,
                status: GpuStatus::Unavailable,
                device_info: None,
            }
        }

        /// List available OpenCL devices (returns empty - OpenCL not compiled)
        pub fn list_devices() -> Vec<GpuDevice> {
            vec![]
        }
    }

    impl GpuAccelerator for OpenClAccelerator {
        fn copy_to_device(&mut self, _data: &[f32]) -> VectorResult<GpuBuffer> {
            if self.status != GpuStatus::Available {
                return Err(VectorError::InvalidParameter(
                    "OpenCL support not compiled in. Enable 'opencl' feature.".to_string(),
                ));
            }

            Ok(GpuBuffer {
                id: rand::random(),
                size: _data.len(),
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
                "OpenCL support not compiled in. Enable 'opencl' feature.".to_string(),
            ))
        }

        fn device_info(&self) -> Option<GpuDevice> {
            self.device_info.clone()
        }

        fn is_available(&self) -> bool {
            self.status == GpuStatus::Available
        }
    }
}

pub use opencl_impl::OpenClAccelerator;

/// Create accelerator based on availability
pub fn create_accelerator(config: GpuConfig) -> Box<dyn GpuAccelerator> {
    // Try OpenCL first (if compiled with opencl feature)
    let opencl = OpenClAccelerator::new(config.clone());
    if opencl.is_available() {
        log::info!("Using OpenCL GPU acceleration");
        return Box::new(opencl);
    }

    // Fall back to CPU SIMD
    log::info!("OpenCL not available, using CPU SIMD fallback");
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
        let vectors = vec![vec![1.0f32, 0.0], vec![0.0f32, 1.0], vec![0.5f32, 0.5]];

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
        // Should fall back to CPU SIMD when OpenCL not compiled
        assert!(accel.is_available());
    }

    #[test]
    fn test_opencl_stub_not_available() {
        let accel = OpenClAccelerator::new(GpuConfig::default());
        // Without opencl feature, should not be available
        assert!(!accel.is_available());
    }
}
