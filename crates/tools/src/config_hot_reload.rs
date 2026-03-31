//! Configuration Hot Reload
//!
//! Provides runtime configuration reloading without restart:
//! - File watching for config changes
//! - Atomic config updates
//! - Validation before apply
//! - Change callbacks

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            max_connections: 100,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogConfig {
    pub level: String,
    pub rotation_size_mb: u64,
    pub retention_days: u32,
    pub format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            rotation_size_mb: 100,
            retention_days: 7,
            format: "json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_mb: 512,
            ttl_seconds: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub log: LogConfig,
    pub cache: CacheConfig,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            log: LogConfig::default(),
            cache: CacheConfig::default(),
            version: "2.1.0".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigChange {
    Database(DatabaseConfig),
    Log(LogConfig),
    Cache(CacheConfig),
    Full(AppConfig),
}

pub trait ConfigListener: Send + Sync {
    fn on_config_change(&self, change: ConfigChange);
}

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    listeners: Vec<Box<dyn ConfigListener>>,
    last_modified: SystemTime,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            let default = AppConfig::default();
            let json = serde_json::to_string_pretty(&default)?;
            fs::create_dir_all(config_path.parent().unwrap_or(&PathBuf::from(".")))?;
            fs::write(&config_path, json)?;
            default
        };

        let metadata = fs::metadata(&config_path)?;
        let last_modified = metadata.modified()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            listeners: Vec::new(),
            last_modified,
        })
    }

    fn load_config(path: &Path) -> Result<AppConfig> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: AppConfig =
            serde_json::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn get_config(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config)
    }

    pub fn get_database_config(&self) -> DatabaseConfig {
        self.config.read().unwrap().database.clone()
    }

    pub fn get_log_config(&self) -> LogConfig {
        self.config.read().unwrap().log.clone()
    }

    pub fn get_cache_config(&self) -> CacheConfig {
        self.config.read().unwrap().cache.clone()
    }

    pub fn add_listener<L: ConfigListener + 'static>(&mut self, listener: L) {
        self.listeners.push(Box::new(listener));
    }

    pub fn reload(&mut self) -> Result<bool> {
        let metadata = fs::metadata(&self.config_path)?;
        let current_modified = metadata.modified()?;

        if current_modified <= self.last_modified {
            return Ok(false);
        }

        let new_config = Self::load_config(&self.config_path)?;

        {
            let mut config = self.config.write().unwrap();
            let old_config = config.clone();

            if old_config.database != new_config.database {
                self.notify_listeners(ConfigChange::Database(new_config.database.clone()));
            }
            if old_config.log != new_config.log {
                self.notify_listeners(ConfigChange::Log(new_config.log.clone()));
            }
            if old_config.cache != new_config.cache {
                self.notify_listeners(ConfigChange::Cache(new_config.cache.clone()));
            }

            *config = new_config;
        }

        self.last_modified = current_modified;
        println!("Configuration reloaded successfully");
        Ok(true)
    }

    fn notify_listeners(&self, change: ConfigChange) {
        for listener in &self.listeners {
            listener.on_config_change(change.clone());
        }
    }

    pub fn update_database_config(&self, new_config: DatabaseConfig) -> Result<()> {
        {
            let mut config = self.config.write().unwrap();
            config.database = new_config.clone();
        }
        self.notify_listeners(ConfigChange::Database(new_config));
        self.save()?;
        Ok(())
    }

    pub fn update_log_config(&self, new_config: LogConfig) -> Result<()> {
        {
            let mut config = self.config.write().unwrap();
            config.log = new_config.clone();
        }
        self.notify_listeners(ConfigChange::Log(new_config));
        self.save()?;
        Ok(())
    }

    pub fn update_cache_config(&self, new_config: CacheConfig) -> Result<()> {
        {
            let mut config = self.config.write().unwrap();
            config.cache = new_config.clone();
        }
        self.notify_listeners(ConfigChange::Cache(new_config));
        self.save()?;
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        let config = self.config.read().unwrap();
        let json = serde_json::to_string_pretty(&*config)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(json.as_bytes())?;
        self.last_modified = SystemTime::now();
        Ok(())
    }
}

