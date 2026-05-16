use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub operator_id: String,
    pub sop_id: String,
    pub completed_at: i64,
    pub expiry_date: Option<i64>,
    pub certificate_id: String,
}

impl TrainingRecord {
    pub fn new(operator_id: &str, sop_id: &str, certificate_id: &str) -> Self {
        Self {
            operator_id: operator_id.to_string(),
            sop_id: sop_id.to_string(),
            completed_at: current_timestamp(),
            expiry_date: None,
            certificate_id: certificate_id.to_string(),
        }
    }

    pub fn with_expiry(mut self, expiry_date: i64) -> Self {
        self.expiry_date = Some(expiry_date);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry_date {
            current_timestamp() > expiry
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmpOperation {
    pub operation_id: String,
    pub operation_type: String,
    pub required_sops: Vec<String>,
    pub requires_training_verification: bool,
}

impl GmpOperation {
    pub fn new(operation_id: &str, operation_type: &str, required_sops: Vec<String>) -> Self {
        Self {
            operation_id: operation_id.to_string(),
            operation_type: operation_type.to_string(),
            required_sops,
            requires_training_verification: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingVerificationResult {
    pub operator_id: String,
    pub operation_id: String,
    pub is_verified: bool,
    pub missing_sops: Vec<String>,
    pub expired_sops: Vec<String>,
}

pub struct SopTrainingBinding {
    training_records: HashMap<String, Vec<TrainingRecord>>,
    operations: HashMap<String, GmpOperation>,
}

impl SopTrainingBinding {
    pub fn new() -> Self {
        Self {
            training_records: HashMap::new(),
            operations: HashMap::new(),
        }
    }

    pub fn record_training(
        &mut self,
        operator_id: &str,
        sop_id: &str,
        certificate_id: &str,
    ) -> TrainingRecord {
        let record = TrainingRecord::new(operator_id, sop_id, certificate_id);
        let key = format!("{}:{}", operator_id, sop_id);
        self.training_records
            .entry(key)
            .or_default()
            .push(record.clone());
        record
    }

    pub fn register_operation(&mut self, operation: GmpOperation) {
        self.operations
            .insert(operation.operation_id.clone(), operation);
    }

    pub fn verify_training(
        &self,
        operator_id: &str,
        operation_id: &str,
    ) -> TrainingVerificationResult {
        let operation = match self.operations.get(operation_id) {
            Some(op) => op,
            None => {
                return TrainingVerificationResult {
                    operator_id: operator_id.to_string(),
                    operation_id: operation_id.to_string(),
                    is_verified: false,
                    missing_sops: vec![],
                    expired_sops: vec![],
                };
            }
        };

        let mut missing_sops = Vec::new();
        let mut expired_sops = Vec::new();

        for sop_id in &operation.required_sops {
            let key = format!("{}:{}", operator_id, sop_id);
            match self.training_records.get(&key) {
                Some(records) => {
                    let valid_record = records.iter().any(|r| !r.is_expired());
                    if !valid_record {
                        expired_sops.push(sop_id.clone());
                    }
                }
                None => {
                    missing_sops.push(sop_id.clone());
                }
            }
        }

        let is_verified = missing_sops.is_empty() && expired_sops.is_empty();

        TrainingVerificationResult {
            operator_id: operator_id.to_string(),
            operation_id: operation_id.to_string(),
            is_verified,
            missing_sops,
            expired_sops,
        }
    }

    pub fn operator_has_training(&self, operator_id: &str, sop_id: &str) -> bool {
        let key = format!("{}:{}", operator_id, sop_id);
        self.training_records
            .get(&key)
            .map(|records| records.iter().any(|r| !r.is_expired()))
            .unwrap_or(false)
    }

    pub fn get_operator_sops(&self, operator_id: &str) -> Vec<&TrainingRecord> {
        self.training_records
            .values()
            .flat_map(|records| records.iter())
            .filter(|r| r.operator_id == operator_id)
            .collect()
    }
}

impl Default for SopTrainingBinding {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_training() {
        let mut binding = SopTrainingBinding::new();
        let record = binding.record_training("op-001", "SOP-001", "cert-123");
        assert_eq!(record.operator_id, "op-001");
        assert_eq!(record.sop_id, "SOP-001");
    }

    #[test]
    fn test_verify_training_success() {
        let mut binding = SopTrainingBinding::new();
        binding.record_training("op-001", "SOP-001", "cert-123");
        binding.register_operation(GmpOperation::new(
            "op-1",
            "PRODUCTION",
            vec!["SOP-001".to_string()],
        ));
        let result = binding.verify_training("op-001", "op-1");
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_training_missing_sop() {
        let mut binding = SopTrainingBinding::new();
        binding.record_training("op-001", "SOP-001", "cert-123");
        binding.register_operation(GmpOperation::new(
            "op-1",
            "PRODUCTION",
            vec!["SOP-001".to_string(), "SOP-002".to_string()],
        ));
        let result = binding.verify_training("op-001", "op-1");
        assert!(!result.is_verified);
        assert!(result.missing_sops.contains(&"SOP-002".to_string()));
    }

    #[test]
    fn test_training_expiry() {
        let mut binding = SopTrainingBinding::new();
        let mut record = TrainingRecord::new("op-001", "SOP-001", "cert-123");
        record.expiry_date = Some(current_timestamp() - 86400);
        let key = "op-001:SOP-001".to_string();
        binding.training_records.insert(key, vec![record]);
        let result = binding.verify_training("op-001", "op-1");
        assert!(!result.is_verified);
    }

    #[test]
    fn test_operator_has_training() {
        let mut binding = SopTrainingBinding::new();
        binding.record_training("op-001", "SOP-001", "cert-123");
        assert!(binding.operator_has_training("op-001", "SOP-001"));
        assert!(!binding.operator_has_training("op-001", "SOP-002"));
    }
}
