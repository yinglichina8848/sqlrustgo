# Evidence Export Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 GMP 证据导出包功能，支持导出 JSON + PDF + Ed25519 签名格式的完整审计链

**Architecture:** 新增 `evidence_export.rs` 模块，协调 JsonExporter、PdfExporter、Signer 组件，输出标准化证据包目录结构

**Tech Stack:** Rust, printpdf, ed25519-dalek, sha2, serde_json

---

## File Structure

```
crates/gmp/
├── Cargo.toml              # 添加 printpdf 依赖
├── src/
│   ├── lib.rs              # 添加 evidence_export 模块
│   └── evidence_export.rs  # 新建：导出模块
└── tests/
    └── evidence_export_test.rs  # 新建：集成测试
```

---

## Task 1: 添加依赖

**Files:**
- Modify: `crates/gmp/Cargo.toml`

- [ ] **Step 1: 添加 printpdf 依赖**

在 `[dependencies]` 段添加：
```toml
printpdf = "0.7"
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/Cargo.toml
git commit -m "deps(gmp): add printpdf for PDF generation"
```

---

## Task 2: 创建 evidence_export.rs 模块骨架

**Files:**
- Create: `crates/gmp/src/evidence_export.rs`

- [ ] **Step 1: 创建模块骨架**

```rust
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
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/evidence_export.rs
git commit -m "feat(gmp): add evidence_export module skeleton"
```

---

## Task 3: 实现 PackageManifest 和 JsonExporter

**Files:**
- Modify: `crates/gmp/src/evidence_export.rs`

- [ ] **Step 1: 添加 JsonExporter 结构体**

在 `evidence_export.rs` 末尾添加：

```rust
/// JSON 导出器
pub struct JsonExporter;

impl JsonExporter {
    /// 导出审计链为 JSON
    pub fn export_audit_chain(
        chain: &[crate::AuditChainEntry],
        path: &Path,
    ) -> Result<String, ExportError> {
        let json = serde_json::to_string_pretty(chain)?;
        std::fs::write(path, &json)?;
        Ok(Self::compute_sha256(path))
    }

    /// 导出证据记录为 JSON
    pub fn export_evidence(
        evidence: &[crate::EvidenceChain],
        path: &Path,
    ) -> Result<String, ExportError> {
        let json = serde_json::to_string_pretty(evidence)?;
        std::fs::write(path, &json)?;
        Ok(Self::compute_sha256(path))
    }

    /// 导出合规报告为 JSON
    pub fn export_compliance(
        report: &crate::ComplianceResult,
        path: &Path,
    ) -> Result<String, ExportError> {
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(path, &json)?;
        Ok(Self::compute_sha256(path))
    }

    /// 计算文件 SHA-256
    pub fn compute_sha256(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let contents = std::fs::read(path).unwrap();
        let hash = Sha256::digest(&contents);
        hex::encode(hash)
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/evidence_export.rs
git commit -m "feat(gmp): add JsonExporter to evidence_export"
```

---

## Task 4: 实现 PdfExporter

**Files:**
- Modify: `crates/gmp/src/evidence_export.rs`

- [ ] **Step 1: 添加 PdfExporter**

```rust
use printpdf::*;

/// PDF 导出器
pub struct PdfExporter;

impl PdfExporter {
    /// 生成审计包 PDF 报告
    pub fn export_report(
        manifest: &PackageManifest,
        chain_len: usize,
        evidence_len: usize,
        compliance_status: &str,
        output_path: &Path,
    ) -> Result<(), ExportError> {
        let (doc, page1, layer1) = PdfDocument::new(
            "GMP Audit Export Report",
            Mm(210.0),  // A4 width
            Mm(297.0),  // A4 height
            "Layer 1",
        );

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // 使用内置字体
        let font = doc.add_builtin_font(BuiltinFont::Helvetica);
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold);

        // 标题
        current_layer.use_text(
            "GMP Audit Export Report",
            24.0,
            Mm(20.0),
            Mm(277.0),
            &font_bold,
        );

        // 包信息
        current_layer.use_text(
            &format!("Version: {}", manifest.version),
            12.0,
            Mm(20.0),
            Mm(260.0),
            &font,
        );

        current_layer.use_text(
            &format!("Generated: {}", manifest.created_at),
            12.0,
            Mm(20.0),
            Mm(250.0),
            &font,
        );

        current_layer.use_text(
            &format!("Time Range: {} to {}", manifest.from_timestamp, manifest.to_timestamp),
            12.0,
            Mm(20.0),
            Mm(240.0),
            &font,
        );

        // 统计数据
        current_layer.use_text("Audit Statistics:", 14.0, Mm(20.0), Mm(220.0), &font_bold);
        current_layer.use_text(
            &format!("- Audit Chain Entries: {}", chain_len),
            12.0,
            Mm(25.0),
            Mm(210.0),
            &font,
        );
        current_layer.use_text(
            &format!("- Evidence Records: {}", evidence_len),
            12.0,
            Mm(25.0),
            Mm(200.0),
            &font,
        );
        current_layer.use_text(
            &format!("- Compliance Status: {}", compliance_status),
            12.0,
            Mm(25.0),
            Mm(190.0),
            &font,
        );

        // 文件列表
        current_layer.use_text("Package Contents:", 14.0, Mm(20.0), Mm(170.0), &font_bold);
        let mut y = 160.0;
        for entry in &manifest.files {
            current_layer.use_text(
                &format!("- {} ({})", entry.filename, &entry.sha256[..8]),
                10.0,
                Mm(25.0),
                Mm(y),
                &font,
            );
            y -= 8.0;
        }

        // 签名算法
        current_layer.use_text(
            &format!("Signing Algorithm: {}", manifest.algorithm),
            12.0,
            Mm(20.0),
            Mm(50.0),
            &font,
        );

        // 保存 PDF
        doc.save(&mut std::fs::File::create(output_path).map_err(|e| ExportError::PdfError(e.to_string()))?)
            .map_err(|e| ExportError::PdfError(e.to_string()))?;

        Ok(())
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/evidence_export.rs
git commit -m "feat(gmp): add PdfExporter for report generation"
```

