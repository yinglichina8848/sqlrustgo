# v3.1.0 系统瓶颈分析与缺失 OO 文档补充计划

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **目标**: 基于 OO 文档深度分析，识别系统瓶颈，制定 v3.1.0/v3.2.0 优化路线

---

## 一、系统瓶颈深度分析（基于 OO 文档）

### 1.1 执行链路瓶颈总览

```
SQL 输入 → Parser → Planner → Optimizer → Executor → Storage → Transaction
             ↓         ↓          ↓           ↓          ↓          ↓
          解析       逻辑       物理       执行       B+Tree    MVCC+
          快速      规划        优化        并发       页I/O     WAL
          ⚠️ 语法   ⚠️ CTE    ⚠️ CBO     ⚠️ 多表   ⚠️ OOM   ⚠️ Gap Lock
          错误      断连       未激活     UPDATE     ⚠️ 聚簇   ⚠️ SSI
                                ⚠️ Join Order    DELETE    缺失   检测
```

### 1.2 五大核心瓶颈

#### 瓶颈 A: 存储层 OOM（mmap 限制）

**来源**: `docs/benchmark/TPCH-SF1-PERFORMANCE-REPORT.md`

| 指标 | v3.0.0 | 问题 |
|------|---------|------|
| MemoryStorage | ✅ 快速 | 大数据 OOM |
| mmap 存储 | ❌ OOM | 单文件 > RAM 时崩溃 |
| 列式存储 | ❌ 不存在 | 分析查询无加速 |

**根因**:
- mmap 将整个文件映射到地址空间，超过物理内存时 page fault
- 没有 LRU 淘汰机制
- 聚簇索引缺失导致所有查询走全表扫描

**v3.1.0 行动**:
- Issue #607 (GMP) 中聚簇索引实现
- Storage 层增加 memory limit 控制
- 列式存储评估（v3.2.0）

#### 瓶颈 B: CBO 代价模型未激活

**来源**: `docs/releases/v3.0.0/oo/cbo/CBO_COST_MODEL.md` (43KB)

```
TotalCost = CPUCost + IOCost + MemoryCost + NetworkCost

CPUCost = Σ(row_count × cpu_cost_per_row) + Σ(tuple_decode_cost)
IOCost = Σ(page_count × io_latency) + buffer_pool_miss_cost
```

**问题**: 43KB 代价公式从未接入 `optimizer.rs`

| 算子 | 公式 | 是否激活 |
|------|------|---------|
| SeqScan | page_count × seq_io_latency | ❌ |
| IndexScan | index_depth × random_io_latency | ❌ |
| HashJoin | build_size + probe_size × hash_probe_cost | ❌ |
| Sort | sort_buffer_pages × io_cost | ❌ |

**v3.1.0 行动**: Issue #616 (CBO 代价模型激活)

#### 瓶颈 C: MVCC 可见性规则未形式化

**来源**: `docs/releases/v3.0.0/oo/transaction/MVCC_IMPLEMENTATION.md` (21KB)

可见性判断算法：
```
VisiblyRead(txn, version, snapshot) =
    version.commit_ts # undefined                          // 版本已提交
    ∧ version.commit_ts < snapshot.ts                     // 提交时间 < 快照时间
    ∧ version.xmax # txn.id                               // 非当前事务删除
    ∧ (version.xmax_is_aborted ∨ version.xmax_committed  // 删除事务已回滚或提交时间 < 快照时间
       ∨ version.xmax < snapshot.ts)
```

**问题**: 
- 无 TLA+ 形式化证明
- 无反例测试（故意构造违反规则的场景）
- SSI 死锁检测不完整

**v3.1.0 行动**: Issue #625 (MVCC 形式化验证)

#### 瓶颈 D: 窗口函数框架完整但核心函数缺失

**来源**: `docs/releases/v3.0.0/oo/query/WINDOW_FUNCTIONS.md` (17KB)

OO 文档定义了完整的窗口函数框架，但：