pub struct ConfigWatcher {
    manager: Arc<RwLock<Option<ConfigManager>>>,
    poll_interval: Duration,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf, poll_interval_secs: u64) -> Result<Self> {
        let manager = ConfigManager::new(config_path)?;
        Ok(Self {
            manager: Arc::new(RwLock::new(Some(manager))),
            poll_interval: Duration::from_secs(poll_interval_secs),
        })
    }

    pub fn start_watching(&self) -> std::thread::JoinHandle<()> {
        let manager = Arc::clone(&self.manager);
        let interval = self.poll_interval;

        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            let mut guard = manager.write().unwrap();
            if let Some(ref mut mgr) = *guard {
                if let Err(e) = mgr.reload() {
                    eprintln!("Config reload error: {}", e);
                }
            }
        })
    }

    pub fn get_manager(&self) -> Arc<RwLock<Option<ConfigManager>>> {
        Arc::clone(&self.manager)
    }
}

pub fn create_config_listener<F>(callback: F) -> Box<dyn ConfigListener>
where
    F: Fn(ConfigChange) + Send + Sync + 'static,
{
    struct CallbackListener<F: Fn(ConfigChange) + Send + Sync> {
        callback: F,
    }

    impl<F: Fn(ConfigChange) + Send + Sync + 'static> ConfigListener for CallbackListener<F> {
        fn on_config_change(&self, change: ConfigChange) {
            (self.callback)(change);
        }
    }

    Box::new(CallbackListener { callback })
}

pub fn load_config(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_json::from_str(&contents).context("Failed to parse config")
}

pub fn save_config(path: &Path, config: &AppConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct ConfigCommand {
    #[structopt(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, StructOpt)]
pub enum ConfigAction {
    Reload {
        #[structopt(short = "c", long = "config", default_value = "config.json")]
        config: PathBuf,
    },
    Show {
        #[structopt(short = "c", long = "config", default_value = "config.json")]
        config: PathBuf,
    },
    UpdateDb {
        #[structopt(short = "c", long = "config", default_value = "config.json")]
        config: PathBuf,
        #[structopt(short = "h", long = "host")]
        host: Option<String>,
        #[structopt(short = "p", long = "port")]
        port: Option<u16>,
        #[structopt(short = "m", long = "max-connections")]
        max_connections: Option<u32>,
    },
    UpdateLog {
        #[structopt(short = "c", long = "config", default_value = "config.json")]
        config: PathBuf,
        #[structopt(short = "l", long = "level")]
        level: Option<String>,
        #[structopt(short = "s", long = "rotation-size")]
        rotation_size: Option<u64>,
    },
    Watch {
        #[structopt(short = "c", long = "config", default_value = "config.json")]
        config: PathBuf,
        #[structopt(short = "i", long = "interval", default_value = "5")]
        interval: u64,
    },
}

pub fn run_config_cmd(cmd: ConfigCommand) -> Result<()> {
    match cmd.action {
        ConfigAction::Reload { config } => {
            let mut manager = ConfigManager::new(config)?;
            manager.reload()?;
        }
        ConfigAction::Show { config } => {
            let cfg = load_config(&config)?;
            println!("{}", serde_json::to_string_pretty(&cfg)?);
        }
        ConfigAction::UpdateDb {
            config,
            host,
            port,
            max_connections,
        } => {
            let mut manager = ConfigManager::new(config)?;
            let mut db_config = manager.get_database_config();
            if let Some(h) = host {
                db_config.host = h;
            }
            if let Some(p) = port {
                db_config.port = p;
            }
            if let Some(m) = max_connections {
                db_config.max_connections = m;
            }
            manager.update_database_config(db_config)?;
            println!("Database config updated");
        }
        ConfigAction::UpdateLog {
            config,
            level,
            rotation_size,
        } => {
            let mut manager = ConfigManager::new(config)?;
            let mut log_config = manager.get_log_config();
            if let Some(l) = level {
                log_config.level = l;
            }
            if let Some(s) = rotation_size {
                log_config.rotation_size_mb = s;
            }
            manager.update_log_config(log_config)?;
            println!("Log config updated");
        }
        ConfigAction::Watch { config, interval } => {
            let watcher = ConfigWatcher::new(config, interval)?;
            println!(
                "Watching configuration... (poll interval: {}s, Ctrl+C to stop)",
                interval
            );
            let _handle = watcher.start_watching();
            loop {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, "2.1.0");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.cache.enabled, true);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_size_mb, 512);
    }
}
