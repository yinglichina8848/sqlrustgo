# SQLRustGo v1.5.0 开发计划 (DEVELOPMENT_PLAN.md)

> **版本**: v1.5.0 Draft
> **阶段**: 开发中 (Craft)
> **更新日期**: 2026-03-15
> **目标**: 事务与插件系统，整合内存管理和安全特性

---

## 一、版本概述

### 1.1 主题

**事务与插件系统**

### 1.2 核心目标

v1.5.0 是 v2.0 分布式内核前的最后一个大版本，主要完成：
1. MVCC 基础实现（快照隔离）
2. 事务隔离级别支持
3. 插件系统完整实现
4. 内存管理基础
5. 安全增强（TLS、认证）

### 1.3 预计周期

8-10 周 (2026-05-11 ~ 2026-07-20)

---

## 二、前置依赖

### 2.1 来自 v1.4.0

| 功能 | 状态 | 说明 |
|------|------|------|
| CBO 成本优化器 | ⏳ | 统计信息系统已就绪 |
| SortMergeJoin | ⏳ | 连接算法已完善 |
| Prometheus 监控 | ⏳ | 指标系统已建立 |

### 2.2 外部依赖

- **无外部强依赖**: 事务和插件系统主要依赖内部模块

---

## 三、功能列表

### 3.1 P0 - 必须完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| TX-01 | transaction | MVCC 基础实现 | - | 快照隔离 (SI) |
| TX-02 | transaction | 事务管理器 | TX-01 | 事务生命周期管理 |
| TX-03 | transaction | 隔离级别支持 | TX-01 | RC, Serializable |
| LK-01 | transaction | 锁管理器 | TX-02 | 行锁和表锁 |
| LK-02 | transaction | 死锁检测 | LK-01 | 防止死锁 |
| PL-01 | plugin | Plugin trait 完善 | - | 插件接口定义 |
| PL-02 | plugin | 插件加载器 | PL-01 | 动态加载 .so 文件 |
| PL-03 | plugin | 存储插件接口 | PL-01 | 自定义存储后端 |
| PL-04 | plugin | UDF 插件支持 | PL-01 | 用户定义函数 |

### 3.2 P1 - 应该完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| MM-01 | memory | 内存池管理 | - | 内存分配器 |
| MM-02 | memory | 内存配额 | MM-01 | 查询内存限制 |
| MM-03 | memory | Spill to Disk | MM-02 | 大数据量溢出到磁盘 |
| SEC-01 | security | TLS/SSL 支持 | - | 传输加密 |
| SEC-02 | security | 认证机制 | SEC-01 | 用户密码认证 |
| SEC-03 | security | 权限控制 | SEC-02 | RBAC 基础 |

### 3.3 P2 - 可选完成

| ID | 模块 | 任务 | 依赖 | 目标 |
|----|------|------|------|------|
| TX-04 | transaction | 两阶段提交 (2PC) | TX-03 | 分布式事务基础 |
| PL-05 | plugin | 插件生命周期 | PL-02 | init/start/stop |
| OBS-01 | observability | 告警规则 | - | Prometheus 告警 |

---

## 四、模块设计

### 4.1 MVCC 实现

```rust
pub struct MVCC {
    tx_manager: Arc<TransactionManager>,
    version_store: Arc<VersionStore>,
}

impl MVCC {
    pub fn begin(&self) -> TransactionId;
    pub fn read(&self, tx_id: TransactionId, key: &Key) -> Option<Value>;
    pub fn write(&self, tx_id: TransactionId, key: &Key, value: Value) -> Result<()>;
    pub fn commit(&self, tx_id: TransactionId) -> Result<()>;
    pub fn abort(&self, tx_id: TransactionId) -> Result<()>;
}
```

### 4.2 事务隔离级别

| 隔离级别 | 实现方式 | 状态 |
|----------|----------|------|
| Read Committed (RC) | 每次读取最新快照 | ⏳ |
| Repeatable Read (RR) | 事务开始快照 | ⏳ |
| Serializable | 序列化执行 | ⏳ |

### 4.3 插件系统

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&self, ctx: &PluginContext) -> Result<()>;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load(&mut self, path: &Path) -> Result<()>;
    pub fn unload(&mut self, name: &str) -> Result<()>;
}
```

### 4.4 内存管理

```rust
pub struct MemoryPool {
    arena: Arena,
    quota: usize,
    used: AtomicUsize,
}

impl MemoryPool {
    pub fn allocate(&self, size: usize) -> Result<*mut u8>;
    pub fn spill(&self) -> Result<()>;
    pub fn limit(&self) -> usize;
}
```

---

## 五、覆盖率要求

### 5.1 目标矩阵

| 模块 | 当前 | 目标 | 优先级 |
|------|------|------|--------|
| 整体 | 80% | ≥82% | P0 |
| transaction | 0% | ≥70% | P0 |
| plugin | 0% | ≥60% | P1 |
| memory | 0% | ≥50% | P1 |

### 5.2 新增测试估算

| 模块 | 需新增测试 |
|------|-----------|
| MVCC | 80+ |
| 锁管理器 | 40+ |
| 插件系统 | 30+ |
| 内存管理 | 20+ |

---

## 六、门禁检查 (Gate Checklist)

### 6.1 构建门禁

| 检查项 | 目标 |
|--------|------|
| cargo build --workspace | ✅ |
| cargo test --workspace | 100% |
| cargo clippy -- -D warnings | 零警告 |
| cargo fmt --all -- --check | 通过 |

### 6.2 覆盖率门禁

| 模块 | 目标 |
|------|------|
| 整体 | ≥82% |
| transaction | ≥70% |
| plugin | ≥60% |

### 6.3 功能门禁

| ID | 检查项 | 说明 |
|----|--------|------|
| TX-01 | MVCC 基础 | 快照隔离可用 |
| TX-02 | 事务管理器 | 事务生命周期管理 |
| TX-03 | 隔离级别 | RC 和 Serializable |
| LK-01 | 锁管理器 | 行锁和表锁 |
| PL-01 | Plugin trait | 插件接口定义 |
| PL-02 | 插件加载器 | 动态加载 |

---

## 七、风险与缓解

### 7.1 技术风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| MVCC 实现复杂度 | 🔴 高 | 先实现快照隔离，简化版本 |
| 插件系统安全 | 🔴 高 | 沙箱隔离，权限控制 |
| 内存管理稳定性 | ⚠️ 中 | 充分测试，渐进实现 |

### 7.2 进度风险

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| 工作量超出预期 | ⚠️ 中 | P2 功能可推迟到 v1.5.x |
| 安全审计不足 | 🔴 高 | 引入外部安全审查 |

---

## 八、评审记录

| 日期 | AI/工具 | 评估结论 |
|------|---------|----------|
| 2026-03-15 | AI Assistant | 初始版本，基于 v1.3.0/v1.4.0 规划 |

---

## 九、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-15 | 初始版本 |

---

**文档状态**: 草稿  
**创建人**: AI Assistant  
**审核人**: 待定
