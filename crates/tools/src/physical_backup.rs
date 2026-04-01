//! Physical Backup CLI Tool
//!
//! Provides commands for physical backup based on storage snapshots:
//! - Create physical backup with storage snapshot and WAL packaging
//! - List physical backups
//! - Verify backup integrity
//! - Restore backup to new instance
//!
//! Physical backup includes:
//! - Complete storage page files (data)
//! - WAL logs packaging
//! - Backup metadata (LSN, timestamp)
//! - One-click backup completion
//! - Backup verification

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::wal::WalArchiveManager;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use structopt::StructOpt;

/// Physical backup metadata stored in manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalBackupManifest {
    /// Backup version
    pub version: String,
    /// Backup type (full or incremental based on LSN)
    pub backup_type: PhysicalBackupType,
    /// Creation timestamp
    pub timestamp: String,
    /// LSN at backup time (Log Sequence Number)
    pub lsn: String,
    /// Parent backup LSN for incremental backups
    pub parent_lsn: Option<String>,
    /// Storage data directory path
    pub data_dir: String,
    /// WAL directory path
    pub wal_dir: String,
    /// Total size of backup in bytes
    pub total_size_bytes: u64,
    /// Number of files in backup
    pub file_count: usize,
    /// Compression enabled
    pub compressed: bool,
    /// Files included in backup
    pub files: Vec<BackupFileInfo>,
    /// WAL archives included
    pub wal_archives: Vec<WalArchiveInfo>,
    /// Checksum for integrity verification
    pub checksum: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PhysicalBackupType {
    Full,
    Incremental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileInfo {
    pub relative_path: String,
    pub size_bytes: u64,
    pub is_compressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalArchiveInfo {
    pub archive_id: u64,
    pub file_name: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub entry_count: u64,
    pub timestamp: u64,
}

/// CLI commands for physical backup operations
#[derive(Debug, StructOpt)]
#[structopt(
    name = "physical-backup",
    about = "Physical backup and restore based on storage snapshots"
)]
pub enum PhysicalBackupCommand {
    /// Create a physical backup
    Backup {
        /// Backup directory (will be created)
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Database data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,

        /// WAL directory
        #[structopt(short = "w", long = "wal-dir", default_value = "./wal")]
        wal_dir: PathBuf,

        /// Enable compression
        #[structopt(short = "c", long = "compress")]
        compress: bool,

        /// Parent backup directory for incremental backup
        #[structopt(long = "parent")]
        parent: Option<PathBuf>,
    },

    /// List physical backups in a directory
    List {
        /// Backup directory to list
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,
    },

    /// Verify physical backup integrity
    Verify {
        /// Backup directory to verify
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,
    },

    /// Restore physical backup to target directory
    Restore {
        /// Backup directory to restore from
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Target data directory
        #[structopt(short = "t", long = "target")]
        target: PathBuf,

        /// Target WAL directory
        #[structopt(short = "w", long = "wal-target")]
        wal_target: Option<PathBuf>,
    },

    /// Prune old physical backups based on retention policy
    Prune {
        /// Backup directory to prune
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Keep N most recent backups
        #[structopt(short = "k", long = "keep")]
        keep: Option<usize>,

        /// Keep backups from last N days
        #[structopt(short = "D", long = "keep-days")]
        keep_days: Option<usize>,

        /// Preview changes without actually deleting
        #[structopt(short = "n", long = "dry-run")]
        dry_run: bool,

        /// Skip confirmation prompt
        #[structopt(short = "f", long = "force")]
        force: bool,
    },
}

