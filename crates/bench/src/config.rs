//! Benchmark Configuration Module
//!
//! Provides configuration file support (YAML) for benchmark settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    #[serde(default = "default_db")]
    pub db: String,

    #[serde(default = "default_workload")]
    pub workload: String,

    #[serde(default = "default_threads")]
    pub threads: usize,

    #[serde(default = "default_duration")]
    pub duration: u64,

    #[serde(default = "default_scale")]
    pub scale: usize,

    #[serde(default)]
    pub cache: bool,

    #[serde(default)]
    pub output: String,

    #[serde(default)]
    pub pg_conn: Option<String>,

    #[serde(default)]
    pub sqlite_path: Option<String>,

    #[serde(default = "default_sqlrustgo_addr")]
    pub sqlrustgo_addr: String,
}

fn default_db() -> String {
    "sqlrustgo".to_string()
}
fn default_workload() -> String {
    "oltp".to_string()
}
fn default_threads() -> usize {
    10
}
fn default_duration() -> u64 {
    60
}
fn default_scale() -> usize {
    10000
}
fn default_sqlrustgo_addr() -> String {
    "127.0.0.1:4000".to_string()
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            db: default_db(),
            workload: default_workload(),
            threads: default_threads(),
            duration: default_duration(),
            scale: default_scale(),
            cache: false,
            output: "json".to_string(),
            pg_conn: None,
            sqlite_path: None,
            sqlrustgo_addr: default_sqlrustgo_addr(),
        }
    }
}

impl BenchmarkConfig {
    pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: BenchmarkConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.threads == 0 {
            warnings.push("threads should be > 0".to_string());
        }

        if self.duration == 0 {
            warnings.push("duration should be > 0".to_string());
        }

        if self.scale == 0 {
            warnings.push("scale should be > 0".to_string());
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.db, "sqlrustgo");
        assert_eq!(config.threads, 10);
        assert_eq!(config.duration, 60);
    }

    #[test]
    fn test_config_yaml() {
        let config = BenchmarkConfig::default();
        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("sqlrustgo"));
    }
}
