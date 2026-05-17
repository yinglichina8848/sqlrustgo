# SQLRustGo 覆盖率测量 SSOT (Single Source of Truth)

> **版本**: v1.0
> **创建日期**: 2026-05-18
> **维护人**: hermes-agent
> **目的**: 统一覆盖率测量标准，杜绝数据矛盾

---

## 一、问题背景

v3.2.0 GA Gate 覆盖率数据曾出现矛盾：
- `GA_GATE_CHECKLIST.md` 声称 85.81%
- `RC_TO_GA_FINAL_REPORT.md` 记录 68.8%
- `COVERAGE_BLIND_SPOT_ANALYSIS.md` 分析为 83.62% (7/8 crates)

**根因**: 测量工具、测量范围、测量时间点不统一。

---

## 二、SSOT 覆盖率测量规范

### 2.1 L1 Crates 定义

> **注意**: 覆盖率测量仅针对 L1 核心 crate，不包含 workspace 所有 crate。

**L1 Crates (按字母顺序)**:
| Crate | 说明 |
|-------|------|
| `sqlrustgo-catalog` | 目录/元数据管理 |
| `sqlrustgo-executor` | 查询执行引擎 |
| `sqlrustgo-optimizer` | CBO 优化器 |
| `sqlrustgo-parser` | SQL 解析器 |
| `sqlrustgo-planner` | 查询规划器 |
| `sqlrustgo-storage` | 存储引擎 |
| `sqlrustgo-transaction` | 事务管理 |
| `sqlrustgo-types` | 类型系统 |

### 2.2 标准测量命令

**L1 整体覆盖率**:
```bash
cargo llvm-cov test \
    -p sqlrustgo-types \
    -p sqlrustgo-parser \
    -p sqlrustgo-planner \
    -p sqlrustgo-optimizer \
    -p sqlrustgo-executor \
    -p sqlrustgo-storage \
    -p sqlrustgo-transaction \
    -p sqlrustgo-catalog \
    --lib
```

**输出示例**:
```
     TOTAL | Lines      |      %
---------|-------------|----------
 types   | 77.1%      |
 parser  | XX.X%      |
 planner | 81.5%      |
 optimizer | 80.3%    |
 executor | 59.0%     |
 storage  | 64.4%     |
 transaction | 78.8%  |
 catalog  | 71.3%     |
---------|-------------|----------
 L1 整体 | 68.8%      |
```

**注意**: 必须使用 `--lib` 标志，仅测量库代码，不测量集成测试。

### 2.3 门禁阈值

| 门禁阶段 | L1 整体阈值 | 说明 |
|----------|-------------|------|
| Alpha (A) | ≥50% | 开发完成，可运行原型 |
| Beta (B) | ≥75% | 功能冻结，进入稳定期 |
| RC (R) | ≥85% | 发布候选 |
| GA (G) | ≥85% | 正式发布 |

### 2.4 不计入覆盖率的 Crates

以下 crate 不计入 L1 覆盖率：
- `sqlrustgo-server` - 服务集成
- `sqlrustgo-mysql-server` - MySQL 协议
- `sqlrustgo-network` - 网络层
- `sqlrustgo-gmp` - GMP 功能 (单独门禁)
- `sqlrustgo-gis` - GIS 扩展
- `sqlrustgo-vector` - 向量存储
- `sqlrustgo-graph` - 图存储
- `sqlrustgo-sql-corpus` - SQL 语料库测试
- 其他辅助 crate

---

## 三、v3.2.0 实测数据 (最终对齐)

### 3.1 各 crate 覆盖率

| Crate | 行数 | 覆盖率 | RC/GA 目标 | 状态 |
|-------|------|--------|------------|------|
| types | 737 | 77.1% | ≥80% | ❌ |
| planner | 3,741 | 81.5% | — | ✅ |
| optimizer | 4,115 | 80.3% | ≥70% | ✅ |
| executor | 10,836 | 59.0% | ≥80% | ❌ |
| storage | 12,312 | 64.4% | ≥40% | ✅ |
| transaction | 4,793 | 78.8% | ≥70% | ✅ |
| catalog | 4,074 | 71.3% | ≥75% | ❌ |
| **L1 整体** | **40,608** | **68.8%** | **≥85%** | **❌** |

### 3.2 差距分析

| 问题 | 差距 | 主要贡献者 |
|------|------|-----------|
| L1 整体 68.8% < 85% | -16.2% | executor (59.0%) |
| executor 59.0% < 80% | -21.0% | stored_proc.rs (41.8%) |
| types 77.1% < 80% | -2.9% | 小差距 |
| catalog 71.3% < 75% | -3.7% | 小差距 |

---

## 四、文档更新要求

### 4.1 文档一致性规则

| 文档类型 | 必须引用的数值 | 来源 |
|----------|---------------|------|
| Gate Checklist | L1 整体覆盖率 | 本文档 §3.1 |
| Gate Report | 各 crate 覆盖率 | 本文档 §3.1 |
| Release Notes | L1 整体覆盖率 | 本文档 §3.1 |
| Legacy Issues | L1 整体覆盖率 | 本文档 §3.1 |

**禁止**: 在任何文档中手动填入与本文档不一致的覆盖率数值。

### 4.2 正确引用方式

```markdown
## 覆盖率

> **SSOT**: 见 `docs/releases/v3.2.0/COVERAGE_SSOT.md`
> - L1 整体: 68.8% (需提升至 85%)
> - 主要差距: executor (59.0%)
```

---

## 五、Issue 追踪

| Issue | 问题 | 状态 |
|-------|------|------|
| #1196/#1197 | executor 覆盖率 70.7% < 85% | v3.3.0 修复 |
| #1202 | Coverage 数据矛盾 | ✅ 已建立 SSOT |

---

## 六、相关文档

- `docs/governance/GATE_SPEC_MASTER.md` - 门禁规范 SSOT
- `scripts/gate/check_coverage.sh` - 覆盖率测量脚本
- `scripts/gate/check_coverage_parallel.sh` - 并行覆盖率测量
- `docs/releases/v3.2.0/COVERAGE_BLIND_SPOT_ANALYSIS.md` - 盲区分析

---

*本文档由 hermes-agent 生成*
*SSOT: COVERAGE_SSOT.md 是覆盖率测量唯一权威来源*