/// Create a physical backup
pub fn create_physical_backup(
    dir: &Path,
    data_dir: &Path,
    wal_dir: &Path,
    compress: bool,
    parent: Option<&Path>,
) -> Result<()> {
    println!("Creating physical backup to: {}", dir.display());
    println!("  Data directory: {}", data_dir.display());
    println!("  WAL directory: {}", wal_dir.display());
    println!(
        "  Compression: {}",
        if compress { "enabled" } else { "disabled" }
    );

    // Generate LSN for this backup
    let current_lsn = generate_lsn();
    let parent_lsn = parent.map(load_parent_lsn).transpose()?;

    if let Some(ref plsn) = parent_lsn {
        println!("  Parent LSN: {}", plsn);
    }
    println!("  Current LSN: {}", current_lsn);

    // Create backup directory structure
    fs::create_dir_all(dir).context("Failed to create backup directory")?;
    let data_backup_dir = dir.join("data");
    let wal_backup_dir = dir.join("wal");
    fs::create_dir_all(&data_backup_dir).context("Failed to create data backup directory")?;
    fs::create_dir_all(&wal_backup_dir).context("Failed to create WAL backup directory")?;

    // Copy storage data files
    let mut files = Vec::new();
    let mut total_size = 0u64;

    if data_dir.exists() {
        for entry in walkdir(data_dir).context("Failed to read data directory")? {
            let path = entry.path();

            if path.is_file() {
                let relative = path
                    .strip_prefix(data_dir)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| path.to_string_lossy().to_string());

                let target_path = data_backup_dir.join(&relative);
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let original_size = fs::metadata(&path)?.len();
                total_size += original_size;

                if compress {
                    let compressed_path = {
                        let ext = target_path
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let new_ext = if ext.is_empty() {
                            "gz".to_string()
                        } else {
                            format!("{}.gz", ext)
                        };
                        target_path.with_extension(new_ext)
                    };
                    compress_file(&path, &compressed_path)?;
                    let compressed_size = fs::metadata(&compressed_path)?.len();
                    files.push(BackupFileInfo {
                        relative_path: relative.clone() + ".gz",
                        size_bytes: compressed_size,
                        is_compressed: true,
                    });
                    println!("  Backed up (compressed): {}", relative);
                } else {
                    fs::copy(&path, &target_path)?;
                    files.push(BackupFileInfo {
                        relative_path: relative.clone(),
                        size_bytes: original_size,
                        is_compressed: false,
                    });
                    println!("  Backed up: {}", relative);
                }
            }
        }
    }

    // Archive WAL files
    let mut wal_archives = Vec::new();
    let wal_archive_dir = wal_backup_dir.join("archives");
    fs::create_dir_all(&wal_archive_dir)?;

    if wal_dir.exists() {
        let mut archive_manager =
            WalArchiveManager::new(wal_dir.to_path_buf(), wal_archive_dir.clone())
                .context("Failed to create WAL archive manager")?;

        if compress {
            archive_manager.set_compression(true);
        }

        // Archive old WAL files
        let archived = archive_manager.archive_wal()?;
        if archived.entry_count > 0 {
            wal_archives.push(WalArchiveInfo {
                archive_id: archived.archive_id,
                file_name: archived.archived_file,
                original_size: archived.original_size,
                compressed_size: archived.archived_size,
                entry_count: archived.entry_count,
                timestamp: archived.timestamp,
            });

            println!(
                "  Archived WAL: {} entries, {} -> {} bytes",
                archived.entry_count, archived.original_size, archived.archived_size
            );
        }
    }

    // Create manifest
    let backup_type = if parent_lsn.is_some() {
        PhysicalBackupType::Incremental
    } else {
        PhysicalBackupType::Full
    };

    let manifest = PhysicalBackupManifest {
        version: "1.0".to_string(),
        backup_type,
        timestamp: chrono_lite_timestamp(),
        lsn: current_lsn.clone(),
        parent_lsn,
        data_dir: data_dir.to_string_lossy().to_string(),
        wal_dir: wal_dir.to_string_lossy().to_string(),
        total_size_bytes: total_size,
        file_count: files.len(),
        compressed: compress,
        files,
        wal_archives,
        checksum: String::new(), // Will be calculated after manifest is created
    };

    // Calculate checksum for manifest
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    let manifest_checksum = calculate_checksum_for_bytes(manifest_json.as_bytes());
    let manifest_with_checksum = PhysicalBackupManifest {
        checksum: manifest_checksum.clone(),
        ..manifest
    };

    // Write manifest
    let manifest_file = dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest_with_checksum)?;
    fs::write(&manifest_file, manifest_json).context("Failed to write manifest.json")?;

    println!();
    println!("✅ Physical backup complete!");
    println!("   Directory: {}", dir.display());
    println!("   LSN: {}", current_lsn);
    println!("   Type: {:?}", backup_type);
    println!("   Files: {}", manifest_with_checksum.file_count);
    println!("   Total size: {} bytes", total_size);
    println!("   Manifest checksum: {}", manifest_checksum);

    Ok(())
}

