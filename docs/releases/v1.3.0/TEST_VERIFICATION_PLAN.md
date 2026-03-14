# v1.3.0 测试验证计划

> **版本**: 1.0
> **创建日期**: 2026-03-15
> **目标**: 确保新增算子和可观测性功能的测试覆盖

---

## 一、测试覆盖矩阵

### 1.1 算子测试覆盖

| 算子 | 单元测试 | 集成测试 | 覆盖率目标 | 当前状态 |
|------|---------|---------|-----------|----------|
| TableScan | 3 | 2 | ≥80% | 🔶 需补充 |
| Projection | 2 | 2 | ≥80% | 🔶 需补充 |
| Filter | 2 | 2 | ≥80% | 🔶 需补充 |
| HashJoin | 2 | 3 | ≥80% | 🔶 需补充 |
| Aggregate | 2 | 2 | ≥80% | 🔶 需补充 |
| Sort | 0 | 0 | ≥60% | ❌ 需新增 |
| Limit | 0 | 0 | ≥60% | ❌ 需新增 |

### 1.2 可观测性测试覆盖

| 功能 | 单元测试 | 集成测试 | 当前状态 |
|------|---------|---------|----------|
| HealthChecker | 10 | 2 | 🔶 需补充 |
| BufferPoolMetrics | 6 | 0 | 🔶 需补充 |
| /metrics 端点 | 0 | 0 | ❌ 待 M-004 |
| Prometheus 格式 | 0 | 0 | ❌ 待 M-003 |

---

## 二、测试用例清单

### 2.1 TableScan 算子测试

```rust
// 目标: 15 个测试用例

// 单元测试 (10)
#[test]
fn test_tablescan_empty_table() { }                    // 空表扫描
#[test]
fn test_tablescan_with_projection() { }               // 列投影
#[test]
fn test_tablescan_with_filter() { }                   // 带过滤条件
#[test]
fn test_tablescan_with_limit() { }                   // 带 LIMIT
#[test]
fn test_tablescan_with_offset() { }                  // 带 OFFSET
#[test]
fn test_tablescan_order_by() { }                      // 排序扫描
#[test]
fn test_tablescan_null_values() { }                   // NULL 值处理
#[test]
fn test_tablescan_large_dataset() { }                 // 大数据集
#[test]
fn test_tablescan_schema_mismatch() { }                // Schema 不匹配
#[test]
fn test_tablescan_concurrent_read() { }               // 并发读取

// 集成测试 (5)
#[test]
fn test_tablescan_with_real_storage() { }              // 真实存储
#[test]
fn test_tablescan_index_usage() { }                   // 索引使用
#[test]
fn test_tablescan_performance() { }                    // 性能基准
#[test]
fn test_tablescan_error_handling() { }                 // 错误处理
#[test]
fn test_tablescan_transaction_isolation() { }          // 事务隔离
```

### 2.2 Projection 算子测试

```rust
// 目标: 12 个测试用例

// 单元测试 (8)
#[test]
fn test_projection_single_column() { }                 // 单列投影
#[test]
fn test_projection_multiple_columns() { }              // 多列投影
#[test]
fn test_projection_with_alias() { }                   // 列别名
#[test]
fn test_projection_with_expression() { }               // 表达式投影
#[test]
fn test_projection_with_function() { }                 // 函数投影
#[test]
fn test_projection_null_handling() { }                // NULL 处理
#[test]
fn test_projection_type_coercion() { }                // 类型转换
#[test]
fn test_projection_duplicate_columns() { }             // 重复列

// 集成测试 (4)
#[test]
fn test_projection_with_aggregate() { }                // 与聚合结合
#[test]
fn test_projection_join_result() { }                  // 连接结果投影
#[test]
fn test_projection_performance() { }                   // 性能基准
#[test]
fn test_projection_complex_expression() { }            // 复杂表达式
```

### 2.3 Filter 算子测试

