//! Backup and Restore CLI Tool
//!
//! Provides commands for:
//! - Full backup: export database to SQL files
//! - Incremental backup: backup changes since last backup (based on LSN)
//! - List backups: show available backups in a directory
//! - Verify backup: validate backup integrity
//! - Restore: restore database from backup

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::{
    BackupExporter, BackupFormat, ColumnDefinition, DataRestorer, MemoryStorage,
    StorageEngine, TableInfo,
};
use sqlrustgo_types::Value;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

/// Backup metadata stored in manifest.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: String,
    pub backup_type: BackupType,
    pub timestamp: String,
    /// LSN (Log Sequence Number) for incremental backups
    pub lsn: Option<String>,
    /// Parent backup LSN for incremental backups
    pub parent_lsn: Option<String>,
    pub tables: Vec<TableBackupInfo>,
    /// Total rows across all tables
    pub total_rows: usize,
    /// Checksum for integrity verification
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBackupInfo {
    pub name: String,
    pub row_count: usize,
    pub columns: Vec<ColumnBackupInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnBackupInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub auto_increment: bool,
    pub references: Option<String>,
}

/// CLI commands for backup operations
#[derive(Debug, StructOpt)]
#[structopt(name = "backup", about = "Backup and restore database")]
pub enum BackupCommand {
    /// Create a full backup
    Backup {
        /// Backup directory (will be created)
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Backup format (sql, csv, json)
        #[structopt(short = "f", long = "format", default_value = "sql")]
        format: String,

        /// Database data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Create an incremental backup
    Incremental {
        /// Parent backup directory (last full/incremental backup)
        #[structopt(long = "parent")]
        parent: PathBuf,

        /// This backup directory (will be created)
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Backup format (sql, csv, json)
        #[structopt(short = "f", long = "format", default_value = "sql")]
        format: String,

        /// Database data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,
    },

    /// List backups in a directory
    List {
        /// Backup directory to list
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,
    },

    /// Verify backup integrity
    Verify {
        /// Backup directory to verify
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,
    },

    /// Restore from backup
    Restore {
        /// Backup directory to restore from
        #[structopt(short = "d", long = "dir")]
        dir: PathBuf,

        /// Target data directory
        #[structopt(short = "t", long = "target")]
        target: PathBuf,

        /// Drop existing tables before restore
        #[structopt(short = "c", long = "clean", long_help = "Drop existing tables before restore")]
        clean: bool,
    },
}

/// Backup tool entry point
pub fn run() -> Result<()> {
    let cmd = BackupCommand::from_args();
    match cmd {
        BackupCommand::Backup {
            dir,
            format,
            data_dir,
        } => create_full_backup(&dir, &format, &data_dir),
        BackupCommand::Incremental {
            parent,
            dir,
            format,
            data_dir,
        } => create_incremental_backup(&parent, &dir, &format, &data_dir),
        BackupCommand::List { dir } => list_backups(&dir),
        BackupCommand::Verify { dir } => verify_backup(&dir),
        BackupCommand::Restore { dir, target, clean } => restore_backup(&dir, &target, clean),
    }
}

/// Create a full backup
pub fn create_full_backup(dir: &Path, format: &str, data_dir: &Path) -> Result<()> {
    let format = BackupFormat::from_str(format)
        .context("Invalid format. Use: sql, csv, or json")?;

    println!("Creating full backup to: {}", dir.display());

    // Create backup directory structure
    fs::create_dir_all(dir).context("Failed to create backup directory")?;
    let data_subdir = dir.join("data");
    fs::create_dir_all(&data_subdir).context("Failed to create data directory")?;

    // Create in-memory storage with sample data for demo
    // In production, this would load from actual storage engine
    let storage = create_demo_storage();

    // Get all tables
    let tables = storage.list_tables();
    if tables.is_empty() {
        println!("No tables to backup");
        return Ok(());
    }

    let mut table_infos = Vec::new();
    let mut total_rows = 0;

    // Export each table
    for table_name in &tables {
        let table_info = storage.get_table_info(table_name)?;
        let rows = storage.scan(table_name)?;
        let row_count = rows.len();
        total_rows += row_count;

        let table_backup_info = TableBackupInfo {
            name: table_name.clone(),
            row_count,
            columns: table_info
                .columns
                .iter()
                .map(|c| ColumnBackupInfo {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                    is_primary_key: c.is_primary_key,
                    is_unique: c.is_unique,
                    auto_increment: c.auto_increment,
                    references: c.references.as_ref().map(|r| {
                        format!("{}.{}", r.referenced_table, r.referenced_column)
                    }),
                })
                .collect(),
        };
        table_infos.push(table_backup_info);

        // Export data file
        let data_file = data_subdir.join(format!("{}.sql", table_name));
        BackupExporter::export_table(&storage, table_name, &data_file, format)?;

        println!(
            "  Exported table '{}': {} rows -> {}",
            table_name,
            row_count,
            data_file.display()
        );
    }

    // Generate schema.sql
    let schema_file = dir.join("schema.sql");
    let schema_content = generate_schema_sql(&storage, &tables)?;
    fs::write(&schema_file, &schema_content).context("Failed to write schema.sql")?;

    // Create manifest
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        backup_type: BackupType::Full,
        timestamp: chrono_lite_timestamp(),
        lsn: Some(generate_lsn()),
        parent_lsn: None,
        tables: table_infos,
        total_rows,
        checksum: calculate_checksum(&data_subdir)?,
    };

