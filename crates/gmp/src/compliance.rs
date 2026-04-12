//! GMP Compliance Checking
//!
//! Provides compliance checking functionality for GMP documents.
//! Checks document status, versioning, approval workflows, and other
//! GMP requirements.

use crate::audit::query_audit_logs;
use crate::document::{query_by_status, query_by_type, DocStatus};
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::StorageEngine;
use std::collections::HashMap;

/// Compliance rule types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceRule {
    DocumentStatus,
    VersionControl,
    ApprovalWorkflow,
    AuditTrail,
    EffectiveDate,
    DataIntegrity,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// A compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub doc_id: i64,
    pub rule: String,
    pub severity: String,
    pub description: String,
    pub detected_at: i64,
    pub remediation: String,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub violations: Vec<Violation>,
    pub checked_at: i64,
    pub documents_checked: i64,
    pub compliance_rate: f64,
}

impl ComplianceResult {
    pub fn new(violations: Vec<Violation>, documents_checked: i64) -> Self {
        let is_compliant = violations.is_empty();
        let compliance_rate = if documents_checked > 0 {
            ((documents_checked - violations.len() as i64) as f64 / documents_checked as f64)
                * 100.0
        } else {
            100.0
        };

        Self {
            is_compliant,
            violations,
            checked_at: current_timestamp(),
            documents_checked,
            compliance_rate,
        }
    }
}

/// Compliance check request parameters
#[derive(Debug, Clone)]
pub struct ComplianceCheckRequest {
    pub doc_type: Option<String>,
    pub check_audit_trail: bool,
    pub check_versioning: bool,
    pub check_approval: bool,
}

impl Default for ComplianceCheckRequest {
    fn default() -> Self {
        Self {
            doc_type: None,
            check_audit_trail: true,
            check_versioning: true,
            check_approval: true,
        }
    }
}

/// Check document compliance
pub fn check_document_compliance(
    storage: &dyn StorageEngine,
    doc_id: i64,
    request: &ComplianceCheckRequest,
) -> SqlResult<ComplianceResult> {
    let mut violations = Vec::new();
    let now = current_timestamp();

    // Get the document by scanning all statuses
    let doc = {
        let mut found_doc = None;
        for status in &[
            DocStatus::Draft,
            DocStatus::Active,
            DocStatus::Archived,
            DocStatus::Superseded,
        ] {
            let docs = query_by_status(storage, status)?;
            if let Some(d) = docs.into_iter().find(|d| d.id == doc_id) {
                found_doc = Some(d);
                break;
            }
        }
        found_doc
    };

    if let Some(doc) = doc {
        // Check 1: Document status
        if doc.status == DocStatus::Draft {
            violations.push(Violation {
                doc_id,
                rule: "DocumentStatus".to_string(),
                severity: Severity::Medium.as_str().to_string(),
                description: "Document is in DRAFT status and not approved".to_string(),
                detected_at: now,
                remediation: "Review and approve document or archive if obsolete".to_string(),
            });
        }

        // Check 2: Version control - documents should have reasonable version numbers
        if doc.version < 1 {
            violations.push(Violation {
                doc_id,
                rule: "VersionControl".to_string(),
                severity: Severity::High.as_str().to_string(),
                description: format!("Invalid version number: {}", doc.version),
                detected_at: now,
                remediation: "Reset version to 1 or higher".to_string(),
            });
        }

        // Check 3: Effective date
        if doc.effective_date > (now / 86400) as i32 {
            violations.push(Violation {
                doc_id,
                rule: "EffectiveDate".to_string(),
                severity: Severity::Medium.as_str().to_string(),
                description: format!(
                    "Document effective date ({}) is in the future",
                    doc.effective_date
                ),
                detected_at: now,
                remediation: "Verify effective date is correctly set".to_string(),
            });
        }

        // Check 4: Audit trail - if enabled
        if request.check_audit_trail {
            let logs = query_audit_logs(storage, None, Some(now), None, None, None)?;
            let has_logs = logs.iter().any(|l| {
                l.record_id
                    .as_ref()
                    .map(|rid| rid == &doc_id.to_string())
                    .unwrap_or(false)
            });

            if !has_logs && doc.status == DocStatus::Active {
                violations.push(Violation {
                    doc_id,
                    rule: "AuditTrail".to_string(),
                    severity: Severity::High.as_str().to_string(),
                    description: "Active document has no audit trail entries".to_string(),
                    detected_at: now,
                    remediation: "Document audit trail may be incomplete".to_string(),
                });
            }
        }

        // Check 5: Approval workflow - check if document was recently modified without update
        if request.check_approval && doc.status == DocStatus::Active {
            if let Some(last_update) = find_last_modification(storage, doc_id)? {
                let age_in_days = (now - last_update) / 86400;
                if age_in_days > 365 {
                    violations.push(Violation {
                        doc_id,
                        rule: "ApprovalWorkflow".to_string(),
                        severity: Severity::Low.as_str().to_string(),
                        description: format!(
                            "Active document not reviewed in {} days",
                            age_in_days
                        ),
                        detected_at: now,
                        remediation: "Schedule periodic review of active documents".to_string(),
                    });
                }
            }
        }
    } else {
        // Document not found - critical violation
        violations.push(Violation {
            doc_id,
            rule: "DataIntegrity".to_string(),
            severity: Severity::Critical.as_str().to_string(),
            description: "Document not found in system".to_string(),
            detected_at: now,
            remediation: "Verify document ID or restore from backup".to_string(),
        });
    }

    Ok(ComplianceResult::new(violations, 1))
}

