use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub total_time_ms: u64,
    pub iterations: u32,
    pub avg_latency_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub qps: f64,
}

impl BenchmarkMetrics {
    pub fn calculate(latencies: &[u64], iterations: u32) -> Self {
        if latencies.is_empty() {
            return Self {
                total_time_ms: 0,
                iterations,
                avg_latency_ms: 0.0,
                p50_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
                qps: 0.0,
            };
        }

        let mut sorted = latencies.to_vec();
        sorted.sort();

        let sum: u64 = sorted.iter().sum();
        let count = sorted.len() as u64;

        Self {
            total_time_ms: sum,
            iterations,
            avg_latency_ms: sum as f64 / count as f64,
            p50_ms: percentile(&sorted, 50) as f64,
            p95_ms: percentile(&sorted, 95) as f64,
            p99_ms: percentile(&sorted, 99) as f64,
            min_ms: sorted.first().copied().unwrap_or(0) as f64,
            max_ms: sorted.last().copied().unwrap_or(0) as f64,
            qps: 1000.0 * count as f64 / sum as f64,
        }
    }
}

fn percentile(sorted: &[u64], p: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (sorted.len() * p / 100).min(sorted.len() - 1);
    sorted[idx]
}

pub struct LatencyCollector {
    samples: Vec<u64>,
}

impl Default for LatencyCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyCollector {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn record(&mut self, latency_ns: u64) {
        self.samples.push(latency_ns / 1_000_000);
    }

    pub fn into_metrics(self, iterations: u32) -> BenchmarkMetrics {
        BenchmarkMetrics::calculate(&self.samples, iterations)
    }
}
