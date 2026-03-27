//! SQLRustGo TCP Server
//!
//! A simple TCP server that accepts SQL queries and returns results.

use sqlrustgo::{parse, ExecutionEngine, SqlError};
use sqlrustgo_storage::MemoryStorage;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};

/// Handle a single client connection
fn handle_client(mut stream: TcpStream, engine: Arc<RwLock<ExecutionEngine>>) -> std::io::Result<()> {
    let mut buffer = [0u8; 4096];

    loop {
        // Read data from client
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => return Ok(()), // Connection closed
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read error: {}", e);
                return Err(e);
            }
        };

        // Convert bytes to string and split by newline
        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        let queries: Vec<&str> = received.lines().collect();

        for query in queries {
            let query = query.trim();
            if query.is_empty() {
                continue;
            }

            // Execute query
            let result = {
                let mut eng = engine.write().unwrap();
                match parse(query) {
                    Ok(statement) => eng.execute(statement),
                    Err(e) => Err(SqlError::ParseError(format!("{:?}", e))),
                }
            };

            // Send response
            let response = match result {
                Ok(result) => {
                    serde_json::json!({
                        "status": "ok",
                        "rows_affected": result.affected_rows,
                        "result": result.rows
                    }).to_string()
                }
                Err(e) => {
                    serde_json::json!({
                        "status": "error",
                        "error": e.to_string()
                    }).to_string()
                }
            };

            // Send response followed by newline
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Write error: {}", e);
                return Err(e);
            }
            if let Err(e) = stream.write_all(b"\n") {
                eprintln!("Write error: {}", e);
                return Err(e);
            }
            if let Err(e) = stream.flush() {
                eprintln!("Flush error: {}", e);
                return Err(e);
            }
        }
    }
}

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let addr = "127.0.0.1:4000";
    println!("SQLRustGo TCP Server v1.6.1");
    println!("Listening on {}", addr);

    // Create storage and execution engine
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let engine = Arc::new(RwLock::new(ExecutionEngine::new(storage)));

    // Create TCP listener
    let listener = TcpListener::bind(addr).expect("Failed to bind to address");
    println!("Ready to accept connections");

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\nShutting down...");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let engine = engine.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, engine) {
                        eprintln!("Client handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}