//! Batch Vector Writer with Buffered Insert and Async Indexing
//!
//! Optimizes write throughput for bulk vector insertions.

use crate::error::{VectorError, VectorResult};
use crate::parallel_knn::ParallelKnnIndex;
use crate::traits::VectorIndex;
use std::thread;
use std::sync::{Arc, RwLock};

/// Batch write configuration
#[derive(Debug, Clone)]
pub struct BatchWriteConfig {
    /// Buffer capacity before auto-flush
    pub buffer_capacity: usize,
    /// Flush threshold (number of vectors)
    pub flush_threshold: usize,
    /// Enable async indexing
    pub async_indexing: bool,
    /// Number of writer threads
    pub num_threads: usize,
}

impl Default for BatchWriteConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 100000,
            flush_threshold: 10000,
            async_indexing: true,
            num_threads: 4,
        }
    }
}

/// Batch vector writer with buffered writes
pub struct BatchVectorWriter {
    index: Arc<RwLock<ParallelKnnIndex>>,
    buffer: Arc<RwLock<Vec<(u64, Vec<f32>)>>>,
    config: BatchWriteConfig,
    pending_count: Arc<RwLock<usize>>,
}

impl BatchVectorWriter {
    pub fn new(metric: crate::metrics::DistanceMetric) -> Self {
        Self {
            index: Arc::new(RwLock::new(ParallelKnnIndex::new(metric))),
            buffer: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            config: BatchWriteConfig::default(),
            pending_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_config(
        metric: crate::metrics::DistanceMetric,
        config: BatchWriteConfig,
    ) -> Self {
        Self {
            index: Arc::new(RwLock::new(ParallelKnnIndex::with_config(
                metric,
                crate::parallel_knn::ParallelKnnConfig {
                    chunk_size: config.num_threads * 1000,
                    simd_enabled: true,
                },
            ))),
            buffer: Arc::new(RwLock::new(Vec::with_capacity(config.buffer_capacity))),
            config,
            pending_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Insert a single vector (buffered)
    pub fn insert(&mut self, id: u64, vector: Vec<f32>) -> VectorResult<()> {
        {
            let mut buffer = self.buffer.write().unwrap();
            buffer.push((id, vector));
        }
        
        {
            let mut count = self.pending_count.write().unwrap();
            *count += 1;
        }
        
        // Auto-flush if threshold reached
        if self.should_flush() {
            self.flush()?;
        }
        
        Ok(())
    }

    /// Batch insert multiple vectors
    pub fn batch_insert(&mut self, vectors: Vec<(u64, Vec<f32>)>) -> VectorResult<usize> {
        let count = vectors.len();
        {
            let mut buffer = self.buffer.write().unwrap();
            buffer.extend(vectors);
        }
        {
            let mut pending = self.pending_count.write().unwrap();
            *pending += count;
        }
        
        if self.should_flush() {
            self.flush()?;
        }
        
        Ok(count)
    }

    /// Check if should trigger flush
    fn should_flush(&self) -> bool {
        let count = self.pending_count.read().unwrap();
        *count >= self.config.flush_threshold
    }

    /// Flush buffer to index
    pub fn flush(&mut self) -> VectorResult<()> {
        let vectors = {
            let mut buffer = self.buffer.write().unwrap();
            std::mem::take(&mut *buffer)
        };
        
        if vectors.is_empty() {
            return Ok(());
        }
        
        let mut count = self.pending_count.write().unwrap();
        *count = 0;
        drop(count);
        
        if self.config.async_indexing {
            let index = Arc::clone(&self.index);
            let metric = self.index.read().unwrap().metric();
            
            thread::spawn(move || {
                let mut index = index.write().unwrap();
                for (id, vector) in vectors {
                    if let Err(e) = index.insert(id, &vector) {
                        eprintln!("Async insert error: {:?}", e);
                    }
                }
            });
        } else {
            let mut index = self.index.write().unwrap();
            for (id, vector) in vectors {
                index.insert(id, &vector)?;
            }
        }
        
        Ok(())
    }

    /// Force flush and wait for completion
    pub fn flush_sync(&mut self) -> VectorResult<()> {
        let vectors = {
            let mut buffer = self.buffer.write().unwrap();
            std::mem::take(&mut *buffer)
        };
        
        let mut count = self.pending_count.write().unwrap();
        *count = 0;
        drop(count);
        
        if vectors.is_empty() {
            return Ok(());
        }
        
        let mut index = self.index.write().unwrap();
        for (id, vector) in vectors {
            index.insert(id, &vector)?;
        }
        
        Ok(())
    }

    /// Get pending buffer size
    pub fn pending(&self) -> usize {
        *self.pending_count.read().unwrap()
    }

    /// Get total indexed vectors
    pub fn indexed_count(&self) -> usize {
        self.index.read().unwrap().len()
    }
}

impl Drop for BatchVectorWriter {
    fn drop(&mut self) {
        // Flush on drop
        let _ = self.flush_sync();
    }
}

/// Estimate write throughput
pub fn estimate_throughput(n: usize, elapsed_secs: f64) -> f64 {
    n as f64 / elapsed_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_writer_insert() {
        let mut writer = BatchVectorWriter::new(crate::metrics::DistanceMetric::Cosine);
        
        for i in 0..100 {
            let v = vec![i as f32; 128];
            writer.insert(i, v).unwrap();
        }
        
        assert_eq!(writer.pending(), 100);
    }

    #[test]
    fn test_batch_writer_auto_flush() {
        let config = BatchWriteConfig {
            buffer_capacity: 1000,
            flush_threshold: 50,
            async_indexing: false,
            num_threads: 1,
        };
        
        let mut writer = BatchVectorWriter::with_config(
            crate::metrics::DistanceMetric::Euclidean,
            config,
        );
        
        for i in 0..100 {
            let v = vec![i as f32; 64];
            writer.insert(i, v).unwrap();
        }
        
        // Should have auto-flushed at 50
        assert_eq!(writer.pending(), 0);
        assert_eq!(writer.indexed_count(), 100);
    }

    #[test]
    fn test_batch_insert() {
        let mut writer = BatchVectorWriter::new(crate::metrics::DistanceMetric::Cosine);
        
        let vectors: Vec<_> = (0..1000)
            .map(|i| (i as u64, vec![i as f32; 128]))
            .collect();
        
        let count = writer.batch_insert(vectors).unwrap();
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_flush_sync() {
        let mut writer = BatchVectorWriter::new(crate::metrics::DistanceMetric::Cosine);
        
        for i in 0..100 {
            let v = vec![i as f32; 64];
            writer.insert(i, v).unwrap();
        }
        
        writer.flush_sync().unwrap();
        
        assert_eq!(writer.pending(), 0);
        assert_eq!(writer.indexed_count(), 100);
    }

    #[test]
    fn test_throughput_estimate() {
        let throughput = estimate_throughput(100000, 1.0);
        assert!((throughput - 100000.0).abs() < 0.01);
    }
}
