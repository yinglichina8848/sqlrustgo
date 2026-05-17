//! Evidence Export Package Module
//!
//! Exports GMP audit chains, evidence records, and compliance reports
//! as a signed JSON + PDF package.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// 包清单结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub version: String,
    pub created_at: i64,
    pub from_timestamp: i64,
    pub to_timestamp: i64,
    pub algorithm: String,
    pub files: Vec<FileEntry>,
}

/// 文件条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub filename: String,
    pub sha256: String,
}

/// 包路径
#[derive(Debug, Clone)]
pub struct PackagePath {
    pub root: std::path::PathBuf,
    pub manifest: std::path::PathBuf,
}

/// 导出错误
#[derive(Error, Debug)]
pub enum ExportError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("PDF error: {0}")]
    PdfError(String),
    #[error("Signature error: {0}")]
    SignatureError(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// 包验证报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub is_valid: bool,
    pub manifest_valid: bool,
    pub files_valid: Vec<bool>,
    pub signatures_valid: Vec<bool>,
    pub errors: Vec<String>,
}

/// JSON Exporter
pub struct JsonExporter;

impl JsonExporter {
    /// Export audit chain records to JSON bytes
    pub fn export_records(records: &[super::AuditChainRecord]) -> Result<Vec<u8>, ExportError> {
        serde_json::to_vec_pretty(records).map_err(ExportError::SerializationError)
    }

    /// Export evidence records to JSON bytes
    pub fn export_evidence(evidence: &[super::EvidenceRecord]) -> Result<Vec<u8>, ExportError> {
        serde_json::to_vec_pretty(evidence).map_err(ExportError::SerializationError)
    }

    /// Export proof to JSON bytes
    pub fn export_proof(proof: &super::types::Proof) -> Result<Vec<u8>, ExportError> {
        serde_json::to_vec_pretty(proof).map_err(ExportError::SerializationError)
    }
}
