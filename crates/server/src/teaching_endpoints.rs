//! Teaching Enhanced API Endpoints
//!
//! Provides Web UI endpoints for visualizing query execution pipelines
//! and operator-level profiling. This is a key differentiation feature
//! that MySQL doesn't provide - making SQLRustGo 2.0 ideal for teaching
//! and learning query optimization.

use crate::metrics_endpoint::MetricsRegistry;
use sqlrustgo_executor::{OperatorProfile, QueryTrace, GLOBAL_PROFILER, GLOBAL_TRACE_COLLECTOR};
use std::sync::{Arc, RwLock};

/// Teaching Enhanced endpoints configuration
#[derive(Clone)]
pub struct TeachingEndpoints {
    /// Enable pipeline visualization
    pub enable_pipeline_viz: bool,
    /// Enable profiling UI
    pub enable_profiling: bool,
    /// Enable trace logging
    pub enable_trace: bool,
    /// Maximum traces to keep
    pub max_traces: usize,
    /// Maximum profiles to keep
    pub max_profiles: usize,
}

impl Default for TeachingEndpoints {
    fn default() -> Self {
        Self {
            enable_pipeline_viz: true,
            enable_profiling: true,
            enable_trace: true,
            max_traces: 1000,
            max_profiles: 100,
        }
    }
}

/// Extended HTTP server with teaching enhanced features
#[derive(Clone)]
pub struct TeachingHttpServer {
    host: String,
    port: u16,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
    teaching_endpoints: TeachingEndpoints,
}

