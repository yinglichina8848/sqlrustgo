# SQLRustGo v3.2.0 性能瓶颈分析报告

> **版本**: v1.0
> **日期**: 2026-05-17
> **状态**: RC Gate 分析
> **基于**: v3.1.0 SYSTEM_BOTTLENECK_ANALYSIS.md + v3.2.0 RC Gate 结果

---

## 一、当前性能基线

### 1.1 QPS 基准测试结果 (v3.2.0)

| 基准测试 | v3.2.0 QPS | v3.1.0 目标 | 状态 |
|----------|-------------|-------------|------|
| simple_select | 324K | ≥400K | ⚠️ 低于目标 |
| insert | 55K | ≥50K | ✅ 达标 |
| update | 58K | ≥550K | ❌ 严重不足 |
| delete | 62K | ≥700K | ❌ 严重不足 |
| **join** | **11K** | ≥50K | ❌ **极低** |
| **aggregation** | **1.27M** | ≥600K | ✅ 优秀 |
| order_by | 20K | ≥50K | ⚠️ 低于目标 |
| concurrent_select_8t | 487K | - | ✅ 良好 |
| **complex_where** | **368** | ≥5K | ❌ **极低** |

### 1.2 v3.1.0 对比分析

| 指标 | v3.0.0 GA | v3.1.0 目标 | v3.2.0 当前 | 变化 |
|------|-----------|-------------|-------------|------|
| Point SELECT | 398K | ≥400K | 324K | -19% |
| UPDATE | 532K | ≥550K | 58K | -89% |
| DELETE | 706K | ≥700K | 62K | -91% |
| INSERT | ~400K | ≥450K | 55K | -86% |
| aggregation | 671K | ≥600K | 1.27M | +89% |

**关键发现**: UPDATE/DELETE/INSERT 性能从 400K-700K 级别暴跌至 55K-62K，降幅超过 85%。

---

## 二、RC Gate 性能问题

### 2.1 门禁检查结果摘要

| 检查项 | 结果 | 问题 |
|--------|------|------|
| R1 Build | ✅ PASS | - |
| R2 Test ≥90% | ✅ PASS | - |
| R3 Clippy | ✅ PASS | - |
| R4 Format | ✅ PASS | - |
| R5 Coverage | ⏸️ SKIP | - |
| R6 SQL Compat | ✅ PASS | - |
| R7 TPC-H SF=1 | ✅ PASS | - |
| R8 Security | ✅ PASS | - |
| R9 **Regression** | **❌ FAIL** | **aggregation -40%** |
| R10 Sysbench | ⚠️ WARNING | **握手失败 (os error 35)** |

### 2.2 R9 回归分析

```
aggregation: 1,269,210 → 767,877 (-40%) FAIL
```

**时间线**:
- 2026-05-16 22:11: baseline.json 建立，aggregation = 1.27M
- 2026-05-17 01:31: RC Gate 检查，aggregation = 768K (-40%)

