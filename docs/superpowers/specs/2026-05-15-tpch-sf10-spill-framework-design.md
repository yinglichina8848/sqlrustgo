# TPC-H SF=10 无 OOM 溢出管理框架设计

**Issue**: #987 - [P0] perf: TPC-H SF=10 22/22 无 OOM
**日期**: 2026-05-15
**状态**: 设计中
**作者**: AI (brainstorming with user)

---

## 1. 概述

### 1.1 目标
优化 TPC-H SF=10 执行（约 6000 万行 lineitem + 关联表），确保 22/22 查询无 OOM，适配 64GB+ 内存环境。

### 1.2 验收条件
- [ ] TPC-H SF=10 22/22 通过
- [ ] 无 OOM 错误
- [ ] 64GB 内存限制下正常运行

---

## 2. 架构设计

### 2.1 核心组件

```
┌─────────────────────────────────────────────────────────────┐
│                    SpillManager                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ Partition    │  │ Fallback    │  │ Adaptive            │ │
│  │ Manager      │  │ Manager     │  │ Memory Tracker      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
    ┌───────────┐   ┌───────────┐   ┌───────────┐
    │SortSpill  │   │HashJoinSpill│  │AggSpill   │
    │Operator   │   │Operator    │  │Operator   │
    └───────────┘   └───────────┘   └───────────┘
```

### 2.2 SpillingIterator Trait

```rust
/// 统一溢出迭代器抽象
pub trait SpillingIterator: Iterator {
    /// 开始溢出到磁盘
    fn start_spill(&mut self) -> Result<(), SpillError>;

    /// 检查是否正在溢出
    fn is_spilling(&self) -> bool;

    /// 获取当前分区数
    fn num_partitions(&self) -> usize;

    /// 标记完成并清理临时文件
    fn finish_spill(&mut self);
}

/// 溢出错误类型
#[derive(Debug, thiserror::Error)]
pub enum SpillError {
    #[error("磁盘空间不足: {available} bytes available, {required} bytes required")]
    OutOfDiskSpace { available: u64, required: u64 },

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("内存限制达到，无法降级: {0}")]
    FallbackFailed(String),
}
```

### 2.3 内存追踪器

```rust
pub struct AdaptiveMemoryTracker {
    /// 当前内存使用
    current_bytes: AtomicU64,
    /// 溢出阈值
    spill_threshold: usize,
    /// 内存限制
    memory_limit: usize,
    /// 降级标志
    fallback_mode: AtomicBool,
}

impl AdaptiveMemoryTracker {
    /// 记录内存分配
    pub fn allocate(&self, bytes: usize) -> bool;

    /// 记录内存释放
    pub fn deallocate(&self, bytes: usize);

    /// 检查是否需要溢出
    pub fn should_spill(&self) -> bool;

    /// 检查是否可以尝试降级
    pub fn can_fallback(&self) -> bool;
}
```

---

## 3. 详细设计

### 3.1 自适应分区策略

**分区触发条件**：
- 当前批次大小 >= `spill_threshold`
- 累积行数 >= 自适应阈值（基于数据类型估算）

**分区大小计算**：
```rust
fn calculate_partition_size(element_size: usize, available_memory: usize) -> usize {
    let rows_per_partition = available_memory / element_size;
    // 添加 10% 安全边际
    (rows_per_partition * 9) / 10
}
```

**数据类型估算**：
| 类型 | 估算大小 |
|------|----------|
| lineitem | 150-200 bytes/row |
| orders | 80-100 bytes/row |
| customer | 200-250 bytes/row |
| part | 150-200 bytes/row |

### 3.2 Sort Spill 实现

```rust
pub struct SortSpillOperator<T: Sortable> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    current_partition: Vec<T>,
    spilled_runs: Vec<SpilledRun>,
    comparator: fn(&T, &T) -> Ordering,
}

impl<T: Sortable> SpillingIterator for SortSpillOperator<T> {
    fn start_spill(&mut self) -> Result<(), SpillError> {
        if self.tracker.can_fallback() {
            // 尝试降级到内存模式
            return self.try_fallback();
        }

        // 排序当前分区并写入磁盘
        self.current_partition.sort_by(self.comparator);
        let run_id = self.partition_manager.write_run(&self.current_partition)?;
        self.spilled_runs.push(run_id);
        self.current_partition.clear();
        Ok(())
    }
}
```

