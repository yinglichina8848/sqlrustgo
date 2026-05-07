// SQLRustGo executor module

pub mod audit_logger;
pub mod executor;
pub mod executor_metrics;
pub mod explain;
pub mod query_cache;
pub mod query_cache_config;
pub mod scan;
pub mod stored_proc;
pub mod trigger;
pub mod trigger_eval;

pub use audit_logger::{
    create_system_audit_log_table, get_all_audit_logs, get_audit_log_by_id, query_audit_logs,
    record_ddl_audit, record_delete_audit, record_insert_audit, record_update_audit,
    AuditAction, AuditLogEntry, AuditLogger, SYSTEM_AUDIT_LOG_TABLE,
};
pub use executor::{Executor, ExecutorResult};
pub use executor_metrics::ExecutorMetrics;
pub use explain::{explain, explain_analyze, ExplainConfig, ExplainLine, ExplainOutput};