/// Load parent LSN from a backup manifest
fn load_parent_lsn(parent_dir: &Path) -> Result<String> {
    let manifest_file = parent_dir.join("manifest.json");
    if !manifest_file.exists() {
        bail!(
            "Parent backup manifest not found: {}",
            manifest_file.display()
        );
    }

    let content = fs::read_to_string(&manifest_file)?;
    let manifest: PhysicalBackupManifest =
        serde_json::from_str(&content).context("Invalid parent manifest format")?;

    Ok(manifest.lsn)
}

/// List all physical backups in a directory
pub fn list_physical_backups(dir: &Path) -> Result<()> {
    if !dir.exists() {
        bail!("Backup directory not found: {}", dir.display());
    }

    println!("Physical backups in: {}", dir.display());
    println!("{}", "=".repeat(70));

    let mut backups = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_file = path.join("manifest.json");
            if manifest_file.exists() {
                let content = fs::read_to_string(&manifest_file)?;
                if let Ok(manifest) = serde_json::from_str::<PhysicalBackupManifest>(&content) {
                    backups.push((path, manifest));
                }
            }
        }
    }

    if backups.is_empty() {
        println!("No physical backups found");
        return Ok(());
    }

    // Sort by timestamp (newest first)
    backups.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

    let backup_count = backups.len();
    for (path, manifest) in backups {
        let type_str = match manifest.backup_type {
            PhysicalBackupType::Full => "FULL",
            PhysicalBackupType::Incremental => "INCR",
        };
        let parent_lsn = manifest.parent_lsn.unwrap_or_else(|| "none".to_string());

        println!();
        println!(
            "📦 Physical Backup: {}",
            path.file_name().unwrap().to_string_lossy()
        );
        println!("   Type: {}", type_str);
        println!("   Timestamp: {}", manifest.timestamp);
        println!("   LSN: {}", manifest.lsn);
        if manifest.backup_type == PhysicalBackupType::Incremental {
            println!("   Parent LSN: {}", parent_lsn);
        }
        println!("   Files: {}", manifest.file_count);
        println!(
            "   Total size: {} bytes ({} KB)",
            manifest.total_size_bytes,
            manifest.total_size_bytes / 1024
        );
        println!(
            "   Compression: {}",
            if manifest.compressed {
                "enabled"
            } else {
                "disabled"
            }
        );

        if !manifest.wal_archives.is_empty() {
            println!("   WAL archives: {}", manifest.wal_archives.len());
            for wal in &manifest.wal_archives {
                println!("     - {}: {} entries", wal.file_name, wal.entry_count);
            }
        }
    }

    println!();
    println!("Total: {} physical backup(s)", backup_count);

    Ok(())
}

