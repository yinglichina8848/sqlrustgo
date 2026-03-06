//! Network Cost Module
//!
//! This module provides network cost estimation interfaces
//! for future distributed queries (v2.0).

/// NetworkCost represents the cost of network operations in distributed queries
///
/// In distributed database systems, data often needs to be transferred between
/// nodes. This cost can be significant and should be considered in query optimization.
#[derive(Debug, Clone, Default)]
pub struct NetworkCost {
    /// Estimated bytes to transfer
    pub bytes: u64,
    /// Estimated latency in milliseconds
    pub latency_ms: f64,
    /// Number of network hops
    pub hops: u32,
}

impl NetworkCost {
    pub fn new(bytes: u64, latency_ms: f64, hops: u32) -> Self {
        Self {
            bytes,
            latency_ms,
            hops,
        }
    }

    /// Calculate total cost based on bytes and latency
    /// Using a simple model: cost = bytes * transfer_time + latency
    pub fn calculate(&self) -> f64 {
        // Assume 1MB/s = 1,048,576 bytes/s network throughput
        let transfer_time_ms = (self.bytes as f64 / 1_048_576.0) * 1000.0;
        transfer_time_ms + self.latency_ms
    }
}

/// NetworkCostEstimator trait - interface for network cost estimation
///
/// # Why (为什么)
/// In distributed scenarios, queries may need to transfer data between nodes.
/// Estimating this cost helps the optimizer choose efficient execution plans.
///
/// # How (如何实现)
/// Implement this trait for your specific distributed environment.
pub trait NetworkCostEstimator {
    /// Estimate network cost for transferring data
    ///
    /// # Arguments
    /// * `source_node` - Source node identifier
    /// * `target_node` - Target node identifier
    /// * `data_size_bytes` - Size of data to transfer in bytes
    fn estimate_cost(
        &self,
        source_node: &str,
        target_node: &str,
        data_size_bytes: u64,
    ) -> NetworkCost;
}

/// SimpleNetworkCostEstimator - basic implementation using constant values
#[derive(Debug, Clone, Default)]
pub struct SimpleNetworkCostEstimator {
    /// Assumed network bandwidth in bytes per second
    #[allow(dead_code)]
    bandwidth_bps: u64,
    /// Assumed latency per hop in milliseconds
    latency_per_hop_ms: f64,
}

impl SimpleNetworkCostEstimator {
    pub fn new(bandwidth_mbps: u64, latency_per_hop_ms: f64) -> Self {
        // Convert Mbps to bytes per second
        let bandwidth_bps = bandwidth_mbps * 125_000;
        Self {
            bandwidth_bps,
            latency_per_hop_ms,
        }
    }

    /// Default estimator assuming 1 Gbps network, 1ms latency per hop
    pub fn default_estimator() -> Self {
        // 1 Gbps = 125,000,000 bytes/s
        Self::new(1000, 1.0)
    }
}

impl NetworkCostEstimator for SimpleNetworkCostEstimator {
    fn estimate_cost(
        &self,
        source_node: &str,
        target_node: &str,
        data_size_bytes: u64,
    ) -> NetworkCost {
        let hops = if source_node == target_node { 0 } else { 1 };
        let latency_ms = hops as f64 * self.latency_per_hop_ms;

        NetworkCost::new(data_size_bytes, latency_ms, hops)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_cost_new() {
        let cost = NetworkCost::new(1024, 10.0, 1);
        assert_eq!(cost.bytes, 1024);
        assert_eq!(cost.latency_ms, 10.0);
        assert_eq!(cost.hops, 1);
    }

    #[test]
    fn test_network_cost_calculate() {
        // 1024 bytes = ~1KB
        // At 1MB/s, transfer time = 1KB / 1MB * 1000ms = ~1ms
        // Plus latency = 10ms, total = ~11ms
        let cost = NetworkCost::new(1024, 10.0, 1);
        let total = cost.calculate();
        assert!(total > 0.0);
    }

    #[test]
    fn test_simple_estimator_default() {
        let estimator = SimpleNetworkCostEstimator::default_estimator();
        let cost = estimator.estimate_cost("node1", "node2", 1000);
        assert_eq!(cost.hops, 1);
    }

    #[test]
    fn test_simple_estimator_same_node() {
        let estimator = SimpleNetworkCostEstimator::default_estimator();
        let cost = estimator.estimate_cost("node1", "node1", 1000);
        assert_eq!(cost.hops, 0);
        assert!(cost.latency_ms >= 0.0);
    }

    #[test]
    fn test_simple_estimator_custom() {
        // 100 Mbps, 2ms latency per hop
        let estimator = SimpleNetworkCostEstimator::new(100, 2.0);
        let cost = estimator.estimate_cost("node1", "node2", 1_000_000);
        assert_eq!(cost.hops, 1);
        assert_eq!(cost.latency_ms, 2.0);
    }
}
