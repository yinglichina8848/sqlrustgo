# 并行覆盖率测试体系改进

## TL;DR

> **Quick Summary**: 将覆盖率测试从单线程 tarpaulin 迁移到并行 llvm-cov，包含集成/E2E/SQL corpus 测试，逐模块并行执行并生成结构化报告。
> 
> **Deliverables**:
> - 并行覆盖率脚本 `scripts/gate/check_coverage_parallel.sh`
> - 每模块 JSON 覆盖率报告 `artifacts/coverage/<module>.json`
> - 聚合总览报告 `artifacts/coverage/total.json`
> - 更新 `COVERAGE_RC_REPORT.md`
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 4 个并行波
> **Critical Path**: 脚本开发 → 模块分组执行 → 报告聚合

---

## Context

### 原始需求
用户要求检查当前测试体系，将覆盖率测试迁移到 llvm-cov，支持并行多模块覆盖率测试，包含集成测试/E2E/SQL corpus，并评估 llvm-cov 对各种测试类型的支持。

### 现有问题
- `scripts/gate/check_coverage.sh` 使用 cargo tarpaulin（慢、单线程）
- 只收集单元测试覆盖率，未包含集成测试、E2E、SQL corpus
- 输出为单一 coverage.xml，无 per-module 明细

### llvm-cov 能力评估
**结论**: cargo llvm-cov 完全支持所有测试类型纳入覆盖率统计。

| 测试类型 | 命令 | llvm-cov 支持 |
|----------|------|:---:|
| 单元测试 | `--lib` | ✅ |
| 集成测试 | `--test <name>` | ✅ |
| Doc 测试 | `--doc` | ✅ |
| E2E 测试 | `--test e2e_*` | ✅ |
| SQL Corpus | `--package sqlrustgo-sql-corpus` | ✅ |
| Benchmarks | `--bench` | ✅ |
| 全工作区 | `--all-features --workspace` | ✅ |
| JSON 输出 | `--json --output-path` | ✅ |

---

## Work Objectives

### Core Objective
建立并行、分模块的覆盖率测试体系，将当前 ~84% 覆盖率提升至 ≥85% 的同时建立可持续的覆盖率监控机制。

### Concrete Deliverables
- `scripts/gate/check_coverage_parallel.sh` - 并行覆盖率脚本
- `artifacts/coverage/<module>.json` - 每模块覆盖率报告
- `artifacts/coverage/total.json` - 聚合总览
- `docs/releases/v2.9.0/COVERAGE_RC_REPORT.md` - 更新报告

### Definition of Done
- [ ] 所有核心模块覆盖率数据生成
- [ ] 聚合覆盖率 ≥85%
- [ ] 脚本支持 `--parallel N` 参数控制并行度
- [ ] 集成测试被计入覆盖率

### Must Have
- 并行执行（利用多 CPU）
- Per-module JSON 报告
- 包含所有测试类型

### Must NOT Have
- 不删除现有 tarpaulin 脚本（保留兼容）
- 不做性能优化分析
- 不修改任何测试文件内容

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: None (这是工具脚本，不是功能测试)
- **Framework**: cargo llvm-cov

### QA Policy
每个 TODO 包含 Agent-Executed QA Scenarios。

---

## Execution Strategy

### 模块分组（4 波并行）

```
Wave 1 (轻量模块 - 快速完成):
├── common, types, expr (共3个)
├── catalog, query-stats, information-schema (共3个)
└── telemetry, security, tools (共3个)

Wave 2 (中型模块):
├── parser, planner (共2个)
├── optimizer, executor (共2个)
├── network, mysql-server (共2个)
└── transaction, server (共2个)

Wave 3 (重/复杂模块):
├── storage (1个)
├── distributed (1个)
├── vector, graph (共2个)
└── sql-corpus (1个)

Wave 4 (辅助工具):
├── agentsql, gmp (共2个)
├── rag, qmd-bridge (共2个)
└── unified-storage, unified-query (共2个)
```

### 覆盖率聚合流程

```
各模块 llvm-cov JSON
       │
       ▼
  extract lines/regions/functions %
       │
       ▼
  merge into total.json (加权平均)
       │
       ▼
  update COVERAGE_RC_REPORT.md
```

---

## TODOs

