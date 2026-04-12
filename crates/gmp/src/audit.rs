//! GMP Audit Logging
//!
//! Provides audit logging functionality for GMP document management.
//! All CREATE, UPDATE, DELETE operations on GMP tables are tracked
//! with tamper-evident checksums.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlrustgo_storage::{ColumnDefinition, StorageEngine};
use sqlrustgo_types::{SqlResult, Value};

/// Audit log action types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "CREATE",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CREATE" => Some(AuditAction::Create),
            "UPDATE" => Some(AuditAction::Update),
            "DELETE" => Some(AuditAction::Delete),
            _ => None,
        }
    }
}

/// Audit log entry representing a single audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
    pub checksum: String,
}

impl AuditLog {
    /// Parse an AuditLog from a database row
    pub fn from_row(row: &[Value]) -> Option<Self> {
        let id = match &row[0] {
            Value::Integer(n) => *n,
            _ => return None,
        };
        let timestamp = match &row[1] {
            Value::Integer(n) => *n,
            Value::Timestamp(n) => *n,
            _ => return None,
        };
        let user_id = match &row[2] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };
        let action = match &row[3] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };
        let table_name = match &row[4] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };
        let record_id = match &row[5] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let old_value = match &row[6] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let new_value = match &row[7] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let ip_address = match &row[8] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let session_id = match &row[9] {
            Value::Text(s) => Some(s.clone()),
            Value::Null => None,
            _ => return None,
        };
        let checksum = match &row[10] {
            Value::Text(s) => s.clone(),
            _ => return None,
        };

        Some(AuditLog {
            id,
            timestamp,
            user_id,
            action,
            table_name,
            record_id,
            old_value,
            new_value,
            ip_address,
            session_id,
            checksum,
        })
    }

    /// Convert AuditLog to a database row
    pub fn to_row(&self) -> Vec<Value> {
        vec![
            Value::Integer(self.id),
            Value::Integer(self.timestamp),
            Value::Text(self.user_id.clone()),
            Value::Text(self.action.clone()),
            Value::Text(self.table_name.clone()),
            self.record_id
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
            self.old_value
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
            self.new_value
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
            self.ip_address
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
            self.session_id
                .as_ref()
                .map(|s| Value::Text(s.clone()))
                .unwrap_or(Value::Null),
            Value::Text(self.checksum.clone()),
        ]
    }

    /// Verify the checksum of this audit log entry
    pub fn verify_checksum(&self) -> bool {
        let data = format!(
            "{}{}{}{}{}{}{}{}{}",
            self.timestamp,
            self.user_id,
            self.action,
            self.table_name,
            self.record_id.as_deref().unwrap_or(""),
            self.old_value.as_deref().unwrap_or(""),
            self.new_value.as_deref().unwrap_or(""),
            self.ip_address.as_deref().unwrap_or(""),
            self.session_id.as_deref().unwrap_or(""),
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

/// GMP audit log table name
pub const TABLE_AUDIT_LOG: &str = "gmp_audit_log";

/// SQL to create the audit log table
pub const CREATE_AUDIT_LOG_TABLE: &str = r#"
CREATE TABLE gmp_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    table_name TEXT NOT NULL,
    record_id TEXT,
    old_value TEXT,
    new_value TEXT,
    ip_address TEXT,
    session_id TEXT,
    checksum TEXT NOT NULL,
    INDEX idx_timestamp (timestamp),
    INDEX idx_user_id (user_id),
    INDEX idx_table_name (table_name),
    INDEX idx_action (action)
)
"#;

/// Create the audit log table
pub fn create_audit_log_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
    if !storage.has_table(TABLE_AUDIT_LOG) {
        let columns = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: true,
                references: None,
                auto_increment: true,
                compression: None,
            },
            ColumnDefinition {
                name: "timestamp".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "user_id".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "action".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "table_name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "record_id".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "old_value".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "new_value".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "ip_address".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "session_id".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
            ColumnDefinition {
                name: "checksum".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
                compression: None,
            },
        ];
        storage.create_table(&sqlrustgo_storage::TableInfo {
            name: TABLE_AUDIT_LOG.to_string(),
            columns,
        })?;
    }
    Ok(())
}

