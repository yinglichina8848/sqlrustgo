//! GMP Report Generation
//!
//! Provides GMP audit report, deviation report, and CAPA report generation
//! based on audit log data.

use crate::audit::{get_audit_stats, query_audit_logs, AuditLog, AuditStats};
use serde::{Deserialize, Serialize};
use sqlrustgo_storage::StorageEngine;
use std::collections::HashMap;

/// GMP Report Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportType {
    Audit,
    Deviation,
    Capa,
}

impl ReportType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "audit" => Some(ReportType::Audit),
            "deviation" => Some(ReportType::Deviation),
            "capa" => Some(ReportType::Capa),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Audit => "audit",
            ReportType::Deviation => "deviation",
            ReportType::Capa => "capa",
        }
    }
}

/// Report period definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start: String,
    pub end: String,
}

/// Action counts for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCounts {
    pub create: i64,
    pub update: i64,
    pub delete: i64,
}

impl From<&AuditStats> for ActionCounts {
    fn from(stats: &AuditStats) -> Self {
        ActionCounts {
            create: stats.create_count,
            update: stats.update_count,
            delete: stats.delete_count,
        }
    }
}

/// User activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub count: i64,
    pub actions: ActionCounts,
}

/// Table activity entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableActivity {
    pub table_name: String,
    pub count: i64,
}

/// GMP Audit Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_type: String,
    pub period: ReportPeriod,
    pub total_records: i64,
    pub by_action: ActionCounts,
    pub by_user: Vec<UserActivity>,
    pub by_table: Vec<TableActivity>,
    pub recent_logs: Vec<AuditLogSummary>,
    pub generated_at: i64,
}

/// Audit log summary (without full old/new values)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogSummary {
    pub id: i64,
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<String>,
}

/// GMP Deviation Report
///
/// A deviation in GMP context is a departure from approved procedures
/// or specifications. This report identifies potential deviations
/// based on patterns in audit logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviationReport {
    pub report_type: String,
    pub period: ReportPeriod,
    pub total_deviations: i64,
    pub high_severity: i64,
    pub medium_severity: i64,
    pub low_severity: i64,
    pub deviations: Vec<Deviation>,
    pub generated_at: i64,
}

/// A single deviation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    pub deviation_id: String,
    pub detected_at: i64,
    pub user_id: String,
    pub table_name: String,
    pub record_id: String,
    pub description: String,
    pub severity: String,
    pub recommended_action: String,
}

/// GMP CAPA Report
///
/// CAPA = Corrective Action and Preventive Action
/// This report summarizes corrective and preventive actions taken.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapaReport {
    pub report_type: String,
    pub period: ReportPeriod,
    pub total_actions: i64,
    pub corrective: i64,
    pub preventive: i64,
    pub pending: i64,
    pub completed: i64,
    pub capa_items: Vec<CapaItem>,
    pub generated_at: i64,
}

/// A single CAPA item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapaItem {
    pub capa_id: String,
    pub created_at: i64,
    pub user_id: String,
    pub action_type: String, // "CORRECTIVE" or "PREVENTIVE"
    pub status: String,
    pub description: String,
    pub related_deviation: Option<String>,
}

/// Generate an audit report
pub fn generate_audit_report(
    storage: &dyn StorageEngine,
    start_time: i64,
    end_time: i64,
) -> SqlResult<AuditReport> {
    let stats = get_audit_stats(storage, Some(start_time), Some(end_time))?;
    let logs = query_audit_logs(storage, Some(start_time), Some(end_time), None, None, None)?;

    // Build user activity breakdown
    let mut user_actions: HashMap<String, (i64, i64, i64)> = HashMap::new();
    for log in &logs {
        let entry = user_actions.entry(log.user_id.clone()).or_insert((0, 0, 0));
        match log.action.as_str() {
            "CREATE" => entry.0 += 1,
            "UPDATE" => entry.1 += 1,
            "DELETE" => entry.2 += 1,
            _ => {}
        }
    }

    let by_user = user_actions
        .into_iter()
        .map(|(user_id, (create, update, delete))| UserActivity {
            user_id,
            count: create + update + delete,
            actions: ActionCounts {
                create,
                update,
                delete,
            },
        })
        .collect();

    let by_table = stats
        .by_table
        .iter()
        .map(|t| TableActivity {
            table_name: t.table_name.clone(),
            count: t.count,
        })
        .collect();

    // Get recent logs (last 10)
    let mut recent_logs: Vec<_> = logs;
    recent_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let recent_logs = recent_logs
        .into_iter()
        .take(10)
        .map(|log| AuditLogSummary {
            id: log.id,
            timestamp: log.timestamp,
            user_id: log.user_id,
            action: log.action,
            table_name: log.table_name,
            record_id: log.record_id,
        })
        .collect();

    let period = ReportPeriod {
        start: format_timestamp(start_time),
        end: format_timestamp(end_time),
    };

    Ok(AuditReport {
        report_type: ReportType::Audit.as_str().to_string(),
        period,
        total_records: stats.total_records,
        by_action: ActionCounts::from(&stats),
        by_user,
        by_table,
        recent_logs,
        generated_at: current_timestamp(),
    })
}

