# v2.6.0 发布门禁检查清单

> **版本**: beta/v2.6.0
> **当前阶段**: `Beta`
> **更新日期**: 2026-04-20
> **说明**: 门禁检查通过引用测试报告验证，不直接执行测试

---

## 一、门禁原则

### 1.1 核心原则

1. **报告驱动**: 门禁检查引用测试报告中的数据，不直接执行测试
2. **证据要求**: 每项检查必须提供报告路径、commit hash、执行日期
3. **阶段隔离**: 不同阶段检查不同报告，不可混淆

### 1.2 证据要求（统一）

| 字段 | 说明 | 示例 |
|------|------|------|
| 报告路径 | 测试报告文件位置 | `report/BETA_TEST_REPORT.md` |
| Commit Hash | 代码版本 | `aac4d64c` |
| 执行日期 | 测试执行日期 | 2026-04-20 |
| 结果摘要 | 通过/失败统计 | "通过 481/481" |
| 产物路径 | 报告或日志位置 | `artifacts/coverage/` |

---

## 二、Alpha 门禁

### 2.1 检查清单

| 检查项 | 引用报告 | 阈值 | 证据要求 |
|--------|----------|------|----------|
| L0 冒烟 | `report/ALPHA_TEST_REPORT.md` - L0 节 | 100% | build, fmt, clippy, smoke |
| L1 parser | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | parser 单测结果 |
| L1 planner | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | planner 单测结果 |
| L1 executor | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | executor 单测结果 |
| L1 storage | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | storage 单测结果 |
| L1 optimizer | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | optimizer 单测结果 |
| L1 transaction | `report/ALPHA_TEST_REPORT.md` - L1 节 | 100% | transaction 单测结果 |

### 2.2 Alpha 通过条件

- [ ] 所有 L0 冒烟测试通过
- [ ] 所有 L1 模块测试通过
- [ ] 报告已生成并位于 `report/ALPHA_TEST_REPORT.md`

---

## 三、Beta 门禁

### 3.1 检查清单

| 检查项 | 引用报告 | 阈值 | 证据要求 |
|--------|----------|------|----------|
| L2 CBO | `report/BETA_TEST_REPORT.md` - L2 节 | 100% | cbo_integration_test |
| L2 WAL | `report/BETA_TEST_REPORT.md` - L2 节 | 100% | wal_integration_test |
| L2 Regression | `report/BETA_TEST_REPORT.md` - L2 节 | 100% | regression_test |
| L2 E2E | `report/BETA_TEST_REPORT.md` - L2 节 | 100% | e2e_*_test |
| SQL Corpus | `report/BETA_TEST_REPORT.md` - L4 节 | ≥95% | sql-corpus 通过率 |
| 覆盖率 | `report/BETA_TEST_REPORT.md` - 覆盖率节 | ≥65% | tarpaulin 报告 |

### 3.2 Beta 通过条件

- [ ] 所有 L2 集成测试通过
- [ ] SQL Corpus 通过率 ≥95%
- [ ] 覆盖率 ≥65%
- [ ] 报告已生成并位于 `report/BETA_TEST_REPORT.md`

---

## 四、RC 门禁

### 4.1 检查清单

| 检查项 | 引用报告 | 阈值 | 证据要求 |
|--------|----------|------|----------|
| 全量 L0~L2 | `report/RC_TEST_REPORT.md` | 100% | 所有测试汇总 |
| 覆盖率 | `report/RC_TEST_REPORT.md` - 覆盖率节 | ≥70% | tarpaulin 报告 |
| TPC-H SF1 | `report/RC_TEST_REPORT.md` - L3 节 | 通过 | bench_tpch 结果 |
| Sysbench | `report/RC_TEST_REPORT.md` - L3 节 | ≥1000 QPS | 外部工具报告 |
| 备份恢复 | `report/RC_TEST_REPORT.md` - L3 节 | 通过 | 手动测试记录 |
| 崩溃恢复 | `report/RC_TEST_REPORT.md` - L3 节 | 通过 | kill -9 测试记录 |

### 4.2 RC 通过条件

- [ ] L0~L2 全部测试通过
- [ ] 覆盖率 ≥70%
- [ ] TPC-H SF1 基准测试通过
- [ ] 备份恢复测试通过
- [ ] 报告已生成并位于 `report/RC_TEST_REPORT.md`

---

## 五、GA 门禁

### 5.1 检查清单

| 检查项 | 引用报告 | 阈值 | 证据要求 |
|--------|----------|------|----------|
| 72h 长稳 | `report/GA_TEST_REPORT.md` - 长稳节 | 稳定 | 压测报告 |
| 全量 A 类为绿 | `report/GA_TEST_REPORT.md` | 100% | 所有检查项汇总 |
| 发布文档完整 | `report/GA_TEST_REPORT.md` - 文档节 | 通过 | 完整性检查 |
| 回滚演练 | `report/GA_TEST_REPORT.md` - 回滚节 | 通过 | 回滚测试记录 |

### 5.2 GA 通过条件

- [ ] 72h 长稳测试通过
- [ ] 所有 A 类检查项为绿
- [ ] 发布文档完整
- [ ] 回滚演练通过
- [ ] 报告已生成并位于 `report/GA_TEST_REPORT.md`

---

## 六、门禁流程

### 6.1 标准流程

```
1. 执行测试 → 生成测试报告
2. 门禁检查 → 引用报告验证
3. 通过 → 进入下一阶段
4. 失败 → 修复后重新执行测试
```

### 6.2 报告命名规范

| 阶段 | 报告路径 |
|------|----------|
| Alpha | `report/ALPHA_TEST_REPORT.md` |
| Beta | `report/BETA_TEST_REPORT.md` |
| RC | `report/RC_TEST_REPORT.md` |
| GA | `report/GA_TEST_REPORT.md` |

### 6.3 报告验证清单

每项检查必须验证：

- [ ] 报告文件存在
- [ ] Commit Hash 与待发布代码一致
- [ ] 执行日期在合理范围内
- [ ] 结果数据完整

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
| 2.0 | 2026-04-20 | 重构：改为引用报告而非直接测试 |

---

## 八、元数据

| 字段 | 值 |
|------|------|
| 工作目录 | /Users/liying/workspace/dev/yinglichina163/sqlrustgo |
| 当前版本 | v2.6.0 |
| 工作分支 | develop/v2.6.0 |

---

*门禁检查清单 v2.6.0*
*本文件规定门禁检查流程，通过引用测试报告验证*
*测试执行和结果见对应阶段的测试报告*