/// Check compliance for multiple documents (batch)
pub fn check_batch_compliance(
    storage: &dyn StorageEngine,
    request: &ComplianceCheckRequest,
) -> SqlResult<ComplianceResult> {
    let mut all_violations = Vec::new();
    #[allow(unused_assignments)]
    let mut docs_checked = 0;

    // Get all documents of specified type, or all documents
    let docs = if let Some(ref doc_type) = request.doc_type {
        query_by_type(storage, doc_type)?
    } else {
        // Get all documents by scanning all statuses
        let mut all_docs = Vec::new();
        for status in &[
            DocStatus::Draft,
            DocStatus::Active,
            DocStatus::Archived,
            DocStatus::Superseded,
        ] {
            let status_docs = query_by_status(storage, status)?;
            all_docs.extend(status_docs);
        }
        all_docs
    };

    docs_checked = docs.len() as i64;

    for doc in docs {
        let result = check_document_compliance(storage, doc.id, request)?;
        all_violations.extend(result.violations);
    }

    Ok(ComplianceResult::new(all_violations, docs_checked))
}

/// Find the last modification timestamp for a document
fn find_last_modification(storage: &dyn StorageEngine, doc_id: i64) -> SqlResult<Option<i64>> {
    let logs = query_audit_logs(storage, None, None, None, None, None)?;
    let last_log = logs
        .into_iter()
        .filter(|log| {
            log.record_id
                .as_ref()
                .map(|rid| rid == &doc_id.to_string())
                .unwrap_or(false)
        })
        .max_by_key(|log| log.timestamp);

    Ok(last_log.map(|log| log.timestamp))
}

/// Get compliance summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_documents: i64,
    pub compliant_documents: i64,
    pub non_compliant_documents: i64,
    pub compliance_rate: f64,
    pub violations_by_rule: HashMap<String, i64>,
    pub violations_by_severity: HashMap<String, i64>,
}

