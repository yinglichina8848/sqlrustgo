# GMP 用户指南

> **版本**: v2.9.0 (RC)
> **更新日期**: 2026-05-05

---

## 1. 概述

GMP (Good Manufacturing Practice / Multi-Version Persistence) 是 SQLRustGo v2.9.0 提供的审计证据链管理扩展模块。GMP 通过多版本持久化机制实现操作溯源、完整性校验和合规审计功能。

### 1.1 主要功能

| 功能 | 说明 |
|------|------|
| 多版本持久化 | MVCC 事务支持，版本链追踪 |
| 审计日志 | 操作审计、合规检查、证据链存储 |
| 证据链 | 操作溯源、完整性校验、防篡改 |
| 报告生成 | 审计报告、偏差报告、CAPA 报告 |
| 合规检查 | 文档合规性验证、规则引擎 |

### 1.2 GMP 核心表

| 表名 | 说明 |
|------|------|
| `gmp_documents` | 文档元数据和版本信息 |
| `gmp_document_contents` | 文档内容（按版本存储） |
| `gmp_document_keywords` | 文档关键词索引 |
| `gmp_embeddings` | 向量嵌入（语义检索） |
| `gmp_audit_log` | 审计日志（证据链） |
| `gmp_version_chain` | 版本链关系（多版本追溯） |
| `gmp_evidence` | 证据存储（防篡改） |

---

## 2. 快速开始

### 2.1 初始化 GMP 环境

```rust
use sqlrustgo_gmp::{GmpExecutor, create_gmp_tables};
use sqlrustgo_storage::{MemoryStorage, DiskStorage};
use std::sync::{Arc, RwLock};

let storage = Arc::new(RwLock::new(MemoryStorage::new()));
let executor = GmpExecutor::new(storage.clone());

executor.init().unwrap();
```

### 2.2 导入文档

```rust
let doc_id = executor
    .import_document(
        "批记录模板",
        "BATCH_RECORD",
        "批号: 20260505\n产品: 维生素C片\n规格: 100mg",
        &["批记录", "生产", "维生素"],
    )
    .unwrap();

println!("文档 ID: {}", doc_id);
```

### 2.3 搜索文档

```rust
let results = executor.search("维生素", 5).unwrap();

for result in results {
    println!("文档: {} - 相似度: {:.2}", result.document_id, result.score);
}
```

---

## 3. 多版本持久化

### 3.1 版本链机制

GMP 通过 `gmp_version_chain` 表维护文档的版本关系：

```rust
use sqlrustgo_gmp::{create_version_record, VersionInfo};

let version = create_version_record(
    &storage,
    doc_id,
    "v1.0",
    "v2.0",
    "user_001",
    "更新生产参数",
).unwrap();
```

### 3.2 版本查询

```rust
use sqlrustgo_gmp::get_version_history;

let history = get_version_history(&storage, doc_id).unwrap();

for v in history {
    println!("版本: {} -> {} (操作人: {})", v.from_ver, v.to_ver, v.operator);
}
```

### 3.3 证据链记录

```rust
use sqlrustgo_gmp::{record_evidence, EvidenceType};

record_evidence(
    &storage,
    doc_id,
    EvidenceType::DocumentUpdate,
    "user_001",
    "更新了批记录内容",
    Some("sha256:abc123..."),
).unwrap();
```

---

## 4. 审计功能

### 4.1 记录审计日志

```rust
use sqlrustgo_gmp::{record_audit_log, AuditAction};

record_audit_log(
    &storage,
    AuditAction::DocumentCreated,
    "user_001",
    "gmp_documents",
    Some(doc_id),
    "创建批记录文档",
).unwrap();
```

### 4.2 查询审计日志

```rust
use sqlrustgo_gmp::query_audit_logs;

let logs = query_audit_logs(
    &storage,
    Some("user_001"),
    None,
    None,
    100,
).unwrap();
```

### 4.3 审计统计

```rust
use sqlrustgo_gmp::get_audit_stats;

let stats = get_audit_stats(&storage).unwrap();

println!("总操作数: {}", stats.total_operations);
println!("活跃用户: {}", stats.active_users);
```

### 4.4 证据完整性校验

```rust
use sqlrustgo_gmp::verify_evidence_chain;

let result = verify_evidence_chain(&storage, doc_id).unwrap();

println!("证据链完整: {}", result.is_valid);
println!("校验时间: {}", result.verification_time);
```

---

## 5. 报告生成

### 5.1 生成审计报告

```rust
use sqlrustgo_gmp::{generate_audit_report, ReportPeriod, ReportType};
use chrono::{Utc, Duration};

let period = ReportPeriod {
    start: Utc::now() - Duration::days(30),
    end: Utc::now(),
};

let report = generate_audit_report(
    &storage,
    ReportType::AuditSummary,
    &period,
).unwrap();
```

### 5.2 生成偏差报告

```rust
use sqlrustgo_gmp::generate_deviation_report;

let deviation_report = generate_deviation_report(&storage, &period).unwrap();

println!("偏差总数: {}", deviation_report.deviations.len());
for dev in &deviation_report.deviations {
    println!("  - {}: {}", dev.id, dev.description);
}
```

### 5.3 生成 CAPA 报告

```rust
use sqlrustgo_gmp::generate_capa_report;

let capa_report = generate_capa_report(&storage, &period).unwrap();

println!("CAPA 项目数: {}", capa_report.items.len());
```

