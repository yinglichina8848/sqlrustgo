//! Checkpoint System
//!
//! Provides periodic checkpoint generation and management for WAL.

use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Checkpoint metadata
#[derive(Debug, Clone)]
pub struct CheckpointMetadata {
    /// Checkpoint LSN
    pub lsn: u64,
    /// Checkpoint timestamp (Unix epoch ms)
    pub timestamp: u64,
    /// Number of transactions logged
    pub tx_count: u64,
    /// Number of dirty pages
    pub dirty_pages: u64,
    /// Checkpoint file path
    pub file_path: PathBuf,
}

impl CheckpointMetadata {
    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"lsn":{},"timestamp":{},"tx_count":{},"dirty_pages":{},"file_path":"{}"}}"#,
            self.lsn,
            self.timestamp,
            self.tx_count,
            self.dirty_pages,
            self.file_path.display()
        )
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Option<Self> {
        let json: serde_json::Value = serde_json::from_str(json).ok()?;
        Some(Self {
            lsn: json["lsn"].as_u64()?,
            timestamp: json["timestamp"].as_u64()?,
            tx_count: json["tx_count"].as_u64()?,
            dirty_pages: json["dirty_pages"].as_u64()?,
            file_path: PathBuf::from(json["file_path"].as_str()?),
        })
    }
}

/// Checkpoint configuration
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Interval between checkpoints
    pub interval: Duration,
    /// Maximum WAL size before checkpoint
    pub max_wal_size_mb: u64,
    /// Enable incremental checkpoint
    pub incremental: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300), // 5 minutes
            max_wal_size_mb: 100,
            incremental: true,
        }
    }
}

/// Checkpoint manager
pub struct CheckpointManager {
    config: CheckpointConfig,
    last_checkpoint: Arc<RwLock<Option<CheckpointMetadata>>>,
    last_checkpoint_time: Arc<RwLock<Instant>>,
    checkpoint_dir: PathBuf,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(checkpoint_dir: PathBuf, config: CheckpointConfig) -> std::io::Result<Self> {
        fs::create_dir_all(&checkpoint_dir)?;
        Ok(Self {
            config,
            last_checkpoint: Arc::new(RwLock::new(None)),
            last_checkpoint_time: Arc::new(RwLock::new(Instant::now())),
            checkpoint_dir,
        })
    }

    /// Create with default config
    pub fn with_dir(checkpoint_dir: PathBuf) -> std::io::Result<Self> {
        Self::new(checkpoint_dir, CheckpointConfig::default())
    }

    /// Check if checkpoint is needed
    pub fn needs_checkpoint(&self) -> bool {
        let elapsed = {
            let last = *self.last_checkpoint_time.read().unwrap();
            last.elapsed()
        };
        
        // Check time interval
        if elapsed >= self.config.interval {
            return true;
        }
        
        // TODO: Check WAL size
        
        false
    }

    /// Record a completed checkpoint
    pub fn record_checkpoint(&self, metadata: CheckpointMetadata) {
        {
            let mut last = self.last_checkpoint.write().unwrap();
            *last = Some(metadata);
        }
        {
            let mut time = self.last_checkpoint_time.write().unwrap();
            *time = Instant::now();
        }
    }

    /// Get the last checkpoint metadata
    pub fn last_checkpoint(&self) -> Option<CheckpointMetadata> {
        self.last_checkpoint.read().unwrap().clone()
    }

    /// Save checkpoint metadata to disk
    pub fn save_metadata(&self, metadata: &CheckpointMetadata) -> std::io::Result<()> {
        let path = self.checkpoint_dir.join("checkpoint.json");
        let mut file = BufWriter::new(File::create(&path)?);
        file.write_all(metadata.to_json().as_bytes())?;
        file.flush()
    }

    /// Load checkpoint metadata from disk
    pub fn load_metadata(&self) -> std::io::Result<Option<CheckpointMetadata>> {
        let path = self.checkpoint_dir.join("checkpoint.json");
        if !path.exists() {
            return Ok(None);
        }
        
        let mut file = BufReader::new(File::open(&path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        Ok(CheckpointMetadata::from_json(&contents))
    }

    /// List all checkpoint files
    pub fn list_checkpoints(&self) -> std::io::Result<Vec<CheckpointMetadata>> {
        let mut checkpoints = Vec::new();
        
        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false)
               && path.file_name().map(|n| n != "checkpoint.json").unwrap_or(false) {
                let mut file = BufReader::new(File::open(&path)?);
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                
                if let Some(meta) = CheckpointMetadata::from_json(&contents) {
                    checkpoints.push(meta);
                }
            }
        }
        
        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(checkpoints)
    }

    /// Clean up old checkpoints, keeping the N most recent
    pub fn cleanup_old_checkpoints(&self, keep: usize) -> std::io::Result<usize> {
        let checkpoints = self.list_checkpoints()?;
        let to_delete = checkpoints.iter().skip(keep);
        let mut removed = 0;
        
        for checkpoint in to_delete {
            if fs::remove_file(&checkpoint.file_path).is_ok() {
                removed += 1;
            }
        }
        
        Ok(removed)
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::new(PathBuf::from("."), CheckpointConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_metadata_serialization() {
        let meta = CheckpointMetadata {
            lsn: 12345,
            timestamp: 1609459200000,
            tx_count: 100,
            dirty_pages: 50,
            file_path: PathBuf::from("/tmp/checkpoint.1"),
        };
        
        let json = meta.to_json();
        let parsed = CheckpointMetadata::from_json(&json).unwrap();
        
        assert_eq!(parsed.lsn, meta.lsn);
        assert_eq!(parsed.timestamp, meta.timestamp);
        assert_eq!(parsed.tx_count, meta.tx_count);
    }

    #[test]
    fn test_checkpoint_manager_default() {
        let manager = CheckpointManager::default();
        assert!(manager.last_checkpoint().is_none());
    }

    #[test]
    fn test_checkpoint_needs() {
        let temp = TempDir::new().unwrap();
        let config = CheckpointConfig {
            interval: Duration::from_secs(3600),
            max_wal_size_mb: 100,
            incremental: true,
        };
        
        let manager = CheckpointManager::new(temp.path().to_path_buf(), config).unwrap();
        
        // Should not need checkpoint immediately
        assert!(!manager.needs_checkpoint());
    }

    #[test]
    fn test_checkpoint_record() {
        let temp = TempDir::new().unwrap();
        let manager = CheckpointManager::with_dir(temp.path().to_path_buf()).unwrap();
        
        let meta = CheckpointMetadata {
            lsn: 1000,
            timestamp: 1609459200000,
            tx_count: 50,
            dirty_pages: 10,
            file_path: temp.path().join("checkpoint.1"),
        };
        
        manager.record_checkpoint(meta.clone());
        
        let last = manager.last_checkpoint().unwrap();
        assert_eq!(last.lsn, 1000);
    }
}
