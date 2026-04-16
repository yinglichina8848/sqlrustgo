# SQLRustGo v2.5.0 覆盖率提升实施计划

> **目标:** 将覆盖率从 49% 提升至 70%+
> **周期:** 6-8周
> **策略:** C → B → A 优先级渐进式提升

---

## 一、当前覆盖率数据

| 指标 | 值 |
|------|------|
| 总体覆盖率 | 49.0% (10,375/21,186) |
| 未覆盖行数 | ~10,800 |
| 测试总数 | 2363+ |

---

## 二、覆盖率提升目标

| 阶段 | 目标覆盖率 | 周期 | 优先级 |
|------|-----------|------|--------|
| Phase 1 | 49% → 55% | 1-2周 | C: 快速见效 |
| Phase 2 | 55% → 62% | 2-3周 | B: 风险高发 |
| Phase 3 | 62% → 70% | 3-4周 | A: 核心功能 |

---

## 三、Phase 1: 快速见效 (C优先级)

### 1.1 Parser覆盖率提升 (88% → 95%)

**目标:** 覆盖剩余 ~1000 行未测试代码

**Files:**
- Modify: `crates/parser/src/parser.rs`
- Modify: `crates/parser/src/expression.rs`
- Modify: `crates/parser/src/lexer.rs`

**任务:**

- [ ] **T1.1: 为 lexer.rs 添加边界测试**
```rust
#[test]
fn test_unicode_identifiers() {
    let tokens = lexer::Tokenizer::new("SELECT 属性, 名称").tokenize();
    assert_eq!(tokens.len(), 4);
}

#[test]
fn test_long_string_literals() {
    let input = format!("SELECT '{}'", "x".repeat(10000));
    let tokens = lexer::Tokenizer::new(&input).tokenize();
    assert!(tokens.len() > 0);
}
```

- [ ] **T1.2: 为 parser.rs 添加错误恢复测试**
```rust
#[test]
fn test_malformed_select_recovery() {
    let sql = "SELECT FROM table";
    let result = parse(sql);
    assert!(result.is_err());
}

#[test]
fn test_unterminated_string() {
    let sql = "SELECT 'unterminated";
    let result = parse(sql);
    assert!(result.is_err());
}
```

- [ ] **T1.3: 为 expression.rs 添加表达式解析测试**
```rust
#[test]
fn test_nested_case_expression() {
    let sql = "SELECT CASE WHEN a = 1 THEN CASE WHEN b = 2 THEN 3 ELSE 4 END ELSE 5 END FROM t";
    let result = parse(sql);
    assert!(result.is_ok());
}
```

### 1.2 Catalog覆盖率提升 (75% → 85%)

**目标:** 覆盖 ~400 行未测试代码

**Files:**
- Modify: `crates/catalog/src/table.rs`
- Modify: `crates/catalog/src/schema.rs`
- Create: `tests/unit/catalog_test.rs`

**任务:**

- [ ] **T1.4: 添加表约束验证测试**
```rust
#[test]
fn test_validate_pk_nullable() {
    let mut table = Table::new("test");
    let col = ColumnDefinition {
        name: "id".to_string(),
        nullable: true, // Primary key cannot be nullable
        is_primary_key: true,
        ..
    };
    let result = table.add_column(col);
    assert!(result.is_err());
}
```

- [ ] **T1.5: 添加模式变更测试**
```rust
#[test]
fn test_schema_add_table() {
    let mut schema = Schema::new("test");
    let table = create_test_table();
    schema.add_table(table).unwrap();
    assert!(schema.get_table("test").is_ok());
}
```

### 1.3 Memory Arena覆盖率提升 (58% → 75%)

**目标:** 覆盖 ~200 行未测试代码

**Files:**
- Modify: `src/memory/reusable_vec.rs`
- Modify: `src/memory/batch_arena.rs`
- Create: `tests/unit/memory_arena_test.rs`

**任务:**

- [ ] **T1.6: 添加 ReusableVec 边界测试**
```rust
#[test]
fn test_reusable_vec_capacity_growth() {
    let mut vec = ReusableVec::new();
    for i in 0..1000 {
        vec.push(i);
    }
    assert!(vec.capacity() >= 1000);
}

#[test]
fn test_reusable_vec_clear_and_reuse() {
    let mut vec = ReusableVec::new();
    vec.push(1);
    vec.clear();
    vec.push(2);
    assert_eq!(vec.len(), 1);
}
```

- [ ] **T1.7: 添加 BatchArena 溢出测试**
```rust
#[test]
fn test_batch_arena_exhaustion() {
    let arena = BatchArena::new(100);
    let result = arena.allocate(200);
    assert!(result.is_err());
}
```