```rust
// 目标: 15 个测试用例

// 单元测试 (10)
#[test]
fn test_filter_simple_condition() { }                 // 简单条件
#[test]
fn test_filter_and_condition() { }                    // AND 条件
#[test]
fn test_filter_or_condition() { }                     // OR 条件
#[test]
fn test_filter_not_condition() { }                    // NOT 条件
#[test]
fn test_filter_comparison_operators() { }            // 比较运算符
#[test]
fn test_filter_null_condition() { }                   // NULL 条件
#[test]
fn test_filter_in_list() { }                          // IN 列表
#[test]
fn test_filter_between() { }                          // BETWEEN
#[test]
fn test_filter_like() { }                             // LIKE
#[test]
fn test_filter_complex_expression() { }               // 复杂表达式

// 集成测试 (5)
#[test]
fn test_filter_with_projection() { }                   // 与投影结合
#[test]
fn test_filter_with_join() { }                       // 与连接结合
#[test]
fn test_filter_pushdown() { }                        // 谓词下推
#[test]
fn test_filter_performance() { }                      // 性能基准
#[test]
fn test_filter_early_termination() { }               // 提前终止
```

### 2.4 HashJoin 算子测试

```rust
// 目标: 20 个测试用例

// 单元测试 (12)
#[test]
fn test_hash_join_inner_join() { }                     // 内连接
#[test]
fn test_hash_join_left_join() { }                     // 左连接
#[test]
fn test_hash_join_right_join() { }                    // 右连接
#[test]
fn test_hash_join_full_outer_join() { }               // 全外连接
#[test]
fn test_hash_join_cross_join() { }                    // 交叉连接
#[test]
fn test_hash_join_multiple_keys() { }                 // 多列连接
#[test]
fn test_hash_join_null_keys() { }                     // NULL 键处理
#[test]
fn test_hash_join_duplicate_keys() { }                // 重复键处理
#[test]
fn test_hash_join_empty_tables() { }                  // 空表连接
#[test]
fn test_hash_join_mismatched_schema() { }             // Schema 不匹配
#[test]
fn test_hash_join_data_type_mismatch() { }             // 类型不匹配
#[test]
fn test_hash_join_build_probe_phases() { }            // 构建和探测阶段

// 集成测试 (8)
#[test]
fn test_hash_join_performance() { }                   // 性能基准
#[test]
fn test_hash_join_large_datasets() { }                // 大数据集
#[test]
fn test_hash_join_memory_usage() { }                  // 内存使用
#[test]
fn test_hash_join_spill_to_disk() { }                 // 溢出到磁盘
#[test]
fn test_hash_join_concurrent() { }                    // 并发连接
#[test]
fn test_hash_join_with_aggregation() { }               // 与聚合结合
#[test]
fn test_hash_join_with_subquery() { }                 // 与子查询结合
#[test]
fn test_hash_join_repartition() { }                   // 重分区
```

### 2.5 Aggregate 算子测试

```rust
// 目标: 18 个测试用例

// 单元测试 (12)
#[test]
fn test_aggregate_count() { }                          // COUNT
#[test]
fn test_aggregate_sum() { }                            // SUM
#[test]
fn test_aggregate_avg() { }                           // AVG
#[test]
fn test_aggregate_min() { }                           // MIN
#[test]
fn test_aggregate_max() { }                           // MAX
#[test]
fn test_aggregate_group_by() { }                      // GROUP BY
#[test]
fn test_aggregate_multiple_groups() { }               // 多列分组
#[test]
fn test_aggregate_with_having() { }                   // HAVING
#[test]
fn test_aggregate_null_handling() { }                 // NULL 处理
#[test]
fn test_aggregate_distinct() { }                      // DISTINCT
#[test]
fn test_aggregate_window_function() { }               // 窗口函数
#[test]
fn test_aggregate_complex_expression() { }             // 复杂表达式

// 集成测试 (6)
#[test]
fn test_aggregate_with_join() { }                     // 与连接结合
#[test]
fn test_aggregate_performance() { }                   // 性能基准
#[test]
fn test_aggregate_large_groups() { }                  // 大分组
#[test]
fn test_aggregate_streaming() { }                     // 流式聚合
#[test]
fn test_aggregate_hash_vs_sort() { }                  // 哈希 vs 排序
#[test]
fn test_aggregate_memory_usage() { }                  // 内存使用
```

