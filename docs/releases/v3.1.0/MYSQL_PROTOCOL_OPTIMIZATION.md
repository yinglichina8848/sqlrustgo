# MySQL 协议性能优化分析

> **任务编号**: TASK-OPT-001
> **来源**: GA-12 Sysbench 失败分析
> **优先级**: P1
> **计划版本**: v3.1.0

---

## 问题描述

GA-12 Sysbench 测试失败，实测 QPS 远低于阈值：

| 测试类型 | 实测 QPS | 阈值 | 状态 |
|---------|----------|------|------|
| point_select | 1,688 | 30,000 | ❌ FAIL |
| oltp_read_write | 71 | 10,000 | ❌ FAIL |
| oltp_write_only | 190 | 8,000 | ❌ FAIL |
| update_index | 468 | 8,000 | ❌ FAIL |

**根本原因**: Sysbench QPS (1,688) 比 cargo bench (183,918) 慢 **109 倍**，瓶颈在 MySQL 协议层，而非 SQL 执行引擎。

### 性能对比

| 测试类型 | Cargo Bench (直接执行) | Sysbench (MySQL协议) | 差距倍数 |
|---------|----------------------|---------------------|---------|
| simple_select | 183,918 QPS | 1,688 QPS | **109x** |
| join | 33,534 QPS | N/A | - |
| aggregation | 499,148 QPS | N/A | - |
| update | 24,977 QPS | 468 QPS | 53x |
| delete | 49,131 QPS | N/A | - |
| insert | 10,446 QPS | N/A | - |

---

## 已识别的性能瓶颈

### 1. `Packet::write_to` 强制 flush

**位置**: `crates/mysql-server/src/lib.rs:786-792`

```rust
pub fn write_to<W: Write>(&self, w: &mut W) -> MySqlResult<()> {
    w.write_u24::<LittleEndian>(self.length)?;
    w.write_u8(self.sequence)?;
    w.write_all(&self.payload)?;
    w.flush()?;  // <-- 每次调用都 flush，性能瓶颈
    Ok(())
}
```

**问题**: 每个数据包都调用 `flush()`，强制内核立即发送数据，触发 Nagle 算法失效。

**影响**: 每个查询产生 3-10 个数据包（握手、查询、结果），每个都单独 flush。

---

### 2. `is_select()` 重复 uppercase

**位置**: `crates/mysql-server/src/lib.rs:1650-1656`

```rust
fn is_select(sql: &str) -> bool {
    sql.trim().to_uppercase().starts_with("SELECT")
}
```

**问题**: 每次调用创建新字符串，执行数万次查询时累积大量分配。

---

### 3. `extract_table_name()` 重复 uppercase

**位置**: `crates/mysql-server/src/lib.rs:1531-1551`

```rust
fn extract_table_name(sql: &str) -> Option<String> {
    // 调用 .to_uppercase() 两次
    let upper = sql.to_uppercase();
    // 再次使用 upper 做匹配...
}
```

**问题**: 对同一字符串进行两次 uppercase 操作。

---

### 4. `send_result_set()` 多次 write_to

**位置**: `crates/mysql-server/src/lib.rs:1138-1197`

```rust
fn send_result_set(&mut self, rows: &[Vec<Value>], columns: &[ColumnDefinition]) -> MySqlResult<()> {
    // 发送列定义包 (每个列一次 write_to + flush)
    for col in columns {
        packet.write_to(&mut self.stream)?;  // flush!
    }
    // 发送行数据 (每行一次 write_to + flush)
    for row in rows {
        packet.write_to(&mut self.stream)?;  // flush!
    }
}
```

**问题**: 100 行结果集 = 100+ 次 flush 调用。

---

### 5. `replace_placeholders()` 字符串分配

**位置**: `crates/mysql-server/src/lib.rs:1515-1529`

```rust
fn replace_placeholders(sql: &str, params: &[Value]) -> String {
    // 每次调用创建新 String
    let mut result = String::with_capacity(sql.len() * 2);
    // 字符串拼接...
    result
}
```

---

## 优化方案

### 方案 1: 移除强制 flush（预计提升 50-100%）

