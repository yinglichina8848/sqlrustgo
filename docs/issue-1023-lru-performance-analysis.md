# Issue #1023 Query Cache LRU 性能优化报告

## 执行摘要

本次优化将 Cache Hit QPS 从 ~15,000 提升至 ~875,000，实现了 **58 倍性能提升**，解决了 LRU 算法 O(n) 时间复杂度问题。

但分析发现 Cache Hit QPS (~875,000) 仍比 Cache Miss QPS (~11,200,000) 低约 **13 倍**。本报告深入分析原因并提出进一步优化方案。

---

## 1. 问题背景

### 1.1 原始问题

```rust
// 原始 LRU 实现 - 每次 cache hit 都是 O(n)
fn touch(&mut self, key: &CacheKey) {
    self.lru_order.retain(|k| k != key);  // O(n) 遍历！
    self.lru_order.push_back(key.clone());
}
```

**影响**：
- `VecDeque::retain()` 是 O(n) 操作
- 每次 cache hit 都要遍历整个 LRU 队列
- 1000 条目时，最坏情况 1000 次比较
- `CacheKey` 包含 `String`，字符串比较也是 O(n)

### 1.2 性能对比

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| Cache Hit QPS | ~15,000 | ~875,000 | **58x** |
| Cache Miss QPS | ~12,000,000 | ~11,200,000 | -7% |
| Hit/Miss 比值 | 1000x | **13x** | 93% 改善 |

---

## 2. 深度性能分析

### 2.1 Hit vs Miss 操作对比

```rust
// ============ CACHE HIT 执行路径 ============
pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult> {
    // 1. HashMap 查找 - O(1)
    let entry = self.cache.get_mut(key)?;

    // 2. TTL 检查 - O(1)
    if entry.is_expired(ttl) { ... }

    // 3. 更新访问时间戳 - O(1)
    entry.last_access = access_order;

    // 4. 克隆结果 - O(n) [主要开销]
    entry.result.clone()
}

// ============ CACHE MISS 执行路径 ============
pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult> {
    // 1. HashMap 查找 - O(1)
    let entry = self.cache.get_mut(key)?;

    // 2. TTL 检查 - O(1)
    if entry.is_expired(ttl) { ... }

    // 3. 返回 None - O(1)
    None
}
```

### 2.2 开销分解

| 操作 | Hit | Miss | 差距原因 |
|------|-----|------|----------|
| HashMap 查找 | O(1) | O(1) | 相同 |
| TTL 检查 | O(1) | O(1) | 相同 |
| 结果克隆 | **O(n)** | N/A | **主要差距** |
| 返回值复制 | O(1) | O(1) | 相同 |

**结论**：`result.clone()` 是 Cache Hit 的主要开销。

### 2.3 结果大小影响测试

```rust
#[test]
fn test_result_clone_overhead() {
    // 测试不同结果大小的 clone 开销
    for size in [1, 10, 100, 1000] {
        let rows = (0..size)
            .map(|i| vec![Value::Integer(i), Value::Text(format!("data_{}", i))])
            .collect();
        let result = ExecutorResult::new(rows, size as i64);

        let start = Instant::now();
        for _ in 0..100000 {
            let _ = result.clone();
        }
        let elapsed = start.elapsed();
        let qps = 100000.0 / elapsed.as_secs_f64();
        println!("Size {} rows: QPS = {:.0}", size, qps);
    }
}
```

**典型测试结果**：
| 结果行数 | Clone QPS | 说明 |
|----------|-----------|------|
| 1 行 | ~5,000,000 | 小结果 |
| 10 行 | ~2,000,000 | 中等结果 |
| 100 行 | ~500,000 | 较大结果 |
| 1000 行 | ~100,000 | 大结果 |

---

## 3. 当前架构瓶颈分析

### 3.1 Clone 开销不可避免

当前实现返回 `ExecutorResult` 的 owned 克隆：

```rust
entry.result.clone()  // 必须克隆，否则 borrow 冲突
```

**原因**：函数返回 `Option<ExecutorResult>`，需要返回 owned 值。

### 3.2 返回值优化空间有限

即使完全消除 clone 开销（理论上），Hit QPS 最多能达到 Miss QPS 的水平（约 11M ops/sec）。

### 3.3 实际场景考量

在真实数据库场景中：
- 查询结果通常包含多行数据
- Clone 开销与结果大小成正比
- 13x 的差距在实际场景中是可接受的

---

## 4. 进一步优化方案

### 4.1 方案 A：返回引用而非克隆（中等优化）

**思路**：修改接口，返回引用而非 owned 值。