/// Verify physical backup integrity
pub fn verify_physical_backup(dir: &Path) -> Result<()> {
    let manifest_file = dir.join("manifest.json");

    if !manifest_file.exists() {
        bail!("Not a physical backup directory: {}", dir.display());
    }

    println!("Verifying physical backup: {}", dir.display());
    println!("{}", "=".repeat(70));

    // Load manifest
    let content = fs::read_to_string(&manifest_file)?;
    let manifest: PhysicalBackupManifest =
        serde_json::from_str(&content).context("Invalid manifest format")?;

    println!("✅ Manifest loaded");
    println!("   Version: {}", manifest.version);
    println!("   Type: {:?}", manifest.backup_type);
    println!("   Timestamp: {}", manifest.timestamp);
    println!("   LSN: {}", manifest.lsn);
    if let Some(ref parent) = manifest.parent_lsn {
        println!("   Parent LSN: {}", parent);
    }

    // Verify manifest checksum
    let manifest_json =
        serde_json::to_string_pretty(&manifest.clone()).context("Failed to serialize manifest")?;
    let calculated_checksum = calculate_checksum_for_bytes(manifest_json.as_bytes());

    if calculated_checksum == manifest.checksum {
        println!("✅ Manifest checksum valid");
    } else {
        println!("❌ Manifest checksum mismatch!");
        println!("   Expected: {}", manifest.checksum);
        println!("   Actual: {}", calculated_checksum);
        bail!("Manifest integrity check failed");
    }

    // Verify data directory
    let data_backup_dir = dir.join("data");
    if !data_backup_dir.exists() {
        bail!(
            "Data backup directory missing: {}",
            data_backup_dir.display()
        );
    }
    println!("✅ Data backup directory exists");

    // Verify each file exists
    let mut all_valid = true;
    for file_info in &manifest.files {
        let file_path = data_backup_dir.join(&file_info.relative_path);
        if !file_path.exists() {
            eprintln!("❌ Missing file: {}", file_info.relative_path);
            all_valid = false;
        } else {
            let metadata = fs::metadata(&file_path)?;
            let actual_size = metadata.len();
            if actual_size != file_info.size_bytes {
                eprintln!(
                    "⚠️  Size mismatch for {}: expected {}, got {}",
                    file_info.relative_path, file_info.size_bytes, actual_size
                );
            }
        }
    }

    if all_valid {
        println!("✅ All {} files present", manifest.files.len());
    }

    // Verify WAL archives
    let wal_backup_dir = dir.join("wal");
    if wal_backup_dir.exists() {
        println!("✅ WAL backup directory exists");
    }

    // Verify total size
    let calculated_size: u64 = manifest.files.iter().map(|f| f.size_bytes).sum();
    if calculated_size == manifest.total_size_bytes {
        println!("✅ Total size matches: {} bytes", manifest.total_size_bytes);
    } else {
        println!(
            "⚠️  Size mismatch: manifest says {}, calculated {}",
            manifest.total_size_bytes, calculated_size
        );
    }

    println!();
    if all_valid {
        println!("✅ Physical backup verification PASSED");
    } else {
        println!("❌ Physical backup verification FAILED");
        std::process::exit(1);
    }

    Ok(())
}

