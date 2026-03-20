use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub connection_timeout: Duration,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            dbname: "tpch".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            connection_timeout: Duration::from_secs(30),
        }
    }
}

impl PostgresConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={} connect_timeout={}",
            self.host,
            self.port,
            self.dbname,
            self.user,
            self.password,
            self.connection_timeout.as_secs()
        )
    }

    pub fn local() -> Self {
        Self::default()
    }

    pub fn docker() -> Self {
        let mut config = Self::default();
        config.host = "127.0.0.1".to_string();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PostgresConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
    }

    #[test]
    fn test_connection_string() {
        let config = PostgresConfig::default();
        let conn_str = config.connection_string();
        assert!(conn_str.contains("host=localhost"));
        assert!(conn_str.contains("dbname=tpch"));
    }
}
