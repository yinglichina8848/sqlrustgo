use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TABLE_PROVENANCE_RECORDS: &str = "gmp_provenance_records";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub id: String,
    pub record_id: String,
    pub source_type: SourceType,
    pub source_id: Option<String>,
    pub creator_id: String,
    pub create_time: i64,
    pub operation_type: OperationType,
    pub lineage_path: Vec<String>,
    pub checksum: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    Insert,
    Import,
    Update,
    Delete,
    Derived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Create,
    Modify,
    Delete,
    Import,
    Export,
}

impl ProvenanceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        record_id: String,
        source_type: SourceType,
        source_id: Option<String>,
        creator_id: String,
        create_time: i64,
        operation_type: OperationType,
        lineage_path: Vec<String>,
    ) -> Self {
        let id = uuid_simple();
        let mut record = Self {
            id: id.clone(),
            record_id,
            source_type,
            source_id,
            creator_id,
            create_time,
            operation_type,
            lineage_path,
            checksum: [0u8; 32],
        };
        record.checksum = compute_checksum(&record);
        record
    }

    pub fn verify_integrity(&self) -> bool {
        self.checksum == compute_checksum(self)
    }
}

pub fn compute_checksum(record: &ProvenanceRecord) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(record.id.as_bytes());
    hasher.update(record.record_id.as_bytes());
    hasher.update(format!("{:?}", record.source_type).as_bytes());
    if let Some(ref sid) = record.source_id {
        hasher.update(sid.as_bytes());
    }
    hasher.update(record.creator_id.as_bytes());
    hasher.update(record.create_time.to_le_bytes());
    hasher.update(format!("{:?}", record.operation_type).as_bytes());
    for pid in &record.lineage_path {
        hasher.update(pid.as_bytes());
    }
    hasher.finalize().into()
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("prov_{:x}", now)
}

pub const CREATE_PROVENANCE_RECORDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_provenance_records (
    id TEXT PRIMARY KEY,
    record_id TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_id TEXT,
    creator_id TEXT NOT NULL,
    create_time INTEGER NOT NULL,
    operation_type TEXT NOT NULL,
    lineage_path TEXT NOT NULL,
    checksum BLOB NOT NULL,
    created_at INTEGER DEFAULT (strftime('%s', 'now'))
)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_record_creation() {
        let record = ProvenanceRecord::new(
            "record_123".to_string(),
            SourceType::Insert,
            None,
            "user_001".to_string(),
            1234567890,
            OperationType::Create,
            vec![],
        );

        assert!(record.id.starts_with("prov_"));
        assert_eq!(record.record_id, "record_123");
        assert!(record.verify_integrity());
    }

    #[test]
    fn test_provenance_record_with_lineage() {
        let record = ProvenanceRecord::new(
            "record_derived".to_string(),
            SourceType::Derived,
            Some("batch_001".to_string()),
            "system".to_string(),
            1234567890,
            OperationType::Create,
            vec!["rec_001".to_string(), "rec_002".to_string()],
        );

        assert_eq!(record.lineage_path.len(), 2);
        assert!(record.verify_integrity());
    }

    #[test]
    fn test_provenance_integrity_failure() {
        let mut record = ProvenanceRecord::new(
            "record_123".to_string(),
            SourceType::Insert,
            None,
            "user_001".to_string(),
            1234567890,
            OperationType::Create,
            vec![],
        );

        assert!(record.verify_integrity());
        record.creator_id = "hacker".to_string();
        assert!(!record.verify_integrity());
    }
}
