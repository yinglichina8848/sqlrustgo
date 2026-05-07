//! Audit Logger - System-level audit logging for GMP compliance
//!
//! This module provides automatic audit logging for all DML operations.
//! All INSERT, UPDATE, DELETE operations are tracked with tamper-evident checksums.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlrustgo_storage::{ColumnDefinition, StorageEngine};
use sqlrustgo_types::{SqlResult, Value};

/// Audit log action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    Insert,
    Update,
    Delete,
    Ddl,
    Login,
    Logout,
    Grant,
    Revoke,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Insert => "INSERT",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
            AuditAction::Ddl => "DDL",
            AuditAction::Login => "LOGIN",
            AuditAction::Logout => "LOGOUT",
            AuditAction::Grant => "GRANT",
            AuditAction::Revoke => "REVOKE",
        }
    }
}

/// Audit log entry representing a single audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub tx_id: i64,
    pub user: String,
    pub client_ip: Option<String>,
    pub operation: String,
    pub table_name: Option<String>,
    pub row_id: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub sql_text: Option<String>,
    pub session_id: Option<i64>,
    pub checksum: String,
}

impl AuditLogEntry {
    /// Parse an AuditLogEntry from a database row
    pub fn from_row(row: &[Value]) -> Option<Self> {
        let id = match &row[0] {
            Value::Integer(n) => *n,
            _ => return None,
        };
        let timestamp = match &row[1] {
            Value::Integer(n) => *n,
            _ => return None,
        };
        let tx_id = match &row[2] {
            Value::Integer(n) => *n,
            _ => return None,
        };
        let user = match &row[3] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };
        let client_ip = match &row[4] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let operation = match &row[5] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };
        let table_name = match &row[6] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let row_id = match &row[7] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let old_value = match &row[8] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let new_value = match &row[9] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let sql_text = match &row[10] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let session_id = match &row[11] {
            Value::Integer(n) => Some(*n),
            Value::Null => None,
            _ => return None,
        };
        let checksum = match &row[12] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };

        Some(AuditLogEntry {
            id,
            timestamp,
            tx_id,
            user,
            client_ip,
            operation,
            table_name,
            row_id,
            old_value,
            new_value,
            sql_text,
            session_id,
            checksum,
        })
    }

    /// Convert AuditLogEntry to a database row
    pub fn to_row(&self) -> Vec<Value> {
        vec![
            Value::Integer(self.id),
            Value::Integer(self.timestamp),
            Value::Integer(self.tx_id),
            Value::Text(self.user.clone()),
            self.client_ip.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            Value::Text(self.operation.clone()),
            self.table_name.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            self.row_id.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            self.old_value.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            self.new_value.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            self.sql_text.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null),
            self.session_id.map(Value::Integer).unwrap_or(Value::Null),
            Value::Text(self.checksum.clone()),
        ]
    }

    /// Verify the checksum of this audit log entry
    pub fn verify_checksum(&self) -> bool {
        let data = format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            self.timestamp,
            self.tx_id,
            self.user,
            self.client_ip.as_deref().unwrap_or(""),
            self.operation,
            self.table_name.as_deref().unwrap_or(""),
            self.row_id.as_deref().unwrap_or(""),
            self.old_value.as_deref().unwrap_or(""),
            self.new_value.as_deref().unwrap_or(""),
            self.sql_text.as_deref().unwrap_or(""),
            self.session_id.map(|n| n.to_string()).unwrap_or_default(),
        );
        let computed = compute_checksum(&data);
        computed == self.checksum
    }
}

