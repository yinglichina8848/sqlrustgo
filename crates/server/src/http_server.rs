//! HTTP Server Module
//!
//! Provides HTTP endpoints for health checks and metrics.

#![allow(clippy::manual_flatten)]

use crate::metrics_endpoint::MetricsRegistry;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};

/// HTTP server configuration
pub struct HttpServer {
    host: String,
    port: u16,
    version: String,
    metrics_registry: Arc<RwLock<MetricsRegistry>>,
}

impl HttpServer {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            version: "1.4.0".to_string(),
            metrics_registry: Arc::new(RwLock::new(MetricsRegistry::new())),
        }
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

    /// Bind to an available port
    pub fn bind_to_available_port(&self) -> u16 {
        if self.port == 0 {
            // Bind to port 0 to get an available port
            if let Ok(listener) = TcpListener::bind(format!("{}:0", self.host)) {
                if let Ok(addr) = listener.local_addr() {
                    return addr.port();
                }
            }
        }
        self.port
    }

    /// Start the HTTP server
    pub fn start(&self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&addr)?;

        println!("HTTP Server started on http://{}", addr);
        println!("Endpoints:");
        println!("  - GET /health/live  - Liveness probe");
        println!("  - GET /health/ready - Readiness probe");
        println!("  - GET /metrics     - Prometheus metrics");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let version = self.version.clone();
                    let metrics_registry = Arc::clone(&self.metrics_registry);

                    std::thread::spawn(move || {
                        let _ = handle_request(&mut stream, &version, &metrics_registry);
                    });
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Start the HTTP server in the background
    pub fn start_background(&self) -> std::thread::JoinHandle<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let version = self.version.clone();
        let metrics_registry = Arc::clone(&self.metrics_registry);

        std::thread::spawn(move || {
            if let Ok(listener) = TcpListener::bind(&addr) {
                println!("HTTP Server started on http://{}", addr);
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let v = version.clone();
                        let mr = Arc::clone(&metrics_registry);
                        std::thread::spawn(move || {
                            let _ = handle_request(&mut stream, &v, &mr);
                        });
                    }
                }
            }
        })
    }
}

/// Handle incoming HTTP request
fn handle_request<T: Read + Write>(
    stream: &mut T,
    version: &str,
    metrics_registry: &Arc<RwLock<MetricsRegistry>>,
) -> Result<(), std::io::Error> {
    let mut buffer = [0u8; 1024];
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
                "/health/live" => {
                    let body = serde_json::json!({
                        "status": "healthy",
                    })
                    .to_string();
                    ("HTTP/1.1 200 OK", "application/json", body)
                }
                "/health/ready" => {
                    let body = serde_json::json!({
                        "status": "ready",
                        "version": version,
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTcpStream {
        read_data: Vec<u8>,
        write_buffer: Vec<u8>,
    }

    impl MockTcpStream {
        fn new(request: &str) -> Self {
            Self {
                read_data: request.as_bytes().to_vec(),
                write_buffer: Vec::new(),
            }
        }
    }

    impl Read for MockTcpStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let len = std::cmp::min(buf.len(), self.read_data.len());
            buf[..len].copy_from_slice(&self.read_data[..len]);
            self.read_data.drain(..len);
            Ok(len)
        }
    }

    impl Write for MockTcpStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.write_buffer.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_http_server_creation() {
        let server = HttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_http_server_with_version() {
        let server = HttpServer::new("127.0.0.1", 8080).with_version("2.0.0");
        assert_eq!(server.version, "2.0.0");
    }

    #[test]
    fn test_http_server_with_metrics_registry() {
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));
        let server = HttpServer::new("127.0.0.1", 8080).with_metrics_registry(registry);
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_http_server_default_version() {
        let server = HttpServer::new("127.0.0.1", 8080);
        assert_eq!(server.version, "1.4.0");
    }

    #[test]
    fn test_http_server_host() {
        let server = HttpServer::new("0.0.0.0", 5432);
        assert_eq!(server.host, "0.0.0.0");
        assert_eq!(server.port, 5432);
    }

    #[test]
    fn test_http_server_with_different_ports() {
        let server1 = HttpServer::new("127.0.0.1", 8080);
        let server2 = HttpServer::new("127.0.0.1", 9000);
        assert_eq!(server1.port, 8080);
        assert_eq!(server2.port, 9000);
    }

    #[test]
    fn test_handle_request_health_live() {
        let request = "GET /health/live HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
        let response = String::from_utf8(stream.write_buffer.clone()).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("application/json"));
        assert!(response.contains("healthy"));
    }

    #[test]
    fn test_handle_request_health_ready() {
        let request = "GET /health/ready HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
        let response = String::from_utf8(stream.write_buffer.clone()).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("application/json"));
        assert!(response.contains("ready"));
        assert!(response.contains("1.4.0"));
    }

    #[test]
    fn test_handle_request_metrics() {
        let request = "GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
        let response = String::from_utf8(stream.write_buffer.clone()).unwrap();
        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("text/plain"));
    }

    #[test]
    fn test_handle_request_not_found() {
        let request = "GET /invalid/path HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
        let response = String::from_utf8(stream.write_buffer.clone()).unwrap();
        assert!(response.contains("HTTP/1.1 404 Not Found"));
        assert!(response.contains("Not Found"));
    }

    #[test]
    fn test_handle_request_bad_request_empty() {
        let request = "";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_request_bad_request_no_path() {
        let request = "GET \r\n";
        let mut stream = MockTcpStream::new(request);
        let registry = Arc::new(RwLock::new(MetricsRegistry::new()));

        let result = handle_request::<MockTcpStream>(&mut stream, "1.4.0", &registry);

        assert!(result.is_ok());
        let response = String::from_utf8(stream.write_buffer.clone()).unwrap();
        assert!(response.contains("HTTP/1.1 400 Bad Request"));
    }
}
