//! Latency Statistics Module
//!
//! Provides percentile-based latency statistics (P50/P95/P99) for benchmark results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    samples: Vec<u64>,
    sorted: bool,
}

impl LatencyStats {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            sorted: false,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            sorted: false,
        }
    }

    pub fn record(&mut self, latency_us: u64) {
        self.samples.push(latency_us);
        self.sorted = false;
    }

    pub fn record_batch(&mut self, latencies: impl IntoIterator<Item = u64>) {
        self.samples.extend(latencies);
        self.sorted = false;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.samples.sort();
            self.sorted = true;
        }
    }

    fn percentile(&self, p: f64) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        if self.samples.len() == 1 {
            return self.samples[0];
        }

        let idx = ((self.samples.len() - 1) as f64 * p) as usize;
        self.samples[idx.min(self.samples.len() - 1)]
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }

    pub fn min(&self) -> Option<u64> {
        self.samples.iter().min().copied()
    }

    pub fn max(&self) -> Option<u64> {
        self.samples.iter().max().copied()
    }

    pub fn sum(&self) -> u64 {
        self.samples.iter().sum()
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.sum() as f64 / self.samples.len() as f64
    }

    pub fn p50(&self) -> u64 {
        self.percentile(0.50)
    }

    pub fn p95(&self) -> u64 {
        self.percentile(0.95)
    }

    pub fn p99(&self) -> u64 {
        self.percentile(0.99)
    }

    pub fn p999(&self) -> u64 {
        self.percentile(0.999)
    }

    pub fn median(&self) -> u64 {
        self.p50()
    }
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_stats_empty() {
        let stats = LatencyStats::new();
        assert_eq!(stats.count(), 0);
        assert_eq!(stats.min(), None);
        assert_eq!(stats.max(), None);
        assert_eq!(stats.avg(), 0.0);
    }

    #[test]
    fn test_latency_stats_single() {
        let mut stats = LatencyStats::new();
        stats.record(100);
        assert_eq!(stats.count(), 1);
        assert_eq!(stats.min(), Some(100));
        assert_eq!(stats.max(), Some(100));
        assert_eq!(stats.avg(), 100.0);
        assert_eq!(stats.p50(), 100);
        assert_eq!(stats.p95(), 100);
        assert_eq!(stats.p99(), 100);
    }

    #[test]
    fn test_latency_stats_percentiles() {
        let mut stats = LatencyStats::new();
        for i in 1..=100 {
            stats.record(i);
        }

        assert_eq!(stats.p50(), 50);
        assert_eq!(stats.p95(), 95);
        assert_eq!(stats.p99(), 99);
    }

    #[test]
    fn test_latency_stats_batch() {
        let mut stats = LatencyStats::new();
        stats.record_batch(vec![10, 20, 30, 40, 50]);

        assert_eq!(stats.count(), 5);
        assert_eq!(stats.min(), Some(10));
        assert_eq!(stats.max(), Some(50));
        assert_eq!(stats.avg(), 30.0);
    }
}