**可能原因**:
1. 合并了 `feat/cold-storage-tiering` (PR #1097, #1091)
2. Storage tier 初始化代码可能影响聚合性能
3. `storage_tier.rs` 的 merge conflict resolution 可能引人问题

### 2.3 R10 Sysbench 问题

```
Handshake read: Resource temporarily unavailable (os error 35)
```

**已知问题**: MySQL 协议层 handshake 在高并发下 Resource temporarily unavailable

**PERF-1 Issue**: https://github.com/openclaw/sqlrustgo/issues/920

---

## 三、五大核心瓶颈 (继承自 v3.1.0)

### 瓶颈 A: 存储层 OOM (mmap 限制)

**根因**:
- mmap 将整个文件映射到地址空间，超过物理内存时 page fault
- 没有 LRU 淘汰机制
- 聚簇索引缺失导致所有查询走全表扫描

**v3.1.0 遗留问题**:

| 指标 | v3.0.0 | 问题 |
|------|---------|------|
| MemoryStorage | ✅ 快速 | 大数据 OOM |
| mmap 存储 | ❌ OOM | 单文件 > RAM 时崩溃 |
| 列式存储 | ❌ 不存在 | 分析查询无加速 |

---

### 瓶颈 B: CBO 代价模型未激活

**根因**: 43KB CBO 代价公式从未接人 `optimizer.rs`

| 算子 | 公式 | 是否激活 |
|------|------|---------|
| SeqScan | page_count × seq_io_latency | ❌ |
| IndexScan | index_depth × random_io_latency | ❌ |
| HashJoin | build_size + probe_size × hash_probe_cost | ❌ |
| Sort | sort_buffer_pages × io_cost | ❌ |

**影响**:
- join QPS 仅 11K (目标 50K+)
- order_by QPS 仅 20K (目标 50K+)
- complex_where 仅 368 (目标 5K+)

---

### 瓶颈 C: MVCC 可见性规则未形式化

**根因**:
- 无 TLA+ 形式化证明
- 无反例测试
- SSI 死锁检测不完整

---

### 瓶颈 D: 窗口函数核心函数缺失

**根因**: OO 文档框架与实现脱节

| 函数 | OO 文档 | 实际代码 | 状态 |
|------|---------|---------|------|
| ROW_NUMBER | ✅ | ✅ | ✅ |
| RANK/DENSE_RANK | ✅ | ✅ | ✅ |
| **NTILE** | ⚠️ 有 | ❌ | **P0** |
| **LEAD** | ⚠️ 有 | ❌ | **P0** |
| **LAG** | ⚠️ 有 | ❌ | **P0** |
| FIRST_VALUE | ⚠️ 有 | ⚠️ 不完整 | P1 |
| LAST_VALUE | ⚠️ 有 | ⚠️ 不完整 | P1 |

---

### 瓶颈 E: 多表 DML 执行链路缺失

**根因**: 多表 DML 需要同时修改多个表，MVCC 可见性判断更复杂

| 语句 | 覆盖率 | 状态 |
|------|--------|------|
| UPDATE (单表) | ~70% | ✅ |
| **UPDATE (多表)** | ~50% | ❌ |
| DELETE (单表) | ~70% | ✅ |
| **DELETE (多表)** | ~45% | ❌ |
| MERGE | 0% | ✅ 已实现 |

---

## 四、v3.2.0 新发现问题

### 4.1 UPDATE/DELETE/INSERT 性能暴跌

**现象**: 从 400K-700K 级别跌至 55K-62K

**可能原因**:
1. Storage tier 初始化引人开销
2. S3 签名计算 (hmac_sha256) 每次调用
3. 冷存储层检查引人延迟

**需要调查**:
```bash
# 对比 PR #1097 前后的性能
git log --oneline PR #1097..HEAD
# 检查 storage_tier.rs 的 merge conflict resolution
```

---

### 4.2 complex_where 仅 368 QPS

**目标**: ≥5K QPS
**当前**: 368 QPS
**差距**: 13.6x

**可能原因**:
1. WHERE 子句无索引支持
2. CBO 未激活导致全表扫描
3. 表达式求值无向量化

---

### 4.3 join QPS 仅 11K

**目标**: ≥50K QPS
**当前**: 11K QPS
**差距**: 4.5x

**可能原因**:
1. Hash join 未优化
2. CBO 未激活，错误执行计划
3. 内存分配开销

---

## 五、根因分析

### 5.1 性能回归根因

```
时间线:
1. 2026-05-16 22:11: f561caa5 - 建立 baseline (aggregation=1.27M)
2. 2026-05-16 22:23: f29ee983 - PR #1098 dead_code clippy 修复
3. 2026-05-17 01:31: 多个 PR 合并后 - RC Gate aggregation=768K

关键 PR:
- PR #1097 (f28bcddb): feat(storage): 冷存储分层 (Issue #993)
- PR #1091 (abd28897): feat(storage): 冷存储完善 (S3签名 + StorageTierManager)
- PR #1093 (5e10ff1f): fix(storage): AWS S3 SigV4 signing
```

**假设**: Storage tier 代码引人了每次 DML 操作的开销

---

### 5.2 性能优化优先级矩阵

```
收益 ↑
     │
  高 ├────────────────────────────
     │  P0: 调查 aggregation 回归  P0: complex_where 优化
     │  P0: 修复 UPDATE/DELETE     P0: CBO 激活
     │                            P1: join 优化
     │
  低 ├────────────────────────────
     │  P2: 列式存储             P2: 窗口函数
     │  P2: MVCC 形式化           P2: 多表 DML
     └────────────────────────────────────→ 紧急度
           低                    高
```

---

## 六、整改建议

### 6.1 立即行动 (P0)

| # | 任务 | 预期收益 | 负责 |
|---|------|----------|------|
| 1 | 调查 aggregation -40% 回归 | 恢复 1.27M QPS | 待分配 |
| 2 | 修复 UPDATE/DELETE 性能 | 恢复 500K+ QPS | 待分配 |
| 3 | 激活 CBO 代价模型 | join 5x, order_by 2.5x | 待分配 |
| 4 | 优化 complex_where | 368 → 5K+ (13.6x) | 待分配 |

### 6.2 短期行动 (P1)

| # | 任务 | 预期收益 |
|---|------|----------|
| 5 | 修复 Sysbench 握手问题 | 基准测试可运行 |
| 6 | 实现 NTILE/LEAD/LAG | SQL 兼容性 90%+ |
| 7 | 多表 UPDATE/DELETE 完善 | InnoDB 兼容性 |
| 8 | LRU Buffer Pool 优化 | 内存效率提升 |

### 6.3 中期行动 (P2)

| # | 任务 | 预期收益 |
|---|------|----------|
| 9 | 列式存储评估 | TPC-H 加速 10-100x |
| 10 | MVCC TLA+ 形式化证明 | SSI 死锁检测完整 |
| 11 | 向量化执行 | 整体 5-10x |

---

## 七、附录

### A. 相关文档

- [v3.1.0 SYSTEM_BOTTLENECK_ANALYSIS.md](../v3.1.0/SYSTEM_BOTTLENECK_ANALYSIS.md)
- [v3.2.0 COMPREHENSIVE_STATUS_REPORT.md](./COMPREHENSIVE_STATUS_REPORT.md)
- [v3.1.0 PERFORMANCE_TARGETS.md](../v3.1.0/PERFORMANCE_TARGETS.md)

### B. 相关 Issue

| Issue | 描述 |
|-------|------|
| #920 | PERF-1: Point SELECT QPS ≥1M |
| #921 | PERF-2: TPC-H SF=10 |
| #922 | PERF-3: 并发增强 200+ |
| #923 | PERF-4: 死锁检测优化 |
| #924 | PERF-5: 内存优化 -15% |
| #619 | 多表 DML 执行链路 |
| #621 | 窗口函数补全 |
| #625 | MVCC TLA+ 形式化 |

### C. 性能基线文件

- `perf_baselines/v3.2.0/current.json` - 当前基线
- `perf_baselines/v3.1.0/baseline.json` - v3.1.0 基线

---

**维护人**: Sisyphus Orchestrator
**生成日期**: 2026-05-17