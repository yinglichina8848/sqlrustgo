//! Metrics trait definitions for SQLRustGo
//!
//! This module defines the core metrics infrastructure for monitoring
//! and observing the database system.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Counter metric - monotonically increasing value
#[derive(Debug)]
pub struct Counter {
    name: String,
    value: AtomicU64,
}

impl Counter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: AtomicU64::new(0),
        }
    }
    
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }
    
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Gauge metric - point-in-time value
#[derive(Debug)]
pub struct Gauge {
    name: String,
    value: AtomicU64,
}

impl Gauge {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: AtomicU64::new(0),
        }
    }
    
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
    }
    
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Histogram metric - distribution of values
#[derive(Debug)]
pub struct Histogram {
    name: String,
    count: AtomicU64,
    sum: AtomicU64,
}

impl Histogram {
    pub fn new(name: &str, _buckets: &[u64]) -> Self {
        Self {
            name: name.to_string(),
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
        }
    }
    
    pub fn observe(&self, value: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);
    }
    
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
    
    pub fn sum(&self) -> u64 {
        self.sum.load(Ordering::Relaxed)
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Metrics trait - base interface for all metrics
pub trait Metric: Send + Sync {
    fn name(&self) -> &str;
    fn metric_type(&self) -> MetricType;
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Metrics registry - collection of all metrics
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    counters: Vec<Arc<Counter>>,
    gauges: Vec<Arc<Gauge>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register_counter(&mut self, counter: Arc<Counter>) {
        self.counters.push(counter);
    }
    
    pub fn register_gauge(&mut self, gauge: Arc<Gauge>) {
        self.gauges.push(gauge);
    }
    
    pub fn get_counter(&self, name: &str) -> Option<Arc<Counter>> {
        self.counters.iter().find(|c| c.name() == name).cloned()
    }
    
    pub fn get_gauge(&self, name: &str) -> Option<Arc<Gauge>> {
        self.gauges.iter().find(|g| g.name() == name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter");
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.add(5);
        assert_eq!(counter.get(), 6);
    }
    
    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge");
        gauge.set(100);
        assert_eq!(gauge.get(), 100);
    }
    
    #[test]
    fn test_registry() {
        let mut registry = MetricsRegistry::new();
        let counter = Arc::new(Counter::new("requests"));
        registry.register_counter(counter.clone());
        
        let found = registry.get_counter("requests");
        assert!(found.is_some());
    }
}