/// Restore physical backup to target directory
pub fn restore_physical_backup(dir: &Path, target: &Path, wal_target: Option<&Path>) -> Result<()> {
    let manifest_file = dir.join("manifest.json");

    if !manifest_file.exists() {
        bail!("Not a physical backup directory: {}", dir.display());
    }

    println!("Restoring physical backup from: {}", dir.display());
    println!("Target data directory: {}", target.display());
    println!("{}", "=".repeat(70));

    // Load manifest
    let content = fs::read_to_string(&manifest_file)?;
    let manifest: PhysicalBackupManifest =
        serde_json::from_str(&content).context("Invalid manifest format")?;

    println!("Loading backup created at: {}", manifest.timestamp);
    println!("LSN: {}", manifest.lsn);
    println!("Files to restore: {}", manifest.files.len());

    // Create target directory
    fs::create_dir_all(target).context("Failed to create target directory")?;

    // Restore data files
    let data_backup_dir = dir.join("data");
    let mut restored_count = 0;

    for file_info in &manifest.files {
        let backup_path = data_backup_dir.join(&file_info.relative_path);
        let target_path = target.join(&file_info.relative_path);

        if !backup_path.exists() {
            eprintln!("⚠️  Backup file not found: {}", file_info.relative_path);
            continue;
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if file_info.is_compressed {
            // Decompress file
            let compressed_file = File::open(&backup_path)?;
            let mut decoder = GzDecoder::new(compressed_file);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            let mut target_file = File::create(
                target_path.with_extension(
                    target_path
                        .extension()
                        .map(|e| {
                            let ext = e.to_string_lossy();
                            ext.strip_suffix(".gz").unwrap_or(&ext).to_string()
                        })
                        .unwrap_or_default(),
                ),
            )?;
            target_file.write_all(&decompressed)?;
        } else {
            fs::copy(&backup_path, &target_path)?;
        }

        restored_count += 1;
        println!("  Restored: {}", file_info.relative_path);
    }

    // Restore WAL archives if target WAL directory specified
    if let Some(wal_dir) = wal_target {
        fs::create_dir_all(wal_dir)?;
        let wal_backup_dir = dir.join("wal");

        if wal_backup_dir.exists() {
            for wal_archive in &manifest.wal_archives {
                let archive_path = wal_backup_dir.join(&wal_archive.file_name);
                if archive_path.exists() {
                    let target_path = wal_dir.join(&wal_archive.file_name);
                    fs::copy(&archive_path, &target_path)?;
                    println!("  Restored WAL: {}", wal_archive.file_name);
                }
            }
        }
    }

    println!();
    println!("✅ Restore complete!");
    println!("   Files restored: {}", restored_count);
    println!("   Data directory: {}", target.display());
    if let Some(wal_dir) = wal_target {
        println!("   WAL directory: {}", wal_dir.display());
    }

    Ok(())
}

/// Prune physical backups based on retention policy
pub fn prune_physical_backups(
    dir: &Path,
    keep: Option<usize>,
    keep_days: Option<usize>,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    if !dir.exists() {
        bail!("Backup directory not found: {}", dir.display());
    }

    // Validate retention policy
    let (retain_by_count, retain_by_days) = match (keep, keep_days) {
        (Some(k), None) => (Some(k), None),
        (None, Some(d)) => (None, Some(d)),
        (Some(k), Some(d)) => (Some(k), Some(d)),
        (None, None) => {
            bail!("Must specify either --keep or --keep-days retention policy");
        }
    };

    println!("Physical backup retention policy:");
    println!("{}", "=".repeat(70));
    if let Some(k) = retain_by_count {
        println!("  Keep: {} most recent backup(s)", k);
    }
    if let Some(d) = retain_by_days {
        println!("  Keep: backups from last {} day(s)", d);
    }
    println!("  Mode: {}", if dry_run { "DRY RUN (no changes)" } else { "LIVE" });

    // Collect all backups
    let mut backups = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_file = path.join("manifest.json");
            if manifest_file.exists() {
                let content = fs::read_to_string(&manifest_file)?;
                if let Ok(manifest) = serde_json::from_str::<PhysicalBackupManifest>(&content) {
                    let size = calculate_backup_size(&path)?;
                    backups.push(BackupToPrune {
                        path,
                        manifest,
                        size,
                    });
                }
            }
        }
    }

    if backups.is_empty() {
        println!("\nNo physical backups found to prune");
        return Ok(());
    }

    // Sort by timestamp (newest first)
    backups.sort_by(|a, b| b.manifest.timestamp.cmp(&a.manifest.timestamp));

    // Determine which backups to delete
    let mut to_delete = Vec::new();
    let mut kept = Vec::new();

    // Apply count-based retention
    if let Some(max_keep) = retain_by_count {
        for (i, backup) in backups.iter().enumerate() {
            if i >= max_keep {
                to_delete.push(backup.clone());
            } else {
                kept.push(backup.clone());
            }
        }
    } else {
        // Keep all if no count-based retention
        kept.extend(backups.iter().cloned());
    }

    // Apply days-based retention on top
    if let Some(days) = retain_by_days {
        let cutoff = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (days as u64 * 86400);

        let before = to_delete.len();
        to_delete.retain(|b| {
            let backup_time = parse_timestamp_to_epoch(&b.manifest.timestamp);
            backup_time < cutoff
        });
        kept.retain(|b| {
            let backup_time = parse_timestamp_to_epoch(&b.manifest.timestamp);
            backup_time >= cutoff
        });

        if to_delete.len() > before {
            println!("  [Days filter] {} backup(s) older than {} days", to_delete.len() - before, days);
        }
    }

    if to_delete.is_empty() {
        println!("\n✅ No backups to prune based on retention policy");
        println!("   Total backups: {}", backups.len());
        println!("   Would keep: {}", kept.len());
        return Ok(());
    }

    // Print what would be deleted
    println!();
    println!("Backup(s) to delete ({} total, {} bytes):", to_delete.len(), to_delete.iter().map(|b| b.size).sum::<u64>());
    for backup in &to_delete {
        println!(
            "  ❌ {} - {} ({} bytes)",
            backup.path.file_name().unwrap().to_string_lossy(),
            backup.manifest.timestamp,
            backup.size
        );
    }

    println!();
    println!("Backup(s) to keep ({} total):", kept.len());
    for backup in &kept {
        println!(
            "  ✅ {} - {} ({} bytes)",
            backup.path.file_name().unwrap().to_string_lossy(),
            backup.manifest.timestamp,
            backup.size
        );
    }

    if dry_run {
        println!();
        println!("🔍 DRY RUN: No backups were deleted");
        return Ok(());
    }

    // Confirm deletion unless force is set
    if !force {
        print!("\n⚠️  Delete {} backup(s)? [y/N] ", to_delete.len());
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!("Aborted.");
            return Ok(());
        }
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Delete backups
    let mut deleted_count = 0u64;
    let mut deleted_size = 0u64;

    for backup in &to_delete {
        match fs::remove_dir_all(&backup.path) {
            Ok(_) => {
                deleted_count += 1;
                deleted_size += backup.size;
                println!("  Deleted: {}", backup.path.display());
            }
            Err(e) => {
                eprintln!("  Failed to delete {}: {}", backup.path.display(), e);
            }
        }
    }

    println!();
    println!("✅ Prune complete!");
    println!("   Deleted: {} backup(s), {} bytes", deleted_count, deleted_size);
    println!("   Kept: {} backup(s)", kept.len());

    Ok(())
}

