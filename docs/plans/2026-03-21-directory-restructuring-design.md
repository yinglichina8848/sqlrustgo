# SQLRustGo 目录结构重构设计

## 1. 背景与目标

### 1.1 当前问题

当前项目存在以下目录混乱问题：

| 问题 | 现状 |
|------|------|
| 性能测试目录重复 | `bench/`、`benches/`、`benchmark/`、`benchmarks/` 四个目录并存 |
| 功能重叠 | `benchmark/` 与 `crates/bench/` 职责不清 |
| 测试命名不统一 | `*_test.rs`、`*_tests.rs`、`ci_test.rs` 混杂 |
| 数据管理分散 | `data/`、`tests/data`、`benches/dataset` 各自存放测试数据 |
| 脚本分散 | `scripts/`、`benchmark/`、`scripts/benchmark/` 多处存放脚本 |

### 1.2 重构目标

1. **符合 Rust 开源项目惯例** - 遵循 Cargo workspace 规范
2. **分层清晰** - 单元测试、集成测试、E2E 测试、性能测试完全分层
3. **职责明确** - 每个目录有单一职责
4. **易于维护** - 统一的命名规范和数据管理

---

## 2. 目标目录结构

### 2.1 整体结构

```
sqlrustgo/
├── src/                      # 主程序入口（可选，当前在根目录）
├── Cargo.toml                # workspace 配置
├── Cargo.lock
├── VERSION                   # 版本号文件
├── tarpaulin.toml            # 覆盖率配置
│
├── crates/                   # 核心代码（workspace members）
│   ├── common/
│   ├── parser/
│   ├── planner/
│   ├── optimizer/
│   ├── executor/
│   ├── storage/
│   ├── catalog/
│   ├── transaction/
│   ├── server/
│   ├── bench-cli/            # 基准测试 CLI 工具
│   └── types/
│
├── benches/                  # 所有性能测试代码
│   ├── Cargo.toml            # bench 入口
│   ├── benches/              # criterion benchmarks（微基准）
│   │   ├── bench_aggregate.rs
│   │   ├── bench_index_scan.rs
│   │   ├── bench_insert.rs
│   │   ├── bench_scan.rs
│   │   ├── lexer_bench.rs
│   │   ├── parser_bench.rs
│   │   └── storage_bench.rs
│   ├── tpch/                 # TPC-H 综合基准
│   │   ├── tpch_bench.rs
│   │   ├── tpch_comprehensive.rs
│   │   └── tpch_streaming_config.rs
│   ├── dataset/              # 基准测试数据集
│   │   ├── generator.rs
│   │   ├── sf01.json
│   │   └── sf1.json
│   └── queries/              # SQL 查询文件
│       └── *.sql
│
├── tests/                    # 集成测试（跨 crate）
│   ├── data/                 # 测试数据
│   │   ├── test_schema.json
│   │   └── sample_data.json
│   ├── e2e/                  # 端到端测试
│   │   ├── e2e_query_test.rs
│   │   └── observability_test.rs
│   ├── integration/          # 模块集成测试
│   │   ├── executor_integration_test.rs
│   │   ├── planner_integration_test.rs
│   │   └── storage_integration_test.rs
│   ├── stress/               # 压力测试
│   │   ├── stress_test.rs
│   │   ├── concurrency_stress_test.rs
│   │   └── crash_recovery_test.rs
│   ├── ci/                   # CI 专用测试
│   │   ├── ci_test.rs
│   │   └── buffer_pool_test.rs
│   ├── unit/                 # 独立单元测试（非模块内）
│   │   └── optimizer_rules_test.rs
│   └── README.md             # 测试说明
│
├── benchmark_results/         # 性能测试结果和报告
│   ├── sf01/
│   │   ├── baseline.json
│   │   └── results/
│   ├── sf1/
│   └── reports/
│       └── BENCHMARK_REPORT.md
│
├── scripts/                   # 统一脚本
│   ├── bench.sh              # 统一 benchmark 入口
│   ├── install.sh            # 安装脚本
│   ├── full_benchmark.sh     # 完整基准测试
│   ├── compare_benchmarks.py # 结果比较脚本
│   ├── sqlite_bench.sh      # SQLite 基准测试
│   ├── cleanup.sh            # 清理脚本
│   ├── gate/                 # gate 检查脚本
│   └── ci/                   # CI 辅助脚本
│
├── docs/                      # 文档
│   ├── plans/                # 设计文档
│   ├── releases/             # 发布文档
│   ├── issues/               # Issue 追踪
│   ├── architecture/         # 架构文档
│   ├── POSTGRESQL_SETUP.md   # PostgreSQL 设置
│   └── BENCHMARK_GUIDE.md    # 基准测试指南
│
└── .github/
    └── workflows/            # CI/CD 工作流
        ├── ci.yml           # 主 CI
        ├── bench-pr.yml     # PR benchmark
        ├── bench-schedule.yml # 定时 benchmark
        └── benchmark.yml    # 基准测试 workflow
```

