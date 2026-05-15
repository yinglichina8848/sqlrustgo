use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileCollectionRecord {
    pub collection_id: String,
    pub device_id: String,
    pub correlation_id: Option<String>,
    pub data_hash: String,
    pub device_signature: String,
    pub trusted_timestamp: i64,
    pub collected_at: i64,
    pub status: CollectionStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollectionStatus {
    Pending,
    Verified,
    Invalid,
}

impl CollectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CollectionStatus::Pending => "PENDING",
            CollectionStatus::Verified => "VERIFIED",
            CollectionStatus::Invalid => "INVALID",
        }
    }

    pub fn parse_status(s: &str) -> Option<CollectionStatus> {
        match s.to_uppercase().as_str() {
            "PENDING" => Some(CollectionStatus::Pending),
            "VERIFIED" => Some(CollectionStatus::Verified),
            "INVALID" => Some(CollectionStatus::Invalid),
            _ => None,
        }
    }
}

impl MobileCollectionRecord {
    pub fn new(
        collection_id: String,
        device_id: String,
        data_hash: String,
        device_signature: String,
        trusted_timestamp: i64,
    ) -> Self {
        Self {
            collection_id,
            device_id,
            correlation_id: None,
            data_hash,
            device_signature,
            trusted_timestamp,
            collected_at: chrono_timestamp(),
            status: CollectionStatus::Pending,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct MobileCollection;

impl MobileCollection {
    pub const TABLE_NAME: &'static str = "gmp_mobile_collection_audit";
}
