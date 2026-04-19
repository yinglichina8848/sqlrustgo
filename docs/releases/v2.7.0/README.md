# v2.7.0 文档索引

> 版本: `v2.7.0`  
> 代号: `Production+Compliance`  
> 当前状态: `规划中`（计划从 `alpha` 启动）  
> 最后更新: 2026-04-19

---

## 一、版本定位

`v2.7.0` 是 SQLRustGo 的生产化版本，目标是：

1. 达到 MySQL 5.7 普通中小企业生产可用水平（必要子集）
2. 支持 GMP 合规审核场景的多路检索
3. 可集成 QMD（全文/语义/重排）能力

---

## 二、核心目标

### 2.1 生产能力目标

1. SQL 主路径稳定可用（DDL/DML/JOIN/GROUP BY/HAVING/子查询）
2. 事务/WAL/恢复链路完整
3. 约束与索引主路径可用
4. 备份恢复、监控、慢查询与连接管理可用

### 2.2 检索能力目标

1. 全文检索（关键词/BM25）
2. 语义检索（向量召回 + 重排）
3. 图检索（关系追溯/影响路径）
4. 混合检索（统一入口与证据包输出）

---

## 三、文档清单

| 文档 | 说明 | 状态 |
|------|------|------|
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 版本定义与分阶段开发计划 | ✅ |
| [VERSION_PLAN.md](./VERSION_PLAN.md) | 里程碑、任务矩阵、交付节奏 | ✅ |
| [TEST_PLAN.md](./TEST_PLAN.md) | 全面测试计划（单测/集成/性能/恢复） | ✅ |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | Alpha/Beta/RC/GA 门禁清单 | ✅ |

---

## 四、阶段路线

1. `Phase A`: 内核生产化（事务、约束、恢复、门禁基线）
2. `Phase B`: 检索融合（lex/vec/graph/hybrid + qmd-bridge）
3. `Phase C`: GMP 场景化（Top 10 审核查询 + 证据链）
4. `Phase D`: RC/GA 冲刺（性能、安全、长稳、发布冻结）

---

## 五、计划时间线

| 里程碑 | 计划日期 | 目标 |
|--------|----------|------|
| v2.7.0-alpha | 2026-06-15 | 完成 Phase A |
| v2.7.0-beta | 2026-07-15 | 完成 Phase B/C 核心能力 |
| v2.7.0-rc1 | 2026-08-01 | 完成 Phase D 核心门禁 |
| v2.7.0-ga | 2026-08-20 | 全部门禁通过并发布 |

---

## 六、相关文档

1. [v2.6.0 文档索引](../v2.6.0/README.md)
2. [长期路线图](../LONG_TERM_ROADMAP.md)
3. [版本演化计划](../VERSION_ROADMAP.md)
4. [GMP 开发计划](../../gmp-audit-db-development-plan.md)
5. [GMP 二次评估](../../gmp-audit-db-evaluation-report.md)