/// Compute SHA256 checksum for audit data
fn compute_checksum(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// System audit log table name
pub const SYSTEM_AUDIT_LOG_TABLE: &str = "system.audit_log";

/// SQL to create the system audit log table
pub const CREATE_SYSTEM_AUDIT_LOG_TABLE: &str = r#"
CREATE TABLE system.audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    tx_id INTEGER NOT NULL,
    user TEXT NOT NULL,
    client_ip TEXT,
    operation TEXT NOT NULL,
    table_name TEXT,
    row_id TEXT,
    old_value TEXT,
    new_value TEXT,
    sql_text TEXT,
    session_id INTEGER,
    checksum TEXT NOT NULL,
    INDEX idx_tx (tx_id),
    INDEX idx_table_row (table_name, row_id),
    INDEX idx_user_time (user, timestamp),
    INDEX idx_operation (operation)
)
"#;

/// Create the system audit log table
pub fn create_system_audit_log_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
    if !storage.has_table(SYSTEM_AUDIT_LOG_TABLE) {
        let columns = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: true,
            },
            ColumnDefinition {
                name: "timestamp".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: false,
            },
            ColumnDefinition {
                name: "tx_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: false,
            },
            ColumnDefinition {
                name: "user".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
            },
            ColumnDefinition {
                name: "client_ip".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "operation".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
            },
            ColumnDefinition {
                name: "table_name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "row_id".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "old_value".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "new_value".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "sql_text".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "session_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: true,
                primary_key: false,
            },
            ColumnDefinition {
                name: "checksum".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
            },
        ];
        storage.create_table(&sqlrustgo_storage::TableInfo {
            name: SYSTEM_AUDIT_LOG_TABLE.to_string(),
            columns,
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
        })?;
    }
    Ok(())
}

/// Record an audit log entry for INSERT
#[allow(clippy::too_many_arguments)]
pub fn record_insert_audit(
    storage: &mut dyn StorageEngine,
    tx_id: i64,
    user: &str,
    client_ip: Option<&str>,
    table_name: &str,
    row_id: &str,
    new_value: &str,
    sql_text: Option<&str>,
    session_id: Option<i64>,
) -> SqlResult<i64> {
    record_audit_log_impl(
        storage,
        tx_id,
        user,
        client_ip,
        AuditAction::Insert.as_str(),
        Some(table_name),
        Some(row_id),
        None,
        Some(new_value),
        sql_text,
        session_id,
    )
}

/// Record an audit log entry for UPDATE
#[allow(clippy::too_many_arguments)]
pub fn record_update_audit(
    storage: &mut dyn StorageEngine,
    tx_id: i64,
    user: &str,
    client_ip: Option<&str>,
    table_name: &str,
    row_id: &str,
    old_value: &str,
    new_value: &str,
    sql_text: Option<&str>,
    session_id: Option<i64>,
) -> SqlResult<i64> {
    record_audit_log_impl(
        storage,
        tx_id,
        user,
        client_ip,
        AuditAction::Update.as_str(),
        Some(table_name),
        Some(row_id),
        Some(old_value),
        Some(new_value),
        sql_text,
        session_id,
    )
}

/// Record an audit log entry for DELETE
#[allow(clippy::too_many_arguments)]
pub fn record_delete_audit(
    storage: &mut dyn StorageEngine,
    tx_id: i64,
    user: &str,
    client_ip: Option<&str>,
    table_name: &str,
    row_id: &str,
    old_value: &str,
    sql_text: Option<&str>,
    session_id: Option<i64>,
) -> SqlResult<i64> {
    record_audit_log_impl(
        storage,
        tx_id,
        user,
        client_ip,
        AuditAction::Delete.as_str(),
        Some(table_name),
        Some(row_id),
        Some(old_value),
        None,
        sql_text,
        session_id,
    )
}

/// Record an audit log entry for DDL
#[allow(clippy::too_many_arguments)]
pub fn record_ddl_audit(
    storage: &mut dyn StorageEngine,
    tx_id: i64,
    user: &str,
    client_ip: Option<&str>,
    sql_text: &str,
    session_id: Option<i64>,
) -> SqlResult<i64> {
    record_audit_log_impl(
        storage,
        tx_id,
        user,
        client_ip,
        AuditAction::Ddl.as_str(),
        None,
        None,
        None,
        None,
        Some(sql_text),
        session_id,
    )
}