---

## 6. 合规检查

### 6.1 文档合规检查

```rust
use sqlrustgo_gmp::{check_document_compliance, ComplianceCheckRequest, ComplianceRule};

let rules = vec![
    ComplianceRule {
        rule_id: "R001".to_string(),
        description: "批记录必须包含批号".to_string(),
        severity: sqlrustgo_gmp::Severity::Critical,
    },
    ComplianceRule {
        rule_id: "R002".to_string(),
        description: "生产日期不得晚于审核日期".to_string(),
        severity: sqlrustgo_gmp::Severity::Major,
    },
];

let request = ComplianceCheckRequest {
    document_id: doc_id,
    rules,
};

let result = check_document_compliance(&storage, &request).unwrap();

println!("合规状态: {}", if result.is_compliant { "通过" } else { "违规" });
for violation in &result.violations {
    println!("  - [{}] {}", violation.severity, violation.description);
}
```

### 6.2 批量合规检查

```rust
use sqlrustgo_gmp::check_batch_compliance;

let batch_result = check_batch_compliance(
    &storage,
    &["R001", "R002", "R003"],
    "BATCH_RECORD",
).unwrap();
```

---

## 7. SQL API

### 7.1 GMP 表查询

```sql
-- 查看所有 GMP 文档
SELECT * FROM gmp_documents WHERE doc_type = 'BATCH_RECORD';

-- 查看文档内容
SELECT d.id, d.title, c.content
FROM gmp_documents d
JOIN gmp_document_contents c ON d.id = c.doc_id;

-- 按关键词搜索
SELECT d.* FROM gmp_documents d
JOIN gmp_document_keywords k ON d.id = k.doc_id
WHERE k.keyword LIKE '%批记录%';
```

### 7.2 版本链查询

```sql
-- 查看文档版本历史
SELECT * FROM gmp_version_chain
WHERE doc_id = 'doc_001'
ORDER BY created_at DESC;

-- 查看证据链
SELECT * FROM gmp_evidence
WHERE doc_id = 'doc_001'
ORDER BY timestamp DESC;
```

### 7.3 审计日志查询

```sql
-- GMP Top 10 审核查询
SELECT action_type, COUNT(*) as cnt
FROM gmp_audit_log
WHERE timestamp > NOW() - INTERVAL '30 days'
GROUP BY action_type
ORDER BY cnt DESC
LIMIT 10;

-- 敏感操作审计
SELECT * FROM gmp_audit_log
WHERE action_type IN ('DELETE', 'UPDATE', 'SENSITIVE_ACCESS')
ORDER BY timestamp DESC
LIMIT 100;
```

---

## 8. 配置

### 8.1 Cargo.toml 依赖

```toml
[dependencies]
sqlrustgo-gmp = { version = "0.9", features = ["full"] }
```

### 8.2 特性开关

| 特性 | 说明 |
|------|------|
| `full` | 启用全部 GMP 功能 |
| `audit` | 仅审计功能 |
| `compliance` | 仅合规检查 |
| `vector` | 向量搜索支持 |
| `evidence` | 证据链功能 |
| `mvcc` | 多版本并发控制 |

### 8.3 环境变量

```bash
# GMP 配置
export GMP_AUDIT_RETENTION_DAYS=1825  # 审计日志保留期限（默认5年）
export GMP_EVIDENCE_HASH=sha256         # 证据哈希算法
export GMP_MVCC_ENABLED=true            # 启用 MVCC
```

---

## 9. 最佳实践

### 9.1 多版本管理

- 文档编号采用唯一标识符
- 重要文档启用版本控制
- 定期清理过期版本（保留主版本）

### 9.2 审计策略

- 关键操作必须记录审计日志
- 审计日志保留期限不少于 5 年
- 定期审查审计统计数据
- 启用证据链完整性校验

### 9.3 合规检查

- 上线前执行完整合规检查
- 定期执行批量合规检查
- 违规项及时整改
- 保存合规检查记录

### 9.4 性能优化

- 批量导入时使用事务包装
- 频繁查询的字段建立索引
- 历史数据定期归档

---

## 10. 故障排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 文档导入失败 | 表未创建 | 调用 `create_gmp_tables()` |
| 版本链断裂 | 并发更新冲突 | 检查 MVCC 配置，使用锁 |
| 审计日志缺失 | 事务未提交 | 检查事务状态 |
| 证据链校验失败 | 数据被篡改 | 立即报警，排查访问日志 |
| 合规检查超时 | 规则过多 | 分批执行检查 |

---

## 11. API 参考

| API | 说明 |
|-----|------|
| `GmpExecutor::new()` | 创建执行器 |
| `create_gmp_tables()` | 创建 GMP 表 |
| `import_document()` | 导入文档 |
| `search()` | 搜索文档 |
| `create_version_record()` | 创建版本记录 |
| `get_version_history()` | 获取版本历史 |
| `record_evidence()` | 记录证据 |
| `verify_evidence_chain()` | 校验证据链 |
| `record_audit_log()` | 记录审计日志 |
| `query_audit_logs()` | 查询审计日志 |
| `generate_audit_report()` | 生成审计报告 |
| `check_document_compliance()` | 合规检查 |

---

*GMP 用户指南 v2.9.0*
*最后更新: 2026-05-05*
