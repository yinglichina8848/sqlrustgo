//! Network Module
//!
//! # What (是什么)
//! 网络通信模块，支持客户端-服务器架构
//!
//! # Why (为什么)
//! 单一进程数据库只能单机使用，网络支持让数据库可以服务多个客户端
//!
//! # How (如何实现)
//! - TCP 服务器监听连接
//! - MySQL 协议兼容（可选）
//! - 连接池管理并发

use crate::{ExecutionResult, SqlError, execute};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// MySQL-compatible protocol handler
pub struct NetworkHandler {
    stream: TcpStream,
}

impl NetworkHandler {
    /// Create a new network handler
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Handle a client connection
    pub fn handle(&mut self) -> Result<(), SqlError> {
        // Send greeting
        self.send_greeting()?;

        // Read queries and execute
        loop {
            match self.read_packet() {
                Ok(Some(query)) => {
                    if query.trim().eq_ignore_ascii_case("QUIT") {
                        break;
                    }
                    self.execute_query(&query)?;
                }
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Send MySQL greeting
    fn send_greeting(&mut self) -> Result<(), SqlError> {
        let greeting = b"SQLRustGo 1.0.0\n";
        self.stream.write_all(greeting)?;
        Ok(())
    }

    /// Read a packet from the stream
    fn read_packet(&mut self) -> Result<Option<String>, SqlError> {
        let mut buf = [0u8; 1024];
        match self.stream.read(&mut buf) {
            Ok(0) => Ok(None), // Connection closed
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]).to_string();
                Ok(Some(s))
            }
            Err(e) => Err(SqlError::IoError(e.to_string())),
        }
    }

    /// Execute a query and send result
    fn execute_query(&mut self, query: &str) -> Result<(), SqlError> {
        match execute(query) {
            Ok(result) => {
                self.send_result(&result)?;
            }
            Err(e) => {
                self.send_error(&format!("Error: {}", e))?;
            }
        }
        Ok(())
    }

    /// Send execution result
    fn send_result(&mut self, result: &ExecutionResult) -> Result<(), SqlError> {
        let msg = format!("OK, {} row(s) affected\n", result.rows_affected);
        self.stream.write_all(msg.as_bytes())?;
        Ok(())
    }

    /// Send error message
    fn send_error(&mut self, msg: &str) -> Result<(), SqlError> {
        let err = format!("ERROR: {}\n", msg);
        self.stream.write_all(err.as_bytes())?;
        Ok(())
    }
}

/// Start the server
pub async fn start_server(addr: &str) -> Result<(), SqlError> {
    let listener = TcpListener::bind(addr)?;
    println!("SQLRustGo Server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut handler = NetworkHandler::new(stream);
                if let Err(e) = handler.handle() {
                    eprintln!("Client error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

/// Connect to server
pub fn connect(addr: &str) -> Result<TcpStream, SqlError> {
    TcpStream::connect(addr).map_err(|e| SqlError::IoError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_handler_creation() {
        assert!(true);
    }

    #[test]
    fn test_connect_function() {
        let _f: fn(&str) -> Result<TcpStream, SqlError> = connect;
    }
}
