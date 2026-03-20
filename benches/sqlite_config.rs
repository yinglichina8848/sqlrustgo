use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct SQLiteConfig {
    pub in_memory: bool,
    pub cache_size: i32,
    pub page_size: u32,
    pub wal_mode: bool,
    pub synchronous: String,
}

impl Default for SQLiteConfig {
    fn default() -> Self {
        Self {
            in_memory: true,
            cache_size: 2000,
            page_size: 4096,
            wal_mode: false,
            synchronous: "OFF",
        }
    }
}

impl SQLiteConfig {
    pub fn to_connection(&self) -> rusqlite::Result<Connection> {
        let conn = if self.in_memory {
            Connection::open_in_memory()
        } else {
            Connection::open("benchmark.db")
        }?;

        conn.execute_batch(&format!("PRAGMA cache_size = {};", self.cache_size))?;
        conn.execute_batch(&format!("PRAGMA page_size = {};", self.page_size))?;
        if self.wal_mode {
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }
        conn.execute_batch(&format!("PRAGMA synchronous = {};", self.synchronous))?;

        Ok(conn)
    }

    pub fn fast() -> Self {
        Self {
            in_memory: true,
            cache_size: 10000,
            page_size: 4096,
            wal_mode: false,
            synchronous: "OFF",
        }
    }

    pub fn durable() -> Self {
        Self {
            in_memory: false,
            cache_size: 10000,
            page_size: 4096,
            wal_mode: true,
            synchronous: "FULL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SQLiteConfig::default();
        assert!(config.in_memory);
        assert!(!config.wal_mode);
    }

    #[test]
    fn test_fast_config() {
        let config = SQLiteConfig::fast();
        assert!(config.in_memory);
        assert_eq!(config.synchronous, "OFF");
    }

    #[test]
    fn test_durable_config() {
        let config = SQLiteConfig::durable();
        assert!(!config.in_memory);
        assert!(config.wal_mode);
        assert_eq!(config.synchronous, "FULL");
    }

    #[test]
    fn test_to_connection() {
        let config = SQLiteConfig::default();
        let conn = config.to_connection();
        assert!(conn.is_ok());
    }
}
