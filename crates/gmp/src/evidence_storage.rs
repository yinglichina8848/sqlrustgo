use crate::evidence::EvidenceChain;
use serde_json;
use sqlrustgo_storage::{ColumnDefinition, StorageEngine, TableInfo};
use sqlrustgo_types::{SqlResult, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TABLE_EVIDENCE_RECORDS: &str = "gmp_evidence_records";

pub fn create_evidence_tables(storage: &mut dyn StorageEngine) -> SqlResult<()> {
    if !storage.has_table(TABLE_EVIDENCE_RECORDS) {
        let columns = vec![
            ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: true,
                auto_increment: true,
            },
            ColumnDefinition {
                name: "chain_id".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "description".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "nodes_json".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "integrity_hash".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "created_at".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
            ColumnDefinition {
                name: "updated_at".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                primary_key: false,
                auto_increment: false,
            },
        ];
        storage.create_table(&TableInfo {
            name: TABLE_EVIDENCE_RECORDS.to_string(),
            columns,
            foreign_keys: vec![],
            unique_constraints: vec![],
            check_constraints: vec![],
            partition_info: None,
            has_hidden_rowid: false,
            next_rowid: 1,
        })?;
    }
    Ok(())
}

pub fn save_evidence_chain(
    storage: &mut dyn StorageEngine,
    chain: &EvidenceChain,
) -> SqlResult<i64> {
    let nodes_json = serde_json::to_string(&chain.nodes).map_err(|e| {
        sqlrustgo_types::SqlError::ExecutionError(format!(
            "Failed to serialize evidence chain: {}",
            e
        ))
    })?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let row = vec![
        Value::Integer(0),
        Value::Text(chain.chain_id.clone()),
        Value::Text(chain.description.clone()),
        Value::Text(nodes_json),
        Value::Text(chain.integrity_hash.clone()),
        Value::Integer(now),
        Value::Integer(now),
    ];

    storage.insert(TABLE_EVIDENCE_RECORDS, vec![row])?;
    Ok(1)
}

pub fn load_evidence_chain(
    storage: &dyn StorageEngine,
    chain_id: &str,
) -> SqlResult<Option<EvidenceChain>> {
    let rows = storage.scan(TABLE_EVIDENCE_RECORDS)?;
    for row in rows {
        if row
            .get(1)
            .map(|v| {
                if let Value::Text(s) = v {
                    s == chain_id
                } else {
                    false
                }
            })
            .unwrap_or(false)
        {
            return Ok(reconstruct_chain_from_row(&row));
        }
    }
    Ok(None)
}

fn reconstruct_chain_from_row(row: &[Value]) -> Option<EvidenceChain> {
    let chain_id = match &row[1] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };
    let description = match &row[2] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };
    let nodes_json = match &row[3] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };
    let integrity_hash = match &row[4] {
        Value::Text(s) => s.clone(),
        _ => return None,
    };

    let nodes: Vec<crate::evidence::EvidenceNode> = serde_json::from_str(&nodes_json).ok()?;

    Some(crate::evidence::EvidenceChain {
        chain_id,
        description,
        nodes,
        integrity_hash,
        created_at: 0,
        updated_at: 0,
    })
}

pub fn get_evidence_by_chain_id(
    storage: &dyn StorageEngine,
    chain_id: &str,
) -> SqlResult<Vec<EvidenceChain>> {
    let rows = storage.scan(TABLE_EVIDENCE_RECORDS)?;
    let mut results = Vec::new();
    for row in rows {
        if row
            .get(1)
            .map(|v| {
                if let Value::Text(s) = v {
                    s == chain_id
                } else {
                    false
                }
            })
            .unwrap_or(false)
        {
            if let Some(chain) = reconstruct_chain_from_row(&row) {
                results.push(chain);
            }
        }
    }
    Ok(results)
}

pub fn get_evidence_by_time_range(
    storage: &dyn StorageEngine,
    from_timestamp: i64,
    to_timestamp: i64,
) -> SqlResult<Vec<EvidenceChain>> {
    let rows = storage.scan(TABLE_EVIDENCE_RECORDS)?;
    let mut results = Vec::new();
    for row in rows {
        let timestamp = match &row[5] {
            Value::Integer(n) => *n,
            _ => continue,
        };
        if timestamp >= from_timestamp && timestamp <= to_timestamp {
            if let Some(chain) = reconstruct_chain_from_row(&row) {
                results.push(chain);
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceChain;
    use crate::immutable_record::ImmutableRecord;

    #[test]
    fn test_create_evidence_table_schema() {
        let columns = vec![ColumnDefinition {
            name: "id".to_string(),
            data_type: "INTEGER".to_string(),
            nullable: false,
            primary_key: true,
            auto_increment: true,
        }];
        assert_eq!(columns[0].name, "id");
        assert_eq!(columns[0].primary_key, true);
    }

    #[test]
    fn test_reconstruct_chain() {
        let record = ImmutableRecord::new("test-chain", "Test", "Content");
        let chain = record.chain().clone();
        let nodes_json = serde_json::to_string(&chain.nodes).unwrap();
        assert!(!nodes_json.is_empty());
    }
}
