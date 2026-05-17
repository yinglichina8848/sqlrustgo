# Evidence Export Package Design

**Date**: 2026-05-18
**Issue**: #1161 - Evidence Export - 合规证据导出
**Status**: Approved

## 1. Overview

实现 GMP 证据导出包功能，支持导出完整的审计链、证据记录、合规报告，输出为 JSON + PDF + 数字签名格式，用于 GA 展示和合规审计。

## 2. Requirements

- 导出完整审计链（基于时间范围）
- 输出 JSON 元数据包
- 生成 PDF 总结报告（使用 printpdf 库）
- Ed25519 数字签名验证
- 包完整性验证

## 3. Architecture

### Module Location
`crates/gmp/src/evidence_export.rs`

### Components

```
evidence_export.rs
├── EvidencePackage       # 包结构定义
├── PackageBuilder        # 构建证据包
├── JsonExporter         # JSON 序列化
├── PdfExporter          # PDF 生成 (printpdf)
├── Signer               # Ed25519 签名
└── verify_package()      # 包完整性验证
```

### Package Output Structure

```
export_<timestamp>/
├── manifest.json          # 包元数据、生成时间、签名算法
├── audit_chain.json      # 完整审计链数据
├── evidence.json         # 证据记录
├── compliance.json       # 合规报告
├── signatures/           # 数字签名目录
│   ├── manifest.json.sig
│   ├── audit_chain.json.sig
│   ├── evidence.json.sig
│   └── compliance.json.sig
└── report.pdf            # PDF 总结报告
```

## 4. API Design

```rust
/// 导出证据包
pub fn export_audit_package(
    storage: &dyn StorageEngine,
    from_ts: i64,
    to_ts: i64,
    output_dir: &Path,
    signing_key: &SigningKey,
) -> SqlResult<PackagePath>;

/// 验证证据包
pub fn verify_package(
    package_path: &Path,
    public_key: &PublicKey,
) -> Result<VerificationReport, ExportError>;

/// 包元数据
#[derive(Serialize, Deserialize)]
pub struct PackageManifest {
    pub version: String,
    pub created_at: i64,
    pub from_timestamp: i64,
    pub to_timestamp: i64,
    pub algorithm: String,  // "Ed25519"
    pub files: Vec<FileEntry>,
}

pub struct FileEntry {
    pub filename: String,
    pub sha256: String,
}
```

## 5. Implementation Details

### 5.1 Dependencies (Cargo.toml)

```toml
printpdf = "0.7"
ed25519-dalek = { version = "2", features = ["std"] }
sha2 = "0.10"
serde_json = "1"
zip = "2"  # 可选：打包成单个 zip
```

### 5.2 JsonExporter

- 序列化 `AuditChain` → `audit_chain.json`
- 序列化 `EvidenceChain` → `evidence.json`
- 序列化 `ComplianceResult` → `compliance.json`

### 5.3 PdfExporter

使用 printpdf 库：
- A4 页面
- 包含：包摘要、审计链长度、证据数量、合规状态、生成时间
- 每页带页眉页脚

### 5.4 Signer

使用 ed25519-dalek：
- 对每个 JSON 文件计算 SHA-256
- 用私钥对 hash 签名
- 输出 `.sig` 文件

### 5.5 PackageBuilder

协调整个导出流程：
1. 查询审计链数据
2. 生成 JSON 文件
3. 生成 PDF 报告
4. 计算签名
5. 写入输出目录

## 6. Error Handling

```rust
pub enum ExportError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    PdfError(printpdf::PdfError),
    SignatureError(ed25519_dalek::SignatureError),
    VerificationFailed(String),
}
```

## 7. Testing

- 单元测试：各组件独立测试
- 集成测试：完整导出流程
- 签名验证测试

## 8. Acceptance Criteria

1. ✅ 能导出完整证据包到指定目录
2. ✅ 包包含 manifest + JSON 文件 + signatures + PDF
3. ✅ Ed25519 签名可验证
4. ✅ PDF 可读且包含关键审计信息
5. ✅ 单元测试覆盖
