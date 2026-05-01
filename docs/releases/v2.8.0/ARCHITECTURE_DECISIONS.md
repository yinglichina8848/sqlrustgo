# 架构决策记录 (ADR)

> **版本**: v2.8.0
> **最后更新**: 2026-04-30

---

## 概述

本文档记录 SQLRustGo v2.8.0 开发过程中的关键架构决策。v2.8.0 是生产化+分布式+安全版本，新增了分布式架构、SIMD 向量化、列级权限等关键架构决策。

---

## ADR-001: 使用 Rust 作为核心开发语言

**状态**: 已批准

**背景**: 选择数据库引擎的开发语言

**决策**: 使用 Rust 作为核心开发语言

**理由**:
- 内存安全，无需 GC
- 高性能，接近 C/C++
- 丰富的生态 (tokio, serde 等)
- 活跃的社区

**后果**:
- 需要处理所有权和生命周期
- 编译时间长
- 生态系统不如 Java 成熟

---

## ADR-002: 分层架构设计

**状态**: 已批准

**背景**: 确定系统架构

**决策**: 采用分层架构: Parser → Planner → Executor → Storage

**分层说明**:

```
┌─────────────────────────────────────┐
│            Network Layer            │
├─────────────────────────────────────┤
│            Parser Layer             │
├─────────────────────────────────────┤
│            Planner Layer            │
├─────────────────────────────────────┤
│           Executor Layer            │
├─────────────────────────────────────┤
│           Storage Layer             │
└─────────────────────────────────────┘
```

**理由**:
- 清晰的职责分离
- 便于独立测试
- 模块可替换

**后果**:
- 层间通信开销
- 需要定义标准接口

---

## ADR-003: 基于成本的查询优化器

**状态**: 已批准

**背景**: 查询优化策略选择

**决策**: 实现基于成本的查询优化器 (CBO)

**理由**:
- 比规则优化更优
- 可适应不同数据分布
- 业界标准实践

**实现**:
- 统计信息收集
- 代价模型
- 动态规划连接顺序

---

## ADR-004: MVCC 事务隔离

**状态**: 已批准

**背景**: 事务隔离实现

**决策**: 实现 MVCC + SSI 隔离级别

**理由**:
- 读不阻塞写
- 写不阻塞读
- 可序列化隔离

**实现**:
- 快照管理
- 版本链
- 冲突检测

---

## ADR-005: Buffer Pool 内存管理

**状态**: 已批准

**背景**: 存储引擎内存管理

**决策**: 实现 Buffer Pool 管理

**理由**:
- 减少磁盘 I/O
- 提高查询性能
- 内存控制

**实现**:
- LRU 淘汰策略
- 页面预取
- 脏页刷新

---

## ADR-006: MySQL 协议兼容

**状态**: 已批准

**背景**: 网络协议选择

**决策**: 兼容 MySQL 协议

**理由**:
- 现有客户端兼容
- 降低用户迁移成本
- 丰富的生态工具

**实现**:
- MySQL C/S 协议
- 连接 handshake
- SQL 命令解析

---

## ADR-007: 向量化执行

**状态**: 已批准

**背景**: 执行引擎优化

**决策**: 实现向量化执行引擎

**理由**:
- SIMD 加速
- 减少分支预测
- 批处理减少开销

**实现**:
- 列式数据组织
- 批量运算符
- SIMD 指令

---

## ADR-008: 预写日志 (WAL)

**状态**: 已批准

**背景**: 持久化策略

**决策**: 实现 WAL 机制

**理由**:
- 崩溃恢复
- 持久性保证
- 写入优化

**实现**:
- 顺序写入
- 检查点
- 崩溃恢复

---

## ADR-009: 索引结构

**状态**: 已批准

**背景**: 索引实现

**决策**: 实现 B+ Tree 作为主索引

**理由**:
- 范围查询高效
- 磁盘友好
- 业界标准

**同时支持**:
- Hash Index (等值查询)
- Vector Index (向量搜索)

---

## ADR-010: 连接池实现

**状态**: 已批准

**背景**: 并发连接管理

**决策**: 实现内置连接池

**理由**:
- 减少连接开销
- 连接复用
- 资源控制

**实现**:
- 连接队列
- 超时管理
- 健康检查

---

## ADR-015: 分布式架构 (T-23 ~ T-27) — v2.8.0 新增

**状态**: 已批准

**背景**: 实现初步分布式能力，包括分区表、主从复制、故障转移和负载均衡

**决策**: 实现分区表 + GTID 主从复制 + 读写分离架构

**架构**:

