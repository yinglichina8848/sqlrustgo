use serde::{Deserialize, Serialize};

use super::algorithms::SignatureAlgorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEntry {
    pub seq: u64,
    pub prev_hash: [u8; 32],
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub signature: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
    pub signer_id: String,
    pub certificate_path: Option<String>,
}

impl SignedAuditEntry {
    pub fn new(
        seq: u64,
        prev_hash: [u8; 32],
        timestamp: i64,
        user_id: String,
        action: String,
        table_name: String,
        signature: Vec<u8>,
        algorithm: SignatureAlgorithm,
        signer_id: String,
    ) -> Self {
        Self {
            seq,
            prev_hash,
            timestamp,
            user_id,
            action,
            table_name,
            record_id: None,
            old_value: None,
            new_value: None,
            signature,
            algorithm,
            signer_id,
            certificate_path: None,
        }
    }
}