| 函数 | OO 文档 | 实际代码 | 状态 |
|------|---------|---------|------|
| ROW_NUMBER | ✅ | ✅ | ✅ |
| RANK/DENSE_RANK | ✅ | ✅ | ✅ |
| COUNT/SUM/AVG/MIN/MAX | ✅ | ✅ | ✅ |
| **NTILE** | ⚠️ 文档有 | ❌ 未实现 | **P0** |
| **LEAD** | ⚠️ 文档有 | ❌ 未实现 | **P0** |
| **LAG** | ⚠️ 文档有 | ❌ 未实现 | **P0** |
| **FIRST_VALUE** | ⚠️ 文档有 | ⚠️ 不完整 | **P1** |
| **LAST_VALUE** | ⚠️ 文档有 | ⚠️ 不完整 | **P1** |
| **NTH_VALUE** | ⚠️ 文档有 | ❌ 未实现 | **P2** |

**根因**: OO 文档框架与实现脱节，文档写了但代码未跟上

**v3.1.0 行动**: Issue #621 (窗口函数补全)

#### 瓶颈 E: 多表 DML 执行链路缺失

**来源**: `docs/releases/v3.0.0/oo/SQL_EXECUTION_MATRIX.md`

| 语句 | 覆盖率 | 状态 |
|------|--------|------|
| UPDATE (单表) | ~70% | ✅ |
| **UPDATE (多表)** | ~50% | ❌ |
| DELETE (单表) | ~70% | ✅ |
| **DELETE (多表)** | ~45% | ❌ |
| MERGE | 0% | **PR #613 进行中** |

**根因**: 多表 DML 需要同时修改多个表，MVCC 可见性判断更复杂

**v3.1.0 行动**: Issue #619 (InnoDB 语义兼容)

---

## 二、缺失的 OO 文档分析领域

### 2.1 高优先级缺失（v3.1.0 必须补充）

| # | 缺失文档 | 涉及瓶颈 | 优先级 |
|---|---------|---------|--------|
| 1 | `oo/storage/COLUMNAR_STORAGE.md` | 存储层 OOM | P1 |
| 2 | `oo/cbo/CBO_INTEGRATION.md` | CBO 未激活 | P0 |
| 3 | `oo/transaction/SERIALIZABLE_SSI.md` | SSI 检测 | P1 |
| 4 | `oo/execution/ITERATOR_MODEL.md` | 执行器并行 | P1 |
| 5 | `oo/security/RBAC_EXECUTION.md` | DCL 执行层 | P1 |
| 6 | `oo/storage/BUFFER_POOL_LRU.md` | Buffer Pool 管理 | P1 |

### 2.2 中优先级缺失（v3.1.0 补充，v3.2.0 实现）

| # | 缺失文档 | 涉及瓶颈 | 优先级 |
|---|---------|---------|--------|
| 7 | `oo/network/PROTOCOL_LAYER.md` | MySQL 协议 | P2 |
| 8 | `oo/optimizer/PARAMETERIZED_QUERY.md` | 执行计划重用 | P2 |
| 9 | `oo/storage/INDEX_MAINTENANCE.md` | 在线索引维护 | P2 |
| 10 | `oo/distributed/FAILOVER.md` | 故障自动转移 | P2 |
| 11 | `oo/security/AUDIT_RBAC.md` | 审计+权限联动 | P2 |
| 12 | `oo/query/INSERT_SELECT.md` | INSERT-SELECT 优化 | P2 |

### 2.3 低优先级（v3.2.0+）

| # | 缺失文档 | 涉及瓶颈 | 优先级 |
|---|---------|---------|--------|
| 13 | `oo/distributed/RAFT_CONSENSUS.md` | Raft 选主 | P3 |
| 14 | `oo/storage/COMPACTION.md` | LSM  vs B+Tree | P3 |
| 15 | `oo/query/PARALLEL_EXECUTION.md` | 向量化执行 | P3 |

---

## 三、瓶颈 → OO 文档 → Issue 映射

### 3.1 当前追踪状态

