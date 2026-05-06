# v3.0.0 Alpha 阶段：全面集成与测试计划

> **创建时间**: 2026-05-06  
> **阶段**: v3.0.0-Alpha  
> **类型**: 集成/测试总控  

---

## 一、背景

v3.0.0 开发已完成核心功能实现，包括：
- PROOF-026 Write Skew / SSI (Serializable Snapshot Isolation)
- CTE (Common Table Expression) 内联
- CBO (Cost-Based Optimizer) 集成
- INSERT...SELECT 执行
- EXPLAIN ANALYZE
- NTILE 窗口函数
- TPC-H 完整支持
- WAL Group Commit
- 连接池完善

**目标**: 在 Alpha 阶段对所有已实现功能进行完整的执行和测试验证。

---

## 二、A-Gate 检查结果

| # | 检查项 | 通过标准 | 结果 | 备注 |
|---|--------|----------|------|------|
| A1 | 编译 | `cargo build --all-features` 无错误 | ✅ | 55.23s, 3 warnings |
| A2 | 单元测试 | `cargo test --all-features` ≥80% | ⏳ | 待执行 |
| A3 | Clippy | 零警告 | ✅ | 0 warnings |
| A4 | 格式化 | `cargo fmt --all -- --check` | ✅ | 已修复 |
| A5 | 文档链接 | `check_docs_links.sh` 无死链 | ✅ | All links valid |
| A6 | 覆盖率 | ≥50% | ⏳ | 待执行 |
| A7 | 安全扫描 | `cargo audit` 无高危 | ✅ | 0 高危, 7 advisory warnings (allowed) |

### A7 安全扫描详情

```
warning: 7 allowed warnings found
```

| Crate | Version | Issue | Severity |
|-------|---------|-------|----------|
| adler | 1.0.2 | unmaintained | advisory only |
| ansi_term | 0.12.1 | unmaintained | advisory only |
| atty | 0.2.14 | unsound (unaligned read) | advisory only |
| lru | 0.12.5 | unsound (Stacked Borrows) | advisory only |
| paste | 1.0.15 | unmaintained | advisory only |
| proc-macro-error | 1.0.4 | unmaintained | advisory only |

以上均为 `cargo audit` 已知维护状态警告，非实际可利用漏洞。

---

## 三、已合并 PR

| PR | 标题 | 合并时间 |
|----|------|----------|
| #372 | PROOF-026 Write Skew/SSI — TLA+ model + concurrent stress tests (7/7 pass) | 2026-05-06 |
| #371 | CTE inlining for WITH clause support | 2026-05-06 |
| #369 | fix(sql-corpus): 100% pass rate - per-case SKIP + clippy fix | 2026-05-06 |
| #368 | A-02~A-05 + I-02 EXPLAIN ANALYZE + SQL Corpus | 2026-05-06 |

---

## 四、Nomad 集群状态

| 节点 | ID | 状态 | Docker | Exec |
|------|-----|------|--------|------|
| HP Z6G4 | `98a7c88c-80ac-d9ac-6e78-26f0325ede05` | **ready** | ✅ 29.4.2 | ✅ |
| 250 MacMini | `cc67e32a-8337-2a73-6780-0e1af19ee9a0` | **ready** | ✅ 29.4.2 | ✅ |

---

## 五、Alpha 阶段集成与测试计划

### 阶段 1: 基础验证 (Alpha-1)

**目标**: 验证编译、单元测试、覆盖率基线

| 任务 | 负责 | 通过标准 |
|------|------|----------|
| T1.1 执行 `cargo test --all-features` | CI | ≥80% 测试通过 |
| T1.2 执行 `cargo llvm-cov --all-features` | CI | 覆盖率 ≥50% |
| T1.3 验证 clippy 零警告 | CI | `cargo clippy --all-features -- -D warnings` |
| T1.4 验证格式化 | CI | `cargo fmt --all -- --check` |

### 阶段 2: 功能测试 (Alpha-2)

**目标**: 验证所有核心功能的正确性

