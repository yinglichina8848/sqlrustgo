//! Performance analysis - automatically detect bottlenecks

use crate::metrics::latency::LatencyStats;

/// Analyze benchmark results and detect bottlenecks
pub fn analyze(stats: &LatencyStats) {
    println!();
    println!("=== ANALYSIS ===");

    // Check for high tail latency
    if stats.p99 > 5000 {
        println!("⚠️ High tail latency detected (P99 > 5ms)");
        println!("   Possible causes:");
        println!("   - Lock contention");
        println!("   - WAL flush bottleneck");
        println!("   - Memory pressure / GC");
    }

    // Check for very high P99
    if stats.p99 > 10000 {
        println!("⚠️ Severe tail latency (P99 > 10ms)");
        println!("   Recommendation: Investigate system bottlenecks");
    }

    // Latency distribution analysis
    let p99_p50_ratio = if stats.p50 > 0 {
        stats.p99 as f64 / stats.p50 as f64
    } else {
        0.0
    };

    if p99_p50_ratio > 10.0 {
        println!("⚠️ High latency variance (P99/P50 > 10x)");
        println!("   This indicates significant tail latency issues");
    }

    // Check sample count
    if stats.count < 1000 {
        println!("⚠️ Low sample count - results may not be statistically significant");
    }

    // Performance classification
    println!();
    println!("=== PERFORMANCE CLASSIFICATION ===");

    if stats.p99 < 1000 {
        println!("⭐ Excellent: P99 < 1ms");
    } else if stats.p99 < 5000 {
        println!("✅ Good: P99 < 5ms");
    } else if stats.p99 < 10000 {
        println!("⚠️ Moderate: P99 < 10ms");
    } else {
        println!("❌ Poor: P99 >= 10ms");
    }
}
