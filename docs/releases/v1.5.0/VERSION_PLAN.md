# SQLRustGo v1.5.0 版本计划 (VERSION_PLAN.md)

> **版本**: v1.5.0 Draft
> **主题**: 事务与插件系统
> **发布日期**: 预计 2026-07-20
> **更新日期**: 2026-03-15

---

## 一、版本主题

**事务与插件系统**

v1.5.0 是 v2.0 分布式内核前的最后一个大版本，完成单机数据库的核心功能：事务支持、插件系统和安全增强。

---

## 二、轨道规划

### 2.1 轨道 A: 事务支持 (必须)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| MVCC 基础 | 多版本并发控制 (快照隔离) | P0 | ⏳ |
| 事务管理器 | 事务生命周期管理 | P0 | ⏳ |
| 隔离级别 | RC, Serializable 支持 | P0 | ⏳ |
| 锁管理器 | 行锁和表锁 | P0 | ⏳ |
| 死锁检测 | 防止死锁策略 | P0 | ⏳ |

### 2.2 轨道 B: 插件系统 (必须)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| Plugin trait | 插件接口定义 | P0 | ⏳ |
| 插件加载器 | 动态加载 .so 文件 | P0 | ⏳ |
| 存储插件 | 自定义存储后端 | P1 | ⏳ |
| UDF 插件 | 用户定义函数支持 | P1 | ⏳ |

### 2.3 轨道 C: 企业级特性 (重要)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| 内存池管理 | 内存分配器 | P1 | ⏳ |
| 内存配额 | 查询内存限制 | P1 | ⏳ |
| Spill to Disk | 大数据量溢出 | P1 | ⏳ |
| TLS/SSL | 传输加密 | P1 | ⏳ |
| 认证机制 | 用户密码认证 | P1 | ⏳ |
| 权限控制 | RBAC 基础 | P2 | ⏳ |

### 2.4 轨道 D: 高级特性 (可选)

| 功能 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| 两阶段提交 | 分布式事务基础 | P2 | ⏳ |
| Prometheus 告警 | 告警规则配置 | P2 | ⏳ |

---

## 三、功能详情

### 3.1 MVCC 实现

#### 3.1.1 快照隔离 (Snapshot Isolation)

- **版本链**: 每个 key 维护版本链
- **清理策略**: 定期清理过期版本 (GC)
- **冲突检测**: 写-写冲突检测

#### 3.1.2 事务状态机

```
START -> ACTIVE -> COMMITTED/ABORTED
                -> BLOCKED (等待锁)
                -> ROLLBACK
```

### 3.2 锁管理器

- **锁类型**: 共享锁 (S), 排他锁 (X)
- **锁粒度**: 行锁, 表锁
- **死锁处理**: 超时检测 + 图检测算法

### 3.3 插件架构

```
┌─────────────────────────────────────┐
│           Plugin Manager            │
├─────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐          │
│  │ Storage │ │   UDF   │   ...    │
│  │ Plugin  │ │ Plugin  │          │
│  └─────────┘ └─────────┘          │
├─────────────────────────────────────┤
│         Plugin API (trait)          │
└─────────────────────────────────────┘
```

### 3.4 内存管理

- **内存池**: 基于 arena 的内存分配
- **配额策略**: 按查询/会话分配内存
- **溢出机制**: 大 operator 结果溢出到磁盘

---

## 四、API 变更

### 4.1 新增公开 API

```rust
// transaction
pub mod transaction {
    pub struct MVCC;
    pub struct TransactionManager;
    pub struct LockManager;
    pub struct TransactionId;
    pub enum IsolationLevel { ReadCommitted, RepeatableRead, Serializable }
}

// plugin
pub mod plugin {
    pub trait Plugin: Send + Sync;
    pub struct PluginManager;
    pub struct PluginContext;
}

// security
pub mod security {
    pub struct TlsConfig;
    pub struct Authenticator;
    pub struct PermissionManager;
}

// memory
pub mod memory {
    pub struct MemoryPool;
    pub struct MemoryQuota;
}
```

### 4.2 配置变更

```toml
[transaction]
isolation_level = "read_committed"
deadlock_timeout_ms = 1000

[plugin]
enabled = true
plugin_dirs = ["./plugins"]

[security]
tls_enabled = false
tls_cert = "cert.pem"
tls_key = "key.pem"
auth_enabled = false

[memory]
query_memory_limit_mb = 1024
spill_threshold_mb = 512
```

---

## 五、性能目标

| 指标 | v1.4.0 基线 | v1.5.0 目标 | 说明 |
|------|-------------|-------------|------|
| 事务吞吐量 | N/A | 10K TPS | 单节点 |
| 锁冲突率 | N/A | <5% | 高并发场景 |
| 内存使用 | 稳定 | 可预测 | 带配额限制 |
| 插件加载 | N/A | <100ms | 动态加载 |

---

## 六、测试计划

### 6.1 事务测试

- MVCC 隔离级别测试
- 死锁检测测试
- 并发写入冲突测试

### 6.2 插件测试

- 插件加载/卸载测试
- 存储插件接口测试
- UDF 插件测试

### 6.3 集成测试

- 事务 + 查询集成测试
- 插件 + 存储集成测试
- 内存配额压力测试

---

## 七、文档更新

### 7.1 用户文档

- 事务使用指南
- 插件开发指南
- 安全配置手册

### 7.2 开发者文档

- MVCC 设计文档
- 插件 API 参考
- 内存管理架构

---

## 八、发布检查清单

- [ ] cargo build --workspace
- [ ] cargo test --workspace
- [ ] cargo clippy -- -D warnings
- [ ] 覆盖率 ≥82%
- [ ] 事务 ACID 测试通过
- [ ] 插件系统集成测试
- [ ] 安全审计 (可选)
- [ ] API 文档更新
- [ ] CHANGELOG.md 更新

---

## 九、预计时间线

| 阶段 | 周 | 日期 | 主要任务 |
|------|-----|------|----------|
| 规划 | 1 | 05/11-05/17 | MVCC 设计 |
| MVCC | 2-3 | 05/18-05/31 | MVCC 核心实现 |
| 锁管理 | 4 | 06/01-06/07 | 锁管理器 |
| 插件 | 5-6 | 06/08-06/21 | 插件系统 |
| 内存/安全 | 7 | 06/22-06/28 | 内存管理和安全 |
| 测试 | 8 | 06/29-07/05 | 测试和修复 |
| 优化 | 9 | 07/06-07/12 | 性能调优 |
| 发布 | 10 | 07/13-07/20 | 文档和发布 |

> 注: 如时间紧张，P2 功能可推迟到 v1.5.x 补丁版本

---

## 十、风险评估

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| MVCC 实现复杂度 | 高 | 高 | 先实现 SI 简化版 |
| 插件安全风险 | 中 | 高 | 沙箱隔离 |
| 安全审计不足 | 中 | 高 | 引入外部审查 |
| 进度延期 | 中 | 中 | P2 功能可调整 |

---

## 十一、与 v2.0 的衔接

v1.5.0 是 v2.0 分布式内核的前置版本：

- **事务层**: MVCC 为分布式事务奠定基础
- **插件系统**: 存储插件接口可扩展到分布式存储
- **内存管理**: 为跨节点内存调度提供参考

v2.0 将在 v1.5.0 基础上引入：
- 节点抽象和集群管理
- 分布式执行
- 日志复制和共识

---

**文档状态**: 草稿  
**创建人**: AI Assistant  
**审核人**: 待定