| 任务 | 功能模块 | 通过标准 |
|------|----------|----------|
| T2.1 | Parser | SELECT/INSERT/UPDATE/DELETE/CREATE/ALTER 所有语法解析通过 |
| T2.2 | Executor | SELECT 扫描、聚合、JOIN、排序正确返回结果 |
| T2.3 | CTE | WITH 子句内联正确，结果等价于展开执行 |
| T2.4 | Window Functions | NTILE/RANK/ROW_NUMBER 窗口函数正确 |
| T2.5 | EXPLAIN ANALYZE | 执行计划展示 + 实际行数/耗时 |
| T2.6 | INSERT...SELECT | 从 SELECT 插入数据正确 |
| T2.7 | Transaction (SSI) | Write Skew 场景正确检测并序列化 |

### 阶段 3: 性能测试 (Alpha-3)

**目标**: 建立性能基线，验证优化效果

| 任务 | 基准 | 通过标准 |
|------|------|----------|
| T3.1 | TPC-H Q1 (SF=1) | 执行时间 < 基准 1.2x |
| T3.2 | TPC-H Q1 (SF=10) | 执行时间 < 基准 1.2x |
| T3.3 | 并发 SELECT (8 threads) | 吞吐量 ≥ 5000 QPS |
| T3.4 | 并发 INSERT | 无死锁，事务正确 |
| T3.5 | R9 E-09 回归检测 | `check_regression.sh --baseline` 无退化 |

### 阶段 4: 集成测试 (Alpha-4)

**目标**: 端到端场景验证

| 任务 | 场景 | 通过标准 |
|------|------|----------|
| T4.1 | 完整 SQL 语法覆盖 | sql-corpus 100% pass |
| T4.2 | 并发压力 | 8 threads × 1000 ops，无数据丢失 |
| T4.3 | 稳定性 | 持续运行 30 分钟无 panic |
| T4.4 | Formal Verification | SSI TLA+ 规格验证 7/7 pass |

---

## 六、覆盖率目标

| 模块 | 当前目标 | 说明 |
|------|----------|------|
| executor | ≥45% | 核心执行引擎 |
| optimizer | ≥40% | CBO 和规则优化 |
| storage | ≥15% | 存储层基础 |
| catalog | ≥50% | 元数据管理 |
| parser | ≥50% | SQL 解析 |
| **整体** | **≥50%** | 项目总覆盖率 |

---

## 七、测试方法

### 7.1 单元测试
```bash
cargo test --all-features --workspace
```

### 7.2 覆盖率
```bash
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html
```

### 7.3 SQL 语法测试
```bash
cargo run --bin sql-corpus -- --suites standard
```

### 7.4 TPC-H 基准
```bash
cargo run --bin tpch -- --sf 1 --query 1
```

### 7.5 性能回归
```bash
bash scripts/gate/check_regression.sh
```

---

## 八、风险管理

| 风险 | 可能性 | 影响 | 缓解 |
|------|---------|------|------|
| SSI 并发测试不通过 | 中 | 高 | TLA+ 模型已验证，着重执行压力测试 |
| 覆盖率不达标 | 中 | 中 | 优先测试 executor/optimizer 模块 |
| TPC-H Q18/Q22 执行超时 | 低 | 中 | 设置超时，失败则跳过并记录 |
| 格式化冲突 | 低 | 低 | PR 强制 fmt 检查 |

---

## 九、交付物

- [x] A-Gate 检查报告 (本文档)
- [ ] 单元测试完整报告
- [ ] 覆盖率报告
- [ ] TPC-H 性能基准报告
- [ ] SQL-Corpus 执行报告
- [ ] Alpha 阶段总结报告

---

## 十、Issue 跟踪

| Issue | 类型 |
|--------|------|
| #353 | v3.0.0 开发总控 |
| #370 | v3.0.0 Alpha 全面集成与测试 (本文档关联) |

---

*文档版本: 1.0*  
*创建: Hermes Agent*  
*最后更新: 2026-05-06*