### 3.3 Hash Join Spill 实现

```rust
pub struct HashJoinSpillOperator {
    build_partition: HashMap<K, Vec<V>>,
    probe_iterator: Box<dyn Iterator<Item = (K, V)>>,
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
}

impl HashJoinSpillOperator {
    /// 分区哈希连接溢出策略
    fn spill_build_side(&mut self) -> Result<(), SpillError> {
        // 当 build 分区超过阈值时，溢出到磁盘
        // 使用 Grace Hash Join 策略
    }
}
```

### 3.4 Aggregate Spill 实现

```rust
pub struct AggregateSpillOperator {
    groups: HashMap<GroupKey, AggregatedState>,
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
}

impl AggregateSpillOperator {
    fn spill_groups(&mut self) -> Result<(), SpillError> {
        // 当 group 数量超过阈值时，溢出部分 group 到磁盘
    }
}
```

### 3.5 降级管理

```rust
pub struct FallbackManager {
    original_memory_limit: usize,
    attempts: AtomicUsize,
    max_attempts: usize,
}

impl FallbackManager {
    /// 尝试降级到纯内存模式
    pub fn try_fallback(&self, tracker: &AdaptiveMemoryTracker) -> Result<(), SpillError> {
        if self.attempts.load(Ordering::SeqCst) >= self.max_attempts {
            return Err(SpillError::FallbackFailed(
                "已达最大降级尝试次数".into()
            ));
        }
        self.attempts.fetch_add(1, Ordering::SeqCst);
        // 扩展内存限制
        Ok(())
    }
}
```

---

## 4. 会话配置集成

### 4.1 环境变量

```bash
# 内存限制（字节）
SQLRUSTGO_MAX_MEMORY_PER_QUERY=68719476736  # 64GB

# 溢出阈值（字节）
SQLRUSTGO_SPILL_THRESHOLD=4294967296  # 4GB

# 溢出目录
SQLRUSTGO_SPILL_DIR=/tmp/sqlrustgo_spill

# 降级最大尝试次数
SQLRUSTGO_FALLBACK_MAX_ATTEMPTS=3
```

### 4.2 SessionConfig 扩展

```rust
impl SessionConfig {
    pub fn with_spill_config(
        mut self,
        max_memory: usize,
        spill_threshold: usize,
        spill_dir: String,
    ) -> Self {
        self.max_memory_per_query = max_memory;
        self.spill_to_disk_threshold = spill_threshold;
        self.spill_dir = Some(spill_dir);
        self
    }
}
```

---

## 5. 优化查询列表

### 5.1 优先级 D: PoC (Q1)
首先实现 spill 框架，验证 Q1 可行性。

### 5.2 优先级 C: Profiling
运行 SF=1 profiling，确认内存峰值排序。

### 5.3 优先级 B: 核心决策支持查询
优化: Q1, Q3, Q5, Q9, Q21

### 5.4 优先级 A: 全部 22 个查询
全面优化所有查询。

---

## 6. 文件结构

```
crates/spill/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── trait.rs           # SpillingIterator trait
│   ├── memory_tracker.rs  # AdaptiveMemoryTracker
│   ├── partition_manager.rs
│   ├── fallback_manager.rs
│   ├── operators/
│   │   ├── mod.rs
│   │   ├── sort_spill.rs
│   │   ├── hash_join_spill.rs
│   │   └── aggregate_spill.rs
│   └── error.rs
└── tests/
    └── spill_test.rs
```

---

## 7. 测试计划

### 7.1 单元测试
- MemoryTracker 分配/释放
- 分区大小计算
- 降级触发逻辑

### 7.2 集成测试
- TPC-H Q1 SF=0.1 (PoC)
- TPC-H Q1 SF=1 (基准)
- TPC-H SF=10 (目标)

### 7.3 内存压测
- 验证 64GB 限制不 OOM
- 验证降级机制有效

---

## 8. 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 溢出性能退化 | 使用 mmap 优化 IO |
| 降级失败导致 OOM | 保守的阈值设置 |
| 磁盘空间不足 | 预检查磁盘空间 |
| 分区不均衡 | 自适应分区策略 |
