use serde::{Deserialize, Serialize};
use sqlrustgo_gmp::audit_chain::{AuditChain, AuditChainError, GENESIS_PREV_HASH};
use sqlrustgo_gmp::audit_chain_tamper::{
    detect_tamper, quick_verify, verify_entry_checksum, TamperViolation,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerifyMode {
    Quick,
    Full,
    Entry,
    Incremental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChainVerifyReport {
    pub metadata: ReportMetadata,
    pub results: VerificationResults,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tamper_detection: Option<TamperDetectionResult>,
    pub chain_stats: ChainStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub version: String,
    pub mode: String,
    pub timestamp: u64,
    pub chain_path: String,
    pub duration_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResults {
    pub passed: bool,
    pub entries_verified: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_results: Option<Vec<EntryVerificationResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_order_violations: Option<Vec<TimestampViolation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orphan_entries: Option<Vec<OrphanInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_gaps: Option<Vec<WorkflowGap>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_gaps: Option<Vec<ProvenanceGap>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampViolation {
    pub seq: u64,
    pub timestamp: i64,
    pub prev_timestamp: i64,
    pub violation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanInfo {
    pub seq: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGap {
    pub seq: u64,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub gap_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceGap {
    pub seq: u64,
    pub missing_field: String,
    pub gap_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryVerificationResult {
    pub seq: u64,
    pub passed: bool,
    pub checksum_valid: bool,
    pub link_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperDetectionResult {
    pub detected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<TamperAlertInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperAlertInfo {
    pub id: u64,
    pub corrupted_seq: u64,
    pub violation_type: String,
    pub violation_details: serde_json::Value,
    pub detected_at: u64,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatistics {
    pub total_entries: u64,
    pub chain_length: u64,
    pub next_seq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_entry_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_entry_time: Option<i64>,
    pub genesis_hash: String,
    pub last_hash: String,
}

pub fn violation_to_json(violation: &TamperViolation) -> serde_json::Value {
    match violation {
        TamperViolation::ChecksumMismatch {
            seq,
            expected,
            actual,
        } => {
            serde_json::json!({
                "type": "ChecksumMismatch",
                "seq": seq,
                "expected": hex::encode(expected),
                "actual": hex::encode(actual)
            })
        }
        TamperViolation::ChainBroken {
            seq,
            expected_prev_hash,
            actual_prev_hash,
        } => {
            serde_json::json!({
                "type": "ChainBroken",
                "seq": seq,
                "expected_prev_hash": hex::encode(expected_prev_hash),
                "actual_prev_hash": hex::encode(actual_prev_hash)
            })
        }
        TamperViolation::SequenceGap { expected, actual } => {
            serde_json::json!({
                "type": "SequenceGap",
                "expected": expected,
                "actual": actual
            })
        }
        TamperViolation::GenesisTampered => {
            serde_json::json!({"type": "GenesisTampered"})
        }
        TamperViolation::EntryNotFound(seq) => {
            serde_json::json!({"type": "EntryNotFound", "seq": seq})
        }
    }
}

pub fn recovery_action_to_string(
    action: &sqlrustgo_gmp::audit_chain_tamper::RecoveryAction,
) -> String {
    match action {
        sqlrustgo_gmp::audit_chain_tamper::RecoveryAction::RecoverFromWal => {
            "RecoverFromWal".to_string()
        }
        sqlrustgo_gmp::audit_chain_tamper::RecoveryAction::ManualAudit => {
            "ManualAudit".to_string()
        }
        sqlrustgo_gmp::audit_chain_tamper::RecoveryAction::FreezeChain => {
            "FreezeChain".to_string()
        }
        sqlrustgo_gmp::audit_chain_tamper::RecoveryAction::NoRecovery => {
            "NoRecovery".to_string()
        }
    }
}

fn minimal_results(
    passed: bool,
    entries_verified: u64,
    error: Option<String>,
) -> VerificationResults {
    VerificationResults {
        passed,
        entries_verified,
        first_failure: None,
        error,
        entry_results: None,
        timestamp_order_violations: None,
        orphan_entries: None,
        workflow_gaps: None,
        provenance_gaps: None,
    }
}

fn run_quick_verify(chain: &AuditChain) -> VerificationResults {
    let passed = quick_verify(chain);
    minimal_results(
        passed,
        if passed { chain.len() } else { 0 },
        if passed {
            None
        } else {
            Some("Quick verify failed".to_string())
        },
    )
}

fn run_full_verify_extended(chain: &AuditChain) -> VerificationResults {
    // Empty chain is valid
    if chain.is_empty() {
        return minimal_results(true, 0, None);
    }

    // Step 1: Core chain verification
    let chain_valid = match chain.verify_chain() {
        Ok(v) => v,
        Err(e) => {
            let (first_failure, error_msg) = match &e {
                AuditChainError::SeqMismatch { expected, actual } => (
                    Some(*actual),
                    format!("SeqMismatch: expected {}, got {}", expected, actual),
                ),
                AuditChainError::HashMismatch { .. } => {
                    (None, "HashMismatch: chain broken".to_string())
                }
                AuditChainError::ChecksumInvalid { seq } => (
                    Some(*seq),
                    format!("ChecksumInvalid at seq {}", seq),
                ),
                AuditChainError::EmptyChain => (None, "EmptyChain".to_string()),
            };
            return VerificationResults {
                passed: false,
                entries_verified: 0,
                first_failure,
                error: Some(error_msg),
                entry_results: None,
                timestamp_order_violations: None,
                orphan_entries: None,
                workflow_gaps: None,
                provenance_gaps: None,
            };
        }
    };

    if !chain_valid {
        return minimal_results(
            false,
            chain.len(),
            Some("Chain verification returned false".to_string()),
        );
    }

    // Step 2: Timestamp ordering (monotonically non-decreasing)
    let entries = chain.entries();
    let mut timestamp_violations = Vec::new();
    for i in 1..entries.len() {
        let curr = &entries[i];
        let prev = &entries[i - 1];
        if curr.timestamp < prev.timestamp {
            timestamp_violations.push(TimestampViolation {
                seq: curr.seq,
                timestamp: curr.timestamp,
                prev_timestamp: prev.timestamp,
                violation_type: "TimestampDecrease".to_string(),
            });
        }
    }

    // Step 3: Orphan detection (sequence gaps)
    let mut orphan_entries = Vec::new();
    for i in 0..entries.len() {
        let entry = &entries[i];
        let expected_seq = (i + 1) as u64;
        if entry.seq != expected_seq {
            orphan_entries.push(OrphanInfo {
                seq: entry.seq,
                reason: format!(
                    "Sequence gap: expected seq {} at position {}, found seq {}",
                    expected_seq,
                    i,
                    entry.seq
                ),
            });
        }
    }

    // Step 4: Workflow gap detection
    let mut workflow_gaps = Vec::new();
    let mut last_action: Option<(String, String)> = None;
    for entry in entries {
        if let Some((prev_table, prev_action)) = last_action {
            let transition_key = format!("{}->{}", prev_action, entry.action);
            if transition_key == "DELETE->CREATE" && prev_table == entry.table_name {
                workflow_gaps.push(WorkflowGap {
                    seq: entry.seq,
                    from_state: Some(prev_action),
                    to_state: Some(entry.action.clone()),
                    gap_description: format!(
                        "Suspicious transition DELETE->CREATE on same table '{}'",
                        entry.table_name
                    ),
                });
            }
        }
        last_action = Some((entry.table_name.clone(), entry.action.clone()));
    }

    // Step 5: Provenance completeness
    let mut provenance_gaps = Vec::new();
    for entry in entries {
        if entry.user_id.is_empty() || entry.user_id == "<unknown>" {
            provenance_gaps.push(ProvenanceGap {
                seq: entry.seq,
                missing_field: "user_id".to_string(),
                gap_type: "EmptyUserId".to_string(),
            });
        }
        if entry.table_name.is_empty() {
            provenance_gaps.push(ProvenanceGap {
                seq: entry.seq,
                missing_field: "table_name".to_string(),
                gap_type: "EmptyTableName".to_string(),
            });
        }
        if entry.checksum == [0u8; 32] {
            provenance_gaps.push(ProvenanceGap {
                seq: entry.seq,
                missing_field: "checksum".to_string(),
                gap_type: "ZeroChecksum".to_string(),
            });
        }
    }

    let passed = chain_valid
        && timestamp_violations.is_empty()
        && orphan_entries.is_empty()
        && workflow_gaps.is_empty()
        && provenance_gaps.is_empty();

    VerificationResults {
        passed,
        entries_verified: chain.len(),
        first_failure: None,
        error: if passed {
            None
        } else {
            Some("Extended verification found issues".to_string())
        },
        entry_results: None,
        timestamp_order_violations: if timestamp_violations.is_empty() {
            None
        } else {
            Some(timestamp_violations)
        },
        orphan_entries: if orphan_entries.is_empty() {
            None
        } else {
            Some(orphan_entries)
        },
        workflow_gaps: if workflow_gaps.is_empty() {
            None
        } else {
            Some(workflow_gaps)
        },
        provenance_gaps: if provenance_gaps.is_empty() {
            None
        } else {
            Some(provenance_gaps)
        },
    }
}

fn run_full_verify(chain: &AuditChain) -> VerificationResults {
    run_full_verify_extended(chain)
}

fn run_entry_verify(chain: &AuditChain, seq: u64) -> VerificationResults {
    match chain.get_entry(seq) {
        Some(entry) => {
            let checksum_valid = verify_entry_checksum(entry).is_ok();
            let link_valid = if entry.seq == 1 {
                entry.prev_hash == GENESIS_PREV_HASH
            } else if let Some(prev) = chain.get_entry(entry.seq - 1) {
                entry.prev_hash == prev.checksum
            } else {
                false
            };
            let entry_results = vec![EntryVerificationResult {
                seq,
                passed: checksum_valid && link_valid,
                checksum_valid,
                link_valid,
                violation: if !checksum_valid {
                    Some("ChecksumInvalid".to_string())
                } else if !link_valid {
                    Some("LinkBroken".to_string())
                } else {
                    None
                },
            }];
            VerificationResults {
                passed: checksum_valid && link_valid,
                entries_verified: 1,
                first_failure: if checksum_valid && link_valid {
                    None
                } else {
                    Some(seq)
                },
                error: None,
                entry_results: Some(entry_results),
                timestamp_order_violations: None,
                orphan_entries: None,
                workflow_gaps: None,
                provenance_gaps: None,
            }
        }
        None => VerificationResults {
            passed: false,
            entries_verified: 0,
            first_failure: Some(seq),
            error: Some(format!("Entry {} not found", seq)),
            entry_results: None,
            timestamp_order_violations: None,
            orphan_entries: None,
            workflow_gaps: None,
            provenance_gaps: None,
        },
    }
}

fn run_tamper_detection(chain: &AuditChain) -> Option<TamperDetectionResult> {
    let alert = detect_tamper(chain);
    Some(TamperDetectionResult {
        detected: alert.is_some(),
        alert: alert.map(|a| TamperAlertInfo {
            id: a.id,
            corrupted_seq: a.corrupted_seq,
            violation_type: format!("{:?}", a.violation),
            violation_details: violation_to_json(&a.violation),
            detected_at: a.detected_at,
            recommended_action: recovery_action_to_string(&a.recommended_action),
        }),
    })
}

fn build_chain_stats(chain: &AuditChain) -> ChainStatistics {
    let state = chain.get_state();
    let first_entry_time = chain.get_entry(1).map(|e| e.timestamp);
    let last_entry_time = chain.entries().last().map(|e| e.timestamp);
    ChainStatistics {
        total_entries: chain.len(),
        chain_length: state.length,
        next_seq: state.next_seq,
        first_entry_time,
        last_entry_time,
        genesis_hash: hex::encode(GENESIS_PREV_HASH),
        last_hash: hex::encode(state.last_hash),
    }
}

pub fn load_chain_from_path(path: &PathBuf) -> Result<AuditChain, String> {
    use sqlrustgo_gmp::audit_chain_wal::AuditChainWalManager;

    let wal_path = if path.is_dir() {
        path.join("audit_chain.wal")
    } else {
        path.clone()
    };

    if !wal_path.exists() {
        return Ok(AuditChain::new());
    }

    let manager = AuditChainWalManager::new(wal_path);
    let (entries, _state) = manager
        .recover()
        .map_err(|e| format!("Failed to recover from WAL: {}", e))?;

    let mut chain = AuditChain::new();
    for entry in entries {
        chain.append(entry).map_err(|e| format!("Failed to append entry: {:?}", e))?;
    }

    Ok(chain)
}

pub fn run_verification(
    mode: &VerifyMode,
    chain_path: &PathBuf,
    entry_seq: Option<u64>,
) -> Result<AuditChainVerifyReport, String> {
    let start = SystemTime::now();
    let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let chain = load_chain_from_path(chain_path)?;

    let (results, tamper_detection) = match mode {
        VerifyMode::Quick => {
            let results = run_quick_verify(&chain);
            let tamper = run_tamper_detection(&chain);
            (results, tamper)
        }
        VerifyMode::Full => {
            let results = run_full_verify(&chain);
            let tamper = run_tamper_detection(&chain);
            (results, tamper)
        }
        VerifyMode::Entry => {
            let seq = entry_seq.ok_or("Entry mode requires --seq argument")?;
            let results = run_entry_verify(&chain, seq);
            (results, None)
        }
        VerifyMode::Incremental => {
            let results = run_full_verify(&chain);
            (results, None)
        }
    };

    let duration_us = start.elapsed().unwrap().as_micros() as u64;

    Ok(AuditChainVerifyReport {
        metadata: ReportMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            mode: format!("{:?}", mode),
            timestamp,
            chain_path: chain_path.to_string_lossy().to_string(),
            duration_us,
        },
        results,
        tamper_detection,
        chain_stats: build_chain_stats(&chain),
    })
}

pub fn write_report(
    report: &AuditChainVerifyReport,
    output_path: Option<&PathBuf>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize report: {}", e))?;

    match output_path {
        Some(path) => {
            std::fs::write(path, &json)
                .map_err(|e| format!("Failed to write report: {}", e))?;
        }
        None => {
            println!("{}", json);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_to_json_checksum_mismatch() {
        let violation = TamperViolation::ChecksumMismatch {
            seq: 1,
            expected: [0u8; 32],
            actual: [1u8; 32],
        };
        let json = violation_to_json(&violation);
        assert_eq!(json["type"], "ChecksumMismatch");
        assert_eq!(json["seq"], 1);
    }

    #[test]
    fn test_chain_statistics_empty() {
        let chain = AuditChain::new();
        let stats = build_chain_stats(&chain);
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.chain_length, 0);
        assert_eq!(stats.genesis_hash, hex::encode(GENESIS_PREV_HASH));
    }

    #[test]
    fn test_verification_results_serialization() {
        let results = minimal_results(true, 10, None);
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"entries_verified\":10"));
    }

    #[test]
    fn test_full_verify_empty_chain() {
        let chain = AuditChain::new();
        let results = run_full_verify(&chain);
        assert!(results.passed, "Empty chain should pass verification");
        assert_eq!(results.entries_verified, 0);
    }

    #[test]
    fn test_load_chain_from_nonexistent_path() {
        let path = PathBuf::from("/tmp/nonexistent_audit_chain_12345.wal");
        let chain = load_chain_from_path(&path).unwrap();
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_extended_results_json_has_violation_fields() {
        let results = VerificationResults {
            passed: true,
            entries_verified: 5,
            first_failure: None,
            error: None,
            entry_results: None,
            timestamp_order_violations: Some(vec![TimestampViolation {
                seq: 2,
                timestamp: 900,
                prev_timestamp: 1000,
                violation_type: "TimestampDecrease".to_string(),
            }]),
            orphan_entries: None,
            workflow_gaps: None,
            provenance_gaps: None,
        };
        let json = serde_json::to_string(&results).unwrap();
        assert!(json.contains("TimestampDecrease"));
        assert!(json.contains("\"seq\":2"));
    }

    #[test]
    fn test_quick_verify_empty_chain() {
        let chain = AuditChain::new();
        let results = run_quick_verify(&chain);
        assert!(results.passed);
        assert_eq!(results.entries_verified, 0);
    }
}
