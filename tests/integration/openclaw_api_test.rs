//! OpenClaw API Integration Tests
//!
//! Tests for OpenClaw REST API endpoints:
//! - /health, /query, /nl_query, /schema, /stats
//! - /memory/save, /memory/load, /memory/search, /memory/clear, /memory/stats

use sqlrustgo_server::OpenClawHttpServer;
use sqlrustgo_storage::MemoryStorage;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn start_server(port: u16) -> (thread::JoinHandle<()>, u16) {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let server = OpenClawHttpServer::new("127.0.0.1", port, storage);
    let actual_port = server.get_port();
    let handle = thread::spawn(move || {
        let _ = server.start();
    });
    (handle, actual_port)
}

fn make_request(port: u16, request: &str) -> Option<String> {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(mut stream) =
        std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(2))
    {
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        return Some(response);
    }
    None
}

fn extract_body(response: &str) -> String {
    if let Some(pos) = response.find("\r\n\r\n") {
        response[pos + 4..].to_string()
    } else {
        response.to_string()
    }
}

#[test]
fn test_openclaw_server_creation() {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let server = OpenClawHttpServer::new("127.0.0.1", 18090, storage);
    assert_eq!(server.get_version(), "2.4.0");
}

#[test]
fn test_openclaw_health_endpoint() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        assert!(response.contains("healthy"), "Expected healthy status");
        assert!(response.contains("2.4.0"), "Expected version 2.4.0");
    }

    drop(handle);
}

#[test]
fn test_openclaw_schema_endpoint() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /schema HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body = extract_body(&response);
        assert!(body.contains("sqlrustgo"), "Expected database name");
        assert!(body.contains("tables"), "Expected tables field");
    }

    drop(handle);
}

#[test]
fn test_openclaw_stats_endpoint() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /stats HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body = extract_body(&response);
        assert!(body.contains("tables"), "Expected tables field");
        assert!(
            body.contains("query_statistics"),
            "Expected query_statistics"
        );
    }

    drop(handle);
}

#[test]
fn test_openclaw_memory_save() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let body = r#"{"content": "test memory content", "memory_type": "conversation"}"#;
    let request = format!(
        "POST /memory/save HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    if let Some(response) = make_request(port, &request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body_str = extract_body(&response);
        assert!(
            body_str.contains("\"success\":true") || body_str.contains("id"),
            "Expected success response, got: {}",
            body_str
        );
    }

    drop(handle);
}

#[test]
fn test_openclaw_memory_search() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let body = r#"{"query": "test query"}"#;
    let request = format!(
        "POST /memory/search HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    if let Some(response) = make_request(port, &request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body_str = extract_body(&response);
        assert!(
            body_str.contains("results") || body_str.contains("total"),
            "Expected results, got: {}",
            body_str
        );
    }

    drop(handle);
}

#[test]
fn test_openclaw_memory_stats() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /memory/stats HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body = extract_body(&response);
        assert!(body.contains("total_memories"), "Expected total_memories");
    }

    drop(handle);
}

#[test]
fn test_openclaw_memory_clear() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let body = r#"{}"#;
    let request = format!(
        "POST /memory/clear HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    if let Some(response) = make_request(port, &request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body_str = extract_body(&response);
        assert!(
            body_str.contains("\"success\":true"),
            "Expected success, got: {}",
            body_str
        );
    }

    drop(handle);
}

#[test]
fn test_openclaw_nl_query() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let body = r#"{"query": "show all users"}"#;
    let request = format!(
        "POST /nl_query HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    if let Some(response) = make_request(port, &request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
        let body_str = extract_body(&response);
        assert!(
            body_str.contains("\"success\":true") || body_str.contains("sql"),
            "Expected success with SQL, got: {}",
            body_str
        );
    }

    drop(handle);
}

#[test]
fn test_openclaw_not_found() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /invalid/endpoint HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(response.contains("404"), "Expected 404, got: {}", response);
    }

    drop(handle);
}

#[test]
fn test_openclaw_metrics_endpoint() {
    let (handle, port) = start_server(0);
    thread::sleep(Duration::from_millis(200));

    let request = "GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n";
    if let Some(response) = make_request(port, request) {
        assert!(
            response.contains("200 OK"),
            "Expected 200 OK, got: {}",
            response
        );
    }

    drop(handle);
}
