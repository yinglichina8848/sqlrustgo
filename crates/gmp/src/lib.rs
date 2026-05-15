//! GMP Document Retrieval Extension for SQLRustGo
//!
//! This crate provides document management and vector-based similarity search
//! capabilities for SQLRustGo databases.
//!
//! # Features
//!
//! - **Document Management**: Store and version documents with typed metadata
//! - **Vector Embeddings**: Hash-based text embedding generation
//! - **Similarity Search**: Find documents by semantic similarity
//! - **Hybrid Search**: Combine text matching with vector similarity
//! - **Audit Logging**: GMP-compliant audit trail with tamper-evident checksums
//! - **Report Generation**: Audit, deviation, and CAPA reports
//! - **Compliance Checking**: GMP document compliance verification
//!
//! # Quick Start
//!
//! ```ignore
//! use sqlrustgo_gmp::{GmpExecutor, create_gmp_tables};
//! use sqlrustgo_storage::MemoryStorage;
//! use std::sync::{Arc, RwLock};
//!
//! let storage = Arc::new(RwLock::new(MemoryStorage::new()));
//! let executor = GmpExecutor::new(storage);
//!
//! executor.init().unwrap();
//!
//! let doc_id = executor
//!     .import_document(
//!         "My Document",
//!         "REPORT",
//!         "Document content here...",
//!         &["keyword1", "keyword2"],
//!     )
//!     .unwrap();
//!
//! let results = executor.search("similar content", 5).unwrap();
//! ```
//!
//! # Tables
//!
//! The GMP extension creates the following tables:
//! - `gmp_documents`: Document metadata (id, title, type, version, dates, status)
//! - `gmp_document_contents`: Document section content
//! - `gmp_document_keywords`: Document keywords for text search
//! - `gmp_embeddings`: Vector embeddings for similarity search
//! - `gmp_audit_log`: Audit trail for all GMP operations

pub mod audit;
pub mod audit_chain;
pub mod audit_chain_tamper;
pub mod audit_chain_wal;
pub mod compliance;
pub mod correction;
pub mod correction_chain;
pub mod document;
pub mod electronic_signature;
pub mod embedding;
pub mod hsm;
pub mod persist_sqlite;
pub mod report;
pub mod scenarios;
pub mod semantic_embedding;
pub mod sql_api;
pub mod vector_search;

// Re-export commonly used types
pub use audit::{
    create_audit_log_table, get_all_audit_logs, get_audit_log_by_id, get_audit_stats,
    query_audit_logs, record_audit_log, AuditAction, AuditLog, AuditStats, TableCount, UserCount,
    TABLE_AUDIT_LOG,
};

pub use audit_chain::{
    AuditChain, AuditChainEntry, AuditChainError, AuditChainState, GENESIS_PREV_HASH,
};

pub use audit_chain_wal::{
    compute_entry_checksum, AuditChainWalEntry, AuditChainWalEntryType, AuditChainWalManager,
    AuditChainWalReader, AuditChainWalWriter,
};

pub use audit_chain_tamper::{
    detect_tamper, incremental_verify, quick_verify, verify_entry_checksum, verify_entry_link,
    RecoveryAction, TamperAlert, TamperViolation, VerificationResult,
};

pub use compliance::{
    check_batch_compliance, check_document_compliance, get_compliance_summary,
    ComplianceCheckRequest, ComplianceResult, ComplianceRule, ComplianceSummary, Severity,
    Violation,
};

pub use correction::{
    CorrectionRecord, CREATE_CORRECTION_RECORDS_TABLE, TABLE_CORRECTION_RECORDS,
};

pub use correction_chain::{CorrectionChain, CorrectionChainEntry};

pub use document::{
    create_gmp_tables, get_content, get_keywords, insert_document, insert_document_content,
    insert_document_keyword, query_by_effective_date, query_by_status, query_by_type, DocStatus,
    Document, DocumentContent, DocumentKeyword, TABLE_DOCUMENTS, TABLE_DOCUMENT_CONTENTS,
    TABLE_DOCUMENT_KEYWORDS,
};

pub use embedding::{
    cosine_similarity, euclidean_distance, generate_embedding, DocumentEmbedding, EmbeddingModel,
    HashEmbeddingModel, CREATE_EMBEDDINGS_TABLE, DEFAULT_MODEL, EMBEDDING_DIM, TABLE_EMBEDDINGS,
};

pub use vector_search::{
    create_embeddings_table, get_all_embeddings, hybrid_search, upsert_embedding, vector_search,
    vector_search_active, SearchResult,
};

pub use report::{
    generate_audit_report, generate_capa_report, generate_deviation_report, ActionCounts,
    AuditLogSummary, AuditReport, CapaItem, CapaReport, Deviation, DeviationReport, ReportPeriod,
    ReportType, TableActivity, UserActivity,
};

pub use semantic_embedding::{
    EmbeddingProvider, EmbeddingProviderConfig, HashConfig, OllamaConfig, OpenAIConfig,
    ProviderFactory,
};

pub use electronic_signature::{
    sql as e_signature_sql, ApprovalPolicy, ApprovalPolicyEvaluator, ElectronicSignature,
    ElectronicSignatureProvider, PolicyEvaluation, PolicyStatus, SignatureError, SignatureRequest,
    SystemTimeProvider, TrustedTimestampProvider,
    CREATE_APPROVAL_POLICIES_TABLE, CREATE_ELECTRONIC_SIGNATURES_TABLE,
    CREATE_SIGNATURE_REQUESTS_TABLE,
};

pub use hsm::{
    HsmConfig, HsmError, HsmProvider, HsmProviderType,
};

pub use hsm::software_tpm::SoftwareTpmProvider;

pub use hsm::software_tpm::create_provider;

pub use sql_api::{sql, GmpExecutor};
