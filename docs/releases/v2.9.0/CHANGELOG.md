# v2.9.0 变更日志

> **版本**: v2.9.0
> **发布日期**: 2026-05-xx

## v2.9.0 (Alpha)

### 新增功能

#### 分布式 (D系列)

- **D-01 Semi-sync 复制** - PR #139
  - 支持半同步复制协议
  - ACK 超时配置

- **D-02 MTS 并行复制** - PR #140
  - Multi-Threaded Slave 支持
  - 冲突检测机制

- **D-03 Multi-source 复制** - PR #143
  - 多主源复制
  - 通道管理

- **D-04 XA 事务协调器** - PR #145/#146
  - 两阶段提交实现
  - XA 事务支持

#### SQL 兼容性 (C系列)

- **C-02 CTE/WITH** - PR #157
  - 通用表表达式支持
  - 递归 CTE 支持

- **C-03 JSON 操作** - PR #160
  - JSON 提取函数
  - JSON 路径支持

- **C-04 窗口函数** - PR #160
  - ROW_NUMBER, RANK, DENSE_RANK
  - PARTITION BY 支持

- **C-06 CASE/WHEN** - PR #160
  - 完整 CASE 表达式支持

#### DDL 命令 (Sprint 1)

- CREATE TABLE IF NOT EXISTS
- DROP TABLE IF EXISTS
- INSERT ON DUPLICATE KEY UPDATE
- ALTER TABLE DROP/MODIFY COLUMN
- CREATE VIEW / DROP VIEW
- CREATE UNIQUE INDEX
- DROP INDEX IF EXISTS
- SHOW DATABASES / SHOW CREATE TABLE

### 改进

- **性能**: Sysbench 基准测试集成
- **安全**: GRANT/REVOKE 完善
- **稳定性**: 移除 #[ignore] 测试

### 已知问题

- E-08: 性能优化目标 (≥10K QPS) 尚未达成

### 测试与质量

- **TPC-H 性能改进** - PR #238
  - RIGHT/FULL JOIN 支持
  - Hash Join NULL 处理优化
  - TPC-H Q1-Q23 基准测试集成

- **Formal Verification Phase B** - PR #239
  - Proof Coverage v2 系统
  - 混沌 CI 集成
  - 不变式保护机制
  - PROOF-011~019 形式化验证

- **Gate OS Lite B-Gate CI** - PR #242
  - .gitea/workflows/gate-ci.yml
  - B-Gate 自动化测试流水线
  - gate_report.json artifact 生成
  - hp-z6g4 nomad runner 执行

### 文档

- **v2.9.0 Beta 测试报告** - PR #237
  - 集成测试状态
  - 覆盖率分析
  - 已知问题清单

---

## v2.8.0 (GA)

### 新增功能

- 完整 MVCC 事务支持
- WAL 检查点机制
- Buffer Pool 优化
- AES-256 静态加密
- 安全审计日志

### 改进

- SQL Parser 覆盖率 85%+
- REPL 改进
- MySQL 协议兼容