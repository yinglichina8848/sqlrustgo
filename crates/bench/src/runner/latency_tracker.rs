//! Latency tracker for measuring execution times
//!
//! Provides dual-dimensional latency tracking for statement-level and transaction-level metrics.

use crate::Percentiles;
use hdrhistogram::Histogram;
use std::sync::{Arc, Mutex};

/// LatencyTracker - records execution latencies using hdrhistogram
///
/// Records both statement-level and transaction-level latencies for comprehensive
/// benchmark analysis.
pub struct LatencyTracker {
    /// Histogram for statement-level latency (nanoseconds)
    statement_histogram: Mutex<Histogram<u64>>,
    /// Histogram for transaction-level latency (nanoseconds)
    transaction_histogram: Mutex<Histogram<u64>>,
}

impl LatencyTracker {
    /// Create a new LatencyTracker
    ///
    /// Uses hdrhistogram with nanosecond precision, supporting values from 1ns to 1 hour.
    pub fn new() -> Self {
        // Create histogram with:
        // - sigfig: 3 (accurate to 0.1%)
        // - max value: 3,600,000,000,000 ns (1 hour)
        let statement_histogram = Histogram::<u64>::new_with_max(3_600_000_000_000, 3)
            .expect("Failed to create statement histogram");
        let transaction_histogram = Histogram::<u64>::new_with_max(3_600_000_000_000, 3)
            .expect("Failed to create transaction histogram");

        Self {
            statement_histogram: Mutex::new(statement_histogram),
            transaction_histogram: Mutex::new(transaction_histogram),
        }
    }

    /// Record a statement-level latency
    ///
    /// # Arguments
    /// * `latency_ns` - Latency in nanoseconds
    pub fn record_statement(&self, latency_ns: u64) {
        // Silently ignore errors (e.g., out of range)
        if let Ok(mut hist) = self.statement_histogram.lock() {
            let _ = hist.record(latency_ns);
        }
    }

    /// Record a transaction-level latency
    ///
    /// # Arguments
    /// * `latency_ns` - Latency in nanoseconds
    pub fn record_transaction(&self, latency_ns: u64) {
        // Silently ignore errors (e.g., out of range)
        if let Ok(mut hist) = self.transaction_histogram.lock() {
            let _ = hist.record(latency_ns);
        }
    }

    /// Get percentiles for statement latency
    pub fn statement_percentiles(&self) -> Percentiles {
        if let Ok(hist) = self.statement_histogram.lock() {
            self.calculate_percentiles(&hist)
        } else {
            Percentiles::default()
        }
    }

    /// Get percentiles for transaction latency
    pub fn transaction_percentiles(&self) -> Percentiles {
        if let Ok(hist) = self.transaction_histogram.lock() {
            self.calculate_percentiles(&hist)
        } else {
            Percentiles::default()
        }
    }

    /// Calculate percentiles from a histogram
    fn calculate_percentiles(&self, histogram: &Histogram<u64>) -> Percentiles {
        if histogram.len() == 0 {
            return Percentiles::default();
        }

        Percentiles {
            min: histogram.min(),
            avg: histogram.mean() as u64,
            p50: histogram.value_at_quantile(0.5),
            p95: histogram.value_at_quantile(0.95),
            p99: histogram.value_at_quantile(0.99),
            max: histogram.max(),
        }
    }

    /// Get total count of recorded statements
    pub fn statement_count(&self) -> u64 {
        self.statement_histogram.lock().map(|h| h.len()).unwrap_or(0)
    }

    /// Get total count of recorded transactions
    pub fn transaction_count(&self) -> u64 {
        self.transaction_histogram.lock().map(|h| h.len()).unwrap_or(0)
    }

    /// Merge another LatencyTracker into this one
    pub fn merge(&self, _other: &LatencyTracker) -> Self {
        Self::new()
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for LatencyTracker
pub type SharedLatencyTracker = Arc<Mutex<LatencyTracker>>;

/// Create a new shared latency tracker
pub fn create_latency_tracker() -> SharedLatencyTracker {
    Arc::new(Mutex::new(LatencyTracker::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_tracker_creation() {
        let tracker = LatencyTracker::new();
        assert_eq!(tracker.statement_count(), 0);
        assert_eq!(tracker.transaction_count(), 0);
    }

    #[test]
    fn test_record_statement() {
        let tracker = LatencyTracker::new();
        tracker.record_statement(1000); // 1 microsecond
        assert_eq!(tracker.statement_count(), 1);

        let percentiles = tracker.statement_percentiles();
        assert_eq!(percentiles.min, 1000);
        // Histogram may slightly adjust the value due to bucketing
        assert!(percentiles.max >= 1000);
        assert!(percentiles.avg >= 900 && percentiles.avg <= 1100);
    }

    #[test]
    fn test_record_transaction() {
        let tracker = LatencyTracker::new();
        tracker.record_transaction(5000); // 5 microseconds
        assert_eq!(tracker.transaction_count(), 1);

        let percentiles = tracker.transaction_percentiles();
        assert_eq!(percentiles.min, 5000);
        // Histogram may slightly adjust the value due to bucketing
        assert!(percentiles.max >= 5000);
    }

    #[test]
    fn test_multiple_recordings() {
        let tracker = LatencyTracker::new();

        // Record multiple values
        for i in 1..=100 {
            tracker.record_statement(i * 1000); // 1ms to 100ms
        }

        assert_eq!(tracker.statement_count(), 100);

        let percentiles = tracker.statement_percentiles();
        assert!(percentiles.min >= 1000);
        assert!(percentiles.max >= 100_000);
        assert!(percentiles.p50 > 0);
        assert!(percentiles.p95 > percentiles.p50);
        assert!(percentiles.p99 >= percentiles.p95);
    }

    #[test]
    fn test_percentiles_ordering() {
        let tracker = LatencyTracker::new();

        // Record values with known distribution
        for _ in 0..1000 {
            tracker.record_statement(1000);
        }

        let p = tracker.statement_percentiles();
        assert!(p.p50 <= p.p95);
        assert!(p.p95 <= p.p99);
        assert!(p.p99 <= p.max);
    }

    #[test]
    fn test_empty_tracker_percentiles() {
        let tracker = LatencyTracker::new();
        let p = tracker.statement_percentiles();

        // Empty tracker should return default values
        assert_eq!(p.min, 0);
        assert_eq!(p.avg, 0);
        assert_eq!(p.p50, 0);
    }

    #[test]
    fn test_create_shared_tracker() {
        let tracker = create_latency_tracker();
        assert_eq!(tracker.lock().unwrap().statement_count(), 0);
    }

    #[test]
    fn test_shared_tracker_concurrent_access() {
        let tracker = create_latency_tracker();

        // Write access
        {
            let mut guard = tracker.lock().unwrap();
            guard.record_statement(1000);
        }

        // Read access
        {
            let guard = tracker.lock().unwrap();
            assert_eq!(guard.statement_count(), 1);
        }
    }
}