# SQLRustGo v1.2.0 测试规划

> **版本**: v1.2.0
> **代号**: Vector Engine
> **目标**: 测试覆盖率 ≥85%
> **日期**: 2026-03-07
> **当前阶段**: alpha/v1.2.0

---

## 1. 当前状态

### 1.1 v1.2.0 开发完成情况

| 轨道 | 任务 | 状态 |
|------|------|------|
| R | 核心接口 (7个) | ✅ 已完成 |
| S | 统计信息 (6个) | ✅ 已完成 |
| C | 向量化执行 (2个) | ✅ 已完成 |
| E | 执行器 (4个) | ✅ 已完成 |
| O | 优化器 (6个) | ✅ 已完成 |
| B | 桥接层 (2个) | ✅ 已完成 |
| A | 性能优化 (4个) | ✅ 已完成 |

**总计**: 31/32 任务已完成

### 1.2 已知问题

- **PR #326 待合并**: `crates/executor/src/executor.rs` 恢复修复仍处于 OPEN，且 `mergeStateStatus=DIRTY`。
- **历史记录保留**: Draft 阶段曾出现编译/依赖冲突，已作为历史信息保留，不删除。

---

## 2. 测试需求分析

### 2.1 v1.2.0 新功能测试

| 模块 | 新增功能 | 测试优先级 | 预计测试数 |
|------|----------|------------|------------|
|**存储引擎**|trait 定义、FileStorage、MemoryStorage| P0 | 20 |
|**记录批次**| 列式数据结构、Array trait | P0 | 25 |
|**统计数据收集器**| 统计收集、持久化、ANALYZE | P0 | 15 |
|**成本模型**| 成本计算、Join优化 | P1 | 15 |
|**本地执行器**| 完整查询流程 | P1 | 20 |
|**缓冲池**| LRU、预取、内存池 | P1 | 15 |
| **WAL** | 组提交、缓冲 | P1 | 10 |
| **B+Tree** | 索引优化 | P1 | 15 |

### 2.2 测试覆盖率目标

| 模块 | 当前覆盖率 | 目标覆盖率 | 差距 |
|------|-----------|------------|------|
|解析器| ~95% | 95% | 0% |
|规划师| ~85% | 90% | 5% |
|优化器| ~60% | 85% | 25% |
|执行人| ~70% | 85% | 15% |
|贮存| ~75% | 85% | 10% |
| **总计** | **~80%** | **85%** | **5%** |

---

## 3. 测试策略

### 3.1 单元测试

```rust
// 示例: StorageEngine trait 测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_engine_read() {
        // 测试 read 方法
    }

    #[test]
    fn test_storage_engine_write() {
        // 测试 write 方法
    }

    #[test]
    fn test_storage_engine_scan() {
        // 测试 scan 方法
    }

    #[test]
    fn test_file_storage_implementation() {
        // 测试 FileStorage 实现
    }

    #[test]
    fn test_memory_storage_implementation() {
        // 测试 MemoryStorage 实现
    }
}
```

### 3.2 集成测试

```rust
// tests/integration_test.rs

#[test]
fn test_full_query_pipeline() {
    // 测试完整查询流程: Parser → Optimizer → Executor → Storage
}

#[test]
fn test_analyze_command() {
    // 测试 ANALYZE 命令
}

#[test]
fn test_cbo_join_optimization() {
    // 测试 CBO Join 优化
}
```

### 3.3 性能测试

```rust
// benches/performance_bench.rs

fn benchmark_1m_row_select(c: &mut Criterion) {
    // 100万行 SELECT 性能测试
    // 目标: <100ms
}

fn benchmark_join(c: &mut Criterion) {
    // Join 性能测试
    // 目标: <1s
}

fn benchmark_memory_usage(c: &mut Criterion) {
    // 内存使用测试
    // 目标: <500MB
}
```

---

## 4. 测试任务分解

### 4.1 StorageEngine 测试 (P0)

