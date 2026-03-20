//! Latency histogram using hdrhistogram

use hdrhistogram::Histogram;
use std::sync::Mutex;

/// Records latency samples and computes percentiles
pub struct LatencyRecorder {
    hist: Mutex<Histogram<u64>>,
}

impl LatencyRecorder {
    /// Create a new latency recorder
    pub fn new() -> Self {
        // Create histogram with precision of 3 decimal digits
        let hist = Histogram::new(3).expect("Failed to create histogram");
        Self {
            hist: Mutex::new(hist),
        }
    }

    /// Record a latency sample (in microseconds)
    pub fn record(&self, value: u64) {
        if let Ok(mut h) = self.hist.lock() {
            let _ = h.record(value);
        }
    }

    /// Get latency statistics
    pub fn snapshot(&self) -> LatencyStats {
        let h = match self.hist.lock() {
            Ok(h) => h,
            Err(_) => return LatencyStats::default(),
        };

        LatencyStats {
            p50: h.value_at_quantile(0.50),
            p95: h.value_at_quantile(0.95),
            p99: h.value_at_quantile(0.99),
            p999: h.value_at_quantile(0.999),
            max: h.max(),
            count: h.len(),
        }
    }
}

impl Default for LatencyRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Latency statistics
#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    /// 50th percentile (median) in microseconds
    pub p50: u64,
    /// 95th percentile in microseconds
    pub p95: u64,
    /// 99th percentile in microseconds
    pub p99: u64,
    /// 99.9th percentile in microseconds
    pub p999: u64,
    /// Maximum value in microseconds
    pub max: u64,
    /// Total number of samples
    pub count: u64,
}

impl LatencyStats {
    /// Print statistics in human-readable format
    pub fn print(&self) {
        println!("Latency (µs):");
        println!("  P50:   {}", self.p50);
        println!("  P95:   {}", self.p95);
        println!("  P99:   {}", self.p99);
        println!("  P999:  {}", self.p999);
        println!("  Max:   {}", self.max);
        println!("  Count: {}", self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_recorder() {
        let recorder = LatencyRecorder::new();

        // Record some samples
        for _ in 0..1000 {
            recorder.record(100);
        }

        let stats = recorder.snapshot();
        assert_eq!(stats.count, 1000);
        assert_eq!(stats.p50, 100);
    }
}
