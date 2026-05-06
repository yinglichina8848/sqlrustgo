# v3.0.0 Alpha 阶段整改清单

> **日期**: 2026-05-06
> **基于**: develop/v3.0.0 @ 59974f48
> **来源**: #353 和 #370 进展审计

---

## 一、必须补充的开发工作

### 1.1 CBO 代价模型 & 索引选择

| 项目 | 内容 |
|------|------|
| **现状** | PP-01 只完成优化器规则桥接，`SimpleCostModel` 未接入 planner |
| **必须补** | SimpleCostModel 集成到 planner 的 plan 选择；基于代价的索引选择规则；Join 重排序 |
| **目标** | EXPLAIN 能选择索引扫描而非全表扫描；多表 JOIN 按代价排序 |
| **工时** | 5-7 天 |

### 1.2 TPC-H 内存治理与 OOM 根治

| 项目 | 内容 |
|------|------|
| **现状** | SF=0.1 ~10.9s 全量可跑；SF=1 无报告（曾 OOM） |
| **必须补** | Hash Join / Sort 内存限额 + 落盘；重复扫描消除；EXPLAIN ANALYZE 对齐真实计划 |
| **目标** | SF=1 22/22 无 OOM，p99 < 5s |
| **工时** | 3-5 天 |

### 1.3 连接池/查询缓存/Group Commit 正确性强化

| 项目 | 内容 |
|------|------|
| **现状** | 标记为"已完成"，但无并发压力测试和崩溃恢复验证 |
| **必须补** | 连接池最大连接限制 + 超时 + 泄漏检测；查询缓存 DML 失效测试；Group Commit WAL 崩溃恢复 |
| **工时** | 2-3 天 |

---

## 二、必须执行的测试工作

### 2.1 基础验证层 (Alpha-1)

| 编号 | 门禁 | 目标 |
|------|------|------|
| A-1 | cargo test --all-features --workspace | ≥80% 通过 |
| A-2 | 覆盖率整体 ≥50% | optimizer ≥40%, parser ≥50%, executor ≥45% |
| A-3 | cargo clippy --all-features | 零警告 |
| A-4 | cargo fmt --check | 零差异 |
| A-5 | check_docs_links.sh | 零死链 |
| A-6 | cargo audit | 无已知漏洞 |

### 2.2 功能深度测试 (Alpha-2)

| 功能 | 关键测试点 |
|------|----------|
| INSERT...SELECT | 不同列数, 类型转换, 自增主键, WITH CTE 内联 |
| 窗口函数 (NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE) | PARTITION BY + ORDER BY, NULL, 边界帧 |
| CTE 执行 | 递归深度限制, 多 CTE 引用, 与 JOIN 混用 |
| SERIALIZABLE 隔离级别 | Proof-026 100 并发压力, 幻读测试 |
| EXPLAIN ANALYZE | 代价估算, 行数预测, JSON/tree 格式 |
| INFORMATION_SCHEMA | TABLES/COLUMNS/STATISTICS 与 SHOW 命令等价 |
| SSL/TLS | --ssl-mode=REQUIRED, 证书链 |

### 2.3 性能回归与压力测试 (Alpha-3)

| 测试 | 目标 |
|------|------|
| TPC-H SF=1 全量 22 查询 | p99 < 10s, 无 OOM |
| Sysbench 全场景 | oltp_read_write, oltp_write_only, oltp_update_index |
| 连接池并发压力 8/32/100 线程 | 无连接泄漏 |

### 2.4 混沌工程与稳定性 (Alpha-4)

| 测试 | 目标 |
|------|------|
| kill -9 崩溃恢复 | 重启后数据完整 |
| 长稳测试 30min+ | 无内存泄漏, 无 panic |

---

## 三、行动清单

| 优先级 | 类别 | 任务 | 工时 |
|--------|------|------|------|
| **P0** | 开发 | CBO 代价模型 + 索引选择 | 5-7d |
| **P0** | 开发 | TPC-H 内存治理防 OOM | 3-5d |
| **P0** | 测试 | Alpha-1 基础验证 | 2d |
| **P1** | 测试 | Alpha-2 功能深度测试 | 3-5d |
| **P1** | 测试 | Alpha-3 性能回归 | 3-5d |
| **P1** | 开发 | 连接池/缓存/Group Commit 正确性 | 2-3d |
| **P2** | 测试 | Alpha-4 混沌工程 | 3-4d |

---

## 四、结论

**当前 develop/v3.0.0 不应直接进入 Beta。**

红线 (阻塞 Beta):
1. Alpha-1 基础验证全部通过
2. CBO 代价模型至少可用
3. TPC-H SF=1 无 OOM

黄线 (建议 Beta 前完成):
1. Alpha-2 功能深度测试 ≥80%
2. Alpha-3 性能基线建立
3. 连接池/缓存/Group Commit 压力测试通过

---

*文档版本: 1.0 | 2026-05-06*
*对应 Issue: #353, #370*