```
┌─────────────────────────────────────────────────────────────┐
│                      SQLRustGo Cluster                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│   │  Primary    │────▶│  Replica 1  │────▶│  Replica 2  │   │
│   │  (主节点)   │ GTID│  (从节点1) │     │  (从节点2)  │   │
│   └─────────────┘     └─────────────┘     └─────────────┘   │
│          │                   │                   │           │
│          ▼                   ▼                   ▼           │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│   │  Partition  │     │  Partition  │     │  Partition  │   │
│   │    A        │     │    B        │     │    C        │   │
│   └─────────────┘     └─────────────┘     └─────────────┘   │
│                                                              │
│   ┌─────────────────────────────────────────────────────┐    │
│   │              Load Balancer (T-26)                   │    │
│   │         Round-Robin / Least-Connections            │    │
│   └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**核心组件**:

| 组件 | 功能 | 状态 |
|------|------|------|
| 分区表 (T-23) | Range/List/Hash/Key 分区 | ⏳ 规划中 |
| GTID 复制 (T-24) | 全局事务 ID + 半同步复制 | ✅ PR#78 已合并 |
| 故障转移 (T-25) | 主节点宕机检测 < 5s，自动切换 < 30s | ⏳ 规划中 |
| 负载均衡 (T-26) | 轮询/最少连接策略 | ✅ PR#45 已合并 |
| 读写分离 (T-27) | SELECT 到从节点，INSERT/UPDATE 到主节点 | ✅ PR#50,55 已合并 |

**分区表策略**:

| 分区类型 | 实现 | 说明 |
|----------|------|------|
| RANGE | ✅ | 按数值范围分区 |
| LIST | ✅ | 按枚举值列表分区 |
| HASH | ✅ | 按 Hash 值均匀分布 |
| KEY | ⏳ | 按主键 Hash 分区 |

**理由**:
- 分区表支持数据垂直/水平拆分
- GTID 提供精确的复制位置追踪
- 读写分离提升查询吞吐量
- 负载均衡提高系统可用性

**后果**:
- 需要处理分布式事务（规划 v2.9.0）
- 跨分区查询复杂度增加
- 复制延迟需要监控

---

## ADR-016: SIMD 向量化加速 (T-14) — v2.8.0 新增

**状态**: 已批准

**背景**: 利用 SIMD 指令加速向量运算，提升性能

**决策**: 实现显式 SIMD 函数接口，支持 AVX2/AVX-512 自动检测

**架构**:

```rust
// crates/vector/src/simd_explicit.rs
pub mod simd_explicit {
    // 单向量距离计算
    pub fn l2_distance_simd(a: &[f32], b: &[f32]) -> f32;
    pub fn cosine_distance_simd(a: &[f32], b: &[f32]) -> f32;
    pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32;

    // 批量计算 (AVX2 加速)
    pub fn batch_l2_distance_simd(query: &[f32], vectors: &[Vec<f32>]) -> Vec<f32>;
    pub fn batch_cosine_distance_simd(query: &[f32], vectors: &[Vec<f32>]) -> Vec<f32>;
    pub fn batch_dot_product_simd(query: &[f32], vectors: &[Vec<f32>]) -> Vec<f32>;

    // SIMD 能力检测
    pub fn detect_simd_lanes() -> usize;  // AVX2=8, AVX-512=16, fallback=1
}
```

**性能目标**:

| 指标 | 目标 | 状态 |
|------|------|------|
| SIMD 加速比 | ≥ 2x | ✅ PR#32 已合并 |
| CPU 利用率 | < 80% | ✅ |
| 向量搜索延迟 (10k) | < 50ms | ✅ |

**理由**:
- SIMD 单指令多数据，加速向量计算
- 自动检测 CPU 能力，fallback 到标量
- 批量计算减少函数调用开销

**实现**:
- `crates/vector/src/simd_explicit.rs` — 显式 SIMD 函数
- `simd_lanes()` — 自动检测可用 SIMD 宽度
- 5 个测试用例全部通过

---

## ADR-017: 列级权限控制 (T-17) — v2.8.0 新增

**状态**: 部分实现

**背景**: 实现细粒度的列级访问控制，满足安全合规需求

**决策**: 实现 ColumnMasker + GRANT/REVOKE 语法

**架构**:

```rust
// ColumnMasker 核心结构
pub struct ColumnMasker {
    rules: Vec<MaskingRule>,
}

pub struct MaskingRule {
    id: String,
    column: String,
    mask_type: MaskingType,
    description: String,
}

pub enum MaskingType {
    Full,      // 完全隐藏
    Partial,   // 部分隐藏 (如 email: t***@example.com)
    Hash,      // Hash 处理
    Null,      // 替换为 NULL
}
```

**SQL 接口**:

```sql
-- 创建带掩码的列权限
GRANT SELECT(email, phone) ON users TO 'reader'@'localhost';
REVOKE SELECT(salary) ON employees FROM 'app'@'localhost';