/// Internal function to record audit log entry
#[allow(clippy::too_many_arguments)]
fn record_audit_log_impl(
    storage: &mut dyn StorageEngine,
    tx_id: i64,
    user: &str,
    client_ip: Option<&str>,
    operation: &str,
    table_name: Option<&str>,
    row_id: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    sql_text: Option<&str>,
    session_id: Option<i64>,
) -> SqlResult<i64> {
    // Ensure audit table exists
    create_system_audit_log_table(storage)?;

    // Get the next ID
    let rows = storage.scan(SYSTEM_AUDIT_LOG_TABLE)?;
    let next_id = rows
        .iter()
        .filter_map(|r| match &r[0] {
            Value::Integer(n) => Some(*n),
            _ => None,
        })
        .max()
        .unwrap_or(0)
        + 1;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Build data for checksum
    let checksum_data = format!(
        "{}{}{}{}{}{}{}{}{}{}",
        timestamp,
        tx_id,
        user,
        client_ip.unwrap_or(""),
        operation,
        table_name.unwrap_or(""),
        row_id.unwrap_or(""),
        old_value.unwrap_or(""),
        new_value.unwrap_or(""),
        sql_text.unwrap_or(""),
    );
    let checksum = compute_checksum(&checksum_data);

    let row = vec![
        Value::Integer(next_id),
        Value::Integer(timestamp),
        Value::Integer(tx_id),
        Value::Text(user.to_string()),
        client_ip.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        Value::Text(operation.to_string()),
        table_name.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        row_id.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        old_value.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        new_value.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        sql_text.map(|s| Value::Text(s.to_string())).unwrap_or(Value::Null),
        session_id.map(Value::Integer).unwrap_or(Value::Null),
        Value::Text(checksum),
    ];

    storage.insert(SYSTEM_AUDIT_LOG_TABLE, vec![row])?;
    Ok(next_id)
}

/// Query audit logs with optional filters
pub fn query_audit_logs(
    storage: &dyn StorageEngine,
    start_time: Option<i64>,
    end_time: Option<i64>,
    user: Option<&str>,
    operation: Option<&str>,
    table_name: Option<&str>,
) -> SqlResult<Vec<AuditLogEntry>> {
    let rows = storage.scan(SYSTEM_AUDIT_LOG_TABLE)?;

    let logs = rows
        .into_iter()
        .filter_map(|row| {
            let log = AuditLogEntry::from_row(&row)?;

            // Apply filters
            if let Some(start) = start_time {
                if log.timestamp < start {
                    return None;
                }
            }
            if let Some(end) = end_time {
                if log.timestamp > end {
                    return None;
                }
            }
            if let Some(u) = user {
                if log.user != u {
                    return None;
                }
            }
            if let Some(op) = operation {
                if log.operation != op {
                    return None;
                }
            }
            if let Some(tbl) = table_name {
                if log.table_name.as_deref() != Some(tbl) {
                    return None;
                }
            }

            Some(log)
        })
        .collect();

    Ok(logs)
}

/// Get all audit logs
pub fn get_all_audit_logs(storage: &dyn StorageEngine) -> SqlResult<Vec<AuditLogEntry>> {
    let rows = storage.scan(SYSTEM_AUDIT_LOG_TABLE)?;
    let logs = rows
        .into_iter()
        .filter_map(|row| AuditLogEntry::from_row(&row))
        .collect();
    Ok(logs)
}

/// Get audit log by ID
pub fn get_audit_log_by_id(
    storage: &dyn StorageEngine,
    id: i64,
) -> SqlResult<Option<AuditLogEntry>> {
    let rows = storage.scan(SYSTEM_AUDIT_LOG_TABLE)?;
    let log = rows
        .into_iter()
        .filter_map(|row| AuditLogEntry::from_row(&row))
        .find(|log| log.id == id);
    Ok(log)
}