- [ ] 1. **创建并行覆盖率脚本骨架**

  **What to do**:
  - 创建 `scripts/gate/check_coverage_parallel.sh`
  - 添加 shebang, set -euo pipefail, 参数解析
  - 支持 `--parallel N` 参数（默认 4）
  - 创建输出目录 `artifacts/coverage/`
  - 定义模块列表（按 Wave 分组）

  **Must NOT do**:
  - 不删除 `check_coverage.sh`
  - 不修改 Cargo.toml 配置

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: none needed

  **References**:
  - `scripts/gate/check_coverage.sh` - 现有脚本结构参考
  - `scripts/gate/check_rc.sh` - gate 检查模式参考

  **QA Scenarios**:
  ```
  Scenario: 脚本可执行且参数有效
    Tool: Bash
    Steps:
      1. bash scripts/gate/check_coverage_parallel.sh --help
      2. 验证输出包含 --parallel N 参数说明
      3. 检查 artifacts/coverage/ 目录已创建
    Expected Result: 目录创建，参数解析正确
    Evidence: .sisyphus/evidence/task-1-script-skeleton.txt
  ```

- [ ] 2. **实现 Wave 1 轻量模块并行覆盖率**

  **What to do**:
  - 在脚本中实现 `run_coverage` 函数
  - 使用后台进程 `&` + `wait` 实现并行
  - Wave 1 模块: common, types, expr, catalog, query-stats, information-schema, telemetry, security, tools
  - 每个模块: `cargo llvm-cov --package <name> --all-features --lib --json --output-path artifacts/coverage/<name>.json`
  - 超时设置: 每模块 300 秒

  **Must NOT do**:
  - 不跳过失败模块（记录错误后继续）

  **QA Scenarios**:
  ```
  Scenario: Wave 1 所有模块完成
    Tool: Bash
    Steps:
      1. bash scripts/gate/check_coverage_parallel.sh --parallel 3 --wave 1
      2. ls artifacts/coverage/common.json artifacts/coverage/types.json artifacts/coverage/expr.json
      3. 每个 JSON 包含 lines/regions/functions 字段
    Expected Result: 9 个 JSON 文件全部生成
    Evidence: .sisyphus/evidence/task-2-wave1.json
  ```

- [ ] 3. **实现 Wave 2 中型模块并行覆盖率**

  **What to do**:
  - Wave 2 模块: parser, planner, optimizer, executor, network, mysql-server, transaction, server
  - 每人模块同时运行 `--lib` 和 `--test` 测试
  - 命令: `cargo llvm-cov --package <name> --all-features --lib --json --output-path <name>.json`

  **QA Scenarios**:
  ```
  Scenario: Wave 2 含集成测试覆盖率
    Tool: Bash
    Steps:
      1. bash scripts/gate/check_coverage_parallel.sh --parallel 4 --wave 2
      2. 验证 executor.json 中存在 coverage 数据
      3. 检查 parser.json 覆盖率 > parser 单独 --lib 的覆盖率
    Expected Result: 8 个 JSON 文件，覆盖率含集成测试数据
    Evidence: .sisyphus/evidence/task-3-wave2.txt
  ```

