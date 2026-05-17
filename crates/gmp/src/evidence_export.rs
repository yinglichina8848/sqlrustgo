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

/// PDF Exporter using printpdf
pub struct PdfExporter;

impl PdfExporter {
    /// Generate a compliance report PDF from audit chain summary
    pub fn generate_compliance_report(
        title: &str,
        records: &[super::AuditChainRecord],
        evidence: &[super::EvidenceRecord],
    ) -> Result<Vec<u8>, ExportError> {
        use printpdf::*;

        let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| ExportError::PdfError(e.to_string()))?;

        current_layer.use_text(title.to_string(), 24.0, Mm(20.0), Mm(277.0), &font);

        let mut y_pos = 260.0;
        current_layer.use_text("Audit Chain Records", 16.0, Mm(20.0), Mm(y_pos), &font);
        y_pos -= 10.0;

        for record in records.iter().take(20) {
            if y_pos < 40.0 { break; }
            let text = format!("- {}: block {} hash={}", record.action, record.block_height, &record.hash[..8]);
            current_layer.use_text(text, 10.0, Mm(25.0), Mm(y_pos), &font);
            y_pos -= 7.0;
        }

        y_pos -= 10.0;
        current_layer.use_text("Evidence Records", 16.0, Mm(20.0), Mm(y_pos), &font);
        y_pos -= 10.0;

        for ev in evidence.iter().take(20) {
            if y_pos < 40.0 { break; }
            let text = format!("- {}: {}", ev.operation, &ev.hash[..8]);
            current_layer.use_text(text, 10.0, Mm(25.0), Mm(y_pos), &font);
            y_pos -= 7.0;
        }

        let mut bytes = Vec::new();
        doc.save(&mut bytes).map_err(|e| ExportError::PdfError(e.to_string()))?;
        Ok(bytes)
    }
}

use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;

pub struct SignerEd25519 {
    signing_key: SigningKey,
}

impl SignerEd25519 {
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self { signing_key }
    }

    pub fn from_secret_key(secret_key: &[u8; 32]) -> Result<Self, ExportError> {
        let signing_key = SigningKey::from_bytes(secret_key);
        Ok(Self { signing_key })
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(data);
        signature.to_bytes().to_vec()
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }
}

impl Default for SignerEd25519 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn verify_signature(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool, ExportError> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    if signature.len() != 64 {
        return Err(ExportError::SignatureError("Invalid signature length".to_string()));
    }

    let signature = Signature::from_bytes(signature.try_into().map_err(|_| ExportError::SignatureError("Signature parse error".to_string()))?);
    let verifying_key = VerifyingKey::from_bytes(public_key.try_into().map_err(|_| ExportError::SignatureError("Public key parse error".to_string()))?)
        .map_err(|e| ExportError::SignatureError(e.to_string()))?;

    Ok(verifying_key.verify(data, &signature).is_ok())
}
