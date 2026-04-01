use sqlrustgo_telemetry::{Metrics, PrometheusRenderer};
use std::time::Duration;

#[test]
fn test_render_queries_total() {
    let metrics = Metrics::new();
    metrics.record_query("SELECT", Duration::from_millis(10));
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_queries_total"));
    assert!(output.contains("# TYPE sqlrustgo_queries_total counter"));
    assert!(output.contains("# HELP sqlrustgo_queries_total"));
}

#[test]
fn test_render_connections() {
    let metrics = Metrics::new();
    metrics.connection_acquired();
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_connections_active"));
    assert!(output.contains("# TYPE sqlrustgo_connections_active gauge"));
    assert!(output.contains("sqlrustgo_connections_total"));
    assert!(output.contains("# TYPE sqlrustgo_connections_total counter"));
}

#[test]
fn test_render_cache() {
    let metrics = Metrics::new();
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_miss();
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_cache_hits"));
    assert!(output.contains("sqlrustgo_cache_misses"));
    assert!(output.contains("sqlrustgo_cache_hit_rate"));
}

#[test]
fn test_render_storage() {
    let metrics = Metrics::new();
    metrics.record_bytes_read(1024);
    metrics.record_bytes_written(512);
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_storage_read_bytes"));
    assert!(output.contains("sqlrustgo_storage_write_bytes"));
}

#[test]
fn test_render_duration_histogram() {
    let metrics = Metrics::new();
    metrics.record_query("SELECT", Duration::from_millis(10));
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_queries_duration_seconds"));
    assert!(output.contains("# TYPE sqlrustgo_queries_duration_seconds histogram"));
    assert!(output.contains("bucket=\"+Inf\""));
    assert!(output.contains("_sum"));
    assert!(output.contains("_count"));
}

#[test]
fn test_render_empty_metrics() {
    let metrics = Metrics::new();
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("sqlrustgo_queries_total 0"));
    assert!(output.contains("sqlrustgo_connections_active 0"));
    assert!(output.contains("sqlrustgo_cache_hit_rate 0"));
}

#[test]
fn test_prometheus_format_compliance() {
    let metrics = Metrics::new();
    metrics.record_query("SELECT", Duration::from_micros(1500));
    let output = PrometheusRenderer::render(&metrics);
    assert!(output.contains("# HELP sqlrustgo_"));
    assert!(output.contains("# TYPE sqlrustgo_"));
    assert!(output.contains("sqlrustgo_"));
}

#[test]
fn test_cache_hit_rate_zero_total() {
    let metrics = Metrics::new();
    assert_eq!(metrics.cache_hit_rate(), 0.0);
}

#[test]
fn test_avg_query_duration_zero_queries() {
    let metrics = Metrics::new();
    assert_eq!(metrics.avg_query_duration_us(), 0);
}

#[test]
fn test_duration_buckets() {
    let metrics = Metrics::new();
    metrics.record_query("Q", Duration::from_micros(50));
    metrics.record_query("Q", Duration::from_micros(300));
    metrics.record_query("Q", Duration::from_millis(2));
    metrics.record_query("Q", Duration::from_millis(100));
    let buckets = metrics.duration_buckets();
    assert_eq!(buckets[0], 1);
    assert_eq!(buckets[1], 1);
    assert_eq!(buckets[3], 1);
    assert_eq!(buckets[6], 1);
}

#[test]
fn test_metrics_connection_pool() {
    let metrics = Metrics::new();
    metrics.connection_acquired();
    metrics.connection_acquired();
    assert_eq!(metrics.connections_active(), 2);
    assert_eq!(metrics.connections_total(), 2);
    metrics.connection_released();
    assert_eq!(metrics.connections_active(), 1);
    assert_eq!(metrics.connections_total(), 2);
}

#[test]
fn test_metrics_storage_io() {
    let metrics = Metrics::new();
    metrics.record_bytes_read(1024);
    metrics.record_bytes_read(512);
    metrics.record_bytes_written(256);
    assert_eq!(metrics.storage_read_bytes(), 1536);
    assert_eq!(metrics.storage_write_bytes(), 256);
}

#[test]
fn test_metrics_reset() {
    let metrics = Metrics::new();
    metrics.record_query("SELECT", Duration::from_millis(10));
    metrics.record_cache_hit();
    metrics.connection_acquired();
    metrics.record_bytes_read(1024);
    metrics.reset();
    assert_eq!(metrics.queries_total(), 0);
    assert_eq!(metrics.cache_hits(), 0);
    assert_eq!(metrics.connections_active(), 0);
    assert_eq!(metrics.storage_read_bytes(), 0);
}

#[test]
fn test_metrics_new() {
    let metrics = Metrics::new();
    assert_eq!(metrics.queries_total(), 0);
    assert_eq!(metrics.connections_active(), 0);
    assert_eq!(metrics.cache_hit_rate(), 0.0);
}

#[test]
fn test_metrics_record_query() {
    let metrics = Metrics::new();
    metrics.record_query("SELECT", Duration::from_millis(10));
    assert_eq!(metrics.queries_total(), 1);
}

#[test]
fn test_metrics_record_cache_hit_miss() {
    let metrics = Metrics::new();
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_miss();
    assert_eq!(metrics.cache_hits(), 2);
    assert_eq!(metrics.cache_misses(), 1);
    assert!((metrics.cache_hit_rate() - 0.666).abs() < 0.01);
}
