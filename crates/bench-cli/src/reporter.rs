use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::metrics::BenchmarkMetrics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub timestamp: String,
    pub version: String,
    pub workload: String,
    pub config: serde_json::Value,
    pub metrics: BenchmarkMetrics,
    pub queries: Vec<QueryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub name: String,
    pub avg_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub iterations: u32,
}

impl BenchmarkResult {
    pub fn new(workload: String, config: serde_json::Value) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workload,
            config,
            metrics: BenchmarkMetrics::calculate(&[], 0),
            queries: Vec::new(),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn print_json(&self) {
        println!("{}", serde_json::to_string_pretty(self).unwrap());
    }
}