/// AuditLogger wrapper that automatically logs DML operations
pub struct AuditLogger {
    enabled: bool,
    user: String,
    client_ip: Option<String>,
    session_id: Option<i64>,
}

impl AuditLogger {
    pub fn new(user: String) -> Self {
        Self {
            enabled: true,
            user,
            client_ip: None,
            session_id: None,
        }
    }

    pub fn with_client_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_session_id(mut self, session_id: i64) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Log an INSERT operation
    #[allow(clippy::too_many_arguments)]
    pub fn log_insert(
        &self,
        storage: &mut dyn StorageEngine,
        tx_id: i64,
        table_name: &str,
        row_id: &str,
        new_value: &str,
        sql_text: Option<&str>,
    ) -> SqlResult<Option<i64>> {
        if !self.enabled {
            return Ok(None);
        }
        let id = record_insert_audit(
            storage,
            tx_id,
            &self.user,
            self.client_ip.as_deref(),
            table_name,
            row_id,
            new_value,
            sql_text,
            self.session_id,
        )?;
        Ok(Some(id))
    }

    /// Log an UPDATE operation
    #[allow(clippy::too_many_arguments)]
    pub fn log_update(
        &self,
        storage: &mut dyn StorageEngine,
        tx_id: i64,
        table_name: &str,
        row_id: &str,
        old_value: &str,
        new_value: &str,
        sql_text: Option<&str>,
    ) -> SqlResult<Option<i64>> {
        if !self.enabled {
            return Ok(None);
        }
        let id = record_update_audit(
            storage,
            tx_id,
            &self.user,
            self.client_ip.as_deref(),
            table_name,
            row_id,
            old_value,
            new_value,
            sql_text,
            self.session_id,
        )?;
        Ok(Some(id))
    }

    /// Log a DELETE operation
    #[allow(clippy::too_many_arguments)]
    pub fn log_delete(
        &self,
        storage: &mut dyn StorageEngine,
        tx_id: i64,
        table_name: &str,
        row_id: &str,
        old_value: &str,
        sql_text: Option<&str>,
    ) -> SqlResult<Option<i64>> {
        if !self.enabled {
            return Ok(None);
        }
        let id = record_delete_audit(
            storage,
            tx_id,
            &self.user,
            self.client_ip.as_deref(),
            table_name,
            row_id,
            old_value,
            sql_text,
            self.session_id,
        )?;
        Ok(Some(id))
    }

