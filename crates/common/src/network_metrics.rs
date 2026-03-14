//! Network Metrics Implementation
//!
//! Implements the Metrics trait for Network to track network performance metrics.

use crate::metrics::{MetricValue, Metrics};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct NetworkMetrics {
    connections_active: AtomicU64,
    connections_total: AtomicU64,
    connections_closed: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    packets_sent: AtomicU64,
    packets_received: AtomicU64,
    errors_total: AtomicU64,
}

impl NetworkMetrics {
    pub fn new() -> Self {
        Self {
            connections_active: AtomicU64::new(0),
            connections_total: AtomicU64::new(0),
            connections_closed: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            packets_sent: AtomicU64::new(0),
            packets_received: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
        }
    }

    pub fn connections_active(&self) -> u64 {
        self.connections_active.load(Ordering::Relaxed)
    }

    pub fn connections_total(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    pub fn connections_closed(&self) -> u64 {
        self.connections_closed.load(Ordering::Relaxed)
    }

    pub fn bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    pub fn bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    pub fn packets_sent(&self) -> u64 {
        self.packets_sent.load(Ordering::Relaxed)
    }

    pub fn packets_received(&self) -> u64 {
        self.packets_received.load(Ordering::Relaxed)
    }

    pub fn errors_total(&self) -> u64 {
        self.errors_total.load(Ordering::Relaxed)
    }

    pub fn record_connection_open(&self) {
        self.connections_active.fetch_add(1, Ordering::Relaxed);
        self.connections_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_connection_close(&self) {
        self.connections_active.fetch_sub(1, Ordering::Relaxed);
        self.connections_closed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_packet_sent(&self) {
        self.packets_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_packet_received(&self) {
        self.packets_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics for NetworkMetrics {
    fn record_query(&mut self, _query_type: &str, _duration_ms: u64) {}

    fn record_error(&mut self) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    fn record_error_with_type(&mut self, _error_type: &str) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }

    fn record_bytes_read(&mut self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    fn record_bytes_written(&mut self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    fn record_cache_hit(&mut self) {}

    fn record_cache_miss(&mut self) {}

    fn get_metric(&self, name: &str) -> Option<MetricValue> {
        match name {
            "connections_active" => Some(MetricValue::Gauge(self.connections_active() as f64)),
            "connections_total" => Some(MetricValue::Counter(self.connections_total())),
            "connections_closed" => Some(MetricValue::Counter(self.connections_closed())),
            "bytes_sent" => Some(MetricValue::Counter(self.bytes_sent())),
            "bytes_received" => Some(MetricValue::Counter(self.bytes_received())),
            "packets_sent" => Some(MetricValue::Counter(self.packets_sent())),
            "packets_received" => Some(MetricValue::Counter(self.packets_received())),
            "errors_total" => Some(MetricValue::Counter(self.errors_total())),
            _ => None,
        }
    }

    fn get_metric_names(&self) -> Vec<String> {
        vec![
            "connections_active".to_string(),
            "connections_total".to_string(),
            "connections_closed".to_string(),
            "bytes_sent".to_string(),
            "bytes_received".to_string(),
            "packets_sent".to_string(),
            "packets_received".to_string(),
            "errors_total".to_string(),
        ]
    }

    fn reset(&mut self) {
        self.connections_active.store(0, Ordering::Relaxed);
        self.connections_total.store(0, Ordering::Relaxed);
        self.connections_closed.store(0, Ordering::Relaxed);
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.packets_sent.store(0, Ordering::Relaxed);
        self.packets_received.store(0, Ordering::Relaxed);
        self.errors_total.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new();
        assert_eq!(metrics.connections_active(), 0);
    }

    #[test]
    fn test_network_metrics_connection_open() {
        let metrics = NetworkMetrics::new();
        metrics.record_connection_open();
        assert_eq!(metrics.connections_active(), 1);
    }

    #[test]
    fn test_network_metrics_connection_lifecycle() {
        let metrics = NetworkMetrics::new();
        metrics.record_connection_open();
        metrics.record_connection_close();
        assert_eq!(metrics.connections_active(), 0);
    }
}
