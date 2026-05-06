# v3.0.0 Alpha 测试计划

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06
> **阶段**: Alpha

---

## 测试目标

| 指标 | Alpha 目标 |
|------|-----------|
| SQL Corpus 通过率 | 100% (485/485) |
| 单元测试通过率 | ≥80% |
| 整体覆盖率 | ≥50% |
| TPC-H SF=0.1 | 22/22 可运行 |
| A-Gate P0 项 | 全部通过 |

---

## 一、Alpha 测试分层

### Alpha-1: 基础验证

| 门禁 | 命令/方法 | 通过标准 |
|------|----------|----------|
| A-1 | `cargo test --all-features --workspace` | ≥80% 通过 |
| A-2 | `cargo llvm-cov --all --all-features` | 整体 ≥50% |
| A-3 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| A-4 | `cargo fmt --all -- --check` | 零差异 |
| A-5 | `bash scripts/gate/check_docs_links.sh` | 零死链 |
| A-6 | `cargo audit` | 无高危漏洞 |

### Alpha-2: 功能深度测试

| 功能 | 测试项 | 验证方法 |
|------|--------|----------|
| IN 子查询 | `WHERE c IN (SELECT ...)` | 端到端测试 |
| EXISTS 子查询 | `WHERE EXISTS (SELECT 1)` | 端到端测试 |
| CASE 表达式 | `CASE WHEN c>0 THEN 1 ELSE 0 END` | 端到端测试 |
| COALESCE | `COALESCE(NULL, NULL, c)` | 端到端测试 |
| INSERT...SELECT | 不同列数、类型转换 | 端到端测试 |
| 窗口函数 | NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE | SQL Corpus |
| CTE | 递归深度、多 CTE 引用、CTE+JOIN | SQL Corpus |
| SERIALIZABLE | Proof-026 100 并发压力 | 零假阳性 |

### Alpha-3: 性能测试

| 测试项 | 方法 | Alpha 目标 |
|--------|------|-----------|
| TPC-H SF=0.1 | 22 查询全部运行 | ≤15s 总耗时 |
| QPS (Point Select) | `test_qps_simple_select` | ≥20,000 QPS |
| QPS (UPDATE) | `test_qps_update` | ≥10,000 QPS |
| QPS (DELETE) | `test_qps_delete` | ≥10,000 QPS |
| 并发 SELECT 8T | `test_qps_concurrent_select` | ≥5,000 QPS |

### Alpha-4: 混沌与稳定性

| 测试项 | 方法 | 目标 |
|--------|------|------|
| kill -9 崩溃恢复 | SIGKILL + 重启数据校验 | 数据完整 |
| 并发压力 | 8T × 1000 ops | 无数据竞争 |
| 长稳测试 | 30min 混合负载 | 无 panic |

---

## 二、模块覆盖率目标

| 模块 | Alpha (≥) | 当前估计 |
|------|-----------|---------|
| executor | 45% | 待实测 |
| optimizer | 40% | 待实测 |
| parser | 50% | 待实测 |
| storage | 15% | 待实测 |
| catalog | 50% | 待实测 |
| **整体** | **50%** | **待实测** |

---

## 三、TPC-H 测试矩阵

| SF | 数据量 | Alpha 目标 | Beta 目标 |
|----|--------|-----------|-----------|
| 0.1 | ~6K 行 | 22/22 可运行 | 22/22 ≤15s |
| 1 | ~60K 行 | 22/22 无 OOM | 22/22 ≤30s |
| 10 | ~600K 行 | — | 22/22 无 OOM |

---

## 四、执行方法

### 本地快速验证

```bash
# Alpha-1: 基础门禁
bash scripts/gate/check_alpha_v300.sh

# Alpha-3: 性能
bash scripts/gate/check_regression.sh

# Alpha-2: 功能（需要 SQL 测试数据）
cargo test --all-features --workspace
```

### CI/Gitea Actions

```bash
# gate-ci.yml (B-Gate) 触发条件
# push to develop/v3.0.0 / alpha/v3.0.0 / beta/v3.0.0

# 完整 Alpha 测试流水线
bash scripts/gate/check_alpha_v300.sh && \
bash scripts/gate/check_regression.sh && \
bash scripts/gate/check_sysbench.sh --alpha
```

---

## 五、测试数据

| 数据集 | 路径 | 用途 |
|--------|------|------|
| TPC-H SF=0.1 | `~/sqlrustgo-tpch/data/` | Alpha 性能验证 |
| TPC-H SF=1 | `~/sqlrustgo-tpch/data/` | Beta/GA 性能验证 |
| SQL 测试 | `crates/sql-corpus/` | 语法兼容性 |

---

## 六、风险项

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| TPC-H SF=1 OOM | 内存溢出 | Beta 阶段加内存限制 |
| 覆盖率不达标 | 无法进入 Beta | 增加单元测试 |
| 并发测试不稳定 | 假阳性 | Proof-026 增加压力 |