---

## 四、Phase 2: 风险高发 (B优先级)

### 2.1 WAL恢复覆盖率提升 (50% → 70%)

**目标:** 覆盖 ~800 行未测试代码

**Files:**
- Modify: `crates/storage/src/wal.rs`
- Create: `tests/unit/wal_recovery_test.rs`

**任务:**

- [ ] **T2.1: 添加 WAL 损坏恢复测试**
```rust
#[test]
fn test_wal_recovery_from_corruption() {
    let dir = TempDir::new().unwrap();
    let mut wal = WalManager::new(dir.path()).unwrap();

    // Write some data
    wal.append(WalEntry::Insert { ... }).unwrap();
    wal.flush().unwrap();

    // Corrupt the WAL file
    std::fs::write(dir.path().join("wal.log"), "corrupted");

    // Recover
    let recovered = wal.recover().unwrap();
    assert!(recovered.is_empty());
}
```

- [ ] **T2.2: 添加 WAL 部分写入恢复测试**
```rust
#[test]
fn test_wal_partial_write_recovery() {
    let dir = TempDir::new().unwrap();
    let mut wal = WalManager::new(dir.path()).unwrap();

    wal.append(WalEntry::Begin { tx_id: 1 }).unwrap();
    wal.append(WalEntry::Insert { tx_id: 1, ... }).unwrap();
    // Simulate crash before commit

    let recovered = wal.recover();
    assert!(recovered.is_empty() || recovered[0].tx_id != 1);
}
```

### 2.2 Distributed 2PC覆盖率提升 (58% → 75%)

**目标:** 覆盖 ~600 行未测试代码

**Files:**
- Modify: `crates/distributed/src/two_phase_commit.rs`
- Create: `tests/unit/two_phase_commit_test.rs`

**任务:**

- [ ] **T2.3: 添加 2PC 超时测试**
```rust
#[test]
fn test_two_phase_commit_timeout() {
    let mut coordinator = TwoPhaseCommit::new();
    coordinator.set_timeout(Duration::from_millis(100));

    let result = coordinator.prepare("node-1").unwrap();
    assert!(result.is_pending());

    // Wait for timeout
    thread::sleep(Duration::from_millis(200));

    let status = coordinator.status("tx-1");
    assert_eq!(status, TransactionStatus::Aborted);
}
```

- [ ] **T2.4: 添加 2PC 协调器失败测试**
```rust
#[test]
fn test_coordinator_failure_recovery() {
    let mut coordinator = TwoPhaseCommit::new();
    coordinator.prepare("node-1").unwrap();
    coordinator.prepare("node-2").unwrap();

    // Simulate coordinator crash
    drop(coordinator);

    // New coordinator should be able to recover
    let new_coordinator = TwoPhaseCommit::recover();
    assert!(new_coordinator.has_pending_transaction("tx-1"));
}
```

### 2.3 Buffer Pool淘汰策略测试 (55% → 70%)

**目标:** 覆盖 ~400 行未测试代码

**Files:**
- Modify: `crates/storage/src/buffer_pool.rs`
- Create: `tests/unit/buffer_pool_eviction_test.rs`

**任务:**

- [ ] **T2.5: 添加 LRU 淘汰测试**
```rust
#[test]
fn test_buffer_pool_lru_eviction() {
    let pool = BufferPool::new(3);
    pool.pin(0, page(0));
    pool.pin(1, page(1));
    pool.pin(2, page(2));

    // Access page 0 to make it recently used
    pool.access(0);

    // Add new page, should evict page 1 (least recently used)
    pool.pin(3, page(3));

    assert!(pool.get(0).is_some()); // Still in pool
    assert!(pool.get(1).is_none());  // Evicted
}
```

---

## 五、Phase 3: 核心功能 (A优先级)

### 3.1 Storage Engine覆盖率提升 (55% → 68%)

**目标:** 覆盖 ~1200 行未测试代码

**Files:**
- Modify: `crates/storage/src/engine.rs`
- Create: `tests/integration/storage_engine_test.rs`

**任务:**

- [ ] **T3.1: 添加事务回滚测试**
```rust
#[test]
fn test_transaction_rollback() {
    let storage = StorageEngine::new();
    let tx = storage.begin_transaction().unwrap();

    storage.insert(&tx, "users", row!(1, "Alice")).unwrap();

    // Rollback
    storage.rollback(tx).unwrap();

    // Data should not exist
    let result = storage.get(&tx, "users", 1);
    assert!(result.is_none());
}
```