impl TeachingHttpServer {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            version: "2.0.0".to_string(),
            metrics_registry: Arc::new(RwLock::new(MetricsRegistry::new())),
            teaching_endpoints: TeachingEndpoints::default(),
        }
    }

    pub fn with_teaching_endpoints(mut self, endpoints: TeachingEndpoints) -> Self {
        self.teaching_endpoints = endpoints;
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_metrics_registry(mut self, registry: Arc<RwLock<MetricsRegistry>>) -> Self {
        self.metrics_registry = registry;
        self
    }

    /// Get server version
    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    /// Get server port
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Start the teaching enhanced HTTP server
    pub fn start(&self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = std::net::TcpListener::bind(&addr)?;

        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║          SQLRustGo 2.0 - Teaching Enhanced Server               ║");
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!(
            "║  Server started on http://{}                                ║",
            addr
        );
        println!("╠══════════════════════════════════════════════════════════════════╣");
        println!("║  Standard Endpoints:                                             ║");
        println!("║    - GET /health/live     - Liveness probe                       ║");
        println!("║    - GET /health/ready    - Readiness probe                      ║");
        println!("║    - GET /metrics         - Prometheus metrics                   ║");

        if self.teaching_endpoints.enable_pipeline_viz {
            println!("║                                                                   ║");
            println!("║  Teaching Enhanced Endpoints:                                    ║");
            println!("║    - GET /teaching/pipeline       - Query pipeline visualization ║");
            println!("║    - GET /teaching/pipeline/json   - Pipeline as JSON            ║");
            println!("║    - GET /teaching/trace          - Vectorized trace log         ║");
        }

        if self.teaching_endpoints.enable_profiling {
            println!("║    - GET /teaching/profile        - Operator profiling report   ║");
            println!("║    - GET /teaching/profile/json   - Profile as JSON             ║");
            println!("║    - GET /teaching/profile/operators - Individual operator stats ║");
        }

        println!("╚══════════════════════════════════════════════════════════════════╝");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let version = self.version.clone();
                    let metrics_registry = Arc::clone(&self.metrics_registry);
                    let teaching = self.teaching_endpoints.clone();

                    std::thread::spawn(move || {
                        let _ = handle_teaching_request(
                            &mut stream,
                            &version,
                            &metrics_registry,
                            &teaching,
                        );
                    });
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// Handle teaching enhanced HTTP requests
fn handle_teaching_request<T: std::io::Read + std::io::Write>(
    stream: &mut T,
    version: &str,
    metrics_registry: &Arc<RwLock<MetricsRegistry>>,
    teaching: &TeachingEndpoints,
) -> Result<(), std::io::Error> {
    let mut buffer = [0u8; 2048];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    let (status, content_type, body) = if let Some(request_line) = lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let path = parts[1];

            match path {
                // Standard endpoints
                "/health/live" => {
                    let body = serde_json::json!({
                        "status": "healthy",
                        "feature": "teaching_enhanced"
                    })
                    .to_string();
                    ("HTTP/1.1 200 OK", "application/json", body)
                }
                "/health/ready" => {
                    let body = serde_json::json!({
                        "status": "ready",
                        "version": version,
                        "edition": "SQLRustGo 2.0 - Teaching Enhanced"
                    })
                    .to_string();
                    ("HTTP/1.1 200 OK", "application/json", body)
                }
                "/metrics" => {
                    let registry = metrics_registry.read().unwrap();
                    let prometheus_output = registry.to_prometheus_format();
                    (
                        "HTTP/1.1 200 OK",
                        "text/plain; version=0.0.4",
                        prometheus_output,
                    )
                }

                // Teaching enhanced endpoints
                "/teaching/pipeline" if teaching.enable_pipeline_viz => {
                    let html = generate_pipeline_html();
                    ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                }

                "/teaching/pipeline/json" if teaching.enable_pipeline_viz => {
                    let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                    let json = serde_json::to_string_pretty(&traces).unwrap_or_default();
                    ("HTTP/1.1 200 OK", "application/json", json)
                }

                "/teaching/pipeline/latest" if teaching.enable_pipeline_viz => {
                    let trace = GLOBAL_TRACE_COLLECTOR.latest();
                    match trace {
                        Some(t) => {
                            let html = generate_pipeline_detail_html(&t);
                            ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                        }
                        None => {
                            let body = serde_json::json!({
                                "message": "No traces available",
                                "hint": "Execute a query to see pipeline visualization"
                            })
                            .to_string();
                            ("HTTP/1.1 200 OK", "application/json", body)
                        }
                    }
                }

                "/teaching/trace" if teaching.enable_trace => {
                    let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                    let html = generate_trace_html(&traces);
                    ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                }

                "/teaching/trace/json" if teaching.enable_trace => {
                    let traces = GLOBAL_TRACE_COLLECTOR.get_traces();
                    let json = serde_json::to_string_pretty(&traces).unwrap_or_default();
                    ("HTTP/1.1 200 OK", "application/json", json)
                }

                "/teaching/profile" if teaching.enable_profiling => {
                    let report = GLOBAL_PROFILER.generate_report();
                    let html = generate_profile_html(&report);
                    ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                }

                "/teaching/profile/json" if teaching.enable_profiling => {
                    let json = GLOBAL_PROFILER.to_json();
                    ("HTTP/1.1 200 OK", "application/json", json)
                }

                "/teaching/profile/operators" if teaching.enable_profiling => {
                    let profiles = GLOBAL_PROFILER.get_sorted_profiles();
                    let html = generate_operators_html(&profiles);
                    ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                }

                "/teaching" => {
                    let html = generate_teaching_index_html();
                    ("HTTP/1.1 200 OK", "text/html; charset=utf-8", html)
                }

                _ => (
                    "HTTP/1.1 404 Not Found",
                    "application/json",
                    serde_json::json!({
                        "error": "Not Found",
                        "message": format!("Path '{}' not found", path)
                    })
                    .to_string(),
                ),
            }
        } else {
            (
                "HTTP/1.1 400 Bad Request",
                "application/json",
                r#"{"error": "Bad Request"}"#.to_string(),
            )
        }
    } else {
        (
            "HTTP/1.1 400 Bad Request",
            "application/json",
            r#"{"error": "Bad Request"}"#.to_string(),
        )
    };

    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

/// Generate HTML for pipeline visualization index
fn generate_teaching_index_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>SQLRustGo 2.0 - Teaching Enhanced</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eee;
        }}
        h1 {{
            color: #00d4ff;
            border-bottom: 2px solid #00d4ff;
            padding-bottom: 10px;
        }}
        .feature {{
            background: #16213e;
            border-radius: 8px;
            padding: 20px;
            margin: 20px 0;
            border-left: 4px solid #00d4ff;
        }}
        .feature h2 {{
            color: #00d4ff;
            margin-top: 0;
        }}
        .feature a {{
            color: #00d4ff;
            text-decoration: none;
            padding: 8px 16px;
            background: #0f3460;
            border-radius: 4px;
            display: inline-block;
            margin: 5px;
        }}
        .feature a:hover {{
            background: #00d4ff;
            color: #1a1a2e;
        }}
        .badge {{
            background: #e94560;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
        }}
        .code {{
            background: #0f0f23;
            padding: 10px;
            border-radius: 4px;
            font-family: monospace;
            overflow-x: auto;
        }}
    </style>
