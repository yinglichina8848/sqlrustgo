# SQLRustGo v3.2.0 问题汇总

> **版本**: v1.0
> **日期**: 2026-05-15
> **维护人**: hermes-z6g4

---

## 一、v3.1.0 遗留问题汇总

### 1.1 OO 文档缺失问题

#### P1 高优先级（v3.1.0 未完成）

| Issue | 文档 | 状态 | 说明 |
|-------|------|------|------|
| #625 | MVCC TLA+ 形式化验证 | ❌ 未完成 | 需 TLA+ 规格 + 反例测试 |
| #626 | WAL + 审计链集成文档 | ❌ 未完成 | WAL chaos 测试文档 |
| #630 | SSI 死锁检测文档 | ❌ 未完成 | SSI 串行化可隔离 |
| #530 | Event Scheduler 设计文档 | ❌ 未完成 | MySQL Event 兼容 |
| — | HASH/MERGE JOIN 算法文档 | ❌ 未创建 | HASH JOIN / MERGE JOIN |

#### P2 中优先级（v3.2.0 继承）

| 文档 | 涉及瓶颈 | 优先级 |
|------|----------|--------|
| `oo/storage/COLUMNAR_STORAGE.md` | 存储层 OOM | P1 |
| `oo/cbo/CBO_INTEGRATION.md` | CBO 未激活 | P0 |
| `oo/transaction/SERIALIZABLE_SSI.md` | SSI 检测 | P1 |
| `oo/execution/ITERATOR_MODEL.md` | 执行器并行 | P1 |
| `oo/storage/BUFFER_POOL_LRU.md` | Buffer Pool 管理 | P1 |
| `oo/security/RBAC_EXECUTION.md` | DCL 执行层 | P1 |

### 1.2 系统瓶颈遗留

| # | 瓶颈 | 影响 | 优先级 | 目标版本 |
|---|------|------|--------|----------|
| A | 存储层 OOM (mmap 限制) | 大数据崩溃 | 🔴 P0 | v3.2.0 |
| B | CBO 代价模型未激活 | 查询计划不优化 | 🔴 P0 | v3.2.0 |
| C | MVCC 可见性规则未形式化 | SSI 不完整 | 🟡 P1 | v3.2.0 |
| D | 窗口函数框架缺失 | NTILE/LEAD/LAG | 🟡 P1 | v3.2.0 |
| E | 多表 DML 执行链路缺失 | UPDATE/DELETE 多表 | 🟡 P1 | v3.2.0 |
| F | MySQL 协议 109x 性能差距 | 生产不可用 | 🔴 P0 | v3.2.0 |

### 1.3 代码级 P0 问题（来自 oo/README.md）

| # | 问题 | 模块 | 影响 | 修复版本 |
|---|------|------|------|----------|
| 1 | B+Tree 分裂不向上传播 | bptree | 大量数据后树退化 | v3.2.0 |
| 2 | Sort/Limit 算子未实现 | executor | ORDER BY/LIMIT 不生效 | v3.2.0 |
| 3 | MySQL 协议 109x 性能 | mysql | 生产不可用 | v3.2.0 |
| 4 | SSI 未实现 | transaction | 无法支持 SERIALIZABLE | v3.2.0 |
| 5 | CBO 无统计信息 | optimizer | 代价估算不准确 | v3.2.0 |
| 6 | CompositeKey 只取首列 | bptree | 多列索引不可用 | v3.2.0 |

### 1.4 窗口函数缺失

| 函数 | OO 文档 | 实际代码 | 状态 |
|------|---------|---------|------|
| NTILE | ✅ | ❌ 未实现 | P0 |
| LEAD | ✅ | ❌ 未实现 | P0 |
| LAG | ✅ | ❌ 未实现 | P0 |
| FIRST_VALUE | ✅ | ⚠️ 不完整 | P1 |
| LAST_VALUE | ✅ | ⚠️ 不完整 | P1 |
| NTH_VALUE | ✅ | ❌ 未实现 | P2 |

### 1.5 多表 DML 覆盖缺口

| 语句 | 覆盖率 | 状态 |
|------|--------|------|
| UPDATE (单表) | ~70% | ✅ |
| **UPDATE (多表)** | ~50% | ❌ |
| DELETE (单表) | ~70% | ✅ |
| **DELETE (多表)** | ~45% | ❌ |
| MERGE | 0% | ❌ 进行中 |

---

## 二、GMP 管理系统核心需求

### 2.1 GMP 合规差距分析