| 测试项 | 说明 | 状态 |
|--------|------|------|
| trait 方法覆盖 |read/write/scan/get_stats 全部覆盖| ⏳ |
|FileStorage 实现| 文件读写、持久化 | ⏳ |
|MemoryStorage 实现| 内存存储 | ⏳ |
| 存储切换 | 切换存储实现无影响 | ⏳ |

### 4.2 RecordBatch 测试 (P0)

| 测试项 | 说明 | 状态 |
|--------|------|------|
|数组特征| 全部方法覆盖 | ⏳ |
|Int32数组| 基础类型数组 | ⏳ |
|字符串数组| 字符串数组 | ⏳ |
|RecordBatch 构建| 构建与访问 | ⏳ |

### 4.3 StatsCollector 测试 (P0)

| 测试项 | 说明 | 状态 |
|--------|------|------|
|表格统计| 表级统计 | ⏳ |
|列统计| 列级统计 | ⏳ |
| 统计收集 | 自动收集 | ⏳ |
|ANALYZE 命令| 手动触发 | ⏳ |

### 4.4 CostModel 测试 (P1)

| 测试项 | 说明 | 状态 |
|--------|------|------|
| 扫描成本计算 |SeqScan 与 IndexScan| ⏳ |
| Join 成本计算 | Join 顺序优化 | ⏳ |
| 成本比较 | 选择最优计划 | ⏳ |

### 4.5 LocalExecutor 测试 (P1)

| 测试项 | 说明 | 状态 |
|--------|------|------|
| 完整流程 |解析器 → 执行器| ⏳ |
|SELECT 查询| 基本查询 | ⏳ |
| JOIN 查询 | 多表查询 | ⏳ |
| 聚合查询 |分组依据| ⏳ |

### 4.6 性能基准测试

| 测试项 | 目标 | 状态 |
|--------|------|------|
|100万行 SELECT| <100ms | ⏳ |
| 复杂 JOIN | <1s | ⏳ |
| 聚合查询 | <500ms | ⏳ |
| 内存使用 | <500MB | ⏳ |

---

## 5. 测试工具

### 5.1 覆盖率工具

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率
cargo tarpaulin --all-features --output-dir ./coverage

# 查看 HTML 报告
open coverage/tarpaulin.html
```

### 5.2 性能基准工具

```bash
# 运行基准测试
cargo bench

# 运行特定基准
cargo bench --bench sql_operations
```

### 5.3 集成测试

```bash
# 运行所有测试
cargo test --all-features

# 运行集成测试
cargo test --test integration_test

# 运行特定模块测试
cargo test --lib storage
```

---

## 6. 测试里程碑

### Phase 1: 编译修复 (第1天)

- [ ] 修复 develop/v1.2.0 编译错误
- [ ] 确保 cargo build 通过
- [ ] 确保 cargo test 通过

### Phase 2: 核心测试 (第2-3天)

- [ ] StorageEngine 测试 (20个)
- [ ] RecordBatch 测试 (25个)
- [ ] StatsCollector 测试 (15个)

### Phase 3: 高级测试 (第4-5天)

- [ ] CostModel 测试 (15个)
- [ ] LocalExecutor 测试 (20个)
- [ ] 集成测试 (10个)

### Phase 4: 性能测试 (第6-7天)

- [ ] 100万行性能测试
- [ ] JOIN 性能测试
- [ ] 内存使用测试

### Phase 5: 覆盖率提升 (第8-10天)

- [ ] 覆盖率分析
- [ ] 补充测试
- [ ] 达到 85% 目标

---

## 7. 验收标准

### 7.1 功能测试

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] cargo test --all-features 通过

### 7.2 覆盖率

- [ ] 测试覆盖率 ≥85%
- [ ] 新增模块覆盖率 ≥85%

### 7.3 性能

- [ ] 100万行 SELECT <100ms
- [ ] JOIN <1s
- [ ] 内存使用 <500MB

---

## 8. 相关文档

- [v1.2.0 发布说明](../releases/v1.2.0/RELEASE_NOTES.md)
- [v1.2.0 门禁检查清单](../releases/v1.2.0/RELEASE_GATE_CHECKLIST.md)
- [v1.1.0 性能报告](../releases/v1.1.0/PERFORMANCE_REPORT.md)