```
瓶颈 A (存储 OOM)
  → oo/storage/COLUMNAR_STORAGE.md [缺失] → Issue ?
  → oo/storage/CLUSTERED_INDEX.md [缺失] → Issue #607 (GMP)
  → oo/storage/BUFFER_POOL_LRU.md [缺失] → Issue #607 (GMP)

瓶颈 B (CBO 未激活)
  → oo/cbo/CBO_INTEGRATION.md [缺失] → Issue #616 (CBO 激活)
  → oo/cbo/CBO_JOIN_ORDERING.md [存在] ✅ → 直接可用

瓶颈 C (MVCC 未形式化)
  → oo/transaction/MVCC_IMPLEMENTATION.md [存在] ✅ → Issue #625 (TLA+)
  → oo/transaction/SERIALIZABLE_SSI.md [缺失] → Issue #625 (SSI)

瓶颈 D (窗口函数)
  → oo/query/WINDOW_FUNCTIONS.md [存在] ✅
  → oo/query/WINDOW_FUNCTION_NTILE.md [缺失] → Issue #621 (NTILE)
  → Issue #621 追踪完整

瓶颈 E (多表 DML)
  → oo/query/MULTI_TABLE_UPDATE_DELETE.md [缺失] → Issue #619 (InnoDB 兼容)

缺失关键 OO 文档（无 Issue 追踪）
  → Issue #628 (OO 文档补全)
```

---

## 四、OO 文档体系增强建议

### 4.1 新增瓶颈分析文档

#### DOC-1: `oo/storage/COLUMNAR_STORAGE.md`

分析列式存储如何解决 OOM：

```
列式 vs 行式：
- 行式: SELECT * → 读取整行 → 投影所需列
- 列式: SELECT col → 只读取 col 列 → 网络传输更少

列式存储格式（Apache Arrow / Parquet）：
- 列数据连续存储 → vectorized scan
- Dictionary encoding → 低基数列压缩
- Bit-squeezing → NULL 位图压缩

对 TPC-H Q1 (聚集函数) 加速 10-100x：
- 列式: 只需读取 l_quantity, l_extendedprice, l_tax 三列
- 行式: 需要读取所有列
```

#### DOC-2: `oo/transaction/SERIALIZABLE_SSI.md`

SSI (Serializable Snapshot Isolation) 死锁检测：

```
SSI 死锁检测：
- 两个事务 T1 和 T2 互相等待对方释放锁
- wait-for graph: T1 → T2 → T1 (环形)

偏序检测算法：
1. 构建 RW-edges: T1 读取 x, T2 写入 x
2. 检测环形依赖
3. 触发 SSI abort

v3.0.0 问题: 缺少 wait-for graph 的偏序检测
```

#### DOC-3: `oo/execution/ITERATOR_MODEL.md`

火山模型 vs 向量化执行：

```
火山模型 (Volcano):
- 每个算子实现 next() 接口
- 一次吐出一行 (tuple-at-a-time)
- 优点: 简单，灵活
- 缺点: 函数调用开销大，CPU 缓存不友好

向量化执行 (Vectorized):
- 每个算子一次处理一批 (batch-at-a-time, 1024 rows)
- SIMD 加速: 一条指令处理多个值
- 缓存友好: 同一列数据连续访问

v3.1.0: 向量化 scanner 实现 → 性能提升 5-10x
```

#### DOC-4: `oo/security/RBAC_EXECUTION.md`

DCL 从解析到执行：

```
当前状态: GRANT/REVOKE 只解析，不执行
目标状态: 完整 RBAC 执行层

GRANT 执行流程：
1. Parser: GRANT SELECT ON db.table TO user
2. Planner: 生成 GrantStatement
3. Executor: 
   - 检查 current_user 是否有 GRANT OPTION
   - 检查目标权限是否存在
   - 写入 catalog.privileges 表
   - 记录审计日志

REVOKE 执行流程：
类似 GRANT，但删除权限条目
```

#### DOC-5: `oo/storage/BUFFER_POOL_LRU.md`

Buffer Pool LRU 淘汰策略：

```
Buffer Pool 结构：
┌──────────────────────────────────────┐
│           Buffer Pool (16GB)         │
│  ┌────────────┐  ┌────────────────┐  │
│  │   LRU     │  │    Free List   │  │
│  │  Hot Pages│  │   Clean Pages  │  │
│  └────────────┘  └────────────────┘  │
└──────────────────────────────────────┘

LRU-K 淘汰策略：
- 跟踪每页最近 K 次访问时间
- 淘汰：最近 K 次访问平均间隔最长
- 优点：防止一次性扫描污染缓存

Clock-Sweep (MySQL InnoDB):
- 近似 LRU，O(1) 复杂度
- 每个页一个 reference bit
- 环形扫描，清除 bits

v3.1.0: 实现 Clock-Sweep，替代简单 LRU
```