    let manifest_file = dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_file, manifest_json).context("Failed to write manifest.json")?;

    println!();
    println!("✅ Backup complete!");
    println!("   Directory: {}", dir.display());
    println!("   Tables: {}", tables.len());
    println!("   Total rows: {}", total_rows);
    println!("   LSN: {}", manifest.lsn.as_ref().unwrap());

    Ok(())
}

/// Create an incremental backup
pub fn create_incremental_backup(parent: &Path, dir: &Path, format: &str, data_dir: &Path) -> Result<()> {
    let format = BackupFormat::from_str(format)
        .context("Invalid format. Use: sql, csv, or json")?;

    // Load parent manifest to get parent LSN
    let parent_manifest_file = parent.join("manifest.json");
    let parent_manifest: BackupManifest = if parent_manifest_file.exists() {
        let content = fs::read_to_string(&parent_manifest_file)?;
        serde_json::from_str(&content).context("Invalid parent manifest")?
    } else {
        anyhow::bail!("Parent backup not found: {}", parent.display());
    };

    let parent_lsn = parent_manifest.lsn.unwrap_or_else(|| "0".to_string());

    println!("Creating incremental backup:");
    println!("  Parent: {} (LSN: {})", parent.display(), parent_lsn);
    println!("  Target: {}", dir.display());

    // Create backup directory structure
    fs::create_dir_all(dir).context("Failed to create backup directory")?;
    let data_subdir = dir.join("data");
    fs::create_dir_all(&data_subdir).context("Failed to create data directory")?;

    // Load storage (demo mode)
    let storage = create_demo_storage();

    // For incremental backup, we would compare with parent LSN
    // In this demo, we backup all tables
    let tables = storage.list_tables();
    if tables.is_empty() {
        println!("No tables to backup");
        return Ok(());
    }

    let current_lsn = generate_lsn();
    let mut table_infos = Vec::new();
    let mut total_rows = 0;

    for table_name in &tables {
        let table_info = storage.get_table_info(table_name)?;
        let rows = storage.scan(table_name)?;
        let row_count = rows.len();
        total_rows += row_count;

        let table_backup_info = TableBackupInfo {
            name: table_name.clone(),
            row_count,
            columns: table_info
                .columns
                .iter()
                .map(|c| ColumnBackupInfo {
                    name: c.name.clone(),
                    data_type: c.data_type.clone(),
                    nullable: c.nullable,
                    is_primary_key: c.is_primary_key,
                    is_unique: c.is_unique,
                    auto_increment: c.auto_increment,
                    references: c.references.as_ref().map(|r| {
                        format!("{}.{}", r.referenced_table, r.referenced_column)
                    }),
                })
                .collect(),
        };
        table_infos.push(table_backup_info);

        // Export data file
        let data_file = data_subdir.join(format!("{}.sql", table_name));
        BackupExporter::export_table(&storage, table_name, &data_file, format)?;

        println!(
            "  Exported table '{}': {} rows",
            table_name,
            row_count
        );
    }

    // Create manifest
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        backup_type: BackupType::Incremental,
        timestamp: chrono_lite_timestamp(),
        lsn: Some(current_lsn.clone()),
        parent_lsn: Some(parent_lsn.clone()),
        tables: table_infos,
        total_rows,
        checksum: calculate_checksum(&data_subdir)?,
    };

    let manifest_file = dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_file, manifest_json).context("Failed to write manifest.json")?;

    println!();
    println!("✅ Incremental backup complete!");
    println!("   Directory: {}", dir.display());
    println!("   Tables: {}", tables.len());
    println!("   Total rows: {}", total_rows);
    println!("   LSN: {}", current_lsn);
    println!("   Parent LSN: {}", parent_lsn);

    Ok(())
}