- [ ] **T3.2: 添加并发插入测试**
```rust
#[test]
fn test_concurrent_insert_same_key() {
    let storage = StorageEngine::new();
    let num_threads = 10;

    let handles: Vec<_> = (0..num_threads)
        .map(|i| {
            thread::spawn(move || {
                let tx = storage.begin_transaction().unwrap();
                storage.insert(&tx, "counters", row!(1, i))
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Only one should succeed
    let successes = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(successes, 1);
}
```

### 3.2 Executor覆盖率提升 (65% → 75%)

**目标:** 覆盖 ~800 行未测试代码

**Files:**
- Modify: `crates/executor/src/executor.rs`
- Create: `tests/integration/executor_path_test.rs`

**任务:**

- [ ] **T3.3: 添加执行器错误路径测试**
```rust
#[test]
fn test_executor_divide_by_zero() {
    let executor = Executor::new();
    let plan = parse("SELECT 1/0 FROM t").unwrap();
    let result = executor.execute(plan);
    assert!(result.is_err());
}
```

- [ ] **T3.4: 添加执行器空表测试**
```rust
#[test]
fn test_executor_empty_table() {
    let executor = Executor::new();
    create_and_populate_table(&executor, "empty_table", 0);

    let plan = parse("SELECT * FROM empty_table").unwrap();
    let result = executor.execute(plan).unwrap();
    assert_eq!(result.rows().len(), 0);
}
```

### 3.3 Optimizer Rules覆盖率提升 (65% → 78%)

**目标:** 覆盖 ~600 行未测试代码

**Files:**
- Modify: `crates/optimizer/src/rules.rs`
- Create: `tests/unit/optimizer_rules_test.rs`

**任务:**

- [ ] **T3.5: 添加谓词下推规则测试**
```rust
#[test]
fn test_predicate_pushdown_join() {
    let optimizer = Optimizer::new();
    let plan = parse("SELECT * FROM a JOIN b ON a.id = b.id WHERE a.x > 5").unwrap();

    let optimized = optimizer.optimize(plan).unwrap();

    // Verify predicate is pushed down to scan
    assert!(has_predicate_on_scan(optimized, "a"));
    assert!(has_predicate_on_scan(optimized, "b"));
}
```

---

## 六、测试命令

### 运行覆盖率测试

```bash
# 单个包覆盖率
cargo tarpaulin -p sqlrustgo-parser --lib --out Html

# 核心包覆盖率
cargo tarpaulin -p sqlrustgo-parser -p sqlrustgo-catalog -p sqlrustgo-storage --lib --out Html

# 全量覆盖率 (CI中运行)
cargo tarpaulin --workspace --lib --tests --out Xml
```

### 验证测试通过

```bash
# 运行所有单元测试
cargo test --lib --workspace

# 运行特定测试
cargo test -p sqlrustgo-parser --lib

# 运行集成测试
cargo test --test regression_test
```

---

## 七、覆盖率目标检查点

| 阶段 | 日期 | 目标覆盖率 | 实际覆盖率 | 状态 |
|------|------|-----------|-----------|------|
| Phase 1 完成 | 2026-04-30 | 55% | TBD | ⏳ |
| Phase 2 完成 | 2026-05-15 | 62% | TBD | ⏳ |
| Phase 3 完成 | 2026-05-31 | 70% | TBD | ⏳ |

---

## 八、关键文件清单

### 需要创建的测试文件

| 文件 | 阶段 | 优先级 |
|------|------|--------|
| `tests/unit/memory_arena_test.rs` | 1 | C |
| `tests/unit/wal_recovery_test.rs` | 2 | B |
| `tests/unit/two_phase_commit_test.rs` | 2 | B |
| `tests/unit/buffer_pool_eviction_test.rs` | 2 | B |
| `tests/integration/storage_engine_test.rs` | 3 | A |
| `tests/integration/executor_path_test.rs` | 3 | A |
| `tests/unit/optimizer_rules_test.rs` | 3 | A |

### 需要修改的测试文件

| 文件 | 阶段 | 优先级 |
|------|------|--------|
| `crates/parser/src/parser.rs` | 1 | C |
| `crates/parser/src/lexer.rs` | 1 | C |
| `crates/catalog/src/table.rs` | 1 | C |
| `crates/storage/src/wal.rs` | 2 | B |
| `crates/distributed/src/two_phase_commit.rs` | 2 | B |
| `crates/storage/src/buffer_pool.rs` | 2 | B |
| `crates/storage/src/engine.rs` | 3 | A |
| `crates/executor/src/executor.rs` | 3 | A |
| `crates/optimizer/src/rules.rs` | 3 | A |

---

*计划创建时间: 2026-04-16*
*计划更新: 2026-04-16*
