# ISSUE E-08 Step 2: Hash Join 性能优化

## 问题描述

### 背景

E-08 TPC-H QPS 优化分为多个步骤：
- **Step 1**: TPC-H 基准测试建立 (已完成)
- **Step 2**: Hash Join 优化 (当前任务)
- **Step 3**: 迭代器扫描优化
- **Step 4**: 谓词下推优化

### Step 2 目标

当前 Hash Join 实现为 O(n×m) 嵌套循环算法，在 TPC-H 查询中性能严重不达标。

| 查询 | 当前耗时 | 目标耗时 | 差距 |
|------|----------|----------|------|
| Q8 | ~400ms | ~50ms | 8x |
| Q9 | ~350ms | ~50ms | 7x |
| Q5 | ~300ms | ~50ms | 6x |
| Q6 | ~250ms | ~30ms | 8x |

---

## 根因分析

### 当前实现 (`local_executor.rs:985-996`)

```rust
fn hash_inner_join_rows(
    left: &[Vec<Value>],
    right: &[Vec<Value>],
    _condition: &Expr,
) -> Vec<Vec<Value>> {
    let mut results = Vec::new();
    for left_row in left {
        for right_row in right {
            results.push(merge_rows(left_row, right_row));  // O(n × m)
        }
    }
    results
}
```

### 问题

1. **O(n×m) 时间复杂度**: 150K × 600K = 900亿次比较
2. **无 HashMap 索引**: 每次都遍历右表
3. **无 build/probe 优化**: 应该用小表作为 build side

---

## 解决方案

### 实现真正的 Hash Join O(n+m)

```rust
fn hash_inner_join_rows(
    left: &[Vec<Value>],
    right: &[Vec<Value>],
    condition: &Expr,
    left_schema: &Schema,
    right_schema: &Schema,
) -> Vec<Vec<Value>> {
    // 1. 选择小表作为 build side
    let (build, probe) = if left.len() < right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let is_swapped = left.len() >= right.len();

    // 2. 构建 HashMap
    let mut hash: HashMap<Vec<Value>, Vec<&[Value]>> = HashMap::new();
    for row in build {
        let key = extract_join_key(row, condition, left_schema, right_schema, is_swapped);
        hash.entry(key).or_default().push(row);
    }

    // 3. 探测
    let mut results = Vec::new();
    for row in probe {
        let key = extract_join_key(row, condition, left_schema, right_schema, is_swapped);
        if let Some(matches) = hash.get(&key) {
            for m in matches {
                if is_swapped {
                    results.push(merge_rows(*m, row));
                } else {
                    results.push(merge_rows(row, *m));
                }
            }
        }
    }
    results
}
```

### 关键优化点

| 优化 | 效果 |
|------|------|
| 小表做 build | 减少 hash 冲突 |
| HashMap 查找 O(1) | 总复杂度 O(n+m) |
| 引用而非克隆 | 减少内存分配 |
| 惰性求值 | 延迟计算直到需要时 |

---

## 实施步骤

### Phase 1: 基础设施

1. **T1.1**: 提取 join key 函数
   - `extract_join_key(row, condition, schema, is_swapped) -> Vec<Value>`
   - 支持单列和多列 join

2. **T1.2**: 合并行函数优化
   - 使用引用避免克隆
   - 支持 swapped 顺序

### Phase 2: Hash Join 实现

3. **T2.1**: 实现 `hash_inner_join_rows`
   - build/probe 优化
   - HashMap 构建
   - 探测阶段

4. **T2.2**: 支持其他 Join 类型
   - LEFT JOIN: 保留未匹配左行
   - RIGHT JOIN: 保留未匹配右行
   - FULL OUTER JOIN: 两者都保留
   - CROSS JOIN: 笛卡尔积

### Phase 3: 集成测试

5. **T3.1**: 修改现有 join 测试验证正确性

6. **T3.2**: TPC-H QPS 验证
   - Q8, Q9, Q5, Q6 达标

---

## 关键文件

| 文件 | 修改 |
|------|------|
| `crates/executor/src/local_executor.rs` | 重写 `hash_inner_join_rows` |
| `tests/executor_join_test.rs` | 添加 Hash Join 测试 |
| `crates/executor/src/join_utils.rs` | 新增 join 工具函数 |

---

## 验证方案

```bash
# 1. 功能测试
cargo test --package sqlrustgo-executor --test join_tests

# 2. TPC-H QPS
cargo run --bin bench-cli -- tpch bench --queries Q5,Q6,Q8,Q9 --iterations 3 --sf 0.1

# 3. 目标
| 查询 | 目标 |
|------|------|
| Q5 | ≤50ms |
| Q6 | ≤30ms |
| Q8 | ≤50ms |
| Q9 | ≤50ms |
```

---

## 风险

- **兼容性**: 确保现有 JOIN 测试继续通过
- **内存**: HashMap 可能占用大量内存，需要考虑 spill-to-disk (后续优化)

---

## 依赖

- Step 1 建立的 TPC-H 基准测试
- 无其他外部依赖