pub fn get_compliance_summary(storage: &dyn StorageEngine) -> SqlResult<ComplianceSummary> {
    let request = ComplianceCheckRequest::default();
    let result = check_batch_compliance(storage, &request)?;

    let mut docs_by_status: HashMap<String, i64> = HashMap::new();
    for status in &[
        DocStatus::Draft,
        DocStatus::Active,
        DocStatus::Archived,
        DocStatus::Superseded,
    ] {
        let docs = query_by_status(storage, status)?;
        docs_by_status.insert(status.as_str().to_string(), docs.len() as i64);
    }

    let total_documents: i64 = docs_by_status.values().sum();
    let compliant_documents = total_documents - result.violations.len() as i64;
    let non_compliant_documents = result
        .violations
        .iter()
        .map(|v| v.doc_id)
        .collect::<std::collections::HashSet<_>>()
        .len() as i64;

    let violations_by_rule: HashMap<String, i64> =
        result.violations.iter().fold(HashMap::new(), |mut acc, v| {
            *acc.entry(v.rule.clone()).or_insert(0) += 1;
            acc
        });

    let violations_by_severity: HashMap<String, i64> =
        result.violations.iter().fold(HashMap::new(), |mut acc, v| {
            *acc.entry(v.severity.clone()).or_insert(0) += 1;
            acc
        });

    Ok(ComplianceSummary {
        total_documents,
        compliant_documents,
        non_compliant_documents,
        compliance_rate: result.compliance_rate,
        violations_by_rule,
        violations_by_severity,
    })
}

/// Get current timestamp
fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// SQL Result type alias
type SqlResult<T> = Result<T, sqlrustgo_types::SqlError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{create_audit_log_table, record_audit_log};
    use crate::document::{create_gmp_tables, insert_document, DocStatus, NewDocument};
    use sqlrustgo_storage::MemoryStorage;

    fn create_test_storage() -> impl StorageEngine {
        let storage = MemoryStorage::new();
        let mut s = storage;
        create_audit_log_table(&mut s).unwrap();
        create_gmp_tables(&mut s).unwrap();
        s
    }

    #[test]
    fn test_compliance_check_document_not_found() {
        let storage = create_test_storage();
        let request = ComplianceCheckRequest::default();

        let result = check_document_compliance(&storage, 9999, &request).unwrap();

        assert!(!result.is_compliant);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "DataIntegrity");
    }

    #[test]
    fn test_compliance_check_draft_document() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Insert a draft document
        let doc_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Test Draft",
                doc_type: "SOP",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: (now / 86400) as i32,
                status: DocStatus::Draft,
            },
        )
        .unwrap();

        let request = ComplianceCheckRequest::default();
        let result = check_document_compliance(&storage, doc_id, &request).unwrap();

        assert!(!result.is_compliant);
        // Should have at least the Draft status violation
        assert!(result.violations.iter().any(|v| v.rule == "DocumentStatus"));
    }

    #[test]
    fn test_compliance_check_active_document() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Insert an active document
        let doc_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Test Active",
                doc_type: "SOP",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: (now / 86400) as i32,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        // Add audit trail
        record_audit_log(
            &mut storage,
            "user1",
            "CREATE",
            "gmp_documents",
            Some(&doc_id.to_string()),
            None,
            Some(r#"{"title":"Test Active"}"#),
            None,
            None,
        )
        .unwrap();

        let request = ComplianceCheckRequest::default();
        let result = check_document_compliance(&storage, doc_id, &request).unwrap();

        // Should be compliant since it has audit trail and is Active
        assert!(result.is_compliant);
    }

    #[test]
    fn test_batch_compliance_check() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Insert multiple documents
        insert_document(
            &mut storage,
            NewDocument {
                title: "Doc 1",
                doc_type: "SOP",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: (now / 86400) as i32,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        insert_document(
            &mut storage,
            NewDocument {
                title: "Doc 2",
                doc_type: "SOP",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: (now / 86400) as i32,
                status: DocStatus::Draft,
            },
        )
        .unwrap();

        let request = ComplianceCheckRequest::default();
        let result = check_batch_compliance(&storage, &request).unwrap();

        assert_eq!(result.documents_checked, 2);
        // Should have at least one violation (the Draft document)
        assert!(!result.violations.is_empty());
    }
}
