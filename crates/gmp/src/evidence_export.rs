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

pub struct PackageBuilder {
    records: Vec<super::AuditChainRecord>,
    evidence: Vec<super::EvidenceRecord>,
    signer: Option<SignerEd25519>,
    from_timestamp: i64,
    to_timestamp: i64,
}

impl PackageBuilder {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            evidence: Vec::new(),
            signer: None,
            from_timestamp: 0,
            to_timestamp: 0,
        }
    }

    pub fn with_records(mut self, records: Vec<super::AuditChainRecord>) -> Self {
        self.records = records;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<super::EvidenceRecord>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_timestamp_range(mut self, from: i64, to: i64) -> Self {
        self.from_timestamp = from;
        self.to_timestamp = to;
        self
    }

    pub fn with_signer(mut self, signer: SignerEd25519) -> Self {
        self.signer = Some(signer);
        self
    }

    pub fn build(self, output_dir: &Path) -> Result<PackagePath, ExportError> {
        let signer = self.signer.unwrap_or_else(SignerEd25519::new);
        let public_key = signer.public_key();

        std::fs::create_dir_all(output_dir)?;

        let records_json = JsonExporter::export_records(&self.records)?;
        let evidence_json = JsonExporter::export_evidence(&self.evidence)?;

        let report_title = format!("GMP Compliance Report {} - {}", self.from_timestamp, self.to_timestamp);
        let pdf_bytes = PdfExporter::generate_compliance_report(&report_title, &self.records, &self.evidence)?;

        let manifest = PackageManifest {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            from_timestamp: self.from_timestamp,
            to_timestamp: self.to_timestamp,
            algorithm: "Ed25519".to_string(),
            files: vec![
                FileEntry { filename: "records.json".to_string(), sha256: Self::sha256(&records_json) },
                FileEntry { filename: "evidence.json".to_string(), sha256: Self::sha256(&evidence_json) },
                FileEntry { filename: "report.pdf".to_string(), sha256: Self::sha256(&pdf_bytes) },
            ],
        };

        let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(ExportError::SerializationError)?;

        let signed_data = [&manifest_json, &records_json, &evidence_json, &pdf_bytes].concat();
        let signature = signer.sign(&signed_data);

        std::fs::write(output_dir.join("manifest.json"), &manifest_json)?;
        std::fs::write(output_dir.join("records.json"), &records_json)?;
        std::fs::write(output_dir.join("evidence.json"), &evidence_json)?;
        std::fs::write(output_dir.join("report.pdf"), &pdf_bytes)?;
        std::fs::write(output_dir.join("signature.bin"), &signature)?;
        std::fs::write(output_dir.join("public_key.bin"), &public_key)?;

        Ok(PackagePath {
            root: output_dir.to_path_buf(),
            manifest: output_dir.join("manifest.json"),
        })
    }

    fn sha256(data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    pub fn verify(path: &Path) -> Result<VerificationReport, ExportError> {
        let manifest_bytes = std::fs::read(path.join("manifest.json"))?;
        let records_json = std::fs::read(path.join("records.json"))?;
        let evidence_json = std::fs::read(path.join("evidence.json"))?;
        let pdf_bytes = std::fs::read(path.join("report.pdf"))?;
        let signature = std::fs::read(path.join("signature.bin"))?;
        let public_key = std::fs::read(path.join("public_key.bin"))?;

        let manifest: PackageManifest = serde_json::from_slice(&manifest_bytes).map_err(ExportError::SerializationError)?;

        let signed_data = [&manifest_bytes, &records_json, &evidence_json, &pdf_bytes].concat();
        let sig_valid = verify_signature(&signed_data, &signature, &public_key)?;

        let manifest_valid = manifest.files.iter().all(|f| {
            let file_data = std::fs::read(path.join(&f.filename)).unwrap_or_default();
            Self::sha256(&file_data) == f.sha256
        });

        Ok(VerificationReport {
            is_valid: sig_valid && manifest_valid,
            manifest_valid,
            files_valid: vec![manifest_valid; manifest.files.len()],
            signatures_valid: vec![sig_valid],
            errors: if sig_valid && manifest_valid { vec![] } else { vec!["Verification failed".to_string()] },
        })
    }
}

impl Default for PackageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