### 4.2 OO 文档质量门禁增强

`check_oo_docs.sh` 当前只验证文件存在。下一步增强：

```bash
# 检查每个 OO 文档是否包含必需的章节
REQUIRED_CHAPTERS=(
    "架构图"
    "时序图" 
    "算法实现"
    "测试计划"
    "覆盖率分析"
)

# 检查文档大小是否足够（防止空文档）
MIN_SIZE=5000  # bytes
```

---

## 五、v3.1.0 行动计划

### 5.1 Issue 补充（立即执行）

| Issue | 标题 | 行动 |
|-------|------|------|
| #629 | v3.1.0 存储层 OOM 修复 + 列式存储评估 | 创建 Issue |
| #630 | v3.1.0 CBO 代价模型接入 optimizer | 补充 Issue #616 |
| #631 | v3.1.0 SSI 死锁检测 + MVCC 形式化 | 补充 Issue #625 |
| #632 | v3.1.0 Buffer Pool LRU 优化 | 创建 Issue |
| #633 | v3.1.0 RBAC 执行层 (DCL) | 创建 Issue |
| #634 | v3.1.0 向量化执行评估 | 创建 Issue |

### 5.2 OO 文档补充（v3.1.0-alpha 前）

| 文档 | 优先级 | 状态 |
|------|--------|------|
| `oo/storage/COLUMNAR_STORAGE.md` | P1 | ⏳ 待创建 |
| `oo/storage/BUFFER_POOL_LRU.md` | P1 | ⏳ 待创建 |
| `oo/transaction/SERIALIZABLE_SSI.md` | P1 | ⏳ 待创建 |
| `oo/execution/ITERATOR_MODEL.md` | P1 | ⏳ 待创建 |
| `oo/cbo/CBO_INTEGRATION.md` | P0 | ⏳ 待创建 |
| `oo/security/RBAC_EXECUTION.md` | P1 | ⏳ 待创建 |

### 5.3 瓶颈 → 代码改进映射

| 瓶颈 | OO 文档 | 代码模块 | v3.1.0 行动 |
|------|---------|---------|-------------|
| 存储 OOM | `COLUMNAR_STORAGE.md` | `crates/storage/src/columnar/` | 列式存储评估 |
| 存储 OOM | `BUFFER_POOL_LRU.md` | `crates/storage/src/buffer_pool.rs` | Clock-Sweep 实现 |
| CBO 未激活 | `CBO_INTEGRATION.md` | `crates/optimizer/src/cost_model.rs` | CostConstants 实现 |
| MVCC 未形式化 | `SERIALIZABLE_SSI.md` | `crates/transaction/src/ssi.rs` | 死锁检测 |
| DCL 执行层 | `RBAC_EXECUTION.md` | `crates/catalog/src/auth.rs` | GRANT/REVOKE 执行 |
| 向量化 | `ITERATOR_MODEL.md` | `crates/executor/src/vectorized/` | VectorizedScanner |

---

## 六、总结：OO 文档 → 系统改进闭环

```
OO 文档（纵向分析）
       ↓
   识别瓶颈
       ↓
   创建 Issue
       ↓
   代码实现
       ↓
   测试验证
       ↓
   覆盖率报告
       ↓
   反哺 OO 文档（更新状态）
```

**v3.1.0 关键闭环**:
1. `CBO_COST_MODEL.md` (43KB) → `cost_model.rs` → Issue #616
2. `MVCC_IMPLEMENTATION.md` (21KB) → TLA+ 规格 → Issue #625
3. `WINDOW_FUNCTIONS.md` (17KB) → `ntile.rs`/`lead_lag.rs` → Issue #621
4. `WAL_PROTOCOL.md` (40KB) → chaos tests → Issue #626
5. `SQL_EXECUTION_MATRIX.md` (18KB) → 覆盖缺口扫描 → Issue #627

**OO 文档的终极价值**: 不是写完就结束，而是成为**从文档到代码到测试**的完整闭环的起点。
