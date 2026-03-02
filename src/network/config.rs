//! Configuration Module - Server configuration from file
//!
//! Provides TOML configuration file support for the server.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Server configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server section
    pub server: ServerSection,
    /// Database section
    pub database: DatabaseSection,
    /// Connection pool section
    pub connection_pool: ConnectionPoolSection,
    /// Logging section
    pub logging: LoggingSection,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerSection::default(),
            database: DatabaseSection::default(),
            connection_pool: ConnectionPoolSection::default(),
            logging: LoggingSection::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSection {
    /// Bind address
    pub address: String,
    /// Port
    pub port: u16,
    /// Maximum connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Enable TCP_NODELAY
    pub tcp_nodelay: bool,
    /// Enable TCP_QUICKACK
    pub tcp_quickack: bool,
}

impl Default for ServerSection {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 3306,
            max_connections: 100,
            connection_timeout: 30,
            tcp_nodelay: true,
            tcp_quickack: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSection {
    /// Database directory path
    pub path: String,
    /// Buffer pool size (in pages)
    pub buffer_pool_size: usize,
    /// Page size (in bytes)
    pub page_size: usize,
    /// Enable WAL
    pub wal_enabled: bool,
}

impl Default for DatabaseSection {
    fn default() -> Self {
        Self {
            path: "./data".to_string(),
            buffer_pool_size: 1000,
            page_size: 4096,
            wal_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSection {
    /// Maximum idle connections
    pub max_idle: usize,
    /// Minimum idle connections
    pub min_idle: usize,
    /// Connection lifetime in seconds
    pub max_lifetime: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Connection acquire timeout in seconds
    pub acquire_timeout: u64,
}

impl Default for ConnectionPoolSection {
    fn default() -> Self {
        Self {
            max_idle: 10,
            min_idle: 2,
            max_lifetime: 1800,
            idle_timeout: 600,
            acquire_timeout: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    /// Log level
    pub level: String,
    /// Enable verbose logging
    pub verbose: bool,
    /// Log file path (optional)
    pub file: Option<String>,
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            verbose: false,
            file: None,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Configuration file not found: {:?}", path));
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read configuration file: {}", e))?;

        toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse configuration file: {}", e))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;

        fs::write(path, contents)
            .map_err(|e| format!("Failed to write configuration file: {}", e))
    }

    /// Get the full server address (address:port)
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.address, self.server.port)
    }
}

/// Generate a default configuration file
pub fn generate_default_config(path: &Path) -> Result<(), String> {
    let config = Config::default();
    config.save(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.address, "127.0.0.1");
        assert_eq!(config.server.port, 3306);
        assert_eq!(config.server.max_connections, 100);
    }

    #[test]
    fn test_config_server_address() {
        let config = Config::default();
        assert_eq!(config.server_address(), "127.0.0.1:3306");
    }

    #[test]
    fn test_config_load() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[server]
address = "0.0.0.0"
port = 3307
max_connections = 50

[database]
path = "/tmp/testdb"

[connection_pool]
max_idle = 5

[logging]
verbose = true
"#
        )
        .unwrap();

        let config = Config::load(file.path()).unwrap();
        assert_eq!(config.server.address, "0.0.0.0");
        assert_eq!(config.server.port, 3307);
        assert_eq!(config.server.max_connections, 50);
        assert_eq!(config.database.path, "/tmp/testdb");
        assert_eq!(config.connection_pool.max_idle, 5);
        assert!(config.logging.verbose);
    }

    #[test]
    fn test_config_save_and_load() {
        let mut file = NamedTempFile::new().unwrap();

        let config = Config {
            server: ServerSection {
                address: "192.168.1.1".to_string(),
                port: 3308,
                max_connections: 200,
                connection_timeout: 60,
                tcp_nodelay: false,
                tcp_quickack: true,
            },
            ..Config::default()
        };

        config.save(file.path()).unwrap();

        let loaded = Config::load(file.path()).unwrap();
        assert_eq!(loaded.server.address, "192.168.1.1");
        assert_eq!(loaded.server.port, 3308);
    }

    #[test]
    fn test_config_load_not_found() {
        let result = Config::load(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }
}
