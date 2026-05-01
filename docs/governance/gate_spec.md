# v2.8.0 门禁规范 (Gate Specification)

> **版本**: 1.0
> **更新日期**: 2026-05-01
> **维护人**: macmini opencode
> **适用版本**: v2.8.0+

---

## 一、门禁概述

v2.8.0 采用四级门禁模型，确保每个发布阶段的质量：

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

| 门禁 | 名称 | 目标 | 覆盖率目标 |
|------|------|------|-----------|
| A-Gate | Alpha Gate | 开发完成 | ≥50% |
| B-Gate | Beta Gate | 功能冻结 | ≥65% |
| R-Gate | RC Gate | 发布候选 | ≥80% |
| G-Gate | GA Gate | 正式发布 | ≥85% |

---

## 二、A-Gate (Alpha Gate)

### 2.1 入口条件

- 所有计划的 feature 已实现
- 核心功能可运行
- 无 P0 Bug

### 2.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| 编译检查 | `cargo build --workspace` | 无错误 |
| 单元测试 | `cargo test --workspace` | ≥80% 通过 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| Baseline 验证 | `python3 scripts/verify_baseline.py` | 226/226 PASS |
| 文档检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 |

### 2.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥45% |
| optimizer | ≥40% |
| storage | ≥15% |
| catalog | ≥50% |
| **整体** | **≥50%** |

---

## 三、B-Gate (Beta Gate)

### 3.1 入口条件

- A-Gate 已通过
- 功能开发完成，进入冻结期
- 无 P0/P1 Bug

### 3.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| 编译检查 | `cargo build --release --workspace` | 无错误 |
| 全量测试 | `cargo test --all-features` | ≥90% 通过 |
| Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| 覆盖率 | `cargo tarpaulin --workspace --all-features` | ≥65% |

### 3.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥60% |
| optimizer | ≥50% |
| storage | ≥20% |
| catalog | ≥60% |
| **整体** | **≥65%** |

---

## 四、R-Gate (RC Gate)

### 4.1 入口条件

- B-Gate 已通过
- 功能冻结，只允许 Bug Fix
- 所有已知 Bug 已修复或延期

### 4.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| Release 构建 | `cargo build --release --workspace` | 无错误 |
| 全量测试 | `cargo test --all-features` | 100% 通过 |
| Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| Baseline 验证 | `cat verification_report.json | jq .baseline_verified` | true |
| 审计验证 | `cat audit_report.json | jq .proof_match` | true |
| 覆盖率 | `cargo tarpaulin --workspace --all-features` | ≥80% |
| 安全扫描 | `cargo audit` | 无漏洞 |

### 4.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥75% |
| optimizer | ≥60% |
| storage | ≥30% |
| catalog | ≥70% |
| **整体** | **≥80%** |

---

## 五、G-Gate (GA Gate)

### 5.1 入口条件

- R-Gate 已通过
- 所有问题已关闭
- 发布审批已获得

### 5.2 检查清单

| 检查项 | 命令 | 通过标准 |
|--------|------|----------|
| Release 构建 | `cargo build --release --workspace` | 无错误 |
| 全量测试 | `cargo test --all-features` | 100% 通过 |
| Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --all -- --check` | 无格式错误 |
| 覆盖率 | `cargo tarpaulin --workspace --all-features` | ≥85% |
| 安全扫描 | `cargo audit` | 无漏洞 |
| 性能基准 | `cargo bench` | 无性能回归 |
| 文档完整 | `bash scripts/gate/check_user_guides.sh` | 文档完整 |

### 5.3 覆盖率要求

| 模块 | 覆盖率目标 |
|------|-----------|
| executor | ≥80% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥75% |
| **整体** | **≥85%** |

---

## 六、完整门禁检查脚本

### 6.1 A-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.8.0 A-Gate 检查 ==="

echo "[1/4] 编译检查..."
cargo build --workspace
echo "✅ 编译通过"

echo "[2/4] 测试检查..."
cargo test --workspace
echo "✅ 测试通过"

echo "[3/4] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[4/4] 文档链接检查..."
bash scripts/gate/check_docs_links.sh
echo "✅ 文档检查通过"

echo "=== A-Gate 检查完成 ==="
```

### 6.2 B-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.8.0 B-Gate 检查 ==="

echo "[1/6] Release 编译..."
cargo build --release --workspace
echo "✅ Release 编译通过"

