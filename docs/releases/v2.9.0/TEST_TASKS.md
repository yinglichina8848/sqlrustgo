# v2.9.0 分阶段测试任务清单

> **文件**: docs/releases/v2.9.0/TEST_TASKS.md
> **日期**: 2026-05-02
> **关联**: TEST_STRATEGY.md → 具体任务

---

## 开发阶段 (develop/v2.9.0) — 回归 + 集成门禁

### 目标: 每 PR 合并前验证全部回归和集成

#### 任务 D1: 集成测试自动化脚本
```yaml
文件: scripts/test/run_integration.sh
执行: bash scripts/test/run_integration.sh
内容: 按模块分组运行 28 个集成测试文件
     tests/aggregate_functions_test.rs
     tests/binary_format_test.rs
     tests/boundary_test.rs
     tests/cbo_integration_test.rs
     tests/concurrency_stress_test.rs
     tests/crash_recovery_test.rs
     tests/distinct_test.rs
     tests/expression_operators_test.rs
     tests/in_value_list_test.rs
     tests/limit_clause_test.rs
     tests/mvcc_transaction_test.rs
     tests/regression_test.rs
     ... (全部 28 文件)
门禁: 100% PASS, 失败则 BLOCK merge
```

#### 任务 D2: 回归测试基线
```yaml
文件: scripts/test/run_regression.sh
基线: 记录当前 3630 个 #[test] 数量
门禁: R6 — test_count 不可减少
输出: regression_baseline.json
```

#### 任务 D3: SQL 缺失命令补齐（高优）
```yaml
来源: SQL Corpus 失败的 SQL + Sysbench 需要的 SQL
已知缺失:
  - INSERT ... ON DUPLICATE KEY UPDATE  ← sysbench 要求
  - DELETE/UPDATE with JOIN               ← sysbench 要求
  - SHOW DATABASES / TABLES               ← mysql-server 协议
  - CREATE INDEX with USING clause        ← SQL corpus
  - ALTER TABLE ADD/DROP COLUMN           ← SQL corpus
  - TRUNCATE TABLE                        ← SQL corpus
  - REPLACE INTO                          ← 已知实现但需验证
门禁: 通过对应的 SQL Corpus 测试用例
```

#### 任务 D4: Sysbench 协议兼容性修复
```yaml
已知问题:
  - MySQL wire protocol auth plugin 名称缺失 (已修复)
  - COM_QUERY 响应格式不完整
  - 预处理语句 (COM_STMT_PREPARE) 不完整
  - EOF packet 编码异常
验证: sysbench point_select.sh --threads=1 可成功运行
      sysbench oltp_read_only.sh --threads=1 可成功运行
门禁: 基础连接 + 查询可通过
```

#### 任务 D5: TPC-H 协议问题修复
```yaml
已知问题:
  - 数据类型不匹配 (REAL vs INTEGER)
  - 缺少标准 SQL 函数 (EXTRACT, DATE)
  - GROUP BY 多列排序不一致
  - 子查询嵌套支持不完整
验证: TPC-H 22 个查询中 ≥ 8 个可执行
门禁: Q1, Q3, Q6, Q11 必须通过
```

#### 任务 D6: CI 门禁检查集成
```yaml
文件: .gitea/workflows/ci.yml
新增 job: test-regression
  步骤:
    1. cargo test --lib --all-features
    2. cargo test --test '*' --all-features
    3. bash scripts/test/run_integration.sh
    4. bash scripts/test/run_regression.sh
触发: 每次 push 到 develop/v2.9.0 + PR
```

---

## Alpha 阶段 (alpha/v2.9.0) — SQL Corpus + Sysbench 门禁

### 目标: 全部 SQL Corpus 通过 + Sysbench 基线建立

#### 任务 A1: SQL Corpus 全量验证
```yaml
目标: 103 个 SQL 文件, 426 用例, 通过率 ≥85%
命令: cargo test -p sqlrustgo-sql-corpus
新增测试:
  - C-02: CTE/WITH 子句 (32 用例)
  - C-03: JSON 操作 (20 用例, 新建)
  - C-04: 窗口函数 (25 用例, 新建)
  - C-05: DISTINCT (15 用例, 已有)
  - C-06: CASE/WHEN (20 用例, 新建)
门禁: BLOCK if < 85%
```

