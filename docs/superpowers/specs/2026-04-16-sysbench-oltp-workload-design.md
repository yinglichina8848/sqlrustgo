# Sysbench OLTP 工作负载实现设计

> 日期: 2026-04-16
> Issue: #1424
> 状态: Approved

## 1. 概述

为 SQLRustGo 实现完整的 Sysbench 风格 OLTP 工作负载，覆盖 point_select, index_scan, range_scan, insert, update, delete, mixed, write_only 等场景。

## 2. 现有实现

| 工作负载 | 文件 | 状态 |
|----------|------|------|
| oltp_point_select | `workload/oltp_point_select.rs` | 框架完成，execute 待实现 |
| oltp_read_only | `workload/oltp_read_only.rs` | 框架完成，execute 待实现 |
| oltp_read_write | `workload/oltp_read_write.rs` | 框架完成，execute 待实现 |

## 3. 需要实现的工作负载

| 工作负载 | SQL 模式 | 操作比例 |
|----------|----------|----------|
| oltp_index_scan | `SELECT * FROM sbtest WHERE id BETWEEN ? AND ?` | SELECT 100% |
| oltp_range_scan | `SELECT * FROM sbtest WHERE k BETWEEN ? AND ?` | SELECT 100% |
| oltp_insert | `INSERT INTO sbtest (id, k, c, pad) VALUES (?, ?, ?, ?)` | INSERT 100% |
| oltp_update_index | `UPDATE sbtest SET k=? WHERE id=?` | UPDATE 100% |
| oltp_update_non_index | `UPDATE sbtest SET c=? WHERE id=?` | UPDATE 100% |
| oltp_delete | `DELETE FROM sbtest WHERE id=?` | DELETE 100% |
| oltp_mixed | 组合 SELECT/UPDATE/INSERT/DELETE | 70/20/5/5 |
| oltp_write_only | UPDATE/INSERT/DELETE | 50/25/25 |

## 4. 架构

### 4.1 文件结构

```
crates/bench/src/workload/
├── mod.rs                     # 导出所有工作负载
├── oltp_point_select.rs     # ✅ 已有
├── oltp_read_only.rs        # ✅ 已有
├── oltp_read_write.rs       # ✅ 已有
├── oltp_index_scan.rs       # ❌ 需创建
├── oltp_range_scan.rs       # ❌ 需创建
├── oltp_insert.rs           # ❌ 需创建
├── oltp_update_index.rs     # ❌ 需创建
├── oltp_update_non_index.rs # ❌ 需创建
├── oltp_delete.rs          # ❌ 需创建
├── oltp_mixed.rs           # ❌ 需创建
└── oltp_write_only.rs     # ❌ 需创建
```

### 4.2 Database Adapter

```rust
#[async_trait]
pub trait Database: Send + Sync {
    async fn execute(&self, sql: &str) -> Result<()>;
    async fn read(&self, key: usize) -> Result<()>;
    async fn update(&self, key: usize) -> Result<()>;
    async fn insert(&self, key: usize) -> Result<()>;
    async fn scan(&self, start: usize, end: usize) -> Result<()>;
}
```

## 5. 工作负载规格

### 5.1 oltp_index_scan

```rust
pub struct OltpIndexScan {
    max_id: u64,
    range_size: u64,
}

impl OltpIndexScan {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let start = rng.gen_range(1..self.max_id);
        let end = start + self.range_size;
        format!("SELECT id, k FROM sbtest WHERE id BETWEEN {} AND {}", start, end)
    }
}
```

### 5.2 oltp_range_scan

```rust
pub struct OltpRangeScan {
    max_id: u64,
    range_size: u64,
}

impl OltpRangeScan {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let start = rng.gen_range(1..self.max_id);
        let end = start + self.range_size;
        format!("SELECT * FROM sbtest WHERE k BETWEEN {} AND {}", start, end)
    }
}
```

### 5.3 oltp_insert

```rust
pub struct OltpInsert {
    max_id: u64,
}

impl OltpInsert {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let k = rng.gen_range(1..100000);
        let c = generate_random_string(120);
        let pad = generate_random_string(60);
        format!("INSERT INTO sbtest (id, k, c, pad) VALUES ({}, {}, '{}', '{}')", id, k, c, pad)
    }
}
```

