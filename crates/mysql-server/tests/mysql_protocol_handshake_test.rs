//! MySQL Protocol Handshake Test - GA-GAP-05
//!
//! Tests that sqlrustgo-mysql-server correctly implements the MySQL protocol handshake.
//! This test starts the server and connects using the mysql crate to verify protocol compatibility.

use mysql::prelude::*;
use mysql::OptsBuilder;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

const TEST_PORT: u16 = 15998;
const SERVER_STARTUP_TIMEOUT: u64 = 10;

fn find_available_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    static PORT: AtomicU16 = AtomicU16::new(TEST_PORT);
    let mut port = PORT.load(Ordering::SeqCst);
    for _ in 0..100 {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_err() {
            return port;
        }
        port = PORT.fetch_add(1, Ordering::SeqCst);
    }
    TEST_PORT
}

fn start_server(port: u16) -> std::process::Child {
    let server_bin = std::env::var("SQLRUSTGO_SERVER_BIN")
        .unwrap_or_else(|_| "target/release/sqlrustgo-mysql-server".to_string());

    let mut child = Command::new(&server_bin)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Wait for server to be ready
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < SERVER_STARTUP_TIMEOUT {
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return child;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // If we couldn't connect, kill and panic
    let _ = child.kill();
    panic!(
        "Server failed to start within {} seconds",
        SERVER_STARTUP_TIMEOUT
    );
}

#[test]
fn test_mysql_protocol_handshake() {
    let port = find_available_port();
    let mut server = start_server(port);

    // Give server a moment to fully initialize
    std::thread::sleep(Duration::from_millis(500));

    // Attempt MySQL connection using mysql crate
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("127.0.0.1"))
        .tcp_port(port)
        .user(Some("root"))
        .pass(Some(""))
        .prefer_socket(false)
        .ssl_opts(mysql::SslOpts::default());

    let conn_result = mysql::Conn::new(opts);

    // Cleanup server
    let _ = server.kill();
    let _ = server.wait();

    // Verify connection succeeded (protocol handshake worked)
    assert!(
        conn_result.is_ok(),
        "MySQL protocol handshake failed: {:?}",
        conn_result.err()
    );

    // Verify we can execute a simple query
    let mut conn = conn_result.unwrap();
    let result: Vec<(i32,)> = conn.query("SELECT 1").unwrap();
    assert_eq!(result, vec![(1,)]);
}

#[test]
fn test_mysql_protocol_multiple_queries() {
    let port = find_available_port();
    let mut server = start_server(port);
    std::thread::sleep(Duration::from_millis(500));

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("127.0.0.1"))
        .tcp_port(port)
        .user(Some("root"))
        .pass(Some(""))
        .prefer_socket(false)
        .ssl_opts(mysql::SslOpts::default());

    let mut conn = mysql::Conn::new(opts).expect("Connection failed");
    let _ = server.kill();
    let _ = server.wait();

    // Test multiple queries in sequence
    let r1: Vec<(i32,)> = conn.query("SELECT 1").unwrap();
    assert_eq!(r1, vec![(1,)]);

    let r2: Vec<(i32, i32)> = conn.query("SELECT 1, 2").unwrap();
    assert_eq!(r2, vec![(1, 2)]);

    let r3: Vec<(String,)> = conn.query("SELECT 'hello'").unwrap();
    assert_eq!(r3, vec![("hello".to_string(),)]);
}

#[test]
fn test_mysql_protocol_error_handling() {
    let port = find_available_port();
    let mut server = start_server(port);
    std::thread::sleep(Duration::from_millis(500));

    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("127.0.0.1"))
        .tcp_port(port)
        .user(Some("root"))
        .pass(Some(""))
        .prefer_socket(false)
        .ssl_opts(mysql::SslOpts::default());

    let mut conn = mysql::Conn::new(opts).expect("Connection failed");
    let _ = server.kill();
    let _ = server.wait();

    // Test that invalid SQL returns proper error (not crash)
    // Use a simple query result type that we'll just discard
    let result: Result<Vec<(i32,)>, _> = conn.query("INVALID SQL SYNTAX");
    assert!(result.is_err(), "Expected error for invalid SQL");
}
