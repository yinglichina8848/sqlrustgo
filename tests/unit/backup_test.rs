use sqlrustgo_tools::{
    BackupManifest, BackupType, ChangeOperation, ChangeRecord, ChangeSet, IncrementalBackupContext,
    TableBackupInfo,
};
use sqlrustgo_types::Value;
use std::env::temp_dir;
use std::path::PathBuf;

#[test]
fn test_backup_manifest_full() {
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        backup_type: BackupType::Full,
        timestamp: "2024-01-01_12:00:00".to_string(),
        lsn: Some("00000001-00000000".to_string()),
        parent_lsn: None,
        tables: vec![TableBackupInfo {
            name: "users".to_string(),
            row_count: 100,
            columns: vec![],
        }],
        total_rows: 100,
        checksum: "abc123".to_string(),
    };

    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.backup_type, BackupType::Full);
    assert!(manifest.parent_lsn.is_none());
    assert_eq!(manifest.tables.len(), 1);
}

#[test]
fn test_backup_manifest_incremental() {
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        backup_type: BackupType::Incremental,
        timestamp: "2024-01-02_12:00:00".to_string(),
        lsn: Some("00000002-00000000".to_string()),
        parent_lsn: Some("00000001-00000000".to_string()),
        tables: vec![],
        total_rows: 10,
        checksum: "def456".to_string(),
    };

    assert_eq!(manifest.backup_type, BackupType::Incremental);
    assert_eq!(manifest.parent_lsn.as_ref().unwrap(), "00000001-00000000");
}

#[test]
fn test_backup_type_serialization() {
    let full = BackupType::Full;
    let incr = BackupType::Incremental;

    let full_json = serde_json::to_string(&full).unwrap();
    let incr_json = serde_json::to_string(&incr).unwrap();

    assert!(full_json.contains("full"));
    assert!(incr_json.contains("incremental"));
}

#[test]
fn test_change_record_insert() {
    let record = ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Insert,
        key_values: vec![Value::Integer(1)],
        row_data: Some(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        lsn: "00000001-00000001".to_string(),
    };

    assert_eq!(record.table, "users");
    assert_eq!(record.operation, ChangeOperation::Insert);
    assert!(record.row_data.is_some());
}

#[test]
fn test_change_record_update() {
    let record = ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Update,
        key_values: vec![Value::Integer(1)],
        row_data: Some(vec![
            Value::Integer(1),
            Value::Text("Alice Updated".to_string()),
        ]),
        lsn: "00000001-00000002".to_string(),
    };

    assert_eq!(record.operation, ChangeOperation::Update);
}

#[test]
fn test_change_record_delete() {
    let record = ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Delete,
        key_values: vec![Value::Integer(1)],
        row_data: None,
        lsn: "00000001-00000003".to_string(),
    };

    assert_eq!(record.operation, ChangeOperation::Delete);
    assert!(record.row_data.is_none());
}

#[test]
fn test_change_set_new() {
    let changeset = ChangeSet::new("00000001-00000000");
    assert!(changeset.is_empty());
    assert_eq!(changeset.len(), 0);
    assert_eq!(changeset.start_lsn, "00000001-00000000");
}

#[test]
fn test_change_set_add_change() {
    let mut changeset = ChangeSet::new("00000001-00000000");

    changeset.add_change(ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Insert,
        key_values: vec![Value::Integer(1)],
        row_data: Some(vec![Value::Integer(1), Value::Text("Bob".to_string())]),
        lsn: "00000001-00000001".to_string(),
    });

    assert!(!changeset.is_empty());
    assert_eq!(changeset.len(), 1);
    assert_eq!(changeset.end_lsn, "00000001-00000001");
}

#[test]
fn test_change_set_export_sql() {
    let mut changeset = ChangeSet::new("00000001-00000000");

    changeset.add_change(ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Insert,
        key_values: vec![Value::Integer(1)],
        row_data: Some(vec![Value::Integer(1), Value::Text("Alice".to_string())]),
        lsn: "00000001-00000001".to_string(),
    });

    let sql = changeset.export_to_sql();
    assert!(sql.contains("INSERT INTO users VALUES"));
    assert!(sql.contains("'Alice'"));
    assert!(sql.contains("-- Incremental Changes"));
}

#[test]
fn test_change_set_export_delete_sql() {
    let mut changeset = ChangeSet::new("00000001-00000000");

    changeset.add_change(ChangeRecord {
        table: "users".to_string(),
        operation: ChangeOperation::Delete,
        key_values: vec![Value::Integer(1)],
        row_data: None,
        lsn: "00000001-00000001".to_string(),
    });

    let sql = changeset.export_to_sql();
    assert!(sql.contains("DELETE FROM users"));
}

#[test]
fn test_incremental_backup_context_insert() {
    let mut ctx = IncrementalBackupContext::new();

    ctx.record_insert(
        "users",
        vec![Value::Integer(1)],
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
    );

    assert_eq!(ctx.total_changes(), 1);

    let changes = ctx.get_changes();
    assert!(changes.contains_key("users"));
    assert_eq!(changes.get("users").unwrap().len(), 1);
}

#[test]
fn test_incremental_backup_context_update() {
    let mut ctx = IncrementalBackupContext::new();

    ctx.record_update(
        "users",
        vec![Value::Integer(1)],
        vec![Value::Integer(1), Value::Text("Updated".to_string())],
    );

    assert_eq!(ctx.total_changes(), 1);
}

#[test]
fn test_incremental_backup_context_delete() {
    let mut ctx = IncrementalBackupContext::new();

    ctx.record_delete("users", vec![Value::Integer(1)]);

    assert_eq!(ctx.total_changes(), 1);
}

#[test]
fn test_incremental_backup_context_multiple_tables() {
    let mut ctx = IncrementalBackupContext::new();

    ctx.record_insert(
        "users",
        vec![Value::Integer(1)],
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
    );

    ctx.record_insert(
        "orders",
        vec![Value::Integer(1)],
        vec![Value::Integer(1), Value::Integer(1), Value::Float(99.99)],
    );

    assert_eq!(ctx.total_changes(), 2);

    let changes = ctx.get_changes();
    assert!(changes.contains_key("users"));
    assert!(changes.contains_key("orders"));
}

#[test]
fn test_incremental_backup_context_empty() {
    let ctx = IncrementalBackupContext::new();
    assert_eq!(ctx.total_changes(), 0);
}

#[test]
fn test_backup_manifest_serde_roundtrip() {
    let manifest = BackupManifest {
        version: "1.0".to_string(),
        backup_type: BackupType::Incremental,
        timestamp: "2024-01-02_12:00:00".to_string(),
        lsn: Some("00000002-00000000".to_string()),
        parent_lsn: Some("00000001-00000000".to_string()),
        tables: vec![TableBackupInfo {
            name: "users".to_string(),
            row_count: 10,
            columns: vec![],
        }],
        total_rows: 10,
        checksum: "xyz789".to_string(),
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let parsed: BackupManifest = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.version, "1.0");
    assert_eq!(parsed.backup_type, BackupType::Incremental);
    assert_eq!(parsed.lsn.as_ref().unwrap(), "00000002-00000000");
    assert_eq!(parsed.tables.len(), 1);
}

#[test]
fn test_table_backup_info() {
    let info = TableBackupInfo {
        name: "orders".to_string(),
        row_count: 500,
        columns: vec![],
    };

    assert_eq!(info.name, "orders");
    assert_eq!(info.row_count, 500);
}