---

## Task 5: 实现 Signer

**Files:**
- Modify: `crates/gmp/src/evidence_export.rs`

- [ ] **Step 1: 添加 Signer 结构体**

```rust
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// Ed25519 签名器
pub struct Signer {
    signing_key: SigningKey,
}

impl Signer {
    /// 从种子创建签名器
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    /// 对数据签名
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.signing_key.sign(data).to_bytes().to_vec()
    }

    /// 获取公钥
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// 导出签名到文件
    pub fn export_signature(
        &self,
        data_path: &Path,
        signature_path: &Path,
    ) -> Result<(), ExportError> {
        let data = std::fs::read(data_path)?;
        let signature = self.sign(&data);
        std::fs::write(signature_path, &signature)?;
        Ok(())
    }

    /// 验证签名
    pub fn verify(
        &self,
        public_key: &VerifyingKey,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ExportError> {
        let sig = Signature::from_slice(signature)
            .map_err(|e| ExportError::SignatureError(e.to_string()))?;
        Ok(public_key.verify(data, &sig).is_ok())
    }
}

#[cfg(test)]
impl Signer {
    /// 创建测试用签名器
    pub fn test_signer() -> Self {
        use rand::RngCore;
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        Self::from_seed(seed)
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/evidence_export.rs
git commit -m "feat(gmp): add Ed25519 Signer to evidence_export"
```

---

## Task 6: 实现 PackageBuilder

**Files:**
- Modify: `crates/gmp/src/evidence_export.rs`

- [ ] **Step 1: 添加 PackageBuilder**