#### 任务 A2: Sysbench 完整 OLTP 套件
```yaml
脚本:
  scripts/sysbench/point_select.sh        ✅ 已有
  scripts/sysbench/oltp_read_only.sh      🆕 新建
  scripts/sysbench/oltp_write_only.sh     🆕 新建
  scripts/sysbench/oltp_read_write.sh     🆕 新建

测试矩阵:
  | 场景      | 线程数 | 时间 | 目标 QPS |
  |-----------|--------|------|---------|
  | Point Select | 8    | 30s  | ≥1,800  |
  | Read Only    | 8    | 30s  | ≥1,500  |
  | Write Only   | 4    | 30s  | ≥500    |
  | Read Write   | 8    | 60s  | ≥1,000  |

数据库: sbtest, table_size=10000
门禁: 全部脚本可运行, QPS 记录基线
```

#### 任务 A3: Alpha CI Job
```yaml
新增 job: test-alpha
  步骤:
    1. cargo test --all-features --test '*'
    2. cargo test -p sqlrustgo-sql-corpus
    3. bash scripts/sysbench/point_select.sh --quick
    4. 覆盖率报告生成 (tarpaulin)
触发: alpha/v2.9.0 分支 push
门禁: 步骤 1-2 BLOCK, 步骤 3-4 记录
```

---

## Beta 阶段 (beta/v2.9.0) — TPC-H 全量 + 安全

### 目标: TPC-H 22 个查询全部可运行 + 安全审计通过

#### 任务 B1: TPC-H 全量查询实现
```yaml
查询清单:
  Q1:  Pricing Summary        ✅ 已有
  Q2:  Minimum Cost Supplier  🆕 需实现
  Q3:  Shipping Priority      ✅ 已有
  Q4:  Order Priority         🆕 需实现
  Q5:  Local Supplier         🆕 需实现
  Q6:  Forecasting Revenue    ✅ 已有
  Q7:  Volume Shipping        🆕 需实现
  Q8:  National Market        🆕 需实现
  Q9:  Product Type Profit    🆕 需实现
  Q10: Returned Items         🆕 需实现
  Q11: Inventory Status       ✅ 已有
  Q12: Shipping Modes         🆕 需实现
  Q13: Customer Distribution  🆕 需实现 (需要 LEFT JOIN)
  Q14: Promotion Effect       🆕 需实现
  Q15: Top Supplier           🆕 需实现 (需要 CREATE VIEW)
  Q16: Parts/Supplier Rel     🆕 需实现
  Q17: Small-Quantity-Order   🆕 需实现
  Q18: Large Volume Customer  🆕 需实现
  Q19: Discounted Revenue     🆕 需实现
  Q20: Potential Part Promo   🆕 需实现
  Q21: Suppliers Who Kept     🆕 需实现 (复杂 JOIN)
  Q22: Global Sales           🆕 需实现

技能依赖: JOIN、子查询、聚合、GROUP BY、ORDER BY、LEFT JOIN、CREATE VIEW、EXISTS
门禁: ≥ 18/22 查询可运行 (≤ 4 因实现限制暂时跳过)
```

#### 任务 B2: TPC-H 性能基线
```yaml
数据规模:
  - Scale Factor 0.01 (快速, ~10MB)
  - Scale Factor 0.1  (中等, ~100MB)
  - Scale Factor 1.0  (标准, ~1GB, 可选)

脚本: scripts/test/run_tpch_baseline.sh
输出: tpch_baseline.json
  { "sf": 0.01, "queries_passed": 18, "total_time_ms": 12345, "qps": 1.5 }
门禁: SF 0.01 全量通过, QPS 记录
```

#### 任务 B3: 安全审计
```yaml
命令: cargo test -p sqlrustgo-security (81 tests)
新增: scripts/gate/check_security.sh + cargo audit
门禁: 100% PASS
```

#### 任务 B4: Beta CI Job
```yaml
新增 job: test-beta
  步骤:
    1. cargo test --all-features
    2. bash scripts/test/run_tpch_baseline.sh --sf=0.01
    3. bash scripts/sysbench/oltp_read_write.sh
    4. cargo test -p sqlrustgo-security
    5. 覆盖率门禁 ≥65%
触发: beta/v2.9.0 分支 push
门禁: 步骤 1,4 必须 PASS
```

---

## RC 阶段 (rc/v2.9.0) — 全量验证 + 性能

### 目标: 全部测试 + 形式化证明 + 稳定性

#### 任务 R1: 全量回归
```yaml
命令: cargo test --all-features
       bash scripts/test/run_integration.sh
       cargo test -p sqlrustgo-sql-corpus
       bash scripts/sysbench/*.sh
       bash scripts/test/run_tpch_baseline.sh --sf=0.1
门禁: 全部 PASS
```

