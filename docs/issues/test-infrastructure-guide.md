# 测试基础设施说明与 CI/CD 集成计划

## 问题描述

PR #258, #259, #262 分别为 planner、optimizer、network 组件添加了共 21 个新测试。需要在其他平台的 OpenCode 代理和 CI/CD 流水线中复用这些测试资产。

## 新增测试清单

| 测试文件 | 组件 | 测试数 | 测试内容 |
|----------|------|--------|----------|
| `tests/planner_multi_join_test.rs` | planner | 4 | 3表JOIN结构、LEFT/RIGHT/CROSS JOIN类型验证 |
| `tests/optimizer_cbo_accuracy_test.rs` | optimizer | 11 | 成本模型准确性：索引vs顺序扫描、Hash vs NLJ、排序聚合、外部排序阈值 |
| `tests/network_tcp_smoke_test.rs` | network | 6 | TCP监听/连接/读写/选项设置/拒绝连接 |

## 运行测试的方法

### 1. 单个测试运行

```bash
# Planner 多表 JOIN 测试
cargo test --test planner_multi_join_test --all-features

# Optimizer CBO 准确性测试
cargo test --test optimizer_cbo_accuracy_test --all-features

# Network TCP 冒烟测试
cargo test --test network_tcp_smoke_test --all-features
```

### 2. 全部新测试一起运行

```bash
# 方式A：分别运行
cargo test --test planner_multi_join_test --all-features && \
cargo test --test optimizer_cbo_accuracy_test --all-features && \
cargo test --test network_tcp_smoke_test --all-features

# 方式B：使用 manifest selector（如果已集成 scripts/test/run.sh）
bash scripts/test/run.sh --group quick
```

### 3. 通过 manifest 运行（推荐用于CI）

```bash
# 查看可用测试组
bash scripts/test/run.sh --list

# 运行正确性测试组（包含新测试）
bash scripts/test/run.sh --group correctness

# 运行快速测试组
bash scripts/test/run.sh --group quick
```

## manifest 结构说明

测试配置位于 `scripts/test/manifest/test_manifest.yaml`:

```yaml
tests:
  - name: planner_multi_join
    type: integration
    component: planner
    dimension: correctness
    command: cargo test --test planner_multi_join_test --all-features
    description: Multi-table JOIN planning tests

  - name: optimizer_cbo_accuracy
    type: integration
    component: optimizer
    dimension: correctness
    command: cargo test --test optimizer_cbo_accuracy_test --all-features
    description: CBO cost model accuracy tests

  - name: network_tcp_smoke
    type: integration
    component: network
    dimension: correctness
    command: cargo test --test network_tcp_smoke_test --all-features
    description: TCP connection smoke tests

groups:
  correctness:
    - planner_multi_join
    - optimizer_cbo_accuracy
    - network_tcp_smoke
  quick:
    - planner_multi_join
    - optimizer_cbo_accuracy
    - network_tcp_smoke
```

## CI/CD 集成清单

### 必须集成到 CI 的内容

1. **测试命令**（添加到 `.github/workflows/ci.yml` 或等价CI配置）:

```yaml
# 测试 job 中添加：
- name: Run new integration tests
  run: |
    cargo test --test planner_multi_join_test --all-features
    cargo test --test optimizer_cbo_accuracy_test --all-features
    cargo test --test network_tcp_smoke_test --all-features
```

2. **Manifest 运行器**（可选但推荐）:

```yaml
- name: Run test manifest
  run: bash scripts/test/run.sh --group correctness
```

### Gate 检查脚本

| 脚本 | 用途 | 建议 |
|------|------|------|
| `scripts/gate/check_docs_links.sh` | 文档链接检查 | PR 前必跑 |
| `scripts/gate/check_coverage.sh` | 覆盖率检查 | 建议 CI 集成 |
| `scripts/gate/check_security.sh` | 安全检查 | 建议定时跑 |

### 本地验证脚本

```bash
# 7步本地验证（推荐 PR 前跑）
bash scripts/verify_local.sh

# 覆盖检查
bash scripts/check_coverage.py --threshold 80

# 格式检查
cargo fmt --all -- --check

# Lint 检查
cargo clippy --all-features -- -D warnings
```

## 其他平台 OpenCode 代理使用指南

### 通用前提

```bash
# 1. 克隆仓库
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# 2. 安装 Rust 工具链（如果未安装）
# 需要 Rust 1.85+

# 3. 构建项目
cargo build --all-features
```

### 运行测试

```bash
# 运行所有测试
cargo test --all-features

# 运行特定组件测试
cargo test --test planner_multi_join_test --all-features
cargo test --test optimizer_cbo_accuracy_test --all-features
cargo test --test network_tcp_smoke_test --all-features

# 查看更多测试选项
cargo test --help
```

### 调试失败的测试

```bash
# 显示详细输出
cargo test --test <test_name> --all-features -- --nocapture

# 运行单个测试
cargo test --test <test_name> --all-features -- test_specific_name

# 查看测试列表
cargo test --test <test_name> --all-features -- --list
```

## 已知失败测试

当前无已知失败测试。如果测试失败，检查：

1. Rust 版本是否 >= 1.85
2. 是否使用了 `--all-features` 标志
3. 是否有未提交的代码变更冲突

## 后续计划

- [ ] 将 `scripts/test/run.sh` 集成到 GitHub Actions
- [ ] 添加测试结果 JSON 报告输出
- [ ] 集成 coverage 到 CI coverage 报告
- [ ] 添加测试稳定性监控

## 相关文件

- `scripts/test/run.sh` - 测试运行器
- `scripts/test/manifest/test_manifest.yaml` - 测试清单
- `scripts/test/manifest/known_failures.yaml` - 已知失败
- `test_inventory.yaml` - 组件测试清单
- `scripts/verify_local.sh` - 本地验证脚本
- `scripts/check_coverage.py` - 覆盖率检查工具