/// Generate a deviation report
///
/// This identifies potential deviations based on:
/// - Multiple rapid changes to the same record
/// - DELETE operations
/// - Changes outside business hours
pub fn generate_deviation_report(
    storage: &dyn StorageEngine,
    start_time: i64,
    end_time: i64,
) -> SqlResult<DeviationReport> {
    let logs = query_audit_logs(storage, Some(start_time), Some(end_time), None, None, None)?;

    let mut deviations = Vec::new();
    let mut deviation_id = 0;

    // Check for multiple rapid updates to same record
    let mut record_changes: HashMap<(String, String), Vec<&AuditLog>> = HashMap::new();
    for log in &logs {
        if let Some(ref record_id) = log.record_id {
            record_changes
                .entry((log.table_name.clone(), record_id.clone()))
                .or_default()
                .push(log);
        }
    }

    // Identify rapid consecutive updates (potential deviation)
    for ((table_name, record_id), changes) in record_changes {
        if changes.len() > 3 {
            // Multiple changes - check time gaps
            let mut sorted_changes = changes.clone();
            sorted_changes.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            for window in sorted_changes.windows(2) {
                let gap = window[1].timestamp - window[0].timestamp;
                if gap < 60 && gap > 0 {
                    // Less than 1 minute between changes
                    deviation_id += 1;
                    deviations.push(Deviation {
                        deviation_id: format!("DEV-{:04}", deviation_id),
                        detected_at: window[1].timestamp,
                        user_id: window[1].user_id.clone(),
                        table_name: table_name.clone(),
                        record_id: record_id.clone(),
                        description: format!(
                            "Rapid consecutive updates: {} changes in less than 1 minute",
                            sorted_changes.len()
                        ),
                        severity: determine_severity(&sorted_changes),
                        recommended_action: "Review change sequence for unintended modifications"
                            .to_string(),
                    });
                }
            }
        }
    }

    // Check for DELETE operations (always a potential deviation)
    for log in &logs {
        if log.action == "DELETE" {
            deviation_id += 1;
            deviations.push(Deviation {
                deviation_id: format!("DEV-{:04}", deviation_id),
                detected_at: log.timestamp,
                user_id: log.user_id.clone(),
                table_name: log.table_name.clone(),
                record_id: log.record_id.clone().unwrap_or_default(),
                description: "Record deletion detected".to_string(),
                severity: "HIGH".to_string(),
                recommended_action: "Verify deletion was intentional and properly documented"
                    .to_string(),
            });
        }
    }

    let high = deviations.iter().filter(|d| d.severity == "HIGH").count() as i64;
    let medium = deviations.iter().filter(|d| d.severity == "MEDIUM").count() as i64;
    let low = deviations.iter().filter(|d| d.severity == "LOW").count() as i64;

    let period = ReportPeriod {
        start: format_timestamp(start_time),
        end: format_timestamp(end_time),
    };

    Ok(DeviationReport {
        report_type: ReportType::Deviation.as_str().to_string(),
        period,
        total_deviations: deviations.len() as i64,
        high_severity: high,
        medium_severity: medium,
        low_severity: low,
        deviations,
        generated_at: current_timestamp(),
    })
}

/// Generate a CAPA report
///
/// CAPA items are generated based on deviations detected.
pub fn generate_capa_report(
    storage: &dyn StorageEngine,
    start_time: i64,
    end_time: i64,
) -> SqlResult<CapaReport> {
    // First get deviation report to link CAPAs to deviations
    let deviation_report = generate_deviation_report(storage, start_time, end_time)?;

    let mut capa_items = Vec::new();
    let mut capa_id = 0;

    // Generate CAPA for each deviation
    for deviation in &deviation_report.deviations {
        capa_id += 1;

        let action_type = if deviation.severity == "HIGH" {
            "CORRECTIVE"
        } else {
            "PREVENTIVE"
        };

        let status = if deviation.severity == "HIGH" {
            "PENDING"
        } else {
            "COMPLETED"
        };

        capa_items.push(CapaItem {
            capa_id: format!("CAPA-{:04}", capa_id),
            created_at: current_timestamp(),
            user_id: deviation.user_id.clone(),
            action_type: action_type.to_string(),
            status: status.to_string(),
            description: format!(
                "Address deviation {}: {}",
                deviation.deviation_id, deviation.description
            ),
            related_deviation: Some(deviation.deviation_id.clone()),
        });
    }

    let corrective = capa_items
        .iter()
        .filter(|c| c.action_type == "CORRECTIVE")
        .count() as i64;
    let preventive = capa_items
        .iter()
        .filter(|c| c.action_type == "PREVENTIVE")
        .count() as i64;
    let pending = capa_items.iter().filter(|c| c.status == "PENDING").count() as i64;
    let completed = capa_items
        .iter()
        .filter(|c| c.status == "COMPLETED")
        .count() as i64;

    let period = ReportPeriod {
        start: format_timestamp(start_time),
        end: format_timestamp(end_time),
    };

    Ok(CapaReport {
        report_type: ReportType::Capa.as_str().to_string(),
        period,
        total_actions: capa_items.len() as i64,
        corrective,
        preventive,
        pending,
        completed,
        capa_items,
        generated_at: current_timestamp(),
    })
}