| GMP 需求 | MySQL | SQLRustGo v3.1.0 | v3.2.0 目标 |
|----------|-------|-------------------|-------------|
| **ACID 事务** | ✅ | ✅ SSI + MVCC | ✅ 保持 |
| **数据完整性** | ✅ | ✅ WAL | ✅ 增强 |
| **审计追溯** | ✅ | ✅ SHA-256 | ✅ 升级为数字签名 |
| **隔离级别** | ✅ | ✅ Snapshot | ✅ SSI |
| **形式化验证** | ❌ | ✅ 31 proofs | ✅ 扩展 |
| **权限控制 RBAC** | ✅ | ✅ 执行层 | ✅ 增强 |
| **SQL 兼容性** | ✅ 100% | ✅ 80% | ✅ 85% |
| **高可用集群** | ✅ | ❌ | 🔴 P0 |
| **HSM/TPM 支持** | ✅ | ❌ | 🔴 P0 |
| **数字签名审计链** | ❌ | ❌ | 🔴 P0 |
| **电子签名** | ❌ | ❌ | 🔴 P0 |
| **Immutable Record** | ❌ | ❌ | 🔴 P0 |
| **Correction Chain** | ❌ | ❌ | 🔴 P0 |
| **Provenance Tracking** | ❌ | ❌ | 🔴 P0 |
| **Trusted Timestamp** | ❌ | ❌ | 🔴 P0 |
| **21 CFR Part 11** | ✅ | ❌ | 🔴 P0 |
| **GMP Workflow** | ❌ | ❌ | 🟡 P1 |

### 2.2 ALCOA+ 支撑矩阵

| ALCOA+ | 说明 | v3.1.0 | v3.2.0 |
|--------|------|---------|---------|
| **A** - Attributable | 可归因 | 用户 ID | 数字签名 + 设备指纹 |
| **L** - Legible | 可读 | ✅ | ✅ |
| **C** - Contemporaneous | 实时 | 时间戳 | 可信时间戳 (RFC3161) |
| **O** - Original | 原始 | WAL | 原始记录 + 哈希锚定 |
| **A** - Accurate | 准确 | 校验 | 签名验证 |
| **+C** - Complete | 完整 | 审计日志 | 完整 Provenance |
| **+I** - Consistent | 一致 | MVCC | 签名链 |
| **+E** - Enduring | 持久 | WAL 持久 | 冷存储集成 |
| **+A** - Available | 可用 | 查询接口 | 审计查询 + 导出 |

### 2.3 可信执行需求

| 需求 | v3.1.0 | v3.2.0 |
|------|---------|---------|
| **TLA+ 形式化验证** | 31 proofs | 扩展至 GMP 模块 |
| **审计链** | WAL + SHA-256 | 数字签名 + 验签 |
| **密钥管理** | 无 | HSM/TPM/KMS |
| **电子签名** | 无 | 私钥签名 |
| **双人复核** | 无 | 四眼原则 |
| **不可变记录** | 无 | CREATE IMMUTABLE |
| **修正链** | 无 | Correction Chain |
| **来源追踪** | 基础 | Provenance Tracking |

---

## 三、MySQL 兼容性差距

### 3.1 SQL 兼容性差距

| 功能 | MySQL | SQLRustGo | 差距 |
|------|-------|-----------|------|
| **窗口函数** | ✅ 完整 | ⚠️ 部分 | NTILE/LEAD/LAG 缺失 |
| **MERGE** | ✅ | ❌ | 未实现 |
| **多表 UPDATE** | ✅ | ⚠️ 部分 | ~50% |
| **多表 DELETE** | ✅ | ⚠️ 部分 | ~45% |
| **Event Scheduler** | ✅ | ❌ | 未实现 |
| **HASH JOIN** | ✅ | ❌ | 未实现 |
| **MERGE JOIN** | ✅ | ❌ | 未实现 |
| **存储过程** | ✅ | ⚠️ 解释执行 | 需预编译 |
| **触发器** | ✅ | ⚠️ 解释执行 | 需优化 |

### 3.2 MySQL 协议性能问题

| 瓶颈 | 问题 | 影响 | v3.2.0 目标 |
|------|------|------|-------------|
| `Packet::write_to` 强制 flush | 每个包单独 flush | 109x 性能差距 | 批量 flush |
| `is_select()` 重复 uppercase | 每次调用创建新字符串 | 内存分配累积 | 缓存 |
| `extract_table_name()` 重复 uppercase | 两次 uppercase | 内存分配累积 | 缓存 |
| `send_result_set()` 多次 write_to | 100 行 = 100+ flush | 性能瓶颈 | 批量发送 |

### 3.3 目标 QPS

| 测试类型 | v3.1.0 实测 | v3.2.0 目标 | 提升 |
|----------|-------------|-------------|------|
| point_select | 1,688 QPS | ≥ 30,000 QPS | 18x |
| oltp_read_write | 71 QPS | ≥ 8,000 QPS | 113x |
| oltp_write_only | 190 QPS | ≥ 6,000 QPS | 32x |
| update_index | 468 QPS | ≥ 6,000 QPS | 13x |

---

## 四、v3.2.0 问题分类汇总

### 4.1 P0 必须完成（GMP Native 核心）