/// Record an audit log entry
#[allow(clippy::too_many_arguments)]
pub fn record_audit_log(
    storage: &mut dyn StorageEngine,
    user_id: &str,
    action: &str,
    table_name: &str,
    record_id: Option<&str>,
    old_value: Option<&str>,
    new_value: Option<&str>,
    ip_address: Option<&str>,
    session_id: Option<&str>,
) -> SqlResult<i64> {
    // Get the next ID
    let rows = storage.scan(TABLE_AUDIT_LOG)?;
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
        "{}{}{}{}{}{}{}{}{}",
        timestamp,
        user_id,
        action,
        table_name,
        record_id.unwrap_or(""),
        old_value.unwrap_or(""),
        new_value.unwrap_or(""),
        ip_address.unwrap_or(""),
        session_id.unwrap_or(""),
    );
    let checksum = compute_checksum(&checksum_data);

    let row = vec![
        Value::Integer(next_id),
        Value::Integer(timestamp),
        Value::Text(user_id.to_string()),
        Value::Text(action.to_string()),
        Value::Text(table_name.to_string()),
        record_id
            .map(|s| Value::Text(s.to_string()))
            .unwrap_or(Value::Null),
        old_value
            .map(|s| Value::Text(s.to_string()))
            .unwrap_or(Value::Null),
        new_value
            .map(|s| Value::Text(s.to_string()))
            .unwrap_or(Value::Null),
        ip_address
            .map(|s| Value::Text(s.to_string()))
            .unwrap_or(Value::Null),
        session_id
            .map(|s| Value::Text(s.to_string()))
            .unwrap_or(Value::Null),
        Value::Text(checksum),
    ];

    storage.insert(TABLE_AUDIT_LOG, vec![row])?;
    Ok(next_id)
}

/// Query audit logs with optional filters
pub fn query_audit_logs(
    storage: &dyn StorageEngine,
    start_time: Option<i64>,
    end_time: Option<i64>,
    user_id: Option<&str>,
    action: Option<&str>,
    table_name: Option<&str>,
) -> SqlResult<Vec<AuditLog>> {
    let rows = storage.scan(TABLE_AUDIT_LOG)?;

    let logs = rows
        .into_iter()
        .filter_map(|row| {
            let log = AuditLog::from_row(&row)?;

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
            if let Some(uid) = user_id {
                if log.user_id != uid {
                    return None;
                }
            }
            if let Some(act) = action {
                if log.action != act {
                    return None;
                }
            }
            if let Some(tbl) = table_name {
                if log.table_name != tbl {
                    return None;
                }
            }

            Some(log)
        })
        .collect();

    Ok(logs)
}

/// Get all audit logs
pub fn get_all_audit_logs(storage: &dyn StorageEngine) -> SqlResult<Vec<AuditLog>> {
    let rows = storage.scan(TABLE_AUDIT_LOG)?;
    let logs = rows
        .into_iter()
        .filter_map(|row| AuditLog::from_row(&row))
        .collect();
    Ok(logs)
}

/// Get audit log by ID
pub fn get_audit_log_by_id(storage: &dyn StorageEngine, id: i64) -> SqlResult<Option<AuditLog>> {
    let rows = storage.scan(TABLE_AUDIT_LOG)?;
    let log = rows
        .into_iter()
        .filter_map(|row| AuditLog::from_row(&row))
        .find(|log| log.id == id);
    Ok(log)
}

/// Get audit statistics for a time period
#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_records: i64,
    pub create_count: i64,
    pub update_count: i64,
    pub delete_count: i64,
    pub by_user: Vec<UserCount>,
    pub by_table: Vec<TableCount>,
}