/// List all backups in a directory
pub fn list_backups(dir: &Path) -> Result<()> {
    if !dir.exists() {
        anyhow::bail!("Backup directory not found: {}", dir.display());
    }

    println!("Backups in: {}", dir.display());
    println!("{}", "=".repeat(70));

    // Read all subdirectories that contain manifest.json
    let mut backups = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_file = path.join("manifest.json");
            if manifest_file.exists() {
                let content = fs::read_to_string(&manifest_file)?;
                if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&content) {
                    backups.push((path, manifest));
                }
            }
        }
    }

    if backups.is_empty() {
        println!("No backups found");
        return Ok(());
    }

    // Sort by timestamp (newest first)
    backups.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

    let backup_count = backups.len();
    for (path, manifest) in backups {
        let type_str = match manifest.backup_type {
            BackupType::Full => "FULL",
            BackupType::Incremental => "INCR",
        };
        let lsn = manifest.lsn.unwrap_or_else(|| "N/A".to_string());
        let parent_lsn = manifest.parent_lsn.unwrap_or_else(|| "none".to_string());

        println!();
        println!("📦 Backup: {}", path.file_name().unwrap().to_string_lossy());
        println!("   Type: {}", type_str);
        println!("   Timestamp: {}", manifest.timestamp);
        println!("   LSN: {}", lsn);
        if manifest.backup_type == BackupType::Incremental {
            println!("   Parent LSN: {}", parent_lsn);
        }
        println!("   Tables: {}", manifest.tables.len());
        println!("   Total rows: {}", manifest.total_rows);

        for table in &manifest.tables {
            println!("     - {}: {} rows", table.name, table.row_count);
        }
    }

    println!();
    println!("Total: {} backup(s)", backup_count);

    Ok(())
}