/// Internal struct for tracking backups to prune
#[derive(Debug, Clone)]
struct BackupToPrune {
    path: PathBuf,
    manifest: PhysicalBackupManifest,
    size: u64,
}

/// Calculate total size of a backup directory
fn calculate_backup_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;

    if path.is_dir() {
        for entry in walkdir(path)? {
            total += entry.metadata()?.len();
        }
    }

    Ok(total)
}

/// Parse timestamp string to epoch seconds
fn parse_timestamp_to_epoch(timestamp: &str) -> u64 {
    // Format: "YYYY-MM-DD_HH:MM:SS"
    let parts: Vec<&str> = timestamp.split(['-', '_', ':']).collect();
    if parts.len() < 6 {
        return 0;
    }

    let year: u64 = parts[0].parse().unwrap_or(1970);
    let month: u64 = parts[1].parse().unwrap_or(1);
    let day: u64 = parts[2].parse().unwrap_or(1);
    let hour: u64 = parts[3].parse().unwrap_or(0);
    let minute: u64 = parts[4].parse().unwrap_or(0);
    let second: u64 = parts[5].parse().unwrap_or(0);

    // Calculate days since epoch (simplified leap year handling)
    let days = days_since_epoch(year, month, day);
    days * 86400 + hour * 3600 + minute * 60 + second
}

