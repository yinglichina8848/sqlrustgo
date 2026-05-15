use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};


#[derive(Debug, Clone)]
pub enum CorrectionError {
    NoPreviousCorrection { record_id: String },
    ChainBroken { record_id: String, expected: String, actual: String },
    InvalidReason { reason: String },
    MissingApprover,
    SignatureInvalid { reason: String },
    RecordNotFound { record_id: String },
}

impl std::fmt::Display for CorrectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorrectionError::NoPreviousCorrection { record_id } => {
                write!(f, "No previous correction found for record: {}", record_id)
            }
            CorrectionError::ChainBroken { record_id, expected, actual } => {
                write!(f, "Correction chain broken for {}: expected {}, actual {}", record_id, expected, actual)
            }
            CorrectionError::InvalidReason { reason } => {
                write!(f, "Invalid correction reason: {}", reason)
            }
            CorrectionError::MissingApprover => {
                write!(f, "Missing approver for correction")
            }
            CorrectionError::SignatureInvalid { reason } => {
                write!(f, "Signature verification failed: {}", reason)
            }
            CorrectionError::RecordNotFound { record_id } => {
                write!(f, "Record not found: {}", record_id)
            }
        }
    }
}

impl std::error::Error for CorrectionError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionReason {
    pub code: String,
    pub explanation: String,
    pub reason_given_by: String,
}