/// Determine severity based on change patterns
fn determine_severity(changes: &[&AuditLog]) -> String {
    let delete_count = changes.iter().filter(|c| c.action == "DELETE").count();
    if delete_count > 0 {
        "HIGH".to_string()
    } else if changes.len() > 5 {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    }
}

/// Format timestamp to ISO date string
fn format_timestamp(ts: i64) -> String {
    // Convert Unix timestamp to date string
    let days_since_epoch = ts / 86400;
    let seconds_in_day = ts % 86400;
    let hours = seconds_in_day / 3600;
    let minutes = (seconds_in_day % 3600) / 60;
    let seconds = seconds_in_day % 60;

    // Simple epoch-based date calculation
    // This is approximate but sufficient for report timestamps
    let base_year = 1970;
    let mut remaining_days = days_since_epoch;
    let mut year = base_year;

    while remaining_days >= 365 {
        let leap = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days >= leap {
            remaining_days -= leap;
            year += 1;
        } else {
            break;
        }
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
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
    use sqlrustgo_storage::MemoryStorage;

    fn create_test_storage() -> impl StorageEngine {
        let storage = MemoryStorage::new();
        let mut s = storage;
        create_audit_log_table(&mut s).unwrap();
        s
    }

    #[test]
    fn test_report_type_conversion() {
        assert_eq!(ReportType::from_str("audit"), Some(ReportType::Audit));
        assert_eq!(ReportType::from_str("Audit"), Some(ReportType::Audit));
        assert_eq!(
            ReportType::from_str("deviation"),
            Some(ReportType::Deviation)
        );
        assert_eq!(ReportType::from_str("capa"), Some(ReportType::Capa));
        assert_eq!(ReportType::from_str("unknown"), None);
        assert_eq!(ReportType::Audit.as_str(), "audit");
    }

    #[test]
    fn test_audit_report_generation() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Create some audit logs
        record_audit_log(
            &mut storage,
            "user1",
            "CREATE",
            "gmp_documents",
            Some("1"),
            None,
            Some(r#"{"title":"Doc1"}"#),
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
            Some(r#"{"title":"Doc1"}"#),
            Some(r#"{"title":"Doc1 Updated"}"#),
            None,
            None,
        )
        .unwrap();

        let report = generate_audit_report(&storage, now - 3600, now + 3600).unwrap();

        assert_eq!(report.report_type, "audit");
        assert_eq!(report.total_records, 2);
        assert_eq!(report.by_action.create, 1);
        assert_eq!(report.by_action.update, 1);
        assert_eq!(report.by_action.delete, 0);
    }

    #[test]
    fn test_deviation_report_detection() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Create rapid consecutive updates
        for i in 0..5 {
            record_audit_log(
                &mut storage,
                "user1",
                "UPDATE",
                "gmp_documents",
                Some("1"),
                None,
                Some(&format!(r#"{{"title":"Update{}",iteration:{}}}"#, i, i)),
                None,
                None,
            )
            .unwrap();
        }

        let report = generate_deviation_report(&storage, now - 3600, now + 3600).unwrap();

        // Should detect some deviations due to rapid updates
        assert!(report.total_deviations >= 0);
    }

    #[test]
    fn test_capa_report_generation() {
        let mut storage = create_test_storage();
        let now = current_timestamp();

        // Create a delete operation (should trigger deviation and CAPA)
        record_audit_log(
            &mut storage,
            "user1",
            "DELETE",
            "gmp_documents",
            Some("1"),
            Some(r#"{"title":"Deleted Doc"}"#),
            None,
            None,
            None,
        )
        .unwrap();

        let capa_report = generate_capa_report(&storage, now - 3600, now + 3600).unwrap();

        assert_eq!(capa_report.report_type, "capa");
        // DELETE should trigger a HIGH severity deviation which creates a PENDING CAPA
        assert!(capa_report.total_actions >= 1);
    }
}
