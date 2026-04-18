# MVCC SSI 实现设计文档

> **版本**: v2.6.0
> **日期**: 2026-04-18
> **状态**: 已批准
> **实现方案**: 方案 A - 最小化实现

## 1. 概述

### 1.1 目标

实现 MVCC SSI (Serializable Snapshot Isolation) 隔离级别，提供串行化事务支持而无需严格锁。

### 1.2 SSI 原理

SSI (Serializable Snapshot Isolation) 通过检测"危险结构"来防止串行化异常：

```
危险结构 = 事务 T1 和 T2 之间存在：
- T1 读取的键被 T2 写入 (T1 -> T2)
- T2 读取的键被 T1 写入 (T2 -> T1)

如果形成循环 T1 -> T2 -> T1，则必须回滚其中一个事务
```

### 1.3 现有组件

| 文件 | 组件 | 状态 |
|------|------|------|
| `mvcc.rs` | Transaction, Snapshot, RowVersion | 已有 |
| `lock_manager.rs` | DistributedLockManager | 已有 |
| `version_chain.rs` | VersionChainMap | 已有 |

## 2. 架构设计

### 2.1 新增文件

```
crates/transaction/src/
└── ssi.rs          # SSI 检测器 (新建)
```

### 2.2 SSI 检测器结构

```rust
/// SSI 检测器 - 检测串行化冲突
pub struct SsiDetector {
    /// 事务读集: tx_id -> Set<键>
    read_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    /// 事务写集: tx_id -> Set<键>
    write_sets: RwLock<HashMap<TxId, HashSet<Vec<u8>>>>,
    /// 活跃事务锁
    locks: Arc<DistributedLockManager>,
}

impl SsiDetector {
    /// 记录读取的键 (SIREAD 锁)
    pub async fn record_read(&self, tx_id: TxId, key: Vec<u8>);

    /// 记录写入的键 (获取 X 锁)
    pub async fn record_write(&self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError>;

    /// 验证提交 - 检测危险结构
    pub async fn validate_commit(&self, tx_id: TxId) -> Result<(), SsiError>;

    /// 释放事务的所有锁
    pub async fn release(&self, tx_id: TxId);
}
```

### 2.3 错误类型

```rust
/// SSI 冲突错误
#[derive(Debug, Clone)]
pub enum SsiError {
    /// 串行化冲突 - 需要回滚
    SerializationConflict {
        our_tx: TxId,
        conflicting_tx: TxId,
        reason: String,
    },
    /// 锁超时
    LockTimeout,
}
```

## 3. 数据流

### 3.1 读取流程

```
BEGIN TRANSACTION (SERIALIZABLE)
    ↓
创建 SsiDetector 快照
    ↓
SELECT 读取数据
    ↓
record_read(tx_id, key)  # 记录 SIREAD 锁
    ↓
返回结果
```

### 3.2 写入流程

```
UPDATE/INSERT/DELETE
    ↓
record_write(tx_id, key)  # 尝试获取 X 锁
    ↓
更新版本链
    ↓
返回结果
```

### 3.3 提交流程

```
COMMIT
    ↓
validate_commit(tx_id)  # SSI 检测
    ↓
无冲突？ → commit_versions() → 释放锁 → SUCCESS
    ↓
有冲突？ → rollback_versions() → 释放锁 → SsiError
```

## 4. SSI 检测算法

### 4.1 validate_commit 伪代码

```
function validate_commit(tx_id):
    read_set = read_sets.get(tx_id)
    write_set = write_sets.get(tx_id)

    for each (other_tx, other_write_set) in write_sets:
        if other_tx == tx_id or other_tx is committed:
            continue

        # 检查其他事务是否读取了当前事务写入的键
        other_read_set = read_sets.get(other_tx)
        if other_read_set ∩ write_set is not empty:
            # 发现 RW 冲突
            # 检查是否形成循环依赖
            if has_cycle(tx_id, other_tx):
                return SERIALIZATION_CONFLICT

    return OK
```

### 4.2 循环检测

```
has_cycle(T1, T2):
    # T1 读取的键被 T2 写入
    # T2 读取的键被 T1 写入
    # 形成 T1 -> T2 -> T1 循环

    T1_reads = read_sets[T1]
    T2_writes = write_sets[T2]

    if T1_reads ∩ T2_writes is not empty:
        T2_reads = read_sets[T2]
        T1_writes = write_sets[T1]

        if T2_reads ∩ T1_writes is not empty:
            return TRUE  # 危险结构！

    return FALSE
```

## 5. 与 Storage Engine 集成

### 5.1 MVCC Storage 接口扩展

```rust
// 在 TransactionalExecutor 或 StorageEngine 中添加：

pub trait SsiAware {
    fn set_ssi_detector(&mut self, detector: Arc<SsiDetector>);

    async fn read_for_update(&self, key: &[u8]) -> Result<Option<Vec<u8>>, SsiError>;
}
```

### 5.2 FileStorage 集成点

```rust
impl StorageEngine for FileStorage {
    async fn read(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        let snapshot = self.create_snapshot();

        // 记录 SSI 读取
        if let Some(ref detector) = self.ssi_detector {
            let key = encode_key(table, key);
            detector.record_read(current_tx_id(), key).await?;
        }

        // ... 原有逻辑
    }

    async fn write(&self, table: &str, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        // 记录 SSI 写入
        if let Some(ref detector) = self.ssi_detector {
            let key = encode_key(table, key);
            detector.record_write(current_tx_id(), key).await?;
        }

        // ... 原有逻辑
    }
}
```

## 6. 测试计划

### 6.1 单元测试

| 测试用例 | 描述 |
|----------|------|
| test_ssi_no_conflict | 无冲突时提交成功 |
| test_ssi_write_write_conflict | WW 冲突检测 |
| test_ssi_read_write_conflict | RW 冲突检测 |
| test_ssi_dangerous_structure | 危险结构检测 |
| test_ssi_cycle_detection | 循环依赖检测 |
| test_ssi_rollback | 冲突时正确回滚 |
| test_ssi_concurrent_reads | 并发读取无冲突 |
| test_ssi_lock_timeout | 锁超时处理 |

### 6.2 集成测试

| 测试用例 | 描述 |
|----------|------|
| test_ssi_file_storage | 与 FileStorage 集成 |
| test_ssi_wal_storage | 与 WalStorage 集成 |
| test_ssi_concurrent_transactions | 并发 SSI 事务 |

## 7. 后续版本计划

### v2.7.0 - PredicateLock

- 实现谓词锁支持
- 支持 WHERE 条件锁定
- 改进冲突检测精度

### v2.8.0 - ConflictMatrix

- 冲突矩阵优化
- 减少检测时间复杂度
- 支持并行验证

### v2.9.0 - ParallelValidation

- 并行 SSI 验证
- 分布式 SSI 支持
- 性能优化

## 8. 风险和缓解

| 风险 | 缓解措施 |
|------|----------|
| 性能开销 | 后续版本优化 |
| 漏检冲突 | 行级锁保守策略 |
| 死锁 | 锁超时机制 |

## 9. 验收标准

- [x] SSI 检测器实现完成
- [x] SIREAD 锁记录读取键
- [x] SerializationGraph 依赖图
- [x] 提交时检测读写冲突
- [x] 冲突时返回 SsiError
- [x] 与 Storage Engine 集成
- [x] 单元测试覆盖率 >80%
