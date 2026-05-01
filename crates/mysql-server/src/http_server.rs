//! HTTP Monitoring Server
//!
//! Provides HTTP endpoints for performance monitoring:
//! - GET /metrics - Prometheus-compatible metrics
//! - GET /health - Health check
//! - GET /stats - JSON statistics

#![allow(clippy::len_zero, clippy::comparison_to_empty, dead_code)]

use crate::monitoring::SharedMonitor;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

const HTTP_OK: &[u8] = b"HTTP/1.1 200 OK\r\n";
const HTTP_NOT_FOUND: &[u8] = b"HTTP/1.1 404 Not Found\r\n";
const HTTP_INTERNAL_ERROR: &[u8] = b"HTTP/1.1 500 Internal Server Error\r\n";
const CONTENT_TYPE_JSON: &[u8] = b"Content-Type: application/json\r\n";
const CONTENT_TYPE_TEXT: &[u8] = b"Content-Type: text/plain; version=0.0.4\r\n";

pub struct MonitoringServer {
    monitor: SharedMonitor,
    port: u16,
}

impl MonitoringServer {
    pub fn new(monitor: SharedMonitor, port: u16) -> Self {
        Self { monitor, port }
    }

    pub fn start(&self) {
        let monitor = self.monitor.clone();
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        thread::spawn(move || {
            let listener = match TcpListener::bind(addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "Failed to bind monitoring server on port {}: {}",
                        addr.port(),
                        e
                    );
                    return;
                }
            };

            println!("Monitoring server listening on http://{}", addr);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let monitor = monitor.clone();
                        thread::spawn(|| handle_connection(stream, monitor));
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream, monitor: SharedMonitor) {
    let mut buffer = [0u8; 8192];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read request: {}", e);
            return;
        }
    };

    if bytes_read == 0 {
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let path = extract_path(&request);

    let (status, content_type, body) = match path.as_str() {
        "/metrics" => {
            let metrics = monitor.prometheus_metrics();
            (
                HTTP_OK.to_vec(),
                CONTENT_TYPE_TEXT.to_vec(),
                metrics.into_bytes(),
            )
        }
        "/health" => {
            let response = serde_json::json!({
                "status": "healthy",
                "service": "sqlrustgo-mysql-server"
            });
            (
                HTTP_OK.to_vec(),
                CONTENT_TYPE_JSON.to_vec(),
                response.to_string().into_bytes(),
            )
        }
        "/stats" => {
            let stats = monitor.json_stats();
            (
                HTTP_OK.to_vec(),
                CONTENT_TYPE_JSON.to_vec(),
                stats.to_string().into_bytes(),
            )
        }
        "/slow-queries" => {
            let slow_queries = monitor.get_slow_queries(100);
            let response = serde_json::json!({
                "slow_queries": slow_queries
            });
            (
                HTTP_OK.to_vec(),
                CONTENT_TYPE_JSON.to_vec(),
                response.to_string().into_bytes(),
            )
        }
        _ => (
            HTTP_NOT_FOUND.to_vec(),
            CONTENT_TYPE_JSON.to_vec(),
            serde_json::json!({"error": "Not Found"})
                .to_string()
                .into_bytes(),
        ),
    };

    let response = build_response(status, content_type, body);
    if let Err(e) = stream.write_all(&response) {
        eprintln!("Failed to write response: {}", e);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Failed to flush stream: {}", e);
    }
}

fn extract_path(request: &str) -> String {
    for line in request.lines() {
        let line = line.trim();
        if line.starts_with("GET ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].to_string();
            }
        }
    }
    String::new()
}

fn build_response(status: Vec<u8>, content_type: Vec<u8>, body: Vec<u8>) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&status);
    response.extend_from_slice(&content_type);
    response.extend_from_slice(b"Content-Length: ");
    response.extend_from_slice(format!("{}\r\n", body.len()).as_bytes());
    response.extend_from_slice(b"\r\n");
    response.extend_from_slice(&body);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path() {
        let request = "GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n";
        assert_eq!(extract_path(request), "/metrics");

        let request2 = "GET /stats?foo=bar HTTP/1.1\r\n\r\n";
        assert_eq!(extract_path(request2), "/stats?foo=bar");
    }
}