impl CorrectionReason {
    pub fn new(code: &str, explanation: &str, reason_given_by: &str) -> Result<Self, CorrectionError> {
        if explanation.trim().is_empty() {
            return Err(CorrectionError::InvalidReason { reason: explanation.to_string() });
        }
        Ok(Self {
            code: code.to_string(),
            explanation: explanation.to_string(),
            reason_given_by: reason_given_by.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionEntry {
    pub correction_id: String,
    pub record_id: String,
    pub table_name: String,
    pub original_value: String,
    pub corrected_value: String,
    pub corrected_fields: Vec<String>,
    pub reason: CorrectionReason,
    pub approver_id: String,
    pub approver_signature: Option<String>,
    pub timestamp: i64,
    pub prev_correction_id: Option<String>,
    pub audit_entry_seq: u64,
    pub checksum: [u8; 32],
}

impl CorrectionEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        correction_id: String,
        record_id: String,
        table_name: String,
        original_value: String,
        corrected_value: String,
        corrected_fields: Vec<String>,
        reason: CorrectionReason,
        approver_id: String,
        approver_signature: Option<String>,
        timestamp: i64,
        prev_correction_id: Option<String>,
        audit_entry_seq: u64,
    ) -> Self {
        let mut entry = Self {
            correction_id: correction_id.clone(),
            record_id,
            table_name,
            original_value,
            corrected_value,
            corrected_fields,
            reason,
            approver_id,
            approver_signature,
            timestamp,
            prev_correction_id,
            audit_entry_seq,
            checksum: [0u8; 32],
        };
        entry.checksum = compute_correction_checksum(&entry);
        entry
    }

    pub fn verify_checksum(&self) -> bool {
        compute_correction_checksum(self) == self.checksum
    }
}

pub fn compute_correction_checksum(entry: &CorrectionEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(entry.correction_id.as_bytes());
    hasher.update(entry.record_id.as_bytes());
    hasher.update(entry.table_name.as_bytes());
    hasher.update(entry.original_value.as_bytes());
    hasher.update(entry.corrected_value.as_bytes());
    for field in &entry.corrected_fields {
        hasher.update(field.as_bytes());
    }
    hasher.update(entry.reason.code.as_bytes());
    hasher.update(entry.reason.explanation.as_bytes());
    hasher.update(entry.reason.reason_given_by.as_bytes());
    hasher.update(entry.approver_id.as_bytes());
    if let Some(ref sig) = entry.approver_signature {
        hasher.update(sig.as_bytes());
    }
    hasher.update(entry.timestamp.to_le_bytes());
    if let Some(ref prev_id) = entry.prev_correction_id {
        hasher.update(prev_id.as_bytes());
    }
    hasher.update(entry.audit_entry_seq.to_le_bytes());
    hasher.finalize().into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordCorrectionChain {
    pub record_id: String,
    pub table_name: String,
    corrections: Vec<CorrectionEntry>,
}

impl RecordCorrectionChain {
    pub fn new(record_id: String, table_name: String) -> Self {
        Self {
            record_id,
            table_name,
            corrections: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.corrections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.corrections.is_empty()
    }

    pub fn get_original_value(&self) -> Option<&str> {
        self.corrections.first().map(|e| e.original_value.as_str())
    }

    pub fn get_current_value(&self) -> Option<&str> {
        self.corrections.last().map(|e| e.corrected_value.as_str())
    }

    pub fn get_corrections(&self) -> &[CorrectionEntry] {
        &self.corrections
    }

    pub fn get_correction(&self, correction_id: &str) -> Option<&CorrectionEntry> {
        self.corrections.iter().find(|e| e.correction_id == correction_id)
    }

    pub fn add_correction(&mut self, correction: CorrectionEntry) -> Result<(), CorrectionError> {
        if let Some(prev) = self.corrections.last() {
            if correction.prev_correction_id.as_ref() != Some(&prev.correction_id) {
                return Err(CorrectionError::ChainBroken {
                    record_id: self.record_id.clone(),
                    expected: prev.correction_id.clone(),
                    actual: correction.prev_correction_id.unwrap_or_default(),
                });
            }
        } else if correction.prev_correction_id.is_some() {
            return Err(CorrectionError::NoPreviousCorrection {
                record_id: self.record_id.clone(),
            });
        }

        if !correction.verify_checksum() {
            return Err(CorrectionError::SignatureInvalid {
                reason: "Checksum verification failed".to_string(),
            });
        }

        self.corrections.push(correction);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CorrectionChain {
    chains: std::collections::HashMap<String, RecordCorrectionChain>,
}

impl CorrectionChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_chain(&mut self, record_id: &str, table_name: &str) -> &mut RecordCorrectionChain {
        let key = format!("{}:{}", table_name, record_id);
        if !self.chains.contains_key(&key) {
            self.chains.insert(key.clone(), RecordCorrectionChain::new(record_id.to_string(), table_name.to_string()));
        }
        self.chains.get_mut(&key).unwrap()
    }

    pub fn get_chain(&self, record_id: &str, table_name: &str) -> Option<&RecordCorrectionChain> {
        let key = format!("{}:{}", table_name, record_id);
        self.chains.get(&key)
    }

    pub fn total_corrections(&self) -> usize {
        self.chains.values().map(|c| c.len()).sum()
    }

    pub fn verify_all(&self) -> Result<(), CorrectionError> {
        for (key, chain) in &self.chains {
            for correction in chain.get_corrections() {
                if !correction.verify_checksum() {
                    return Err(CorrectionError::SignatureInvalid {
                        reason: format!("Checksum failed for correction {} in chain {}", correction.correction_id, key),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_reason_validation() {
        let valid = CorrectionReason::new("DATA_ENTRY_ERROR", "Typed wrong value", "operator-1");
        assert!(valid.is_ok());

        let invalid = CorrectionReason::new("DATA_ENTRY_ERROR", "", "operator-1");
        assert!(matches!(invalid, Err(CorrectionError::InvalidReason { .. })));
    }

    #[test]
    fn test_correction_entry_checksum() {
        let reason = CorrectionReason::new("CLARITY", "Initial entry unclear", "operator-1").unwrap();
        let entry = CorrectionEntry::new(
            "corr-1".to_string(),
            "record-1".to_string(),
            "batch_records".to_string(),
            r#"{"quantity": 100}"#.to_string(),
            r#"{"quantity": 1000}"#.to_string(),
            vec!["quantity".to_string()],
            reason,
            "approver-1".to_string(),
            None,
            1234567890,
            None,
            1,
        );

        assert!(entry.verify_checksum());
    }

    #[test]
    fn test_correction_chain_add_corrections() {
        let mut chain = RecordCorrectionChain::new("record-1".to_string(), "batch_records".to_string());

        let reason1 = CorrectionReason::new("DATA_ENTRY_ERROR", "Initial typo", "operator-1").unwrap();
        let entry1 = CorrectionEntry::new(
            "corr-1".to_string(),
            "record-1".to_string(),
            "batch_records".to_string(),
            r#"{"quantity": 100}"#.to_string(),
            r#"{"quantity": 1000}"#.to_string(),
            vec!["quantity".to_string()],
            reason1,
            "approver-1".to_string(),
            None,
            1234567890,
            None,
            1,
        );

        chain.add_correction(entry1).unwrap();

        let reason2 = CorrectionReason::new("CLARITY", "Needed more precision", "operator-2").unwrap();
        let entry2 = CorrectionEntry::new(
            "corr-2".to_string(),
            "record-1".to_string(),
            "batch_records".to_string(),
            r#"{"quantity": 1000}"#.to_string(),
            r#"{"quantity": 1000.5}"#.to_string(),
            vec!["quantity".to_string()],
            reason2,
            "approver-2".to_string(),
            None,
            1234567900,
            Some("corr-1".to_string()),
            2,
        );

        chain.add_correction(entry2).unwrap();

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.get_original_value(), Some(r#"{"quantity": 100}"#));
        assert_eq!(chain.get_current_value(), Some(r#"{"quantity": 1000.5}"#));
    }

    #[test]
    fn test_correction_chain_broken() {
        let mut chain = RecordCorrectionChain::new("record-1".to_string(), "batch_records".to_string());

        let reason1 = CorrectionReason::new("DATA_ENTRY_ERROR", "Initial typo", "operator-1").unwrap();
        let entry1 = CorrectionEntry::new(
            "corr-1".to_string(),
            "record-1".to_string(),
            "batch_records".to_string(),
            r#"{"quantity": 100}"#.to_string(),
            r#"{"quantity": 1000}"#.to_string(),
            vec!["quantity".to_string()],
            reason1,
            "approver-1".to_string(),
            None,
            1234567890,
            None,
            1,
        );

        chain.add_correction(entry1).unwrap();

        let reason2 = CorrectionReason::new("CLARITY", "Wrong chain", "operator-2").unwrap();
        let entry2 = CorrectionEntry::new(
            "corr-2".to_string(),
            "record-1".to_string(),
            "batch_records".to_string(),
            r#"{"quantity": 1000}"#.to_string(),
            r#"{"quantity": 1000.5}"#.to_string(),
            vec!["quantity".to_string()],
            reason2,
            "approver-2".to_string(),
            None,
            1234567900,
            Some("wrong-id".to_string()),
            2,
        );

        let result = chain.add_correction(entry2);
        assert!(matches!(result, Err(CorrectionError::ChainBroken { .. })));
    }
}