### 2.2 目录职责表

| 目录 | 职责 | 迁移来源 |
|------|------|----------|
| `crates/` | 核心业务代码 | 保持不变 |
| `crates/bench-cli/` | Benchmark CLI 工具 | 保持不变 |
| `benches/` | 所有性能测试代码 | 从 `benches/`、`crates/bench/` 迁移 |
| `tests/` | 集成/E2E/压力测试 | 从根目录 `tests/` 迁移并重组 |
| `benchmark_results/` | 性能测试结果 | 从 `benchmark_results/` 迁移并清理 |
| `scripts/` | 统一脚本入口 | 合并 `scripts/`、`benchmark/` |

---

## 3. 测试分层设计

### 3.1 测试层次结构

```
┌─────────────────────────────────────────────────────────────┐
│                         CI/CD 流程                          │
│    - 自动执行测试 & 覆盖率报告                                │
│    - benchmark 报告生成 & 阈值检查                            │
└─────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┴─────────────────┐
            │                                   │
   ┌─────────────────────┐           ┌─────────────────────┐
   │     单元测试          │           │    模块集成测试       │
   │   (Unit Test)        │           │ (Module Integration) │
   │ crates/*/src 内嵌     │           │ tests/integration/   │
   │ #[cfg(test)] mod     │           │ crates/*/tests/       │
   │ 模块级测试            │           │ 模块间调用            │
   └─────────────────────┘           └─────────────────────┘
            │                                   │
            └─────────────────┬─────────────────┘
                              │
                    ┌─────────────────────┐
                    │   系统 / E2E 测试     │
                    │   (System / E2E)     │
                    │   tests/e2e/         │
                    │   tests/stress/      │
                    │ 独立数据库/临时 schema │
                    └─────────────────────┘
                              │
                              │
                    ┌─────────────────────┐
                    │   性能 / Benchmark   │
                    │   (Benchmark)        │
                    │   benches/           │
                    │   benchmark_results/ │
                    └─────────────────────┘
                              │
                              │
                    ┌─────────────────────┐
                    │     测试数据管理      │
                    │ tests/data/          │
                    │ benches/dataset/     │
                    │ DataLoader helper    │
                    └─────────────────────┘
```

### 3.2 测试命名规范

| 类型 | 位置 | 文件命名 | 示例 |
|------|------|----------|------|
| 单元测试 | `crates/*/src` 内 | `#[cfg(test)] mod tests {}` | 内嵌于源文件 |
| 模块集成 | `crates/*/tests/` | `*_integration.rs` | `executor_integration.rs` |
| E2E 测试 | `tests/e2e/` | `e2e_*.rs` | `e2e_query_test.rs` |
| 压力测试 | `tests/stress/` | `*_stress.rs` | `stress_test.rs` |
| CI 测试 | `tests/ci/` | `ci_*.rs` | `ci_test.rs` |
| 微基准 | `benches/benches/` | `bench_*.rs` | `bench_scan.rs` |
| TPC-H | `benches/tpch/` | `tpch_*.rs` | `tpch_bench.rs` |

### 3.3 数据加载机制

提供统一的 DataLoader helper：

```rust
// tests/data_loader.rs
pub struct TestDataLoader;

impl TestDataLoader {
    pub fn load_json(path: &str) -> serde_json::Value {
        let full_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join(path);
        serde_json::from_str(&std::fs::read_to_string(full_path).unwrap()).unwrap()
    }

    pub fn load_schema() -> Schema {
        Self::load_json("test_schema.json")
    }
}
```

---

## 4. 迁移计划

### 4.1 阶段一：准备（不影响开发）

1. **创建目标目录结构**
2. **创建 `tests/data_loader.rs`** - 统一数据加载
3. **创建 `benches/Cargo.toml`** - bench workspace 入口

### 4.2 阶段二：迁移 benches

