# SQLRustGo 2.0 风险矩阵

> **版本**: 2.0  
> **更新日期**: 2026-03-05  
> **状态**: 风险可控

---

## 一、风险等级定义

| 等级 | 说明 | 应对策略 |
|------|------|----------|
| 🔴 高 | 必须控制，否则影响项目成功 | 立即处理 |
| 🟠 中 | 需要关注，可能影响进度 | 计划处理 |
| 🟡 低 | 可接受，后期迭代 | 监控即可 |

---

## 二、高风险（必须控制）

### 2.1 Trait 设计不稳定

| 项目 | 说明 |
|------|------|
| **风险** |Executor/Storage trait 接口频繁变更|
| **概率** | 中 |
| **影响** | 高 - 影响所有实现 |
| **应对策略** | Phase0 阶段冻结核心 Trait |

**缓解措施**:

```rust
// 冻结的核心 Trait
pub trait Executor: Send {
    fn open(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
    fn next(&mut self, ctx: &dyn ExecutionContext) -> Result<Option<RecordBatch>>;
    fn close(&mut self, ctx: &dyn ExecutionContext) -> Result<()>;
}

// 扩展通过组合实现
pub trait ExecutorExt: Executor {
    fn statistics(&self) -> ExecutorStatistics;
}
```

### 2.2 HashJoin 内存爆炸

| 项目 | 说明 |
|------|------|
| **风险** | 大表 Join 导致内存溢出 |
| **概率** | 中 |
| **影响** | 高 - 系统崩溃 |
| **应对策略** | 提前设计 spill 接口 |

**缓解措施**:

```rust
pub trait Spillable {
    fn can_spill(&self) -> bool;
    fn spill_to_disk(&mut self) -> Result<()>;
    fn load_from_disk(&mut self) -> Result<()>;
}

pub struct HashJoinExecutor {
    build_side: HashMap<JoinKey, Vec<RecordBatch>>,
    spill_threshold: usize,
    spill_file: Option<PathBuf>,
}
```

### 2.3 协议不可扩展

| 项目 | 说明 |
|------|------|
| **风险** | MySQL 协议实现硬编码，无法扩展 |
| **概率** | 中 |
| **影响** | 高 - 无法支持新功能 |
| **应对策略** | 协议中增加版本字段 |

**缓解措施**:

```rust
pub struct ProtocolHeader {
    pub version: u8,
    pub flags: u16,
    pub reserved: u32,
}

pub const PROTOCOL_VERSION_V1: u8 = 1;
pub const PROTOCOL_VERSION_V2: u8 = 2;
```

### 2.4 并发死锁

| 项目 | 说明 |
|------|------|
| **风险** | 多线程访问共享资源导致死锁 |
| **概率** | 低 |
| **影响** | 高 - 系统挂起 |
| **应对策略** | 避免共享可变状态 |

**缓解措施**:

```rust
// 使用消息传递替代共享状态
pub struct StorageActor {
    receiver: mpsc::Receiver<StorageMessage>,
}

pub enum StorageMessage {
    Scan { table: String, reply: oneshot::Sender<Result<RecordBatch>> },
    Insert { table: String, data: RecordBatch, reply: oneshot::Sender<Result<()>> },
}
```

---

## 三、中风险

### 3.1 优化器规则冲突

| 项目 | 说明 |
|------|------|
| **风险** | 多个优化规则相互冲突 |
| **概率** | 中 |
| **影响** | 中 - 查询结果错误 |
| **应对策略** | 控制规则执行顺序 |

**缓解措施**:

```rust
pub struct Optimizer {
    rules: Vec<Box<dyn OptimizationRule>>,
    rule_order: RuleOrder,
}

pub enum RuleOrder {
    Sequential,
    Priority(Vec<RulePriority>),
}

pub struct RulePriority {
    pub rule_name: String,
    pub priority: u32,
    pub dependencies: Vec<String>,
}
```

### 3.2 性能低于预期

| 项目 | 说明 |
|------|------|
| **风险** | 实现性能未达到设计目标 |
| **概率** | 中 |
| **影响** | 中 - 用户体验差 |
| **应对策略** | 提前加入 Benchmark 测试 |

**缓解措施**:

```rust
// CI 中运行基准测试
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_scan(c: &mut Criterion) {
        c.bench_function("scan_1m_rows", |b| {
            b.iter(|| {
                let executor = ScanExecutor::new(table);
                while let Some(batch) = executor.next(ctx).unwrap() {
                    black_box(batch);
                }
            })
        });
    }
}
```

### 3.3 错误模型混乱

| 项目 | 说明 |
|------|------|
| **风险** | 错误类型不统一，难以处理 |
| **概率** | 中 |
| **影响** | 中 - 调试困难 |
| **应对策略** | 统一 Error 枚举 |

**缓解措施**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SqlError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}
```

---

## 四、低风险

### 4.1 CLI 功能不丰富

| 项目 | 说明 |
|------|------|
| **风险** | 命令行工具功能简单 |
| **概率** | 高 |
| **影响** | 低 - 非核心功能 |
| **应对策略** | 后期迭代 |

### 4.2 日志系统基础

| 项目 | 说明 |
|------|------|
| **风险** | 日志功能简单，缺少结构化日志 |
| **概率** | 高 |
| **影响** | 低 - 可在未来增强 |
| **应对策略** | 后期增强 |

---

## 五、风险监控

### 5.1 监控指标

| 指标 | 阈值 | 说明 |
|------|------|------|
| Trait 变更次数 | < 3 次/月 | 接口稳定性 |
| 内存使用峰值 | < 80% 堆内存 | 内存安全 |
| 死锁检测 | 0 次 | 并发安全 |
| 性能退化 | < 10% | 性能稳定 |

### 5.2 风险评审周期

| 周期 | 内容 |
|------|------|
| 每日 | CI 检查结果 |
| 每周 | 高风险项状态 |
| 每月 | 风险矩阵更新 |

---

## 六、应急响应

### 6.1 高风险触发响应

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          高风险触发响应流程                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   发现高风险问题                                                             │
│        │                                                                     │
│        ▼                                                                     │
│   立即暂停相关开发                                                           │
│        │                                                                     │
│        ▼                                                                     │
│   评估影响范围                                                               │
│        │                                                                     │
│        ├── 影响核心 Trait ──► 召开架构评审会议                              │
│        │                                                                     │
│        ├── 影响内存模型 ──► 设计替代方案                                    │
│        │                                                                     │
│        └── 影响协议 ──► 评估兼容性                                          │
│                                                                              │
│   实施修复                                                                   │
│        │                                                                     │
│        ▼                                                                     │
│   验证修复效果                                                               │
│        │                                                                     │
│        ▼                                                                     │
│   更新风险矩阵                                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 联系人

| 风险类型 | 负责人 |
|----------|--------|
| Trait 设计 |开放的心|
| 内存管理 |敞开心扉|
| 网络协议 |开放的心|
| 并发安全 |维护者|

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-05 | 初始版本 |
