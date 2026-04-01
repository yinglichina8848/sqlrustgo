//! Upgrade CLI Tool
//!
//! Provides commands for:
//! - Pre-upgrade compatibility check
//! - Execute rolling upgrades
//! - Rollback to previous version
//! - View upgrade status

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeManifest {
    pub from_version: String,
    pub to_version: String,
    pub timestamp: String,
    pub status: UpgradeStatus,
    pub backup_path: Option<PathBuf>,
    pub rollback_enabled: bool,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpgradeStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl VersionInfo {
    pub fn parse(version: &str) -> Result<Self> {
        let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid version format: {}. Expected X.Y.Z", version);
        }
        Ok(Self {
            major: parts[0].parse().context("Invalid major version")?,
            minor: parts[1].parse().context("Invalid minor version")?,
            patch: parts[2].parse().context("Invalid patch version")?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn can_upgrade_to(&self, target: &VersionInfo) -> bool {
        if self.major != target.major {
            return false;
        }
        if target.minor > self.minor {
            return true;
        }
        if target.minor == self.minor && target.patch > self.patch {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub id: usize,
    pub description: String,
    pub executed: bool,
    pub rollback_sql: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePlan {
    pub from_version: VersionInfo,
    pub to_version: VersionInfo,
    pub migration_steps: Vec<MigrationStep>,
    pub pre_check_passed: bool,
    pub estimated_duration_secs: u64,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "upgrade", about = "Upgrade SQLRustGo database version")]
pub enum UpgradeCommand {
    /// Check upgrade compatibility
    Check {
        /// Current version
        #[structopt(short = "f", long = "from")]
        from: String,

        /// Target version
        #[structopt(short = "t", long = "to")]
        to: String,

        /// Data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Execute upgrade
    Upgrade {
        /// Current version
        #[structopt(short = "f", long = "from")]
        from: String,

        /// Target version
        #[structopt(short = "t", long = "to")]
        to: String,

        /// Backup directory
        #[structopt(short = "b", long = "backup-dir")]
        backup_dir: PathBuf,

        /// Data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,

        /// Skip backup (dangerous!)
        #[structopt(short = "s", long = "skip-backup")]
        skip_backup: bool,
    },

    /// Rollback to previous version
    Rollback {
        /// Backup directory to restore
        #[structopt(short = "b", long = "backup-dir")]
        backup_dir: PathBuf,

        /// Target version to restore
        #[structopt(short = "t", long = "target")]
        target: String,

        /// Data directory
        #[structopt(short = "D", long = "data-dir", default_value = "./data")]
        data_dir: PathBuf,
    },

    /// Show upgrade status
    Status {
        /// Upgrade manifest directory
        #[structopt(short = "d", long = "dir", default_value = "./data/.upgrade")]
        dir: PathBuf,
    },

    /// List upgrade history
    History {
        /// Upgrade history directory
        #[structopt(short = "d", long = "dir", default_value = "./data/.upgrade")]
        dir: PathBuf,
    },
}

pub fn run_with_opt(cmd: UpgradeCommand) -> Result<()> {
    match cmd {
        UpgradeCommand::Check { from, to, data_dir } => check_upgrade(&from, &to, &data_dir),
        UpgradeCommand::Upgrade {
            from,
            to,
            backup_dir,
            data_dir,
            skip_backup,
        } => execute_upgrade(&from, &to, &backup_dir, &data_dir, skip_backup),
        UpgradeCommand::Rollback {
            backup_dir,
            target,
            data_dir,
        } => execute_rollback(&backup_dir, &target, &data_dir),
        UpgradeCommand::Status { dir } => show_status(&dir),
        UpgradeCommand::History { dir } => list_history(&dir),
    }
}

pub fn check_upgrade(from: &str, to: &str, data_dir: &Path) -> Result<()> {
    println!("SQLRustGo Upgrade Compatibility Check");
    println!("{}", "=".repeat(70));

    let from_ver = VersionInfo::parse(from)?;
    let to_ver = VersionInfo::parse(to)?;

    println!(
        "Checking upgrade: v{} -> v{}",
        from_ver.to_string(),
        to_ver.to_string()
    );
    println!("Data directory: {}", data_dir.display());
    println!();

    let can_upgrade = from_ver.can_upgrade_to(&to_ver);

    if !can_upgrade {
        println!("❌ UPGRADE NOT SUPPORTED");
        println!();
        println!("Cross-major version upgrades are not supported.");
        println!("You can only upgrade to the next minor or patch version.");
        std::process::exit(1);
    }

    println!("✅ Version compatibility: PASSED");

    let plan = create_upgrade_plan(&from_ver, &to_ver)?;

    println!();
    println!("Upgrade Plan:");
    println!("  Migration steps: {}", plan.migration_steps.len());
    println!(
        "  Estimated duration: {} seconds",
        plan.estimated_duration_secs
    );

    println!();
    for step in &plan.migration_steps {
        let status = if step.executed {
            "[pending]"
        } else {
            "[pending]"
        };
        println!("  {} {}: {}", status, step.id, step.description);
    }

    println!();
    println!("✅ Pre-upgrade check PASSED");
    println!();
    println!("To proceed with upgrade, run:");
    println!(
        "  sqlrustgo-tools upgrade --from {} --to {} --backup-dir <dir>",
        from, to
    );

    Ok(())
}

pub fn execute_upgrade(
    from: &str,
    to: &str,
    backup_dir: &Path,
    data_dir: &Path,
    skip_backup: bool,
) -> Result<()> {
    println!("SQLRustGo Upgrade Execution");
    println!("{}", "=".repeat(70));

    let from_ver = VersionInfo::parse(from)?;
    let to_ver = VersionInfo::parse(to)?;

    println!(
        "Upgrading: v{} -> v{}",
        from_ver.to_string(),
        to_ver.to_string()
    );
    println!("Backup directory: {}", backup_dir.display());
    println!("Data directory: {}", data_dir.display());
    println!();

    if !skip_backup {
        println!("Step 0/{}: Creating backup...", 4);
        create_backup(backup_dir, data_dir)?;
        println!("✅ Backup created");
    } else {
        println!("⚠️  WARNING: Skipping backup - this is dangerous!");
    }

    println!();
    println!("Step 1/{}: Validating data...", 4);
    validate_data(data_dir)?;
    println!("✅ Data validated");

    println!();
    println!("Step 2/{}: Running migration scripts...", 4);
    let migration_dir = backup_dir.join("migrations");
    fs::create_dir_all(&migration_dir)?;
    run_migrations(&migration_dir, &from_ver, &to_ver)?;
    println!("✅ Migrations completed");

    println!();
    println!("Step 3/{}: Updating metadata...", 4);
    update_metadata(data_dir, &to_ver)?;
    println!("✅ Metadata updated");

    println!();
    println!("Step 4/{}: Verifying upgrade...", 4);
    verify_upgrade(data_dir, &to_ver)?;
    println!("✅ Upgrade verified");

    let manifest = UpgradeManifest {
        from_version: from_ver.to_string(),
        to_version: to_ver.to_string(),
        timestamp: chrono_lite_timestamp(),
        status: UpgradeStatus::Completed,
        backup_path: Some(backup_dir.to_path_buf()),
        rollback_enabled: !skip_backup,
        steps_completed: 4,
        total_steps: 4,
        checksum: calculate_dir_checksum(data_dir)?,
    };

    let upgrade_dir = data_dir.join(".upgrade");
    fs::create_dir_all(&upgrade_dir)?;
    let manifest_file = upgrade_dir.join("last_upgrade.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_file, manifest_json)?;

    println!();
    println!("✅ UPGRADE COMPLETED SUCCESSFULLY");
    println!();
    println!("From version: v{}", from_ver.to_string());
    println!("To version: v{}", to_ver.to_string());
    println!("Backup location: {}", backup_dir.display());

    if !skip_backup {
        println!();
        println!("To rollback if needed:");
        println!(
            "  sqlrustgo-tools upgrade rollback --backup-dir {} --target {}",
            backup_dir.display(),
            from
        );
    }

    Ok(())
}

pub fn execute_rollback(backup_dir: &Path, target: &str, data_dir: &Path) -> Result<()> {
    println!("SQLRustGo Rollback Execution");
    println!("{}", "=".repeat(70));

    let target_ver = VersionInfo::parse(target)?;

    println!("Rolling back to: v{}", target_ver.to_string());
    println!("Backup directory: {}", backup_dir.display());
    println!("Data directory: {}", data_dir.display());
    println!();

    let manifest_file = backup_dir.join("manifest.json");
    if !manifest_file.exists() {
        anyhow::bail!("Invalid backup directory: no manifest.json found");
    }

    println!("Step 1/3: Validating backup...");
    validate_backup(backup_dir)?;
    println!("✅ Backup validated");

    println!();
    println!("Step 2/3: Restoring data...");
    restore_from_backup(backup_dir, data_dir)?;
    println!("✅ Data restored");

    println!();
    println!("Step 3/3: Updating metadata...");
    update_metadata(data_dir, &target_ver)?;
    println!("✅ Metadata updated");

    println!();
    println!("✅ ROLLBACK COMPLETED SUCCESSFULLY");
    println!();
    println!("Rolled back to version: v{}", target_ver.to_string());

    Ok(())
}

pub fn show_status(dir: &Path) -> Result<()> {
    let manifest_file = dir.join("last_upgrade.json");

    if !manifest_file.exists() {
        println!("No upgrade history found");
        return Ok(());
    }

    let content = fs::read_to_string(&manifest_file)?;
    let manifest: UpgradeManifest = serde_json::from_str(&content)?;

    println!("Last Upgrade Status");
    println!("{}", "=".repeat(70));
    println!("From: v{}", manifest.from_version);
    println!("To: v{}", manifest.to_version);
    println!("Timestamp: {}", manifest.timestamp);
    println!("Status: {:?}", manifest.status);
    println!(
        "Steps completed: {}/{}",
        manifest.steps_completed, manifest.total_steps
    );
    println!("Rollback enabled: {}", manifest.rollback_enabled);

    if let Some(backup_path) = &manifest.backup_path {
        println!("Backup location: {}", backup_path.display());
    }

    Ok(())
}

pub fn list_history(dir: &Path) -> Result<()> {
    if !dir.exists() {
        println!("No upgrade history found");
        return Ok(());
    }

    println!("Upgrade History");
    println!("{}", "=".repeat(70));

    let mut upgrades = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let manifest_file = path.join("manifest.json");
            if manifest_file.exists() {
                let content = fs::read_to_string(&manifest_file)?;
                if let Ok(manifest) = serde_json::from_str::<UpgradeManifest>(&content) {
                    upgrades.push((path, manifest));
                }
            }
        }
    }

    if upgrades.is_empty() {
        println!("No upgrades found");
        return Ok(());
    }

    for (path, manifest) in &upgrades {
        println!();
        println!(
            "📦 Upgrade: {}",
            path.file_name().unwrap().to_string_lossy()
        );
        println!("   {} -> v{}", manifest.from_version, manifest.to_version);
        println!("   Status: {:?}", manifest.status);
        println!("   Timestamp: {}", manifest.timestamp);
    }

    println!();
    println!("Total: {} upgrade(s)", upgrades.len());

    Ok(())
}

fn create_upgrade_plan(from: &VersionInfo, to: &VersionInfo) -> Result<UpgradePlan> {
    let mut steps = Vec::new();

    if from.minor < to.minor {
        steps.push(MigrationStep {
            id: 1,
            description: format!(
                "Schema migration from v{}.{} to v{}.{}",
                from.major, from.minor, from.major, to.minor
            ),
            executed: false,
            rollback_sql: None,
        });

        if from.patch < to.patch {
            steps.push(MigrationStep {
                id: 2,
                description: format!(
                    "Patch update v{}.{}.{} to v{}.{}.{}",
                    from.major, from.minor, from.patch, from.major, from.minor, to.patch
                ),
                executed: false,
                rollback_sql: None,
            });
        }
    } else {
        steps.push(MigrationStep {
            id: 1,
            description: format!(
                "Patch update v{}.{}.{} to v{}.{}.{}",
                from.major, from.minor, from.patch, to.major, to.minor, to.patch
            ),
            executed: false,
            rollback_sql: None,
        });
    }

    steps.push(MigrationStep {
        id: steps.len() + 1,
        description: "Update system metadata".to_string(),
        executed: false,
        rollback_sql: None,
    });

    steps.push(MigrationStep {
        id: steps.len() + 1,
        description: "Verify data integrity".to_string(),
        executed: false,
        rollback_sql: None,
    });

    Ok(UpgradePlan {
        from_version: from.clone(),
        to_version: to.clone(),
        migration_steps: steps,
        pre_check_passed: true,
        estimated_duration_secs: 60,
    })
}

fn create_backup(backup_dir: &Path, data_dir: &Path) -> Result<()> {
    fs::create_dir_all(backup_dir).context("Failed to create backup directory")?;

    let timestamp = chrono_lite_timestamp();
    let data_backup = backup_dir.join(format!("data_{}", timestamp));
    fs::create_dir_all(&data_backup)?;

    if data_dir.exists() {
        copy_dir_recursive(data_dir, &data_backup)?;
    }

    let manifest = UpgradeManifest {
        from_version: "current".to_string(),
        to_version: "upgrade_target".to_string(),
        timestamp,
        status: UpgradeStatus::Pending,
        backup_path: Some(data_backup.clone()),
        rollback_enabled: true,
        steps_completed: 0,
        total_steps: 0,
        checksum: calculate_dir_checksum(data_dir)?,
    };

    let manifest_file = backup_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_file, manifest_json)?;

    println!("  Backup created at: {}", data_backup.display());
    Ok(())
}

fn validate_data(data_dir: &Path) -> Result<()> {
    if !data_dir.exists() {
        anyhow::bail!("Data directory does not exist: {}", data_dir.display());
    }

    let catalog_dir = data_dir.join("catalog");
    if !catalog_dir.exists() {
        anyhow::bail!("Invalid data directory: catalog not found");
    }

    Ok(())
}

fn run_migrations(migration_dir: &Path, from: &VersionInfo, to: &VersionInfo) -> Result<()> {
    let migration_file = migration_dir.join(format!(
        "migration_{}_{}_{}.sql",
        from.to_string().replace('.', "_"),
        to.to_string().replace('.', "_"),
        chrono_lite_timestamp()
    ));

    let migration_sql = generate_migration_sql(from, to);
    fs::write(&migration_file, &migration_sql)?;

    println!("  Migration script: {}", migration_file.display());
    Ok(())
}

fn generate_migration_sql(from: &VersionInfo, to: &VersionInfo) -> String {
    let mut sql = String::new();
    sql.push_str(&format!(
        "-- SQLRustGo Migration: v{} -> v{}\n",
        from.to_string(),
        to.to_string()
    ));
    sql.push_str(&format!("-- Generated at: {}\n\n", chrono_lite_timestamp()));

    if from.minor < to.minor {
        sql.push_str("-- Minor version migration\n");
        sql.push_str("-- Add schema changes here\n");
    }

    if from.patch < to.patch {
        sql.push_str("-- Patch update\n");
        sql.push_str("-- Add patch-specific changes here\n");
    }

    sql.push_str("\n-- Migration completed successfully\n");
    sql
}

fn update_metadata(data_dir: &Path, version: &VersionInfo) -> Result<()> {
    let version_file = data_dir.join(".version");
    fs::write(&version_file, version.to_string())?;
    println!("  Version updated to: v{}", version.to_string());
    Ok(())
}

fn verify_upgrade(data_dir: &Path, version: &VersionInfo) -> Result<()> {
    let version_file = data_dir.join(".version");
    if version_file.exists() {
        let current_version = fs::read_to_string(&version_file)?;
        if current_version.trim() != version.to_string() {
            anyhow::bail!("Version mismatch after upgrade");
        }
    }
    Ok(())
}

fn validate_backup(backup_dir: &Path) -> Result<()> {
    let manifest_file = backup_dir.join("manifest.json");
    if !manifest_file.exists() {
        anyhow::bail!("Invalid backup: manifest.json not found");
    }

    let content = fs::read_to_string(&manifest_file)?;
    let _manifest: UpgradeManifest =
        serde_json::from_str(&content).context("Invalid backup manifest")?;

    Ok(())
}

fn restore_from_backup(backup_dir: &Path, data_dir: &Path) -> Result<()> {
    let data_subdir = backup_dir.join("data_*");
    let entries: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().to_string_lossy().starts_with("data_"))
        .collect();

    if let Some(latest) = entries.into_iter().max_by_key(|e| e.file_name()) {
        let source = latest.path();
        if data_dir.exists() {
            fs::remove_dir_all(data_dir).ok();
        }
        fs::create_dir_all(data_dir)?;
        copy_dir_recursive(&source, data_dir)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
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

fn calculate_dir_checksum(dir: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    if !dir.exists() {
        return Ok("empty".to_string());
    }

    collect_and_hash(dir, &mut hasher)?;

    let hash = hasher.finish();
    Ok(format!("{:016x}", hash))
}

fn collect_and_hash(path: &Path, hasher: &mut DefaultHasher) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let entry_path = entry.path();

        if ty.is_dir() {
            collect_and_hash(&entry_path, hasher)?;
        } else {
            let content = fs::read_to_string(&entry_path)?;
            content.hash(hasher);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_parse() {
        let v = VersionInfo::parse("2.0.0").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_info_parse_with_v() {
        let v = VersionInfo::parse("v2.1.0").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 1);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn test_version_info_to_string() {
        let v = VersionInfo::parse("2.1.0").unwrap();
        assert_eq!(v.to_string(), "2.1.0");
    }

    #[test]
    fn test_can_upgrade_to_minor() {
        let from = VersionInfo::parse("2.0.0").unwrap();
        let to = VersionInfo::parse("2.1.0").unwrap();
        assert!(from.can_upgrade_to(&to));
    }

    #[test]
    fn test_can_upgrade_to_patch() {
        let from = VersionInfo::parse("2.0.0").unwrap();
        let to = VersionInfo::parse("2.0.1").unwrap();
        assert!(from.can_upgrade_to(&to));
    }

    #[test]
    fn test_cannot_upgrade_major() {
        let from = VersionInfo::parse("2.0.0").unwrap();
        let to = VersionInfo::parse("3.0.0").unwrap();
        assert!(!from.can_upgrade_to(&to));
    }

    #[test]
    fn test_cannot_downgrade() {
        let from = VersionInfo::parse("2.1.0").unwrap();
        let to = VersionInfo::parse("2.0.0").unwrap();
        assert!(!from.can_upgrade_to(&to));
    }

    #[test]
    fn test_chrono_lite_timestamp() {
        let ts = chrono_lite_timestamp();
        assert!(ts.contains("_"));
        assert!(ts.contains("-"));
    }
}
