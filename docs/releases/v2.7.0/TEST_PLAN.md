# v2.7.0 全面测试计划（MySQL 5.7 生产可用 + GMP 检索能力）

> 版本: `v2.7.0`  
> 目标: 覆盖所有已规划/已集成功能，形成可发布的综合评估证据  
> 更新日期: 2026-04-19

---

## 1. 测试目标与原则

1. 全覆盖：单元、集成、性能、覆盖率、SQL Corpus、TPC-H、Sysbench、安装、备份恢复、崩溃恢复、长稳。
2. 全链路：每个功能至少具备“单元 + 集成 + 回归”三层证据。
3. 可复现：每项结论必须有命令、环境、commit、产物路径。
4. 可发布：测试计划直接映射到 RC/GA 门禁。

---

## 2. 测试分层（L0~L3）

### L0 冒烟（5-10 分钟）

目标：快速判断分支可用性。

必测：
1. 构建/格式/静态检查（`cargo build`, `cargo fmt --check`, `cargo clippy`）
2. 核心路径冒烟（SQL 主路径、WAL 基础、SQL Corpus 快速集）
3. 基本安装与启动（本地二进制启动 + 简单查询）

### L1 模块回归（20-40 分钟）

目标：核心 crate 行为正确。

必测：
1. parser/planner/optimizer/executor/storage/transaction/server
2. vector/graph/unified-query/unified-storage/gmp
3. 关键边界与错误路径

### L2 集成回归（40-90 分钟）

目标：跨模块、跨引擎、跨协议验证。

必测：
1. SQL->Planner->Executor->Storage 全链路
2. 事务 + WAL + 约束 + 恢复
3. SQL + Vector + Graph + RAG 混合检索
4. MySQL 协议兼容与连接池

### L3 深度验证（夜间/周跑）

目标：发布级稳定性与性能。

必测：
1. TPC-H（SF1 必跑，SF10 夜间）
2. Sysbench（OLTP）
3. 72h 长稳与崩溃注入
4. 备份恢复与回滚演练

---

## 3. 测试矩阵（功能 -> 用例 -> 证据）

## 3.1 单元测试（Unit）

覆盖模块：
1. `crates/parser`
2. `crates/planner`
3. `crates/optimizer`
4. `crates/executor`
5. `crates/storage`
6. `crates/transaction`
7. `crates/server`
8. `crates/vector`
9. `crates/graph`
10. `crates/gmp`
11. `crates/unified-query`
12. `crates/unified-storage`

通过标准：
1. 模块单测通过率 100%
2. 关键模块（parser/executor/storage/transaction）行覆盖率 >= 75%

---

## 3.2 集成测试（Integration）

覆盖场景：
1. SQL DDL/DML 主路径（含 JOIN/GROUP BY/HAVING/Subquery）
2. 事务隔离（RC/RR/Serializable 路径）
3. WAL 崩溃恢复
4. FK/约束检查与级联
5. Prepared/Procedure/Trigger（若版本内声明支持）
6. MySQL 协议访问 + 多连接并发
7. SQL + Vector + Graph 联合查询
8. GMP 审核证据查询（Top10 场景）

通过标准：
1. P0 集成用例通过率 100%
2. 全部集成用例通过率 >= 95%

---

## 3.3 覆盖率测试（Coverage）

工具：
1. `cargo tarpaulin`（或 `cargo llvm-cov`，按 CI 环境固定一种主工具）

覆盖率门槛：
1. 全仓总覆盖率 >= 70%
2. 关键路径覆盖率：
- parser >= 80%
- executor >= 75%
- storage >= 75%
- transaction >= 80%
- server >= 70%

---

## 3.4 SQL Corpus 测试

目标：
1. SQL 语法回归通过率 >= 95%
2. P0 语法（SELECT/INSERT/UPDATE/DELETE/JOIN/GROUP/HAVING）必须 100%

要求：
1. 记录 case 总量、通过量、失败量、失败分类
2. 输出按类别统计（DML/DDL/Transactions/Expressions）

---

## 3.5 TPC-H 测试

必跑：
1. SF=1 全量查询回归（RC/GA 必跑）

夜间跑：
1. SF=10（性能趋势）

目标：
1. 正确性通过率 100%
2. 与 v2.6.0 基线对比，核心查询不退化

---

## 3.6 Sysbench 测试

场景：
1. `oltp_point_select`
2. `oltp_read_only`
3. `oltp_read_write`

指标：
1. QPS/TPS
2. P95/P99 延迟
3. 错误率/超时率

目标：
1. 达到 v2.7.0 目标阈值（由性能门禁统一维护）
2. 与 v2.6.0 相比不出现显著回退

---

## 3.7 安装与升级测试

覆盖：
1. 源码构建安装
2. 二进制包安装（macOS/Linux）
3. 基础配置启动
4. 从 v2.6.0 升级到 v2.7.0（配置与数据兼容）

通过标准：
1. 安装手册命令可执行
2. 升级后核心业务 SQL 可正常执行

---

## 3.8 备份/恢复/崩溃测试

覆盖：
1. 全量备份 + 恢复
2. 增量备份（若版本内支持）
3. 崩溃恢复（`kill -9` / 进程异常退出）
4. 回滚演练（版本回退 + 数据一致性）

通过标准：
1. 数据完整性校验通过
2. 恢复时间符合 RTO 目标
3. 数据丢失窗口符合 RPO 目标

---

## 4. 执行节奏

日常：
1. 每日 L0 + L1

合并前：
1. L0 + L1 必跑
2. 影响核心路径时补跑 L2

夜间：
1. L3（TPC-H/Sysbench/长稳/崩溃）

发布前：
1. 全量 L0~L3
2. 生成综合评估报告

---

## 5. 综合评估与交付物

必须产出：
1. `docs/releases/v2.7.0/PERFORMANCE_REPORT.md`
2. `docs/releases/v2.7.0/COVERAGE_REPORT.md`
3. `docs/releases/v2.7.0/SECURITY_REPORT.md`
4. `docs/releases/v2.7.0/STABILITY_REPORT.md`
5. `docs/releases/v2.7.0/TEST_SUMMARY.md`

每份报告必须包含：
1. 环境（CPU/内存/OS/Rust）
2. commit hash
3. 执行命令
4. 原始结果与产物路径（`artifacts/`）
5. 结论与风险

---

## 6. 发布门禁映射（摘要）

1. Alpha：L0/L1 全绿 + 核心恢复链路通过
2. Beta：L2 全绿 + SQL Corpus >= 95%
3. RC：TPC-H SF1 + Sysbench + 覆盖率达标
4. GA：72h 长稳 + 备份恢复 + 崩溃恢复 + 回滚演练全通过