-- 列级 INSERT/UPDATE 权限
GRANT INSERT(name, age) ON users TO 'writer'@'localhost';
GRANT UPDATE(email) ON users TO 'editor'@'localhost';
```

**当前状态**:
- ColumnMasker 数据结构已实现
- GRANT/REVOKE 解析器部分实现
- 78 个审计测试通过

**理由**:
- 满足 GDPR 等合规要求
- 保护敏感个人信息
- 最小权限原则

**待完成**:
- GRANT/REVOKE 解析器完整实现
- 运行时权限检查

---

## ADR-018: 审计告警系统 (T-18) — v2.8.0 新增

**状态**: 已批准

**背景**: 实现完整的审计日志，支持合规性要求和安全监控

**决策**: 实现安全审计模块，记录所有关键操作

**审计事件类型**:

| 事件 | 说明 |
|------|------|
| `Login` | 用户登录 |
| `Logout` | 用户登出 |
| `ExecuteSql` | SQL 执行 |
| `DDL` | CREATE/ALTER/DROP |
| `DML` | INSERT/UPDATE/DELETE |
| `Grant` | 权限授予 |
| `Revoke` | 权限撤销 |
| `Error` | 错误发生 |

**核心 API**:

```rust
use sqlrustgo_security::{AuditManager, AuditEvent, AuditFilter};

let audit = AuditManager::new();

// 记录审计事件
audit.log(AuditEvent::ExecuteSql {
    user: "root".to_string(),
    sql: "SELECT * FROM users".to_string(),
    duration_ms: 5,
    rows: 100,
    session_id: 12345,
});

// 查询审计日志
let filter = AuditFilter::new()
    .with_users(vec!["admin".to_string()])
    .with_event_types(vec!["DDL".to_string(), "DML".to_string()]);

let records = audit.query(&filter);
```

**状态**: ✅ PR#76 已合并，78 tests passing

---

## ADR-019: TriBool 三值逻辑 — v2.8.0 新增

**状态**: 已批准

**背景**: SQL 标准要求三值逻辑 (TRUE, FALSE, UNKNOWN/NULL)，需要正确处理 NULL 比较

**决策**: 实现 TriBool 类型，正确处理 SQL NULL 语义

**TriBool 定义**:

```rust
pub enum TriBool {
    True,    // SQL: TRUE
    False,   // SQL: FALSE
    Unknown, // SQL: NULL (UNKNOWN)
}

impl TriBool {
    // AND 运算: UNKNOWN AND x = UNKNOWN (除 FALSE)
    pub fn and(self, other: TriBool) -> TriBool;

    // OR 运算: UNKNOWN OR x = UNKNOWN (除 TRUE)
    pub fn or(self, other: TriBool) -> TriBool;

    // NOT 运算: NOT UNKNOWN = UNKNOWN
    pub fn not(self) -> TriBool;
}
```

**真值表**:

| AND | True | False | Unknown |
|-----|------|-------|---------|
| **True** | True | False | Unknown |
| **False** | False | False | False |
| **Unknown** | Unknown | False | Unknown |

| OR | True | False | Unknown |
|-----|------|-------|---------|
| **True** | True | True | True |
| **False** | True | False | Unknown |
| **Unknown** | True | Unknown | Unknown |

**理由**:
- SQL 标准要求三值逻辑
- 正确处理 NULL = NULL (结果为 UNKNOWN，不是 TRUE)
- 避免常见的 NULL 比较陷阱

---

## ADR-020: MySQL Native Password 认证 (Issue #1838) — v2.8.0 新增

**状态**: 已批准

**背景**: 修复 MySQL 客户端认证兼容性问题

**决策**: 实现 `mysql_native_password` 认证插件

**实现**:

```rust
// crates/network/src/auth/mysql_native_password.rs
pub struct MysqlNativePassword {
    salt: [u8; 20],
}

impl MysqlNativePassword {
    // MySQL 4.1+ 密码哈希算法
    pub fn scramble(&self, password: &str) -> [u8; 20];
    pub fn verify(&self, password: &str, scramble: &[u8]) -> bool;
}
```

**状态**: ✅ PR#75 已合并，Issue #1838 已修复

---

## 决策模板

```markdown
## ADR-XXX: 决策标题

**状态**: [已批准/待定/已拒绝]

**背景**: 决策背景

**决策**: 具体决策

**理由**: 决策理由

**后果**: 预期后果
```

---

## 相关文档

- [架构设计](../architecture.md)
- [安全加固指南](./SECURITY_HARDENING.md)
- [API 使用示例](./API_USAGE_EXAMPLES.md)
- [性能基准](./BENCHMARK.md)

---

*本文档由 SQLRustGo Team 维护*