#[derive(Debug, Clone)]
pub struct UserCount {
    pub user_id: String,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct TableCount {
    pub table_name: String,
    pub count: i64,
}

pub fn get_audit_stats(
    storage: &dyn StorageEngine,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> SqlResult<AuditStats> {
    let logs = query_audit_logs(storage, start_time, end_time, None, None, None)?;

    let total_records = logs.len() as i64;
    let create_count = logs.iter().filter(|l| l.action == "CREATE").count() as i64;
    let update_count = logs.iter().filter(|l| l.action == "UPDATE").count() as i64;
    let delete_count = logs.iter().filter(|l| l.action == "DELETE").count() as i64;

    // Count by user
    let mut user_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for log in &logs {
        *user_counts.entry(log.user_id.clone()).or_insert(0) += 1;
    }
    let by_user = user_counts
        .into_iter()
        .map(|(user_id, count)| UserCount { user_id, count })
        .collect();

    // Count by table
    let mut table_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for log in &logs {
        *table_counts.entry(log.table_name.clone()).or_insert(0) += 1;
    }
    let by_table = table_counts
        .into_iter()
        .map(|(table_name, count)| TableCount { table_name, count })
        .collect();

    Ok(AuditStats {
        total_records,
        create_count,
        update_count,
        delete_count,
        by_user,
        by_table,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::MemoryStorage;

    #[test]
    fn test_audit_action_conversion() {
        assert_eq!(AuditAction::from_str("CREATE"), Some(AuditAction::Create));
        assert_eq!(AuditAction::from_str("create"), Some(AuditAction::Create));
        assert_eq!(AuditAction::from_str("UPDATE"), Some(AuditAction::Update));
        assert_eq!(AuditAction::from_str("DELETE"), Some(AuditAction::Delete));
        assert_eq!(AuditAction::from_str("UNKNOWN"), None);
        assert_eq!(AuditAction::Create.as_str(), "CREATE");
    }

    #[test]
    fn test_create_audit_table() {
        let mut storage = MemoryStorage::new();
        create_audit_log_table(&mut storage).unwrap();
        assert!(storage.has_table(TABLE_AUDIT_LOG));
    }

    #[test]
    fn test_record_and_query_audit_log() {
        let mut storage = MemoryStorage::new();
        create_audit_log_table(&mut storage).unwrap();

        let log_id = record_audit_log(
            &mut storage,
            "user1",
            "CREATE",
            "gmp_documents",
            Some("1"),
            None,
            Some(r#"{"title":"Test"}"#),
            Some("192.168.1.1"),
            Some("session123"),
        )
        .unwrap();

        assert!(log_id > 0);

        let logs = query_audit_logs(&storage, None, None, None, None, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "user1");
        assert_eq!(logs[0].action, "CREATE");
        assert_eq!(logs[0].table_name, "gmp_documents");
    }

    #[test]
    fn test_audit_log_checksum() {
        let mut storage = MemoryStorage::new();
        create_audit_log_table(&mut storage).unwrap();

        let log_id = record_audit_log(
            &mut storage,
            "user1",
            "UPDATE",
            "gmp_documents",
            Some("1"),
            Some(r#"{"title":"Old"}"#),
            Some(r#"{"title":"New"}"#),
            None,
            None,
        )
        .unwrap();

        let log = get_audit_log_by_id(&storage, log_id).unwrap().unwrap();
        assert!(log.verify_checksum());
    }

    #[test]
    fn test_audit_stats() {
        let mut storage = MemoryStorage::new();
        create_audit_log_table(&mut storage).unwrap();

        // Create multiple audit entries
        record_audit_log(
            &mut storage,
            "user1",
            "CREATE",
            "gmp_documents",
            Some("1"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        record_audit_log(
            &mut storage,
            "user1",
            "UPDATE",
            "gmp_documents",
            Some("1"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        record_audit_log(
            &mut storage,
            "user2",
            "CREATE",
            "gmp_documents",
            Some("2"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        record_audit_log(
            &mut storage,
            "user2",
            "DELETE",
            "gmp_documents",
            Some("1"),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let stats = get_audit_stats(&storage, None, None).unwrap();
        assert_eq!(stats.total_records, 4);
        assert_eq!(stats.create_count, 2);
        assert_eq!(stats.update_count, 1);
        assert_eq!(stats.delete_count, 1);
        assert_eq!(stats.by_user.len(), 2);
    }

    #[test]
    fn test_audit_log_filtering() {
        let mut storage = MemoryStorage::new();
        create_audit_log_table(&mut storage).unwrap();

        record_audit_log(
            &mut storage,
            "user1",
            "CREATE",
            "gmp_documents",
            Some("1"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        record_audit_log(
            &mut storage,
            "user2",
            "CREATE",
            "gmp_documents",
            Some("2"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        record_audit_log(
            &mut storage,
            "user1",
            "DELETE",
            "gmp_document_contents",
            Some("1"),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // Filter by user
        let user1_logs = query_audit_logs(&storage, None, None, Some("user1"), None, None).unwrap();
        assert_eq!(user1_logs.len(), 2);

        // Filter by action
        let create_logs =
            query_audit_logs(&storage, None, None, None, Some("CREATE"), None).unwrap();
        assert_eq!(create_logs.len(), 2);

        // Filter by table
        let content_logs = query_audit_logs(
            &storage,
            None,
            None,
            None,
            None,
            Some("gmp_document_contents"),
        )
        .unwrap();
        assert_eq!(content_logs.len(), 1);
    }
}