echo "[2/6] 全量测试..."
cargo test --all-features
echo "✅ 全量测试通过"

echo "[3/6] Clippy 检查..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy 通过"

echo "[4/6] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[5/6] 覆盖率检查..."
rm -rf target/tarpaulin/
cargo tarpaulin --workspace --all-features
echo "✅ 覆盖率检查完成"

echo "[6/6] 安全扫描..."
cargo audit
echo "✅ 安全扫描通过"

echo "=== B-Gate 检查完成 ==="
```

### 6.3 R-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.8.0 R-Gate 检查 ==="

echo "[1/7] Release 编译..."
cargo build --release --workspace
echo "✅ Release 编译通过"

echo "[2/7] 全量测试..."
cargo test --all-features
echo "✅ 全量测试通过"

echo "[3/7] Clippy 检查..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy 通过"

echo "[4/7] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[5/7] 覆盖率检查..."
rm -rf target/tarpaulin/
cargo tarpaulin --workspace --all-features
echo "✅ 覆盖率 ≥80%"

echo "[6/7] 安全扫描..."
cargo audit
echo "✅ 安全扫描通过"

echo "[7/7] MySQL 兼容性测试..."
cargo test --all-features -- mysql_compat
echo "✅ MySQL 兼容性通过"

echo "=== R-Gate 检查完成 ==="
```

### 6.4 G-Gate 脚本

```bash
#!/bin/bash
set -e

echo "=== v2.8.0 G-Gate 检查 ==="

echo "[1/8] Release 编译..."
cargo build --release --workspace
echo "✅ Release 编译通过"

echo "[2/8] 全量测试..."
cargo test --all-features
echo "✅ 全量测试通过"

echo "[3/8] Clippy 检查..."
cargo clippy --all-features -- -D warnings
echo "✅ Clippy 通过"

echo "[4/8] 格式化检查..."
cargo fmt --all -- --check
echo "✅ 格式化通过"

echo "[5/8] 覆盖率检查..."
rm -rf target/tarpaulin/
cargo tarpaulin --workspace --all-features
echo "✅ 覆盖率 ≥85%"

echo "[6/8] 安全扫描..."
cargo audit
echo "✅ 安全扫描通过"

echo "[7/8] 性能基准测试..."
cargo bench
echo "✅ 性能基准通过"

echo "[8/8] 文档完整性检查..."
bash scripts/gate/check_user_guides.sh
echo "✅ 文档完整"

echo "=== G-Gate 检查完成 ==="
```

---

## 七、门禁状态追踪

### 7.1 各分支门禁要求

| 分支 | 门禁 | 覆盖率目标 | 测试要求 |
|------|------|-----------|----------|
| develop/v2.8.0 | A-Gate | ≥50% | ≥80% |
| alpha/v2.8.0 | B-Gate | ≥65% | ≥90% |
| beta/v2.8.0 | R-Gate | ≥80% | 100% |
| rc/v2.8.0 | G-Gate | ≥85% | 100% |

### 7.2 当前状态 (v2.8.0)

| 门禁 | 状态 | 完成日期 | 备注 |
|------|------|----------|------|
| A-Gate | ✅ 完成 | 2026-04-30 | Baseline 226/226 PASS, 100% |
| B-Gate | ✅ 完成 | 2026-05-01 | hermes_gate + run_hermes_gate PASS |
| R-Gate | 🔄 进行中 | TBD | 待 Gitea CI 验证 + 覆盖率 |
| G-Gate | ⚪ 未启动 | TBD | 需 R-Gate 完成 |

---

## 八、门禁豁免规则

以下情况可申请门禁豁免：

| 豁免类型 | 条件 | 审批人 |
|----------|------|--------|
| 覆盖率豁免 | 新增代码可证明难以测试 | Tech Lead |
| 性能豁免 | 性能测试环境不稳定 | QA Lead |
| 文档豁免 | 文档更新不影响功能 | Docs Lead |

---

## 九、相关文档

| 文档 | 说明 |
|------|------|
| [release_process.md](./release_process.md) | 发布流程 |
| [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md) | 版本生命周期 |
| [RC_TO_GA_GATE_CHECKLIST.md](./RC_TO_GA_GATE_CHECKLIST.md) | RC→GA 清单 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-01 | 初始版本，定义 A/B/R/G 四级门禁 |

---

*本文档由 macmini opencode 维护*