#### 任务 R2: 形式化证明集成
```yaml
证明:
  PROOF-011: 类型系统安全性 (Dafny)
  PROOF-012: WAL ACID (TLA+)
  PROOF-013: B+Tree 不变式 (Dafny)
  PROOF-014: 查询等价性 (Formulog)
命令: bash scripts/verify/*.sh
门禁: P011+P012 verified, P013+P014 记录进度
```

#### 任务 R3: 72h 稳定性测试
```yaml
命令: cargo test --test long_run_stability_72h_test -- --nocapture
运行: 非 CI (定时系统), 72h
门禁: 无 crash, 无内存泄漏
```

#### 任务 R4: RC CI Job
```yaml
新增 job: test-rc
  步骤:
    1. 全量 cargo test --all-features
    2. cargo test -p sqlrustgo-sql-corpus (≥90%)
    3. bash scripts/test/run_tpch_baseline.sh --sf=0.1
    4. bash scripts/sysbench/oltp_read_write.sh (≥5K QPS)
    5. 覆盖率门禁 ≥80%
门禁: 全部必须通过 (BLOCK)
```

---

## GA 阶段 (v2.9.0) — 最终验证

### 目标: 所有前序阶段错误修正 + 性能改进 + 全面审计

#### 任务 G1: 前序错误修正回溯
```yaml
审核: 从 Alpha/Beta/RC 阶段的所有 FAIL 记录
修复: 针对每个已知问题修复
验证: 对应测试重新运行通过
```

#### 任务 G2: 性能改进验证
```yaml
对比:
  | 指标           | Beta 基线 | GA 目标  |
  |---------------|----------|---------|
  | Sysbench OLTP | ≥1K QPS  | ≥10K QPS |
  | TPC-H SF 0.01 | baseline | -10% 内  |
  | 点查询延迟     | <10ms    | <5ms     |
  | 写入吞吐       | ≥500/s   | ≥2K/s    |
```

#### 任务 G3: 混沌工程
```yaml
实验:
  - CPU 80% 压力 + TPC-H 并发
  - 网络 30% 丢包 + Sysbench 持续
  - PostgreSQL 重启 + 查询恢复
  - 磁盘满模拟 + 优雅降级
门禁: 全部自愈, 无数据丢失
```

#### 任务 G4: 覆盖率门禁最终检查
```yaml
命令: cargo tarpaulin --ignore-tests --out Json --output-dir artifacts/coverage/
目标: ≥85%
模块目标:
  executor ≥80%, parser ≥75%, planner ≥70%, storage ≥60%
```

#### 任务 G5: GA 发布检查清单
```yaml
文件: docs/governance/RELEASE_GATE_CHECKLIST.md
包含:
  ☐ Develop 回归: 全部 PASS
  ☐ Alpha SQL Corpus: ≥90% PASS
  ☐ Alpha Sysbench: 基线记录
  ☐ Beta TPC-H: ≥18/22 PASS
  ☐ Beta 安全: 100% PASS
  ☐ RC 形式化证明: P011+P012 verified
  ☐ RC 稳定性: 72h no crash
  ☐ GA 混沌工程: 全部通过
  ☐ GA 覆盖率: ≥85%
  ☐ GA 性能: 达到目标
```

---

## 阶段转换门禁汇总

```yaml
develop → alpha:
  cargo test --lib:              100% PASS
  cargo test --test '*':         100% PASS
  cargo clippy:                  零警告
  cargo fmt:                     无 diff
  SQL Corpus (已有):            ≥85%
  集成测试:                      100% PASS

alpha → beta:
  cargo test --all-features:     100% PASS
  SQL Corpus (全部 103 文件):    ≥90%
  Sysbench point_select:         ≥1,800 QPS
  Sysbench oltp_read_write:      ≥1,000 QPS
  覆盖率:                         ≥65%

beta → rc:
  TPC-H SF 0.01:                ≥18/22 PASS
  Sysbench OLTP:                 ≥5K QPS
  安全测试 (81):                 100% PASS
  覆盖率:                         ≥80%

rc → ga:
  TPC-H SF 0.1:                 全部 PASS
  Sysbench OLTP:                 ≥10K QPS
  形式化证明:                     P011+P012 verified
  72h 稳定性:                    无 crash
  混沌工程:                      全部通过
  覆盖率:                         ≥85%
```
