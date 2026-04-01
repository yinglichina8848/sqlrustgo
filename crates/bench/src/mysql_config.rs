use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MySqlConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
    pub connection_timeout: Duration,
}

impl Default for MySqlConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            dbname: "tpch".to_string(),
            user: "root".to_string(),
            password: "".to_string(),
            connection_timeout: Duration::from_secs(30),
        }
    }
}

impl MySqlConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        )
    }

    pub fn local() -> Self {
        Self::default()
    }

    pub fn docker() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MySqlConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
    }

    #[test]
    fn test_connection_string() {
        let config = MySqlConfig::default();
        let conn_str = config.connection_string();
        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("localhost"));
    }
}
