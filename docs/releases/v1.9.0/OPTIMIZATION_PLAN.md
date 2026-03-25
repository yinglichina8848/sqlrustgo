# SQLRustGo 性能优化开发计划

## 背景

v1.9.0 基准测试显示：
- **优势**: 谓词下推 (Q6) 比 SQLite 快 3.27x
- **劣势**: Join 操作比 SQLite 慢 7-13x

## 优化目标

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| Q3 (Join) | 17ms | <5ms | 3x |
| Q10 (复杂Join) | 17ms | <5ms | 3x |
| Q1 (全表扫描) | 1.68ms | <1ms | 1.5x |

## 开发计划

### Phase 1: 流式 Hash Join (P0)

**问题**: 当前全量加载左右表到内存

**解决方案**:
```rust
// 当前实现 (executor.rs)
while let Some(row) = self.right.next()? {
    self.right_hash.entry(key).or_default().push(row);
}

// 改进: 分批处理 + 溢出磁盘
pub struct StreamingHashJoin {
    build_partition: Vec<HashMapPartition>,
    probe_buffer: Vec<RecordBatch>,
    max_memory_per_partition: usize,
}
```

**任务**:
- [ ] 实现分批构建哈希表
- [ ] 实现溢出到磁盘机制
- [ ] 添加内存限制和监控

### Phase 2: Join 重排序 (P0)

**问题**: 固定 join 顺序，未利用统计信息

**解决方案**:
```rust
pub struct JoinReorderOptimizer {
    stats_provider: Arc<StatisticsProvider>,
}

fn optimize_join_order(
    &self, 
    joins: &[JoinNode], 
    stats: &Statistics
) -> Vec<JoinNode> {
    // 选择小表先 join，减少中间结果
}
```

**任务**:
- [ ] 添加表统计信息收集
- [ ] 实现基于代价的 join 重排序
- [ ] 处理复杂 join 依赖

### Phase 3: 查询计划缓存 (P1)

**问题**: 每次查询都重新生成执行计划

**解决方案**:
```rust
pub struct QueryPlanCache {
    cache: DashMap<u64, Arc<CachedPlan>>,
    max_entries: usize,
}

impl QueryPlanCache {
    pub fn get(&self, sql: &str, schema: &str) -> Option<Arc<CachedPlan>> {
        let key = self.compute_key(sql, schema);
        self.cache.get(&key)
    }
}
```

**任务**:
- [ ] 实现计划缓存结构
- [ ] 添加 SQL 规范化
- [ ] 实现 LRU 淘汰策略

### Phase 4: 向量化执行 (P1)

**问题**: 逐行处理，CPU 利用率低

**解决方案**:
```rust
// 改进: 批量处理
pub trait VectorizedExecutor {
    fn next_batch(&mut self) -> SqlResult<Option<RecordBatch>>;
}
```

**任务**:
- [ ] 定义向量化算子接口
- [ ] 实现向量化 Filter
- [ ] 实现向量化 Projection

### Phase 5: 并行执行 (P2)

**任务**:
- [ ] 实现并行扫描
- [ ] 实现并行 Hash Join
- [ ] 添加任务调度器

## 时间线

| Phase | 预估工作量 | 优先级 |
|-------|-----------|--------|
| Phase 1 流式 Hash Join | 2 周 | P0 |
| Phase 2 Join 重排序 | 1 周 | P0 |
| Phase 3 计划缓存 | 1 周 | P1 |
| Phase 4 向量化 | 2 周 | P1 |
| Phase 5 并行执行 | 2 周 | P2 |

## 验收标准

- [ ] Q3 (Join) 性能提升 3x
- [ ] Q10 (复杂Join) 性能提升 3x  
- [ ] 内存使用降低 50%
- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试通过

## 相关 ISSUE

- #XXX: 流式 Hash Join 实现
- #XXX: Join 重排序优化
- #XXX: 查询计划缓存
- #XXX: 向量化执行引擎