- [ ] 4. **实现 Wave 3 重要模块覆盖率** (storage, distributed, vector, graph, sql-corpus)

  **What to do**:
  - Wave 3: storage, distributed, vector, graph, sql-corpus
  - storage 模块需包含 all-features
  - sql-corpus 用 `--lib --test`
  - 超时 600s（模块较大）

  **QA Scenarios**:
  ```
  Scenario: SQL Corpus 覆盖率被计入
    Tool: Bash
    Steps:
      1. bash scripts/gate/check_coverage_parallel.sh --parallel 2 --wave 3
      2. 验证 sql-corpus.json 覆盖率 > 0
      3. 检查 distributed.json 存在
  Expected Result: 5 个 JSON 全部生成
    Evidence: .sisyphus/evidence/task-4-wave3.txt

- [ ] 5. **实现覆盖率聚合脚本**

  **What to do**:
  - 创建 `scripts/gate/aggregate_coverage.sh`
  - 读取 `artifacts/coverage/*.json` 所有模块报告
  - 从每个 JSON 提取: lines (covered/total), regions, functions
  - 计算加权总覆盖率并写入 `artifacts/coverage/total.json`
  - 输出格式化表格到 stdout

  **References**:
  - llvm-cov JSON 格式: `{"data":[{"totals":{"lines":{"count":N,"covered":N,"percent":NN}}]}`

  **QA Scenarios**:
  ```
  Scenario: 聚合报告包含所有模块
    Tool: Bash
    Steps:
      1. bash scripts/gate/aggregate_coverage.sh
      2. 验证 total.json 包含 total_lines_pct 字段
      3. 验证 total.json 包含 modules 数组，长度 ≥20
      4. 检查 stdout 表格中 executor 覆盖率 ≥70%
    Expected Result: 聚合报告正确，覆盖率数值合理
    Evidence: .sisyphus/evidence/task-5-aggregate.txt
  ```

- [ ] 6. **E2E 测试覆盖率集成**

  **What to do**:
  - 独立运行 E2E 测试覆盖: `cargo llvm-cov --test e2e_query_test --json --output-path artifacts/coverage/e2e.json`
  - 同样处理 e2e_observability_test, e2e_monitoring_test
  - 对比仅单元测试 vs 含 E2E 的覆盖率差异
  - 将 E2E 覆盖数据合并到汇总中

  **QA Scenarios**:
  ```
  Scenario: E2E 测试产生覆盖率数据
    Tool: Bash
    Steps:
      1. cargo llvm-cov --test e2e_query_test --all-features --json --output-path artifacts/coverage/e2e_query.json
      2. python3 -c "import json; d=json.load(open('artifacts/coverage/e2e_query.json')); assert d['data'][0]['totals']['lines']['covered'] > 0"
    Expected Result: E2E 覆盖率 >0
    Evidence: .sisyphus/evidence/task-6-e2e.txt
  ```

- [ ] 7. **完整端到端运行 + 覆盖率报告更新**

  **What to do**:
  - 清空 artifacts/coverage/ 目录
  - 运行全部 Wave (1-4)：`bash scripts/gate/check_coverage_parallel.sh --parallel 4`
  - 运行聚合：`bash scripts/gate/aggregate_coverage.sh`
  - 更新 `docs/releases/v2.9.0/COVERAGE_RC_REPORT.md` 使用新数据
  - 执行 `bash scripts/gate/check_rc.sh` 验证 B1/B2 gate 通过

  **QA Scenarios**:
  ```
  Scenario: 完整流程通过 RC Gate
    Tool: Bash
    Steps:
      1. rm -rf artifacts/coverage/
      2. time bash scripts/gate/check_coverage_parallel.sh --parallel 4 2>&1 | tee /tmp/coverage_run.log
      3. bash scripts/gate/aggregate_coverage.sh
      4. 检查 total.json 中 lines.percent ≥ 85
      5. 检查 artifacts/coverage/executor.json lines.percent ≥ 60
    Expected Result: 总体 ≥85%, executor ≥60%, 耗时 <15min
    Evidence: .sisyphus/evidence/task-7-full-run.log
  ```

- [ ] 8. **验证与文档更新**

  **What to do**:
  - 对比新旧覆盖率数据差异
  - 更新 COVERAGE_RC_REPORT.md 写入 Phase 3 结果
  - 提交所有变更

  **QA Scenarios**:
  ```
  Scenario: 覆盖率报告数据一致
    Tool: Bash
    Steps:
      1. python3 -c "import json; d=json.load(open('artifacts/coverage/total.json')); lines=d.get('data',[{}])[0].get('totals',{}).get('lines',{}); print(f'Total: {lines.get(\"percent\",0):.2f}%')"
      2. grep "Overall Lines Coverage" docs/releases/v2.9.0/COVERAGE_RC_REPORT.md
    Expected Result: 报告数值与 JSON 一致
    Evidence: .sisyphus/evidence/task-8-verify.txt
  ```

---

## Final Verification Wave

- [ ] F1. 运行脚本验证所有模块覆盖率报告生成
- [ ] F2. 检查 total.json 覆盖率 ≥85%
- [ ] F3. 验证集成测试覆盖率 >0（说明集测被计入）

## Commit Strategy

- **1**: `feat(scripts): Add parallel llvm-cov coverage script` - 新脚本
- **2**: `docs: Update COVERAGE_RC_REPORT.md with parallel results` - 更新报告

## Success Criteria

### Verification Commands
```bash
# 并行覆盖率
bash scripts/gate/check_coverage_parallel.sh --parallel 4

# 验证每个模块报告
ls artifacts/coverage/*.json | wc -l  # ≥20 个模块

# 总覆盖率
python3 -c "import json; d=json.load(open('artifacts/coverage/total.json')); print(d['lines_pct'])"
```