| 迁移源 | 迁移目标 |
|--------|----------|
| `crates/bench/src/*.rs` | `benches/benches/` |
| `benches/*.rs` | `benches/tpch/` 或 `benches/benches/` |
| `benches/dataset/` | `benches/dataset/` |
| `benches/queries/` | `benches/queries/` |

### 4.3 阶段三：重组 tests

| 迁移源 | 迁移目标 |
|--------|----------|
| `tests/*_test.rs` → e2e | `tests/e2e/` |
| `tests/*_stress.rs` | `tests/stress/` |
| `tests/ci_test.rs` | `tests/ci/` |
| `tests/buffer_pool_test.rs` | `tests/ci/` |
| `crates/executor/tests/*.rs` | `tests/integration/` |
| `crates/planner/tests/*.rs` | `tests/integration/` |

### 4.4 阶段四：清理

**删除的目录：**
- `bench/`（空目录）
- `benchmark/`（冗余）
- `benchmark_results/`（迁移到 `benchmark_results/`）
- `scripts/benchmark/`（合并到 `scripts/`）

**保留但清理的目录：**
- `crates/bench/` → 删除（代码已迁移到 `benches/`）
- `benchmark_results/` → 合并到根目录 `benchmark_results/`

---

## 5. 关键文件更新

### 5.1 Cargo.toml 变更

**根目录 `Cargo.toml`：**
```toml
[workspace]
members = [
    "crates/common",
    "crates/parser",
    "crates/planner",
    "crates/optimizer",
    "crates/executor",
    "crates/storage",
    "crates/catalog",
    "crates/transaction",
    "crates/server",
    "crates/bench-cli",
    "benches",              # 新增
]

[[bench]]
name = "sql_operations"
harness = false
path = "benches/benches/bench_scan.rs"
```

**新建 `benches/Cargo.toml`：**
```toml
[package]
name = "sqlrustgo-benches"
version.workspace = true
edition.workspace = true

[dependencies]
sqlrustgo-common.workspace = true
sqlrustgo-types.workspace = true
# ... 其他依赖
```

### 5.2 CI 配置更新

`.github/workflows/ci.yml`:
```yaml
test:
  script:
    - cargo test --lib                    # 单元测试
    - cargo test --tests --tests-dir tests/integration  # 模块集成
    - cargo test --test ci_test           # CI 测试
```

`.github/workflows/bench-pr.yml`:
```yaml
bench:
  script:
    - cargo bench --bench benches/benches  # 微基准
    - ./scripts/bench.sh --target=tpch     # TPC-H
```

---

## 6. 预期效果

### 6.1 重构前后对比

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 性能测试目录 | 4 个（bench, benches, benchmark, benchmarks） | 1 个（benches/） |
| 测试入口 | 分散 | 统一（tests/） |
| 数据加载 | 散乱 | 统一 DataLoader |
| 脚本位置 | 3+ 处 | 1 处（scripts/） |
| 命名规范 | 不统一 | 明确分层命名 |

### 6.2 符合开源数据库惯例

参考项目：
- **PostgreSQL**: `src/` + `test/` + `contrib/`
- **TiDB**: `tidb/` + `tests/` + `benchmarks/`
- **ClickHouse**: `src/` + `tests/` + `utils/` + `docs/`

本方案结合 Rust Cargo 惯例和数据库项目特点，达到行业标准。

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| CI 路径变更 | 测试可能失败 | 提前更新 CI 配置 |
| 历史 benchmark 数据丢失 | 无法对比 | 先备份 `benchmark_results/` 到 `benchmark_results/backup/` |
| 命名变更导致 git history 碎片 | 后续追溯困难 | 保持文件内容不变，仅移动位置 |

---

## 8. 实施检查清单

- [ ] 创建 `benches/` 目录结构
- [ ] 创建 `tests/data_loader.rs`
- [ ] 迁移 `crates/bench/` → `benches/benches/`
- [ ] 迁移 `benches/` → `benches/tpch/` 或 `benches/benches/`
- [ ] 重组 `tests/` 子目录
- [ ] 更新根 `Cargo.toml` workspace members
- [ ] 创建 `benches/Cargo.toml`
- [ ] 更新 CI workflows
- [ ] 更新 `scripts/` 引用
- [ ] 删除冗余目录
- [ ] 验证 `cargo test` 通过
- [ ] 验证 `cargo bench` 通过
- [ ] 更新 `BENCHMARK_GUIDE.md`
- [ ] 提交 PR 并 review