### 2.6 可观测性测试

```rust
// HealthChecker 补充测试 (10)
#[test]
fn test_health_checker_with_storage_component() { }   // 存储组件健康
#[test]
fn test_health_checker_with_executor_component() { }  // 执行器组件健康
#[test]
fn test_health_checker_degraded_status() { }           // 降级状态
#[test]
fn test_health_checker_unhealthy_status() { }          // 不健康状态
#[test]
fn test_health_checker_latency_tracking() { }         // 延迟追踪
#[test]
fn test_health_checker_concurrent_checks() { }        // 并发检查
#[test]
fn test_component_health_serialization() { }          // 序列化
#[test]
fn test_health_report_json_format() { }               // JSON 格式
#[test]
fn test_health_status_transitions() { }               // 状态转换
#[test]
fn test_health_checker_uptime() { }                   // 运行时间

// BufferPoolMetrics 补充测试 (10)
#[test]
fn test_metrics_query_type_breakdown() { }             // 按类型统计
#[test]
fn test_metrics_error_rate_calculation() { }           // 错误率计算
#[test]
fn test_metrics_latency_histogram() { }                // 延迟直方图
#[test]
fn test_metrics_cache_hit_ratio() { }                 // 缓存命中率
#[test]
fn test_metrics_concurrent_recording() { }            // 并发记录
#[test]
fn test_metrics_reset_functionality() { }             // 重置功能
#[test]
fn test_metrics_thread_safety() { }                   // 线程安全
#[test]
fn test_metrics_prometheus_format() { }               // Prometheus 格式
#[test]
fn test_metrics_memory_overhead() { }                  // 内存开销
#[test]
fn test_metrics_integration_with_executor() { }        // 与执行器集成
```

---

## 三、执行计划

### 3.1 阶段划分

| 阶段 | 时间 | 任务 | 目标测试数 |
|------|------|------|-----------|
| Phase 1 | 第1周 | 算子单元测试 | 50+ |
| Phase 2 | 第2周 | 算子集成测试 | 30+ |
| Phase 3 | 第3周 | 可观测性测试 | 20+ |
| Phase 4 | 第4周 | 性能基准测试 | 20+ |

### 3.2 测试优先级

1. **P0 (必须)**: HashJoin, Filter, Aggregate 核心测试
2. **P1 (重要)**: TableScan, Projection 完整测试
3. **P2 (可选)**: Sort, Limit, 性能基准测试

### 3.3 验收标准

- [ ] 所有 P0 测试通过 (100%)
- [ ] 所有 P1 测试通过 (≥90%)
- [ ] 整体测试覆盖率 ≥80%
- [ ] 无性能回归

---

## 四、质量保证

### 4.1 测试规范

```bash
# 测试命名规范
test_<module>_<scenario>_<expected_result>

# 示例
test_hash_join_inner_join_returns_matching_rows
test_filter_null_condition_returns_empty_result
```

### 4.2 代码审查

- 所有测试必须经过 PR 审查
- 至少 1 人批准
- 检查测试覆盖率报告

### 4.3 持续集成

```yaml
# .github/workflows/test.yml
- name: Run Unit Tests
  run: cargo test --workspace

- name: Run Integration Tests
  run: cargo test --workspace --test integration

- name: Check Coverage
  run: cargo tarpaulin --workspace
```

---

## 五、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 测试用例遗漏 | 高 | 使用矩阵检查确保覆盖 |
| 测试不稳定 | 中 | 修复 flaky tests |
| 性能回归 | 高 | 自动化性能基准测试 |
| 覆盖不足 | 中 | 定期检查覆盖率报告 |

---

## 六、跟踪机制

| 指标 | 目标 | 当前 |
|------|------|------|
| 算子单元测试 | 50+ | ~20 |
| 算子集成测试 | 30+ | ~10 |
| 可观测性测试 | 20+ | ~16 |
| 总体测试数 | 100+ | ~50 |

---

**文档状态**: 草稿
**创建人**: AI Assistant
**审核人**: 待定
