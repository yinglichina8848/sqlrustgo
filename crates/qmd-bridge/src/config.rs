//! QMD Bridge configuration

use serde::{Deserialize, Serialize};

/// Configuration for QMD Bridge connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QmdConfig {
    /// QMD server address
    pub server_addr: String,
    /// QMD server port
    pub server_port: u16,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Batch size for sync operations
    pub batch_size: usize,
    /// Enable compression
    pub compression: bool,
    /// Retry attempts for failed operations
    pub retry_attempts: u32,
}

impl Default for QmdConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1".to_string(),
            server_port: 8080,
            timeout_secs: 30,
            batch_size: 1000,
            compression: false,
            retry_attempts: 3,
        }
    }
}

impl QmdConfig {
    /// Create a new config with custom server address
    pub fn with_server(mut self, addr: &str, port: u16) -> Self {
        self.server_addr = addr.to_string();
        self.server_port = port;
        self
    }

    /// Get the full server URL
    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.server_addr, self.server_port)
    }
}