**修改位置**: `Packet::write_to`

```rust
// Before
pub fn write_to<W: Write>(&self, w: &mut W) -> MySqlResult<()> {
    w.write_u24::<LittleEndian>(self.length)?;
    w.write_u8(self.sequence)?;
    w.write_all(&self.payload)?;
    w.flush()?;  // 删除这行
    Ok(())
}

// After
pub fn write_to<W: Write>(&self, w: &mut W) -> MySqlResult<()> {
    w.write_u24::<LittleEndian>(self.length)?;
    w.write_u8(self.sequence)?;
    w.write_all(&self.payload)?;
    Ok(())  // 不 flush，由调用者批量 flush
}

// 新增批量写入方法
pub fn write_all_to<W: Write>(packets: &[Packet], w: &mut W) -> MySqlResult<()> {
    for pkt in packets {
        pkt.write_to(w)?;
    }
    w.flush()?;  // 只在最后 flush 一次
    Ok(())
}
```

**修改位置**: `send_result_set` 等方法改为批量发送后统一 flush。

---

### 方案 2: 缓存 uppercase 表名（预计提升 10-20%）

```rust
use std::collections::HashMap;
use std::sync::RwLock;

static TABLE_NAME_CACHE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());

fn cached_uppercase(s: &str) -> String {
    if let Ok(cache) = TABLE_NAME_CACHE.read() {
        if let Some(cached) = cache.get(s) {
            return cached.clone();
        }
    }
    let upper = s.to_uppercase();
    if let Ok(mut cache) = TABLE_NAME_CACHE.write() {
        cache.insert(s.to_string(), upper.clone());
    }
    upper
}
```

---

### 方案 3: 使用栈分配替代堆分配（预计提升 20-30%）

```rust
// Before: String 分配
fn build_packet_header(length: u32, sequence: u8) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4);
    // ...
}

// After: 栈分配
fn build_packet_header(length: u32, sequence: u8, buf: &mut [u8; 4]) {
    // 直接写入固定大小数组
}
```

---

## 验收条件

| 指标 | 当前值 | 目标值 |
|------|--------|--------|
| Sysbench point_select | 1,688 QPS | ≥ 30,000 QPS |
| Sysbench overhead ratio | 109x | < 5x |

**验收测试**:
```bash
# Sysbench 验证
bash scripts/gate/check_sysbench.sh

# 或 cargo bench 验证（如果 GA-12 已切换）
cargo test --test qps_benchmark_test -- --ignored
```

---

## 影响范围

### 修改的文件

| 文件 | 修改类型 |
|------|----------|
| `crates/mysql-server/src/lib.rs` | 重构 `Packet::write_to`, `send_result_set` |
| `crates/mysql-server/src/packets.rs` | 新增批量写入方法 |

### 相关测试

| 测试 | 验证内容 |
|------|----------|
| `mysql_protocol_handshake_test` | 握手协议兼容 |
| `qps_benchmark_test` | QPS 无回归 |
| `integration_tests` | 整体功能正常 |

---

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 移除 flush 导致数据滞留 | 低 | flush 只在批量发送时调用，确保数据最终发送 |
| 缓存占用过多内存 | 低 | 使用 LRU 缓存，限制大小 |
| 破坏协议兼容性 | 中 | 使用 mysql_protocol_handshake_test 验证 |

---

## 实施计划

### Phase 1: 快速修复（预计 2 小时）

1. 移除 `Packet::write_to` 中的 `flush()`
2. 在 `send_result_set` 末尾添加单次 `flush()`
3. 运行 `mysql_protocol_handshake_test` 验证兼容性

### Phase 2: 优化缓存（预计 4 小时）

1. 实现 `cached_uppercase()` 函数
2. 替换 `is_select()`, `extract_table_name()` 中的调用
3. 添加 LRU 缓存限制

### Phase 3: 内存优化（预计 8 小时）

1. 分析热路径上的堆分配
2. 使用栈分配或 `smallvec` 替代
3. 运行 `cargo bench` 验证性能提升

---

*本文档由 GA-12 Sysbench 失败分析生成，用于指导 v3.1.0 MySQL 协议优化工作。*
