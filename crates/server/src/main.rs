//! SQLRustGo TCP Server
//!
//! A simple TCP server that accepts SQL queries and returns results.

use sqlrustgo::{parse, ExecutionEngine, SqlError};
use sqlrustgo_common::logging::{init_logging, LogFormat, LogLevel};
use sqlrustgo_server::SecurityIntegration;
use sqlrustgo_storage::MemoryStorage;
use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

/// Handle a single client connection
fn handle_client(
    mut stream: TcpStream,
    storage: Arc<RwLock<MemoryStorage>>,
    security: Arc<SecurityIntegration>,
) -> std::io::Result<()> {
    let peer_addr = stream
        .peer_addr()
        .unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    let ip = peer_addr.ip().to_string();
    let user = "anonymous".to_string();

    let session_id = security.create_secure_session(user.clone(), ip.clone());

    let mut engine =
        ExecutionEngine::new_with_session(storage.clone(), security.sessions().clone(), session_id);

    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                security.close_secure_session(session_id, user);
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                security.log_error(&user, &format!("Read error: {}", e), session_id);
                security.close_secure_session(session_id, user);
                return Err(e);
            }
        };

        let received = String::from_utf8_lossy(&buffer[..bytes_read]);
        let queries: Vec<&str> = received.lines().collect();

        for query in queries {
            let query = query.trim();
            if query.is_empty() {
                continue;
            }

            let start = std::time::Instant::now();

            let result = {
                match parse(query) {
                    Ok(statement) => {
                        security.reset_session_query_state(session_id);
                        if let Some(flag) = security.get_session_cancel_flag(session_id) {
                            engine.storage.write().unwrap().set_cancel_flag(flag);
                        }
                        if let Err(e) = security.check_session_and_reset(session_id) {
                            Err(SqlError::ExecutionError(e))
                        } else {
                            engine.execute(statement)
                        }
                    }
                    Err(e) => Err(SqlError::ParseError(format!("{:?}", e))),
                }
            };

            let duration_ms = start.elapsed().as_millis() as u64;
            let rows = result.as_ref().map(|r| r.affected_rows as u64).unwrap_or(0);

            if parse(query).is_ok() && is_ddl(query) {
                security.log_ddl(&user, query, session_id);
            }

            security.log_sql_execution(&user, query, duration_ms, rows, session_id);

            let response = match result {
                Ok(result) => serde_json::json!({
                    "status": "ok",
                    "rows_affected": result.affected_rows,
                    "result": result.rows
                })
                .to_string(),
                Err(e) => {
                    security.log_error(&user, &e.to_string(), session_id);
                    serde_json::json!({
                        "status": "error",
                        "error": e.to_string()
                    })
                    .to_string()
                }
            };

            if let Err(e) = stream.write_all(response.as_bytes()) {
                security.log_error(&user, &format!("Write error: {}", e), session_id);
                return Err(e);
            }
            if let Err(e) = stream.write_all(b"\n") {
                return Err(e);
            }
            if let Err(e) = stream.flush() {
                return Err(e);
            }
        }
    }
}

fn is_ddl(query: &str) -> bool {
    let upper = query.to_uppercase();
    upper.starts_with("CREATE")
        || upper.starts_with("DROP")
        || upper.starts_with("ALTER")
        || upper.starts_with("TRUNCATE")
}

fn main() {
    let log_dir = env::var("SQLRUSTGO_LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
    let log_level = env::var("SQLRUSTGO_LOG_LEVEL")
        .map(|v| LogLevel::from_str(&v))
        .unwrap_or(LogLevel::Info);
    let log_format = if env::var("SQLRUSTGO_LOG_JSON").is_ok() {
        LogFormat::Json
    } else {
        LogFormat::Text
    };
    let max_file_size: u64 = env::var("SQLRUSTGO_LOG_MAX_SIZE")
        .unwrap_or_else(|_| "10485760".to_string())
        .parse()
        .unwrap_or(10 * 1024 * 1024);
    let max_files: usize = env::var("SQLRUSTGO_LOG_MAX_FILES")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    if let Err(e) = init_logging(&log_dir, log_level, log_format, max_file_size, max_files) {
        eprintln!("Failed to initialize logging: {}", e);
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    log::info!("SQLRustGo TCP Server v1.6.1 starting...");
    log::info!("Log directory: {}", log_dir);
    log::info!("Log level: {:?}", log_level);

    let addr = "127.0.0.1:4000";
    println!("SQLRustGo TCP Server v1.6.1");
    println!("Listening on {}", addr);

    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let security = Arc::new(SecurityIntegration::new());

    println!("Security: audit logging enabled");

    let listener = TcpListener::bind(addr).expect("Failed to bind to address");
    println!("Ready to accept connections");

    let security_for_shutdown = security.clone();
    ctrlc::set_handler(move || {
        println!("\nShutting down...");
        let stats = security_for_shutdown.get_security_stats();
        println!(
            "Security stats: {} events, {} sessions",
            stats.audit_total_events, stats.total_sessions
        );
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let storage = storage.clone();
                let security = security.clone();
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, storage, security) {
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
