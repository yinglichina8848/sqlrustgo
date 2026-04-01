// SQLRustGo 物理备份工具集成测试
//
// 运行方式:
//   cargo test --test physical_backup_test -- --nocapture
//   cargo test --test physical_backup_test -- --nocapture --test-threads=1

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_physical_backup_help() {
    let output = Command::new("cargo")
        .args(&["run", "-p", "sqlrustgo-tools", "--", "physical-backup", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        stderr
    );

    // 验证输出包含关键信息
    assert!(
        stdout.contains("Physical backup") || stdout.contains("physical-backup"),
        "Help output should mention physical backup"
    );
}

#[test]
fn test_physical_backup_subcommand_help() {
    let subcommands = vec!["backup", "list", "verify", "restore"];

    for subcommand in subcommands {
        let output = Command::new("cargo")
            .args(&[
                "run",
                "-p",
                "sqlrustgo-tools",
                "--",
                "physical-backup",
                subcommand,
                "--help",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "Subcommand {} should succeed, stderr: {}",
            subcommand,
            stderr
        );
    }
}

#[test]
fn test_physical_backup_backup_requires_args() {
    // backup 命令缺少必需参数时应失败
    let output = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "sqlrustgo-tools",
            "--",
            "physical-backup",
            "backup",
        ])
        .output()
        .expect("Failed to execute command");

    // 应该失败，因为缺少必需参数
    assert!(
        !output.status.success(),
        "Backup without required args should fail"
    );
}

#[test]
fn test_physical_backup_list_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let empty_dir = temp_dir.path().join("empty_backups");

    fs::create_dir_all(&empty_dir).expect("Failed to create empty directory");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "sqlrustgo-tools",
            "--",
            "physical-backup",
            "list",
            "--dir",
            empty_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List on empty directory should succeed, stderr: {}",
        stderr
    );

    // 空目录应该显示相应消息
    assert!(
        stdout.contains("No backups found") || stdout.is_empty() || !stdout.contains("Physical Backup"),
        "Empty directory should not show backups"
    );
}

#[test]
fn test_physical_backup_verify_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent = temp_dir.path().join("nonexistent_backup");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "sqlrustgo-tools",
            "--",
            "physical-backup",
            "verify",
            "--dir",
            nonexistent.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // 应该失败，因为备份不存在
    assert!(
        !output.status.success(),
        "Verify nonexistent backup should fail"
    );
}

#[test]
fn test_physical_backup_restore_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent = temp_dir.path().join("nonexistent_backup");
    let target = temp_dir.path().join("restore_target");

    let output = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "sqlrustgo-tools",
            "--",
            "physical-backup",
            "restore",
            "--dir",
            nonexistent.to_str().unwrap(),
            "--target",
            target.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // 应该失败，因为备份不存在
    assert!(
        !output.status.success(),
        "Restore from nonexistent backup should fail"
    );
}

#[test]
fn test_physical_backup_unit_tests() {
    // 运行物理备份单元测试
    let output = Command::new("cargo")
        .args(&[
            "test",
            "-p",
            "sqlrustgo-tools",
            "physical_backup",
            "--",
            "--nocapture",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Physical backup unit tests should pass, stderr: {}",
        stderr
    );

    // 验证测试数量
    assert!(
        stdout.contains("11 passed"),
        "Should have 11 passing tests, output: {}",
        stdout
    );
}

#[test]
fn test_physical_backup_all_unit_tests() {
    // 运行 sqlrustgo-tools 的所有单元测试
    let output = Command::new("cargo")
        .args(&[
            "test",
            "-p",
            "sqlrustgo-tools",
            "--",
            "--nocapture",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "All tools unit tests should pass, stderr: {}",
        stderr
    );

    // 验证有测试通过
    assert!(
        stdout.contains("test result: ok"),
        "Tests should pass, output: {}",
        stdout
    );
}