    /// Log a DDL operation
    pub fn log_ddl(
        &self,
        storage: &mut dyn StorageEngine,
        tx_id: i64,
        sql_text: &str,
    ) -> SqlResult<Option<i64>> {
        if !self.enabled {
            return Ok(None);
        }
        let id = record_ddl_audit(
            storage,
            tx_id,
            &self.user,
            self.client_ip.as_deref(),
            sql_text,
            self.session_id,
        )?;
        Ok(Some(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::MemoryStorage;

    #[test]
    fn test_audit_action_conversion() {
        assert_eq!(AuditAction::Insert.as_str(), "INSERT");
        assert_eq!(AuditAction::Update.as_str(), "UPDATE");
        assert_eq!(AuditAction::Delete.as_str(), "DELETE");
        assert_eq!(AuditAction::Ddl.as_str(), "DDL");
    }

    #[test]
    fn test_create_audit_table() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();
        assert!(storage.has_table(SYSTEM_AUDIT_LOG_TABLE));
    }

    #[test]
    fn test_record_and_query_insert_audit() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let log_id = record_insert_audit(
            &mut storage,
            1,
            "test_user",
            Some("127.0.0.1"),
            "test_table",
            "row_1",
            r#"{"id":1,"name":"test"}"#,
            Some("INSERT INTO test_table VALUES (1, 'test')"),
            Some(123),
        )
        .unwrap();

        assert!(log_id > 0);

        let logs = query_audit_logs(&storage, None, None, None, None, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user, "test_user");
        assert_eq!(logs[0].operation, "INSERT");
        assert_eq!(logs[0].table_name, Some("test_table".to_string()));
    }

    #[test]
    fn test_record_update_audit() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let log_id = record_update_audit(
            &mut storage,
            1,
            "test_user",
            None,
            "test_table",
            "row_1",
            r#"{"id":1,"name":"old"}"#,
            r#"{"id":1,"name":"new"}"#,
            None,
            None,
        )
        .unwrap();

        assert!(log_id > 0);

        let logs = query_audit_logs(&storage, None, None, None, Some("UPDATE"), None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].old_value, Some(r#"{"id":1,"name":"old"}"#.to_string()));
        assert_eq!(logs[0].new_value, Some(r#"{"id":1,"name":"new"}"#.to_string()));
    }

    #[test]
    fn test_record_delete_audit() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let log_id = record_delete_audit(
            &mut storage,
            1,
            "test_user",
            None,
            "test_table",
            "row_1",
            r#"{"id":1,"name":"test"}"#,
            None,
            None,
        )
        .unwrap();

        assert!(log_id > 0);

        let logs = query_audit_logs(&storage, None, None, None, Some("DELETE"), None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].operation, "DELETE");
    }

    #[test]
    fn test_record_ddl_audit() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let log_id = record_ddl_audit(
            &mut storage,
            1,
            "admin",
            None,
            "CREATE TABLE test (id INTEGER PRIMARY KEY)",
            None,
        )
        .unwrap();

        assert!(log_id > 0);

        let logs = query_audit_logs(&storage, None, None, None, Some("DDL"), None).unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].sql_text.is_some());
    }

    #[test]
    fn test_audit_log_checksum() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let log_id = record_insert_audit(
            &mut storage,
            1,
            "test_user",
            None,
            "test_table",
            "row_1",
            r#"{"id":1}"#,
            None,
            None,
        )
        .unwrap();

        let log = get_audit_log_by_id(&storage, log_id).unwrap().unwrap();
        assert!(log.verify_checksum());
    }

    #[test]
    fn test_audit_log_filtering() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        // Insert from different users
        record_insert_audit(&mut storage, 1, "user1", None, "t1", "r1", "{}", None, None).unwrap();
        record_insert_audit(&mut storage, 1, "user1", None, "t2", "r1", "{}", None, None).unwrap();
        record_insert_audit(&mut storage, 1, "user2", None, "t1", "r1", "{}", None, None).unwrap();

        // Filter by user
        let user1_logs = query_audit_logs(&storage, None, None, Some("user1"), None, None).unwrap();
        assert_eq!(user1_logs.len(), 2);

        // Filter by table
        let t1_logs = query_audit_logs(&storage, None, None, None, None, Some("t1")).unwrap();
        assert_eq!(t1_logs.len(), 2);

        // Filter by operation
        let insert_logs = query_audit_logs(&storage, None, None, None, Some("INSERT"), None).unwrap();
        assert_eq!(insert_logs.len(), 3);
    }

    #[test]
    fn test_audit_logger_disable() {
        let mut storage = MemoryStorage::new();
        create_system_audit_log_table(&mut storage).unwrap();

        let mut logger = AuditLogger::new("test_user".to_string());
        logger.disable();

        let result = logger.log_insert(&mut storage, 1, "t1", "r1", "{}", None).unwrap();
        assert!(result.is_none());

        // Enable and try again
        logger.enable();
        let result = logger.log_insert(&mut storage, 1, "t1", "r1", "{}", None).unwrap();
        assert!(result.is_some());

        let logs = get_all_audit_logs(&storage).unwrap();
        assert_eq!(logs.len(), 1);
    }
}