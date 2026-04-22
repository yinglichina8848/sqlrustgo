# GMP 用户指南

> **版本**: v2.7.0 (GA)
> **更新日期**: 2026-04-22

---

## 1. 概述

GMP (Good Manufacturing Practice) 是 SQLRustGo v2.7.0 提供的药品生产质量管理扩展模块。GMP 用户指南详细说明了如何在 SQLRustGo 中部署、配置和使用 GMP 审计功能。

### 1.1 主要功能

| 功能 | 说明 |
|------|------|
| 文档管理 | 文档版本控制、状态追踪 |
| 审计日志 | 操作审计、合规检查 |
| 报告生成 | 偏差报告、CAPA 报告、审计报告 |
| 合规检查 | 文档合规性验证 |
| 证据链 | 操作溯源、完整性校验 |

### 1.2 GMP 核心表

| 表名 | 说明 |
|------|------|
| `gmp_documents` | 文档元数据 |
| `gmp_document_contents` | 文档内容 |
| `gmp_document_keywords` | 文档关键词 |
| `gmp_embeddings` | 向量嵌入 |
| `gmp_audit_log` | 审计日志 |

---

## 2. 快速开始

### 2.1 初始化 GMP 环境

```rust
use sqlrustgo_gmp::{GmpExecutor, create_gmp_tables};
use sqlrustgo_storage::MemoryStorage;
use std::sync::{Arc, RwLock};

let storage = Arc::new(RwLock::new(MemoryStorage::new()));
let executor = GmpExecutor::new(storage);

executor.init().unwrap();
```

### 2.2 导入文档

```rust
let doc_id = executor
    .import_document(
        "批记录模板",
        "BATCH_RECORD",
        "批号: 20260422\n产品: 维生素C片\n规格: 100mg",
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

## 3. 审计功能

### 3.1 记录审计日志

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

### 3.2 查询审计日志

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

### 3.3 审计统计

```rust
use sqlrustgo_gmp::get_audit_stats;

let stats = get_audit_stats(&storage).unwrap();

println!("总操作数: {}", stats.total_operations);
println!("活跃用户: {}", stats.active_users);
```

---

## 4. 报告生成

### 4.1 生成审计报告

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

### 4.2 生成偏差报告

```rust
use sqlrustgo_gmp::generate_deviation_report;

let deviation_report = generate_deviation_report(&storage, &period).unwrap();

println!("偏差总数: {}", deviation_report.deviations.len());
for dev in &deviation_report.deviations {
    println!("  - {}: {}", dev.id, dev.description);
}
```

### 4.3 生成 CAPA 报告

```rust
use sqlrustgo_gmp::generate_capa_report;

let capa_report = generate_capa_report(&storage, &period).unwrap();

println!("CAPA 项目数: {}", capa_report.items.len());
```

---

## 5. 合规检查

### 5.1 文档合规检查

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

### 5.2 批量合规检查

```rust
use sqlrustgo_gmp::check_batch_compliance;

let batch_result = check_batch_compliance(
    &storage,
    &["R001", "R002", "R003"],
    "BATCH_RECORD",
).unwrap();
```

---

## 6. SQL API

### 6.1 GMP 表查询

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

### 6.2 审计日志查询

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

## 7. 配置

### 7.1 Cargo.toml 依赖

```toml
[dependencies]
sqlrustgo-gmp = { version = "0.7", features = ["full"] }
```

### 7.2 特性开关

| 特性 | 说明 |
|------|------|
| `full` | 启用全部 GMP 功能 |
| `audit` | 仅审计功能 |
| `compliance` | 仅合规检查 |
| `vector` | 向量搜索支持 |

---

## 8. 最佳实践

### 8.1 文档管理

- 文档编号采用唯一标识符
- 重要文档启用版本控制
- 定期备份审计日志

### 8.2 审计策略

- 关键操作必须记录审计日志
- 审计日志保留期限不少于 5 年
- 定期审查审计统计数据

### 8.3 合规检查

- 上线前执行完整合规检查
- 定期执行批量合规检查
- 违规项及时整改

---

## 9. 故障排查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| 文档导入失败 | 表未创建 | 调用 `create_gmp_tables()` |
| 审计日志缺失 | 事务未提交 | 检查事务状态 |
| 合规检查超时 | 规则过多 | 分批执行检查 |

---

## 10. API 参考

| API | 说明 |
|-----|------|
| `GmpExecutor::new()` | 创建执行器 |
| `create_gmp_tables()` | 创建 GMP 表 |
| `import_document()` | 导入文档 |
| `search()` | 搜索文档 |
| `record_audit_log()` | 记录审计日志 |
| `query_audit_logs()` | 查询审计日志 |
| `generate_audit_report()` | 生成审计报告 |
| `check_document_compliance()` | 合规检查 |

---

*GMP 用户指南 v2.7.0*
*最后更新: 2026-04-22*