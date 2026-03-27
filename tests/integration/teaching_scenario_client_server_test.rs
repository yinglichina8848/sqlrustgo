//! Teaching Scenario Client-Server Tests
//!
//! These tests verify real client-server SQL execution through the HTTP API.
//! This is the proper way to test database system behavior - through the server path,
//! not direct ExecutionEngine calls.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};
use sqlrustgo_server::teaching_endpoints::TeachingHttpServer;

struct TestServer {
    handle: thread::JoinHandle<()>,
    port: u16,
}

impl TestServer {
    fn new() -> Self {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let server = TeachingHttpServer::new("127.0.0.1", 0)
            .with_storage(storage);

        // Clone server for the thread
        let server_clone = server.clone();
        let handle = thread::spawn(move || {
            let _ = server_clone.start();
        });

        // Give the server time to start and bind to a port
        thread::sleep(Duration::from_millis(100));

        // Get the actual port after server has started
        let port = server.get_port();

        Self { handle, port }
    }

    fn sql(&self, sql: &str) -> SqlResponse {
        let request = format!(
            "POST /sql HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            format!(r#"{{"sql":"{}"}}"#, sql).len(),
            format!(r#"{{"sql":"{}"}}"#, sql)
        );

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();
        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        let mut reader = std::io::BufReader::new(&stream);
        reader.read_to_end(&mut response).unwrap();

        let response_str = String::from_utf8_lossy(&response);

        // Find JSON body (after blank line)
        if let Some(body_start) = response_str.find("\r\n\r\n") {
            let body = &response_str[body_start + 4..];
            serde_json::from_str(body).unwrap_or_else(|_| SqlResponse {
                columns: None,
                rows: None,
                affected_rows: 0,
                error: Some(format!("Failed to parse response: {}", body)),
            })
        } else {
            SqlResponse {
                columns: None,
                rows: None,
                affected_rows: 0,
                error: Some("Invalid HTTP response".to_string()),
            }
        }
    }

    fn get(&self, path: &str) -> serde_json::Value {
        let request = format!("GET {} HTTP/1.1\r\nHost: localhost\r\n\r\n", path);

        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();
        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        let mut reader = std::io::BufReader::new(&stream);
        reader.read_to_end(&mut response).unwrap();

        let response_str = String::from_utf8_lossy(&response);

        // Find JSON body
        if let Some(body_start) = response_str.find("\r\n\r\n") {
            let body = &response_str[body_start + 4..];
            serde_json::from_str(body).unwrap_or(serde_json::json!({"error": "parse error"}))
        } else {
            serde_json::json!({"error": "invalid response"})
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        // Server thread will terminate when the test drops
    }
}

#[derive(serde::Deserialize, Debug)]
struct SqlResponse {
    columns: Option<Vec<String>>,
    rows: Option<Vec<Vec<serde_json::Value>>>,
    affected_rows: usize,
    error: Option<String>,
}

#[test]
fn test_client_server_create_table() {
    let server = TestServer::new();

    let result = server.sql("CREATE TABLE users (id INT, name TEXT)");

    assert!(result.error.is_none(), "CREATE TABLE should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 0);
}

#[test]
fn test_client_server_insert() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");

    let result = server.sql("INSERT INTO t VALUES (1, 'Alice')");

    assert!(result.error.is_none(), "INSERT should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 1);
}

#[test]
fn test_client_server_select() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");
    server.sql("INSERT INTO t VALUES (1, 'Alice')");
    server.sql("INSERT INTO t VALUES (2, 'Bob')");

    let result = server.sql("SELECT * FROM t");

    assert!(result.error.is_none(), "SELECT should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 0);
    assert!(result.rows.is_some());
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn test_client_server_select_with_filter() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");
    server.sql("INSERT INTO t VALUES (1, 'Alice')");
    server.sql("INSERT INTO t VALUES (2, 'Bob')");

    let result = server.sql("SELECT * FROM t WHERE id = 1");

    assert!(result.error.is_none(), "SELECT WHERE should succeed: {:?}", result.error);
    let rows = result.rows.unwrap();
    assert_eq!(rows.len(), 1);
}

#[test]
fn test_client_server_update() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");
    server.sql("INSERT INTO t VALUES (1, 'Alice')");

    let result = server.sql("UPDATE t SET name = 'Bob' WHERE id = 1");

    assert!(result.error.is_none(), "UPDATE should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 1);
}

#[test]
fn test_client_server_delete() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");
    server.sql("INSERT INTO t VALUES (1, 'Alice')");
    server.sql("INSERT INTO t VALUES (2, 'Bob')");

    let result = server.sql("DELETE FROM t WHERE id = 1");

    assert!(result.error.is_none(), "DELETE should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 1);
}

#[test]
fn test_client_server_multiple_inserts() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");

    let result = server.sql("INSERT INTO t VALUES (1, 'A'), (2, 'B'), (3, 'C')");

    assert!(result.error.is_none(), "Bulk INSERT should succeed: {:?}", result.error);
    assert_eq!(result.affected_rows, 3);

    let select_result = server.sql("SELECT * FROM t");
    assert_eq!(select_result.rows.unwrap().len(), 3);
}

#[test]
fn test_client_server_drop_table() {
    let server = TestServer::new();

    server.sql("CREATE TABLE t (id INT, name TEXT)");
    let drop_result = server.sql("DROP TABLE t");

    assert!(drop_result.error.is_none(), "DROP TABLE should succeed: {:?}", drop_result.error);

    // Verify table is gone
    let select_result = server.sql("SELECT * FROM t");
    assert!(select_result.error.is_some());
}

#[test]
fn test_client_server_error_invalid_sql() {
    let server = TestServer::new();

    let result = server.sql("INVALID SQL SYNTAX");

    assert!(result.error.is_some(), "Invalid SQL should return error");
    assert!(result.columns.is_none());
    assert!(result.rows.is_none());
}

#[test]
fn test_client_server_error_nonexistent_table() {
    let server = TestServer::new();

    let result = server.sql("SELECT * FROM nonexistent_table");

    assert!(result.error.is_some(), "Query on nonexistent table should return error");
}

#[test]
fn test_client_server_health_endpoint() {
    let server = TestServer::new();

    let health = server.get("/health/live");

    assert_eq!(health["status"], "healthy");
}

#[test]
fn test_client_server_metrics_endpoint() {
    let server = TestServer::new();

    // Execute some queries
    server.sql("CREATE TABLE t (id INT)");
    server.sql("INSERT INTO t VALUES (1)");

    let metrics = server.get("/metrics");

    // Metrics should be a JSON object
    assert!(metrics.is_object());
}