</head>
<body>
    <h1>🎓 SQLRustGo 2.0 - Teaching Enhanced</h1>
    <p>Differentiating features that MySQL doesn't provide!</p>
    
    <div class="feature">
        <h2>📊 Visual Pipeline Execution</h2>
        <p>Visualize how your SQL query is executed as a pipeline of operators.</p>
        <a href="/teaching/pipeline">Pipeline Visualization</a>
        <a href="/teaching/pipeline/json">JSON API</a>
        <a href="/teaching/pipeline/latest">Latest Query</a>
    </div>
    
    <div class="feature">
        <h2>🔍 Vectorized Trace Log</h2>
        <p>Detailed execution traces showing vectorized batch processing.</p>
        <a href="/teaching/trace">Trace Log</a>
        <a href="/teaching/trace/json">JSON API</a>
    </div>
    
    <div class="feature">
        <h2>⚡ Operator Profiling</h2>
        <p>Per-operator performance profiling to understand query bottlenecks.</p>
        <a href="/teaching/profile">Profiling Report</a>
        <a href="/teaching/profile/json">JSON API</a>
        <a href="/teaching/profile/operators">Operator Stats</a>
    </div>
    
    <div class="feature">
        <h2>✨ Key Differentiators</h2>
        <ul>
            <li><strong>MySQL can't do this:</strong> Real-time pipeline visualization</li>
            <li><strong>MySQL can't do this:</strong> Vectorized execution tracing</li>
            <li><strong>MySQL can't do this:</strong> Operator-level profiling UI</li>
            <li>Perfect for teaching query optimization</li>
            <li>Great for learning how databases work</li>
        </ul>
    </div>
</body>
</html>"#
    )
}

