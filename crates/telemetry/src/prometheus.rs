//! Prometheus Metrics Renderer
//!
//! Renders metrics in Prometheus text exposition format 0.0.4

use crate::Metrics;

/// Prometheus metric types
#[derive(Debug, Clone, Copy)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
        }
    }
}

/// PrometheusRenderer - Renders Metrics to Prometheus text format
#[derive(Debug)]
pub struct PrometheusRenderer;

impl PrometheusRenderer {
    /// Render metrics to Prometheus exposition format
    ///
    /// Output format follows Prometheus text specification version 0.0.4
    pub fn render(metrics: &Metrics) -> String {
        let mut output = String::with_capacity(2048);

        // HELP and TYPE for queries_total
        Self::render_queries_total(metrics, &mut output);

        // HELP and TYPE for queries_duration_seconds (histogram)
        Self::render_queries_duration(metrics, &mut output);

        // HELP and TYPE for connections_active
        Self::render_connections_active(metrics, &mut output);

        // HELP and TYPE for connections_total
        Self::render_connections_total(metrics, &mut output);

        // HELP and TYPE for cache_hits
        Self::render_cache_hits(metrics, &mut output);

        // HELP and TYPE for cache_misses
        Self::render_cache_misses(metrics, &mut output);

        // HELP and TYPE for cache_hit_rate (gauge)
        Self::render_cache_hit_rate(metrics, &mut output);

        // HELP and TYPE for storage_read_bytes
        Self::render_storage_read_bytes(metrics, &mut output);

        // HELP and TYPE for storage_write_bytes
        Self::render_storage_write_bytes(metrics, &mut output);

        output
    }

    fn render_queries_total(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_queries_total Total number of queries executed"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_queries_total counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_queries_total {}",
            metrics.queries_total()
        );
    }

    fn render_queries_duration(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;

        // Prometheus histogram buckets (in seconds, using le = "+Inf" convention)
        // Bucket boundaries match our internal buckets (in microseconds)
        let bucket_boundaries_us = [100, 500, 1000, 5000, 10000, 50000, 100000, 500000, 1000000, 5000000];
        let bucket_boundaries_s: Vec<f64> = bucket_boundaries_us.iter().map(|&u| u as f64 / 1_000_000.0).collect();
        let buckets = metrics.duration_buckets();

        let _ = writeln!(
            output,
            "# HELP sqlrustgo_queries_duration_seconds Query duration histogram"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_queries_duration_seconds histogram"
        );

        // Cumulative count for histogram
        let mut cumulative = 0u64;
        for (i, &count) in buckets.iter().enumerate() {
            cumulative += count;
            let le = bucket_boundaries_s[i];
            let _ = writeln!(
                output,
                "sqlrustgo_queries_duration_seconds{{bucket=\"{:.6}\"}} {}",
                le, cumulative
            );
        }
        // +Inf bucket
        let _ = writeln!(
            output,
            "sqlrustgo_queries_duration_seconds{{bucket=\"+Inf\"}} {}",
            cumulative
        );

        // Sum and count for histogram
        let sum_us = metrics.queries_duration_us();
        let sum_s = sum_us as f64 / 1_000_000.0;
        let _ = writeln!(
            output,
            "sqlrustgo_queries_duration_seconds_sum {}",
            sum_s
        );
        let _ = writeln!(
            output,
            "sqlrustgo_queries_duration_seconds_count {}",
            metrics.queries_total()
        );
    }

    fn render_connections_active(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_connections_active Number of active connections"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_connections_active gauge"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_connections_active {}",
            metrics.connections_active()
        );
    }

    fn render_connections_total(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_connections_total Total number of connections created"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_connections_total counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_connections_total {}",
            metrics.connections_total()
        );
    }

    fn render_cache_hits(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_cache_hits Total number of cache hits"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_cache_hits counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_cache_hits {}",
            metrics.cache_hits()
        );
    }

    fn render_cache_misses(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_cache_misses Total number of cache misses"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_cache_misses counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_cache_misses {}",
            metrics.cache_misses()
        );
    }

    fn render_cache_hit_rate(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_cache_hit_rate Cache hit rate (0.0 to 1.0)"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_cache_hit_rate gauge"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_cache_hit_rate {}",
            metrics.cache_hit_rate()
        );
    }

    fn render_storage_read_bytes(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_storage_read_bytes Total bytes read from storage"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_storage_read_bytes counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_storage_read_bytes {}",
            metrics.storage_read_bytes()
        );
    }

    fn render_storage_write_bytes(metrics: &Metrics, output: &mut String) {
        use std::fmt::Write;
        let _ = writeln!(
            output,
            "# HELP sqlrustgo_storage_write_bytes Total bytes written to storage"
        );
        let _ = writeln!(
            output,
            "# TYPE sqlrustgo_storage_write_bytes counter"
        );
        let _ = writeln!(
            output,
            "sqlrustgo_storage_write_bytes {}",
            metrics.storage_write_bytes()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_queries_total() {
        let metrics = Metrics::new();
        metrics.record_query("SELECT", std::time::Duration::from_millis(10));
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
        metrics.record_query("SELECT", std::time::Duration::from_millis(10));
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
        // All metrics should be present even if zero
        assert!(output.contains("sqlrustgo_queries_total 0"));
        assert!(output.contains("sqlrustgo_connections_active 0"));
        assert!(output.contains("sqlrustgo_cache_hit_rate 0"));
    }

    #[test]
    fn test_prometheus_format_compliance() {
        let metrics = Metrics::new();
        metrics.record_query("SELECT", std::time::Duration::from_micros(1500));
        let output = PrometheusRenderer::render(&metrics);

        // Check HELP comments are present
        assert!(output.contains("# HELP sqlrustgo_"));
        // Check TYPE comments are present
        assert!(output.contains("# TYPE sqlrustgo_"));
        // Check metric name format
        assert!(output.contains("sqlrustgo_"));
    }
}