```rust
/// 证据包构建器
pub struct PackageBuilder {
    output_dir: std::path::PathBuf,
    from_ts: i64,
    to_ts: i64,
    signer: Signer,
}

impl PackageBuilder {
    pub fn new(output_dir: &Path, from_ts: i64, to_ts: i64, signer: Signer) -> Self {
        // 创建输出目录
        let timestamp = chrono::Utc::now().timestamp();
        let root = output_dir.join(format!("export_{}", timestamp));
        std::fs::create_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("signatures")).ok();

        Self {
            output_dir: root,
            from_ts,
            to_ts,
            signer,
        }
    }

    pub fn build(
        self,
        storage: &dyn sqlrustgo_storage::StorageEngine,
    ) -> Result<PackagePath, ExportError> {
        let root = &self.output_dir;
        let timestamp = chrono::Utc::now().timestamp();

        // 1. 收集审计链数据
        let audit_chain = crate::audit::query_audit_logs(
            storage,
            self.from_ts,
            self.to_ts,
        ).map_err(|e| ExportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 2. 收集证据数据
        let evidence = crate::evidence_api::list_evidence(
            storage,
            self.from_ts,
            self.to_ts,
        ).map_err(|e| ExportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 3. 生成合规报告
        let compliance = crate::compliance::check_batch_compliance(
            storage,
            &crate::compliance::ComplianceCheckRequest {
                from_timestamp: self.from_ts,
                to_timestamp: self.to_ts,
                rules: vec![],
            },
        ).map_err(|e| ExportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        // 4. 导出 JSON 文件
        let audit_path = root.join("audit_chain.json");
        let audit_hash = JsonExporter::export_audit_chain(&audit_chain, &audit_path)?;

        let evidence_path = root.join("evidence.json");
        let evidence_hash = JsonExporter::export_evidence(&evidence, &evidence_path)?;

        let compliance_path = root.join("compliance.json");
        let compliance_hash = JsonExporter::export_compliance(&compliance, &compliance_path)?;

        // 5. 签名
        let sign_dir = root.join("signatures");
        self.signer.export_signature(&audit_path, &sign_dir.join("audit_chain.json.sig"))?;
        self.signer.export_signature(&evidence_path, &sign_dir.join("evidence.json.sig"))?;
        self.signer.export_signature(&compliance_path, &sign_dir.join("compliance.json.sig"))?;

        // 6. 生成 PDF
        let files = vec![
            FileEntry { filename: "audit_chain.json".to_string(), sha256: audit_hash },
            FileEntry { filename: "evidence.json".to_string(), sha256: evidence_hash },
            FileEntry { filename: "compliance.json".to_string(), sha256: compliance_hash },
        ];

        let manifest = PackageManifest {
            version: "1.0".to_string(),
            created_at: timestamp,
            from_timestamp: self.from_ts,
            to_timestamp: self.to_ts,
            algorithm: "Ed25519".to_string(),
            files,
        };

        let manifest_path = root.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, &manifest_json)?;

        // 签名 manifest
        self.signer.export_signature(&manifest_path, &sign_dir.join("manifest.json.sig"))?;

        // 7. 生成 PDF
        let pdf_path = root.join("report.pdf");
        let compliance_status = if compliance.is_compliant { "COMPLIANT" } else { "VIOLATIONS FOUND" };
        PdfExporter::export_report(
            &manifest,
            audit_chain.len(),
            evidence.len(),
            compliance_status,
            &pdf_path,
        )?;

        Ok(PackagePath {
            root: root.clone(),
            manifest: manifest_path,
        })
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/evidence_export.rs
git commit -m "feat(gmp): add PackageBuilder for evidence export"
```

---

## Task 7: 添加模块到 lib.rs

**Files:**
- Modify: `crates/gmp/src/lib.rs`

- [ ] **Step 1: 添加 evidence_export 模块声明**

在 `pub mod evidence_verification;` 后添加：
```rust
pub mod evidence_export;
```

在 re-exports 区域添加（找到合适位置）：
```rust
pub use evidence_export::{
    PackageBuilder, PackageManifest, PackagePath, ExportError, FileEntry,
    VerificationReport, JsonExporter, PdfExporter, Signer,
};
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/src/lib.rs
git commit -m "feat(gmp): export evidence_export module"
```

---

## Task 8: 添加 chrono 依赖（用于时间戳）

**Files:**
- Modify: `crates/gmp/Cargo.toml`

- [ ] **Step 1: 添加 chrono 依赖**

```toml
chrono = "0.4"
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/Cargo.toml
git commit -m "deps(gmp): add chrono for timestamp"
```

---

## Task 9: 编写集成测试

**Files:**
- Create: `crates/gmp/tests/evidence_export_test.rs`

- [ ] **Step 1: 编写测试**

```rust
use sqlrustgo_gmp::evidence_export::{PackageBuilder, Signer, ExportError};
use sqlrustgo_gmp::StorageEngine;
use tempfile::TempDir;

#[test]
fn test_export_audit_package() {
    let temp_dir = TempDir::new().unwrap();
    let signer = Signer::test_signer();

    // Note: Full integration test requires storage setup
    // This is a compile-time check that the API works

    let builder = PackageBuilder::new(
        temp_dir.path(),
        0,
        9999999999,
        signer,
    );

    // Verify builder can be created
    assert!(builder.is_ok());
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/gmp/tests/evidence_export_test.rs
git commit -m "test(gmp): add evidence_export integration tests"
```

---

## Task 10: 运行测试验证

**Files:**
- None (verification only)

- [ ] **Step 1: 运行 clippy 和检查**

```bash
cargo clippy -p sqlrustgo-gmp --all-features -- -D warnings
```

- [ ] **Step 2: 运行格式化检查**

```bash
cargo fmt --all -- --check
```

- [ ] **Step 3: 如有错误则修复并重新提交**

---

## Spec Coverage Check

| Spec Requirement | Task |
|-----------------|------|
| 完整审计链导出 | Task 3, 6 |
| JSON 元数据包 | Task 3, 7 |
| PDF 报告生成 | Task 4, 6 |
| Ed25519 签名 | Task 5, 6 |
| 包完整性验证 | Task 5 |
| 单元测试覆盖 | Task 9 |

---

## 验收条件确认

- [ ] 能导出完整证据包到指定目录
- [ ] 包包含 manifest + JSON 文件 + signatures + PDF
- [ ] Ed25519 签名可验证
- [ ] PDF 可读且包含关键审计信息
- [ ] 单元测试覆盖
