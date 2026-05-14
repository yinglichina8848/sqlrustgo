use crate::{ClientGtid, VectorClock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub op: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OperationType {
    Business(BusinessOperation),
    Sql(SqlOperation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessOperation {
    pub op_type: String,
    pub entity_type: String,
    pub entity_id: String,
    #[serde(default)]
    pub params: std::collections::HashMap<String, String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlOperation {
    pub sql: String,
    #[serde(default)]
    pub params: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub cgtid: ClientGtid,
    #[serde(default)]
    pub operations: Vec<Operation>,
    pub device_info: Option<String>,
    pub client_timestamp: Option<i64>,
}

impl SyncRequest {
    pub fn new(cgtid: ClientGtid, operations: Vec<Operation>) -> Self {
        Self {
            cgtid,
            operations,
            device_info: None,
            client_timestamp: None,
        }
    }

    pub fn with_device_info(mut self, device_info: impl Into<String>) -> Self {
        self.device_info = Some(device_info.into());
        self
    }

    pub fn with_client_timestamp(mut self, ts: i64) -> Self {
        self.client_timestamp = Some(ts);
        self
    }

    pub fn request_hash(&self) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use serde_json::to_string;

        let mut hasher = DefaultHasher::new();
        self.cgtid.hash(&mut hasher);
        if let Ok(json) = to_string(&self.operations) {
            json.hash(&mut hasher);
        }
        let hash = hasher.finish();
        hash.to_le_bytes().to_vec()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub result: ResponseResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseResult {
    Commit(CommitResult),
    Conflict(ConflictResult),
    Error(ErrorResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitResult {
    pub cgtid: ClientGtid,
    pub gtid: String,
    pub commit_timestamp: i64,
    pub response_blob: Option<Vec<u8>>,
    pub updated_clock: VectorClock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResult {
    pub cgtid: ClientGtid,
    pub transformed_ops: Vec<Operation>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResult {
    pub cgtid: ClientGtid,
    pub error_code: String,
    pub error_message: String,
    pub retryable: bool,
}