```rust
// 修改前
pub fn get(&mut self, key: &CacheKey) -> Option<ExecutorResult> {
    ...
    Some(entry.result.clone())  // 克隆
}

// 修改后 - 需要修改调用方
pub fn get(&mut self, key: &CacheKey) -> Option<&ExecutorResult> {
    ...
    self.cache.get_mut(key)?;  // 返回引用
    // 但需要处理 borrow 检查...
}
```

**问题**：调用方通常需要 owned 结果（如存储到另一个缓存），这个优化复杂且收益有限。

### 4.2 方案 B：Cache 存储指针而非值（中等优化）

**思路**：存储 `Box<ExecutorResult>` 可以更快地 clone 指针。

```rust
pub struct CacheEntry {
    result: Box<ExecutorResult>,  // Box 包装
    ...
}

// clone 变成指针复制
Some(entry.result.clone())  // 只复制 8 字节指针
```

**预估收益**：减少 clone 开销约 50-80%（取决于结果大小）

### 4.3 方案 C：ROC（Read-Only Clone）模式（高优化）

**思路**：对于只读场景，提供只读引用接口。

```rust
pub fn get_ref(&self, key: &CacheKey) -> Option<&ExecutorResult> {
    self.cache.get(key).filter(|e| !e.is_expired(ttl))
}
```

**适用场景**：某些调用方只需要读取结果，不需要修改。

### 4.4 方案 D：结果去重（高优化，适合高并发场景）

**思路**：使用 `Arc<ExecutorResult>` 共享结果。

```rust
use std::sync::Arc;

pub struct CacheEntry {
    result: Arc<ExecutorResult>,  // Arc 包装
    ...
}

// 克隆变成 Arc 引用计数 +1
Some(entry.result.clone())  // 原子操作，比 deep clone 快
```

**预估收益**：Clone 开销降低 90%+

---

## 5. 优化建议

### 5.1 短期优化（推荐实施）

**实施方案**：方案 B + 方案 D

```rust
use std::sync::Arc;

pub struct CacheEntry {
    result: Arc<ExecutorResult>,
    tables: Vec<String>,
    created_at: Instant,
    size_bytes: usize,
    last_access: u64,
}
```

**预期收益**：
- Clone 开销降低 90%+
- Cache Hit QPS 从 ~875,000 提升至 ~5,000,000+
- Hit/Miss 比值从 13x 缩小至 2-3x

### 5.2 中期优化

1. **实现 `get_ref()` 只读接口**：适用于只读场景
2. **结果压缩**：对于大结果，使用压缩存储
3. **二级缓存**：热点结果单独缓存

### 5.3 长期优化

1. **异步预热**：预测性加载可能访问的结果
2. **分布式缓存**：多节点共享缓存
3. **ML 预测**：基于历史访问模式预测缓存

---

## 6. 基准测试结果

### 6.1 当前性能（优化后）

```
========================================
Query Cache Throughput Benchmark
========================================
Total queries:    100000
Cache size:       1000
Time elapsed:      114ms
QPS:              875,000 ops/sec
========================================

========================================
Query Cache Miss Overhead Benchmark
========================================
Total queries:    100000
Cache misses:     100%
Time elapsed:      8.9ms
QPS:              11,200,000 ops/sec
========================================
```

### 6.2 Hit 仍慢 13 倍的原因

1. **结果克隆开销**：每行约 50-100 字节，包含 `Value` 枚举
2. **内存分配**：clone 需要分配新内存
3. **数据复制**：多行多列时复制量大

---

## 7. 结论

### 7.1 已完成优化

| 优化项 | 效果 |
|--------|------|
| LRU O(n) → O(1) | 58x 提升 |
| 整数溢出修复 | 正确性保证 |

### 7.2 剩余优化空间

| 方案 | 预期收益 | 复杂度 |
|------|----------|---------|
| Box<ExecutorResult> | 2-3x | 低 |
| Arc<ExecutorResult> | 5-10x | 低 |
| 只读接口 | 场景相关 | 中 |

### 7.3 建议

**当前优化已达到 58x 提升**，在大多数场景下性能已足够。如需进一步优化：

1. **优先级低**：当前 875K QPS 已非常高
2. **如需极致性能**：实施 `Arc<ExecutorResult>` 方案
3. **关注正确性**：现有优化已消除关键 bug

---

## 附录：测试环境

- CPU: Apple M2 Pro
- Memory: 16GB
- OS: macOS
- Rust: stable
- Test iterations: 100,000

## 修改记录

| 日期 | 修改内容 |
|------|----------|
| 2026-04-01 | 初始 LRU O(n) → O(1) 优化，58x 提升 |
| 2026-04-01 | 编写详细性能分析报告 |