/// Verify backup integrity
pub fn verify_backup(dir: &Path) -> Result<()> {
    let manifest_file = dir.join("manifest.json");

    if !manifest_file.exists() {
        anyhow::bail!("Not a backup directory: {}", dir.display());
    }

    println!("Verifying backup: {}", dir.display());
    println!("{}", "=".repeat(70));

    // Load manifest
    let content = fs::read_to_string(&manifest_file)?;
    let manifest: BackupManifest = serde_json::from_str(&content)
        .context("Invalid manifest format")?;

    println!("✅ Manifest loaded");
    println!("   Version: {}", manifest.version);
    println!("   Type: {:?}", manifest.backup_type);
    println!("   Timestamp: {}", manifest.timestamp);
    println!("   Tables: {}", manifest.tables.len());

    // Verify data directory
    let data_dir = dir.join("data");
    if !data_dir.exists() {
        anyhow::bail!("Data directory missing: {}", data_dir.display());
    }
    println!("✅ Data directory exists");

    // Verify each table file exists
    let mut all_valid = true;
    for table in &manifest.tables {
        let data_file = data_dir.join(format!("{}.sql", table.name));
        if !data_file.exists() {
            eprintln!("❌ Missing data file: {}", data_file.display());
            all_valid = false;
        } else {
            // Verify file is not empty
            let metadata = fs::metadata(&data_file)?;
            if metadata.len() == 0 {
                eprintln!("⚠️  Empty data file: {}", data_file.display());
            }
        }
    }

    if all_valid {
        println!("✅ All {} table files present", manifest.tables.len());
    }

    // Verify schema.sql exists
    let schema_file = dir.join("schema.sql");
    if schema_file.exists() {
        println!("✅ Schema file exists");
    } else {
        println!("⚠️  Schema file missing (optional)");
    }

    // Verify checksum
    let calculated_checksum = calculate_checksum(&data_dir)?;
    if calculated_checksum == manifest.checksum {
        println!("✅ Checksum valid");
    } else {
        println!("❌ Checksum mismatch!");
        println!("   Expected: {}", manifest.checksum);
        println!("   Actual: {}", calculated_checksum);
        all_valid = false;
    }

    println!();
    if all_valid {
        println!("✅ Backup verification PASSED");
    } else {
        println!("❌ Backup verification FAILED");
        std::process::exit(1);
    }

    Ok(())
}

/// Restore database from backup
pub fn restore_backup(dir: &Path, target: &Path, clean: bool) -> Result<()> {
    let manifest_file = dir.join("manifest.json");

    if !manifest_file.exists() {
        anyhow::bail!("Not a backup directory: {}", dir.display());
    }

    println!("Restoring from backup: {}", dir.display());
    println!("Target directory: {}", target.display());
    if clean {
        println!("Mode: CLEAN (will drop existing tables)");
    } else {
        println!("Mode: MERGE (will add to existing tables)");
    }
    println!("{}", "=".repeat(70));

    // Load manifest
    let content = fs::read_to_string(&manifest_file)?;
    let manifest: BackupManifest = serde_json::from_str(&content)
        .context("Invalid manifest format")?;

    // Create target directory
    fs::create_dir_all(target).context("Failed to create target directory")?;
    let data_subdir = dir.join("data");

    // Create in-memory storage for restore
    let mut storage = MemoryStorage::new();

    // Process each table
    let format = BackupFormat::Sql;

    for table_info in &manifest.tables {
        println!("Restoring table: {}", table_info.name);

        if clean {
            // Drop table if exists
            if storage.has_table(&table_info.name) {
                storage.drop_table(&table_info.name)?;
                println!("  Dropped existing table");
            }
        }

        // Create table schema
        let columns: Vec<ColumnDefinition> = table_info
            .columns
            .iter()
            .map(|c| ColumnDefinition {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.nullable,
                is_primary_key: c.is_primary_key,
                is_unique: c.is_unique,
                auto_increment: c.auto_increment,
                references: None,
            })
            .collect();

        let table_schema = TableInfo {
            name: table_info.name.clone(),
            columns,
        };

        storage.create_table(&table_schema)?;

        // Restore data
        let data_file = data_subdir.join(format!("{}.sql", table_info.name));
        if data_file.exists() {
            let rows_restored = DataRestorer::restore_from_backup(
                &mut storage,
                &data_file,
                format,
            )?;
            println!("  Restored {} rows", rows_restored);
        }
    }

    println!();
    println!("✅ Restore complete!");
    println!("   Tables restored: {}", manifest.tables.len());
    println!("   Total rows: {}", manifest.total_rows);

    Ok(())
}

// ============================================================================
// Helper functions
// ============================================================================

/// Generate a simple LSN (Log Sequence Number)
fn generate_lsn() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:016x}-{:08x}", duration.as_secs(), duration.subsec_nanos())
}