/// Calculate days since epoch for a given date
fn days_since_epoch(year: u64, month: u64, day: u64) -> u64 {
    let mut total_days = 0u64;

    // Add days for complete years
    for y in 1970..year {
        total_days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for months in current year
    let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = is_leap_year(year);
    for m in 1..month {
        total_days += if is_leap && m == 2 { 29 } else { days_per_month[(m - 1) as usize] };
    }

    // Add days
    total_days += day - 1;

    total_days
}

/// Check if a year is a leap year
fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

// ============================================================================
// Helper functions
// ============================================================================

/// Compress a file using GZIP
fn compress_file(source: &Path, target: &Path) -> Result<()> {
    let source_file = File::open(source)?;
    let target_file = File::create(target)?;
    let mut encoder = GzEncoder::new(target_file, Compression::default());
    let mut reader = BufReader::new(source_file);
    io::copy(&mut reader, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Walk directory recursively and collect all file paths
fn walkdir(path: &Path) -> Result<Vec<fs::DirEntry>> {
    let mut results = Vec::new();

    if !path.exists() {
        return Ok(results);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(walkdir(&path)?);
        } else {
            results.push(entry);
        }
    }

    Ok(results)
}

/// Generate a simple LSN (Log Sequence Number)
fn generate_lsn() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!(
        "{:016x}-{:08x}",
        duration.as_secs(),
        duration.subsec_nanos()
    )
}

/// Generate a simple ISO8601 timestamp
fn chrono_lite_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let mut year = 1970;
    let mut remaining_days = days as i64;

    while remaining_days >= 365 {
        let leap_years = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days >= leap_years {
            remaining_days -= leap_years;
            year += 1;
        } else {
            break;
        }
    }

    let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

    let mut month = 1;
    for (i, days_in_month) in days_per_month.iter().enumerate() {
        let actual_days = if is_leap && i == 1 {
            29
        } else {
            *days_in_month
        };
        if remaining_days < actual_days as i64 {
            break;
        }
        remaining_days -= actual_days as i64;
        month = i + 2;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}_{:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

/// Calculate checksum for bytes
fn calculate_checksum_for_bytes(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_lsn() {
        let lsn = generate_lsn();
        assert!(!lsn.is_empty());
        assert!(lsn.contains("-"));
        assert_eq!(lsn.len(), 25); // 16 hex digits + dash + 8 hex digits
    }

    #[test]
    fn test_physical_backup_manifest_serialization() {
        let manifest = PhysicalBackupManifest {
            version: "1.0".to_string(),
            backup_type: PhysicalBackupType::Full,
            timestamp: "2024-01-01_12:00:00".to_string(),
            lsn: "00000001-00000000".to_string(),
            parent_lsn: None,
            data_dir: "./data".to_string(),
            wal_dir: "./wal".to_string(),
            total_size_bytes: 1024,
            file_count: 1,
            compressed: true,
            files: vec![BackupFileInfo {
                relative_path: "test.dat".to_string(),
                size_bytes: 1024,
                is_compressed: true,
            }],
            wal_archives: vec![],
            checksum: "abc123".to_string(),
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: PhysicalBackupManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.backup_type, PhysicalBackupType::Full);
        assert_eq!(parsed.file_count, 1);
    }

    #[test]
    fn test_physical_backup_type_serialization() {
        let full = PhysicalBackupType::Full;
        let incremental = PhysicalBackupType::Incremental;

        let full_json = serde_json::to_string(&full).unwrap();
        let incr_json = serde_json::to_string(&incremental).unwrap();

        assert!(full_json.contains("full"));
        assert!(incr_json.contains("incremental"));
    }

    #[test]
    fn test_walkdir_empty_directory() {
        let dir = tempdir().unwrap();
        let entries = walkdir(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_walkdir_with_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let entries = walkdir(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_walkdir_nested_directories() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::write(subdir.join("file2.txt"), "content2").unwrap();

        let entries = walkdir(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_checksum_calculation() {
        let data = b"test data";
        let checksum1 = calculate_checksum_for_bytes(data);
        let checksum2 = calculate_checksum_for_bytes(data);
        assert_eq!(checksum1, checksum2);

        let different_data = b"different data";
        let different_checksum = calculate_checksum_for_bytes(different_data);
        assert_ne!(checksum1, different_checksum);
    }

    #[test]
    fn test_compress_and_decompress_file() {
        let dir = tempdir().unwrap();
        let original_path = dir.path().join("original.txt");
        let compressed_path = dir.path().join("compressed.txt.gz");
        let decompressed_path = dir.path().join("decompressed.txt");

        // Write original content
        std::fs::write(&original_path, "test content for compression").unwrap();

        // Compress
        compress_file(&original_path, &compressed_path).unwrap();
        assert!(compressed_path.exists());

        // Decompress
        let compressed_file = File::open(&compressed_path).unwrap();
        let mut decoder = GzDecoder::new(compressed_file);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        let decompressed_content = String::from_utf8(decompressed).unwrap();
        assert_eq!(decompressed_content, "test content for compression");
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let timestamp = chrono_lite_timestamp();
        assert!(!timestamp.is_empty());
        // Format: YYYY-MM-DD_HH:MM:SS
        assert!(timestamp.len() == 19);
        assert!(timestamp.contains('_'));
        assert!(timestamp.contains('-'));
    }

    #[test]
    fn test_backup_file_info_serialization() {
        let file_info = BackupFileInfo {
            relative_path: "data/table1.dat".to_string(),
            size_bytes: 4096,
            is_compressed: false,
        };

        let json = serde_json::to_string(&file_info).unwrap();
        let parsed: BackupFileInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.relative_path, "data/table1.dat");
        assert_eq!(parsed.size_bytes, 4096);
        assert!(!parsed.is_compressed);
    }

    #[test]
    fn test_wal_archive_info_serialization() {
        let wal_info = WalArchiveInfo {
            archive_id: 1,
            file_name: "archive_1.wal".to_string(),
            original_size: 10000,
            compressed_size: 5000,
            entry_count: 100,
            timestamp: 1704067200,
        };

        let json = serde_json::to_string(&wal_info).unwrap();
        let parsed: WalArchiveInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.archive_id, 1);
        assert_eq!(parsed.entry_count, 100);
        assert_eq!(parsed.compressed_size, 5000);
    }

    #[test]
    fn test_is_leap_year() {
        // Regular years
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(2025));

        // Leap years
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2016));

        // Century years
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(2000)); // Divisible by 400
    }

    #[test]
    fn test_days_since_epoch() {
        // Jan 1, 1970 = 0 days
        assert_eq!(days_since_epoch(1970, 1, 1), 0);

        // Jan 1, 1971 = 365 days (1970 was not a leap year)
        assert_eq!(days_since_epoch(1971, 1, 1), 365);

        // Jan 1, 1972 = 730 days (1971 was not a leap year)
        assert_eq!(days_since_epoch(1972, 1, 1), 730);

        // Jan 1, 1973 = 1096 days (1972 was a leap year, 366 days)
        assert_eq!(days_since_epoch(1973, 1, 1), 1096);

        // Test Feb 29 in leap year
        let feb29_leap = days_since_epoch(2024, 2, 29);
        let feb28_leap = days_since_epoch(2024, 2, 28);
        assert_eq!(feb29_leap - feb28_leap, 1);
    }

    #[test]
    fn test_parse_timestamp_to_epoch() {
        // 2024-01-01_00:00:00 should be around 1704067200
        let epoch = parse_timestamp_to_epoch("2024-01-01_00:00:00");
        assert!(epoch > 0);

        // Same day should have same day component
        let epoch2 = parse_timestamp_to_epoch("2024-01-02_00:00:00");
        let diff = epoch2 - epoch;
        assert_eq!(diff, 86400); // 1 day = 86400 seconds
    }

    #[test]
    fn test_calculate_backup_size() {
        let dir = tempdir().unwrap();
        let file1 = dir.path().join("file1.txt");
        let file2 = dir.path().join("file2.txt");
        std::fs::write(&file1, "content1").unwrap();
        std::fs::write(&file2, "longer content here").unwrap();

        let size = calculate_backup_size(dir.path()).unwrap();
        // "content1" = 8 bytes, "longer content here" = 19 bytes, total = 27
        assert_eq!(size, 27);
    }

    #[test]
    fn test_calculate_backup_size_empty_dir() {
        let dir = tempdir().unwrap();
        let size = calculate_backup_size(dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_physical_backup_command_prune_variants() {
        // Test that Prune command can be parsed (structopt will handle this)
        use std::process::Command;

        // Just verify the command help works
        let output = Command::new("cargo")
            .args(&[
                "run",
                "-p",
                "sqlrustgo-tools",
                "--",
                "physical-backup",
                "prune",
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        assert!(
            output.status.success(),
            "Prune command should be valid: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("prune") || stdout.contains("keep"));
    }
}