| 类别 | 问题 | 来源 | 工作量 |
|------|------|------|--------|
| **GMP-1** | 数字签名审计链 | GMP 需求 | L |
| **GMP-2** | 电子签名 + 双人复核 | GMP 需求 | M |
| **GMP-3** | Immutable Record | GMP 需求 | M |
| **GMP-4** | Correction Chain | GMP 需求 | M |
| **GMP-5** | Provenance Tracking | GMP 需求 | L |
| **GMP-6** | Trusted Timestamp | GMP 需求 | M |
| **GMP-7** | HSM/KMS 集成 | GMP 需求 | L |
| **GMP-8** | 高可用架构设计 | GMP 需求 | L |
| **PERF-1** | MySQL 协议性能优化 | 系统瓶颈 | M |
| **PERF-2** | CBO 代价模型激活 | 系统瓶颈 | L |
| **PERF-3** | B+Tree 分裂修复 | 系统瓶颈 | M |
| **PERF-4** | Sort/Limit 算子实现 | 系统瓶颈 | M |
| **PERF-5** | SSI 死锁检测实现 | 系统瓶颈 | L |

### 4.2 P1 重要（平台能力）

| 类别 | 问题 | 来源 | 工作量 |
|------|------|------|--------|
| **GMP-9** | GMP Workflow Engine | GMP 需求 | L |
| **GMP-10** | 移动端可信采集协议 | GMP 需求 | M |
| **GMP-11** | SOP/培训绑定 | GMP 需求 | M |
| **GMP-12** | 设备校准管理 | GMP 需求 | M |
| **SQL-1** | 窗口函数补全 (NTILE/LEAD/LAG) | MySQL 兼容 | M |
| **SQL-2** | 多表 UPDATE/DELETE | MySQL 兼容 | L |
| **SQL-3** | HASH JOIN | MySQL 兼容 | L |
| **SQL-4** | MERGE 语句 | MySQL 兼容 | L |
| **SQL-5** | Event Scheduler | MySQL 兼容 | M |
| **OO-1** | TLA+ MVCC 形式化验证 | OO 文档 | M |
| **OO-2** | WAL + 审计链集成文档 | OO 文档 | M |
| **OO-3** | CBO_INTEGRATION 文档 | OO 文档 | S |
| **OO-4** | SERIALIZABLE_SSI 文档 | OO 文档 | S |
| **OO-5** | BUFFER_POOL_LRU 文档 | OO 文档 | S |
| **OO-6** | ITERATOR_MODEL 文档 | OO 文档 | S |

### 4.3 P2 后续（增强能力）

| 类别 | 问题 | 来源 | 工作量 |
|------|------|------|--------|
| **GMP-13** | 列式分析引擎 | GMP 需求 | L |
| **GMP-14** | Syslog/SIEM 集成 | GMP 需求 | M |
| **GMP-15** | Agent Audit | GMP 需求 | M |
| **SQL-6** | MERGE JOIN | MySQL 兼容 | M |
| **SQL-7** | 存储过程预编译 | MySQL 兼容 | L |
| **SQL-8** | 触发器优化 | MySQL 兼容 | M |
| **OO-7** | COLUMNAR_STORAGE 文档 | OO 文档 | S |
| **OO-8** | Event Scheduler 设计文档 | OO 文档 | S |

---

## 五、问题优先级矩阵

```
                    技术难度
              低        中        高
         ┌────────┬────────┬────────┐
    高   │ GMP-1  │ GMP-2  │ GMP-3  │
        │ GMP-5  │ GMP-6  │ GMP-8  │
        │ PERF-1  │ PERF-2  │        │
        ├────────┼────────┼────────┤
业  中   │ GMP-9  │ GMP-10 │ GMP-11 │
务  优先 │ SQL-1  │ GMP-12 │ SQL-2  │
重  度   │ SQL-3  │ SQL-4  │ SQL-5  │
        │ PERF-3  │ PERF-4 │ PERF-5 │
        ├────────┼────────┼────────┤
    低   │ OO-1   │ OO-2   │ GMP-13 │
        │ OO-3   │ OO-4   │ SQL-6  │
        │ OO-5   │ OO-6   │ SQL-7  │
        └────────┴────────┴────────┘
```

---

## 六、关键依赖关系

```
GMP-7 (HSM/KMS)
    ↓
GMP-1 (数字签名审计链)
    ↓
GMP-2 (电子签名)
    ↓
GMP-3 (Immutable Record)
    ↓
GMP-4 (Correction Chain)

PERF-1 (MySQL协议) ← 独立
PERF-2 (CBO) ← OO-3 文档完成后
PERF-3 (B+Tree) ← OO 文档
PERF-4 (Sort/Limit) ← OO 文档
PERF-5 (SSI) ← OO-4 文档

SQL-1 (窗口函数) ← OO 文档
SQL-2 (多表DML) ← MVCC 增强
SQL-3 (HASH JOIN) ← CBO
SQL-4 (MERGE) ← SQL-3 依赖

GMP-9 (Workflow) ← GMP-1~5 完成后
GMP-10 (移动端) ← GMP-1~5 完成后
```

---

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-15*
