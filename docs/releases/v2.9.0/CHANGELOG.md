# v2.9.0 变更日志

> **版本**: v2.9.0
> **发布日期**: 2026-05-05

---

## v2.9.0 (2026-05-05)

### 新增功能

#### 分布式 (D系列)

- **D-01 Semi-sync 复制** - PR #139
- **D-02 MTS 并行复制** - PR #140
- **D-03 Multi-source 复制** - PR #143
- **D-04 XA 事务协调器** - PR #145/#146

#### SQL 兼容性 (C系列)

- **C-02 CTE/WITH** - PR #157
- **C-03 JSON 操作** - PR #160
- **C-04 窗口函数** - PR #160
- **C-06 CASE/WHEN** - PR #160

#### 形式化验证

- **Proof Registry v2** - PR #239
- **Proof Coverage 报告** - PR #239
- **Gate OS Lite B-Gate CI** - PR #242

#### DDL 命令

- CREATE TABLE IF NOT EXISTS
- DROP TABLE IF EXISTS
- INSERT ON DUPLICATE KEY UPDATE
- ALTER TABLE DROP/MODIFY COLUMN
- CREATE VIEW / DROP VIEW

### 改进

- TPC-H RIGHT/FULL JOIN 支持 - PR #233
- Hash Join NULL 处理优化
- B+Tree 索引优化

### 测试与质量

- SQL Corpus 覆盖率 92.6% (449/485)
- TPC-H 13/22 查询通过
- executor 覆盖率 71.08%
- 总覆盖率 84.18%

### 文档

- v2.9.0 综合文档更新 - PR #280
- TPC-H 测试指南 - PR #275
- RC 门禁检查清单 - PR #272

---

## v2.8.0 (2026-05-02)

### 新增功能

- 完整 MVCC 事务支持
- WAL 检查点机制
- Buffer Pool 优化
- AES-256 静态加密
- 安全审计日志
- SIMD 向量化加速

### 改进

- SQL Parser 覆盖率 85%+
- REPL 改进
- MySQL 协议兼容

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