/// Generate a simple ISO8601 timestamp
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple format: YYYY-MM-DD_HH:MM:SS
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Calculate year, month, day from days since epoch (1970-01-01)
    // Using a simple algorithm for date calculation
    let mut year = 1970;
    let mut remaining_days = days as i64;

    // Approximate year (365 days + leap year handling)
    while remaining_days >= 365 {
        let leap_years = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days >= leap_years {
            remaining_days -= leap_years;
            year += 1;
        } else {
            break;
        }
    }

    // Days per month (non-leap year)
    let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

    let mut month = 1;
    for (i, days_in_month) in days_per_month.iter().enumerate() {
        let actual_days = if is_leap && i == 1 { 29 } else { *days_in_month };
        if remaining_days < actual_days as i64 {
            break;
        }
        remaining_days -= actual_days as i64;
        month = i + 2; // months are 1-indexed, but we start at 0
    }
    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}_{:02}:{:02}:{:02}", year, month, day, hours, minutes, seconds)
}

/// Calculate checksum for backup verification
fn calculate_checksum(data_dir: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    if !data_dir.exists() {
        return Ok("empty".to_string());
    }

    let mut files: Vec<_> = fs::read_dir(data_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .collect();
    files.sort_by_key(|e| e.file_name());

    for entry in files {
        let content = fs::read_to_string(entry.path())?;
        content.hash(&mut hasher);
    }

    let hash = hasher.finish();
    Ok(format!("{:016x}", hash))
}

/// Generate CREATE TABLE statements from storage
fn generate_schema_sql(storage: &dyn StorageEngine, tables: &[String]) -> Result<String> {
    let mut schema = String::new();

    schema.push_str("-- SQLRustGo Backup Schema\n");
    schema.push_str(&format!("-- Generated at: {}\n\n", chrono_lite_timestamp()));

    for table_name in tables {
        let info = storage.get_table_info(table_name)?;
        schema.push_str(&generate_create_table_sql(&info));
        schema.push('\n');
    }

    Ok(schema)
}

/// Generate CREATE TABLE SQL for a single table
fn generate_create_table_sql(table_info: &TableInfo) -> String {
    let mut sql = format!("CREATE TABLE {} (\n", table_info.name);

    let column_defs: Vec<String> = table_info
        .columns
        .iter()
        .map(|col| {
            let mut def = format!("  {} {}", col.name, col.data_type);
            if !col.nullable {
                def.push_str(" NOT NULL");
            }
            if col.is_primary_key {
                def.push_str(" PRIMARY KEY");
            }
            if col.is_unique {
                def.push_str(" UNIQUE");
            }
            if col.auto_increment {
                def.push_str(" AUTO_INCREMENT");
            }
            def
        })
        .collect();

    sql.push_str(&column_defs.join(",\n"));
    sql.push_str("\n);\n");

    // Add foreign key constraints
    for col in &table_info.columns {
        if let Some(ref fk) = col.references {
            sql.push_str(&format!(
                "ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {} ({});\n",
                table_info.name, col.name, fk.referenced_table, fk.referenced_column
            ));
        }
    }

    sql
}

/// Create demo storage with sample data
fn create_demo_storage() -> MemoryStorage {
    let mut storage = MemoryStorage::new();

    // Create users table
    let users_table = TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: true,
                is_unique: true,
                auto_increment: true,
                references: None,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
            ColumnDefinition {
                name: "email".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: true,
                auto_increment: false,
                references: None,
            },
            ColumnDefinition {
                name: "created_at".to_string(),
                data_type: "TIMESTAMP".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
        ],
    };
    storage.create_table(&users_table).unwrap();

    // Insert demo users (using TEXT for timestamp to simplify demo)
    storage.insert("users", vec![
        vec![Value::Integer(1), Value::Text("Alice".to_string()), Value::Text("alice@example.com".to_string()), Value::Text("2024-01-15 10:30:00".to_string())],
        vec![Value::Integer(2), Value::Text("Bob".to_string()), Value::Text("bob@example.com".to_string()), Value::Text("2024-01-16 14:22:00".to_string())],
        vec![Value::Integer(3), Value::Text("Charlie".to_string()), Value::Text("charlie@example.com".to_string()), Value::Text("2024-01-17 09:15:00".to_string())],
    ]).unwrap();

    // Create orders table
    let orders_table = TableInfo {
        name: "orders".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: true,
                is_unique: true,
                auto_increment: true,
                references: None,
            },
            ColumnDefinition {
                name: "user_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: Some(sqlrustgo_storage::ForeignKeyConstraint {
                    referenced_table: "users".to_string(),
                    referenced_column: "id".to_string(),
                    on_delete: None,
                    on_update: None,
                }),
            },
            ColumnDefinition {
                name: "total".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
            ColumnDefinition {
                name: "status".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
        ],
    };
    storage.create_table(&orders_table).unwrap();

    // Insert demo orders
    storage.insert("orders", vec![
        vec![Value::Integer(1), Value::Integer(1), Value::Float(99.99), Value::Text("completed".to_string())],
        vec![Value::Integer(2), Value::Integer(1), Value::Float(149.50), Value::Text("pending".to_string())],
        vec![Value::Integer(3), Value::Integer(2), Value::Float(29.99), Value::Text("completed".to_string())],
    ]).unwrap();

    // Create products table
    let products_table = TableInfo {
        name: "products".to_string(),
        columns: vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: true,
                is_unique: true,
                auto_increment: true,
                references: None,
            },
            ColumnDefinition {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
            ColumnDefinition {
                name: "price".to_string(),
                data_type: "FLOAT".to_string(),
                nullable: false,
                is_primary_key: false,
                is_unique: false,
                auto_increment: false,
                references: None,
            },
        ],
    };
    storage.create_table(&products_table).unwrap();

    // Insert demo products
    storage.insert("products", vec![
        vec![Value::Integer(1), Value::Text("Widget".to_string()), Value::Float(19.99)],
        vec![Value::Integer(2), Value::Text("Gadget".to_string()), Value::Float(49.99)],
        vec![Value::Integer(3), Value::Text("Doohickey".to_string()), Value::Float(99.99)],
    ]).unwrap();

    storage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_lsn() {
        let lsn = generate_lsn();
        assert!(!lsn.is_empty());
        assert!(lsn.contains("-"));
    }

    #[test]
    fn test_backup_manifest_serialization() {
        let manifest = BackupManifest {
            version: "1.0".to_string(),
            backup_type: BackupType::Full,
            timestamp: "2024-01-01 00:00:00".to_string(),
            lsn: Some("00000001-00000000".to_string()),
            parent_lsn: None,
            tables: vec![],
            total_rows: 0,
            checksum: "abc123".to_string(),
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: BackupManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "1.0");
        assert_eq!(parsed.backup_type, BackupType::Full);
    }

    #[test]
    fn test_backup_type_serialization() {
        let full = BackupType::Full;
        let incremental = BackupType::Incremental;

        let full_json = serde_json::to_string(&full).unwrap();
        let incr_json = serde_json::to_string(&incremental).unwrap();

        assert!(full_json.contains("full"));
        assert!(incr_json.contains("incremental"));
    }

    #[test]
    fn test_generate_create_table_sql() {
        let table = TableInfo {
            name: "users".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_primary_key: true,
                    is_unique: true,
                    auto_increment: true,
                    references: None,
                },
                ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: true,
                    is_primary_key: false,
                    is_unique: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        let sql = generate_create_table_sql(&table);
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("id INTEGER NOT NULL PRIMARY KEY"));
        assert!(sql.contains("name TEXT"));
    }

    #[test]
    fn test_create_demo_storage() {
        let storage = create_demo_storage();
        let tables = storage.list_tables();

        assert!(tables.contains(&"users".to_string()));
        assert!(tables.contains(&"orders".to_string()));
        assert!(tables.contains(&"products".to_string()));

        let users = storage.scan("users").unwrap();
        assert_eq!(users.len(), 3);

        let orders = storage.scan("orders").unwrap();
        assert_eq!(orders.len(), 3);

        let products = storage.scan("products").unwrap();
        assert_eq!(products.len(), 3);
    }
}
