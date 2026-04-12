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

        for _ in 0..1000 {
            recorder.record(100);
        }

        let stats = recorder.snapshot();
        assert_eq!(stats.count, 1000);
        assert_eq!(stats.p50, 100);
    }

    #[test]
    fn test_latency_recorder_empty() {
        let recorder = LatencyRecorder::new();
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_latency_recorder_single_sample() {
        let recorder = LatencyRecorder::new();
        recorder.record(500);
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.p50, 500);
        assert_eq!(stats.p95, 500);
        assert_eq!(stats.p99, 500);
    }

    #[test]
    fn test_latency_recorder_multiple_values() {
        let recorder = LatencyRecorder::new();
        for i in 1..=100 {
            recorder.record(i * 100);
        }
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 100);
        assert!(stats.p50 > 0);
        assert!(stats.p95 > stats.p50);
        assert!(stats.p99 > stats.p95);
    }

    #[test]
    fn test_latency_recorder_varied_latency() {
        let recorder = LatencyRecorder::new();
        for i in 1..=1000 {
            let latency = if i % 100 == 0 { 10000 } else { 100 };
            recorder.record(latency);
        }
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 1000);
        assert!(stats.max >= stats.p50);
    }

    #[test]
    fn test_latency_recorder_concurrent() {
        let recorder = LatencyRecorder::new();
        std::thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| {
                    for i in 1..=250 {
                        recorder.record(i);
                    }
                });
            }
        });
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 1000);
    }

    #[test]
    fn test_latency_stats_default() {
        let stats = LatencyStats::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.p50, 0);
        assert_eq!(stats.p95, 0);
    }

    #[test]
    fn test_latency_recorder_default() {
        let recorder = LatencyRecorder::default();
        let stats = recorder.snapshot();
        assert_eq!(stats.count, 0);
    }
}