### 5.4 oltp_update_index

```rust
pub struct OltpUpdateIndex {
    max_id: u64,
}

impl OltpUpdateIndex {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let k = rng.gen_range(1..100000);
        format!("UPDATE sbtest SET k = {} WHERE id = {}", k, id)
    }
}
```

### 5.5 oltp_update_non_index

```rust
pub struct OltpUpdateNonIndex {
    max_id: u64,
}

impl OltpUpdateNonIndex {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        let c = generate_random_string(120);
        format!("UPDATE sbtest SET c = '{}' WHERE id = {}", c, id)
    }
}
```

### 5.6 oltp_delete

```rust
pub struct OltpDelete {
    max_id: u64,
}

impl OltpDelete {
    fn generate_sql(&self, rng: &mut SmallRng) -> String {
        let id = rng.gen_range(1..self.max_id);
        format!("DELETE FROM sbtest WHERE id = {}", id)
    }
}
```

### 5.7 oltp_mixed

```rust
pub struct OltpMixed {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpMixed {
    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String> {
        // 70% SELECT, 20% UPDATE, 5% INSERT, 5% DELETE
        (0..self.statements_per_tx)
            .map(|_| {
                let op = rng.gen_range(0..100);
                match op {
                    0..70 => self.generate_select(rng),   // 70%
                    70..90 => self.generate_update(rng),  // 20%
                    90..95 => self.generate_insert(rng),  // 5%
                    _ => self.generate_delete(rng),       // 5%
                }
            })
            .collect()
    }
}
```

### 5.8 oltp_write_only

```rust
pub struct OltpWriteOnly {
    max_id: u64,
    statements_per_tx: usize,
}

impl OltpWriteOnly {
    fn generate_transaction(&self, rng: &mut SmallRng) -> Vec<String> {
        // 50% UPDATE, 25% INSERT, 25% DELETE
        (0..self.statements_per_tx)
            .map(|_| {
                let op = rng.gen_range(0..100);
                match op {
                    0..50 => self.generate_update(rng),  // 50%
                    50..75 => self.generate_insert(rng), // 25%
                    _ => self.generate_delete(rng),      // 25%
                }
            })
            .collect()
    }
}
```

## 6. 实施计划

### Task 1: 实现 oltp_index_scan
- 创建 `oltp_index_scan.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 2: 实现 oltp_range_scan
- 创建 `oltp_range_scan.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 3: 实现 oltp_insert
- 创建 `oltp_insert.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 4: 实现 oltp_update_index
- 创建 `oltp_update_index.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 5: 实现 oltp_update_non_index
- 创建 `oltp_update_non_index.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 6: 实现 oltp_delete
- 创建 `oltp_delete.rs`
- 实现 generate_sql 和 generate_transaction
- 实现 execute 方法
- 添加单元测试

### Task 7: 实现 oltp_mixed
- 创建 `oltp_mixed.rs`
- 实现 generate_transaction (多操作组合)
- 实现 execute 方法
- 添加单元测试

### Task 8: 实现 oltp_write_only
- 创建 `oltp_write_only.rs`
- 实现 generate_transaction (写操作组合)
- 实现 execute 方法
- 添加单元测试

### Task 9: 更新 mod.rs
- 导出所有新工作负载
- 更新 create_workload 工厂函数

## 7. 验收标准

| 工作负载 | generate_sql | generate_transaction | execute |
|----------|-------------|-------------------|---------|
| oltp_index_scan | ✅ | ✅ | ✅ |
| oltp_range_scan | ✅ | ✅ | ✅ |
| oltp_insert | ✅ | ✅ | ✅ |
| oltp_update_index | ✅ | ✅ | ✅ |
| oltp_update_non_index | ✅ | ✅ | ✅ |
| oltp_delete | ✅ | ✅ | ✅ |
| oltp_mixed | ✅ | ✅ | ✅ |
| oltp_write_only | ✅ | ✅ | ✅ |
| oltp_point_select | ✅ | ✅ | ✅ |
| oltp_read_only | ✅ | ✅ | ✅ |
| oltp_read_write | ✅ | ✅ | ✅ |

所有工作负载单元测试通过。