/// Generate HTML for pipeline visualization
fn generate_pipeline_html() -> String {
    let traces = GLOBAL_TRACE_COLLECTOR.get_traces();

    let mut trace_list = String::new();
    for trace in traces.iter().take(20) {
        trace_list.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{} rows</td>
                <td><a href="/teaching/pipeline/latest">View</a></td>
            </tr>"#,
            &trace.query_id[..8],
            truncate_string(&trace.sql, 40),
            format_duration(trace.total_duration_ns),
            trace.total_rows
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Pipeline Visualization - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #58a6ff; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #30363d;
        }}
        th {{ background: #161b22; color: #58a6ff; }}
        tr:hover {{ background: #161b22; }}
        a {{ color: #58a6ff; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .empty {{ color: #8b949e; font-style: italic; }}
        .home-link {{
            display: inline-block;
            margin-bottom: 20px;
            padding: 8px 16px;
            background: #21262d;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <a href="/teaching" class="home-link">← Back to Teaching Enhanced</a>
    <h1>📊 Query Pipeline Visualization</h1>
    <p>These are the queries that have been executed with pipeline tracing enabled.</p>
    
    <table>
        <thead>
            <tr>
                <th>Query ID</th>
                <th>SQL</th>
                <th>Duration</th>
                <th>Rows</th>
                <th>Action</th>
            </tr>
        </thead>
        <tbody>
            {trace_list}
        </tbody>
    </table>
    
    {empty_msg}
</body>
</html>"#,
        trace_list = if trace_list.is_empty() {
            r#"<p class="empty">No queries executed yet. Run a query to see the pipeline visualization.</p>"#.to_string()
        } else {
            String::new()
        },
        empty_msg = ""
    )
}

/// Generate HTML for detailed pipeline view
fn generate_pipeline_detail_html(trace: &QueryTrace) -> String {
    let viz = trace.visualize_pipeline();
    let escaped_viz = viz.replace("<", "&lt;").replace(">", "&gt;");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Pipeline Detail - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #58a6ff; }}
        .pipeline {{
            background: #161b22;
            padding: 20px;
            border-radius: 8px;
            overflow-x: auto;
            white-space: pre;
            font-size: 14px;
            line-height: 1.6;
        }}
        .query-info {{
            background: #21262d;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .json-link {{
            display: inline-block;
            margin-top: 20px;
            padding: 8px 16px;
            background: #238636;
            border-radius: 4px;
        }}
    </style>
</head>
<body>
    <a href="/teaching/pipeline" style="color: #58a6ff;">← Back to Pipeline List</a>
    <h1>📊 Pipeline Execution Detail</h1>
    
    <div class="query-info">
        <p><strong>Query ID:</strong> {query_id}</p>
        <p><strong>SQL:</strong> {sql}</p>
        <p><strong>Duration:</strong> {duration}</p>
        <p><strong>Operators:</strong> {operators}</p>
        <p><strong>Total Rows:</strong> {total_rows}</p>
    </div>
    
    <h2>Execution Tree</h2>
    <div class="pipeline">
{visualization}
    </div>
    
    <a href="/teaching/pipeline/json" class="json-link">View as JSON</a>
</body>
</html>"#,
        query_id = trace.query_id,
        sql = trace.sql,
        duration = format_duration(trace.total_duration_ns),
        operators = trace.operator_count,
        total_rows = trace.total_rows,
        visualization = escaped_viz
    )
}

/// Generate HTML for trace log
fn generate_trace_html(traces: &[QueryTrace]) -> String {
    let mut trace_entries = String::new();
    for trace in traces.iter().take(50) {
        trace_entries.push_str(&format!(
            r#"<div class="trace-entry">
                <div class="trace-header">
                    <span class="trace-id">#{trace_id}</span>
                    <span class="trace-sql">{sql}</span>
                    <span class="trace-time">{duration}</span>
                </div>
            </div>"#,
            trace_id = &trace.query_id[..8],
            sql = truncate_string(&trace.sql, 60),
            duration = format_duration(trace.total_duration_ns)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Vectorized Trace Log - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #a371f7; }}
        .trace-entry {{
            background: #161b22;
            margin: 10px 0;
            padding: 15px;
            border-radius: 6px;
            border-left: 3px solid #a371f7;
        }}
        .trace-header {{
            display: flex;
            justify-content: space-between;
        }}
        .trace-id {{
            color: #a371f7;
            font-weight: bold;
        }}
        .trace-sql {{
            color: #c9d1d9;
        }}
        .trace-time {{
            color: #8b949e;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .empty {{ color: #8b949e; font-style: italic; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>🔍 Vectorized Trace Log</h1>
    <p>Detailed execution traces showing vectorized batch processing.</p>
    
    {trace_entries}
    
    {empty_msg}
</body>
</html>"#,
        trace_entries = if trace_entries.is_empty() {
            r#"<p class="empty">No traces available. Execute a query to generate traces.</p>"#
                .to_string()
        } else {
            trace_entries
        },
        empty_msg = ""
    )
}

/// Generate HTML for profiling report
fn generate_profile_html(report: &str) -> String {
    let escaped_report = report.replace("<", "&lt;").replace(">", "&gt;");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Operator Profiling - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Courier New', monospace;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #3fb950; }}
        .report {{
            background: #161b22;
            padding: 20px;
            border-radius: 8px;
            overflow-x: auto;
            white-space: pre;
            font-size: 13px;
            line-height: 1.5;
        }}
        a {{ color: #58a6ff; text-decoration: none; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>⚡ Operator Profiling Report</h1>
    <p>Per-operator performance profiling to identify bottlenecks.</p>
    
    <div class="report">
{report}
    </div>
    
    <a href="/teaching/profile/json" style="display: inline-block; margin-top: 20px; padding: 8px 16px; background: #238636; border-radius: 4px;">View as JSON</a>
</body>
</html>"#,
        report = escaped_report
    )
}

/// Generate HTML for individual operator stats
fn generate_operators_html(profiles: &[OperatorProfile]) -> String {
    let mut operator_rows = String::new();
    for profile in profiles.iter().take(50) {
        operator_rows.push_str(&format!(
            r#"<tr>
                <td>{name}</td>
                <td>{execs}</td>
                <td>{avg}</td>
                <td>{total}</td>
                <td>{min}</td>
                <td>{max}</td>
                <td>{rows}</td>
                <td>{rps}</td>
            </tr>"#,
            name = profile.operator_name,
            execs = profile.execution_count,
            avg = profile.format_avg_time(),
            total = profile.format_total_time(),
            min = profile.format_min_time(),
            max = profile.format_max_time(),
            rows = profile.rows_processed,
            rps = format!("{:.0}", profile.rows_per_second)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Operator Statistics - SQLRustGo 2.0</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            background: #0d1117;
            color: #c9d1d9;
        }}
        h1 {{ color: #3fb950; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #30363d;
        }}
        th {{ background: #161b22; color: #3fb950; }}
        tr:hover {{ background: #161b22; }}
        a {{ color: #58a6ff; text-decoration: none; }}
        .empty {{ color: #8b949e; font-style: italic; }}
    </style>
</head>
<body>
    <a href="/teaching" style="color: #58a6ff;">← Back to Teaching Enhanced</a>
    <h1>⚡ Individual Operator Statistics</h1>
    
    <table>
        <thead>
            <tr>
                <th>Operator</th>
                <th>Executions</th>
                <th>Avg Time</th>
                <th>Total Time</th>
                <th>Min Time</th>
                <th>Max Time</th>
                <th>Rows</th>
                <th>Rows/sec</th>
            </tr>
        </thead>
        <tbody>
            {operator_rows}
        </tbody>
    </table>
    
    {empty_msg}
</body>
</html>"#,
        operator_rows = if operator_rows.is_empty() {
            r#"<p class="empty">No profiling data available. Execute some queries first.</p>"#
                .to_string()
        } else {
            operator_rows
        },
        empty_msg = ""
    )
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_duration(ns: u64) -> String {
    if ns < 1_000 {
        format!("{}ns", ns)
    } else if ns < 1_000_000 {
        format!("{:.1}µs", ns as f64 / 1_000.0)
    } else if ns < 1_000_000_000 {
        format!("{:.2}ms", ns as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", ns as f64 / 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teaching_endpoints_default() {
        let endpoints = TeachingEndpoints::default();
        assert!(endpoints.enable_pipeline_viz);
        assert!(endpoints.enable_profiling);
        assert!(endpoints.enable_trace);
    }

    #[test]
    fn test_teaching_http_server_creation() {
        let server = TeachingHttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_teaching_http_server_with_endpoints() {
        let endpoints = TeachingEndpoints {
            enable_pipeline_viz: false,
            enable_profiling: true,
            enable_trace: true,
            max_traces: 100,
            max_profiles: 50,
        };
        let server = TeachingHttpServer::new("127.0.0.1", 8080).with_teaching_endpoints(endpoints);
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
    }

    #[test]
    fn test_format_duration() {
        assert!(format_duration(500).contains("ns"));
        assert!(format_duration(50_000).contains("µs"));
        assert!(format_duration(5_000_000).contains("ms"));
    }
}
