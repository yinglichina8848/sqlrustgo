# SQLRustGo Test System – Phase 1 Execution Spec (MVP)

> 范围：**仅覆盖 Unit + Integration 测试的统一覆盖率采集与 CI 输出**
> 原则：**小步快跑、强约束、可回滚、可验收**

---

# 0. 成功定义（Definition of Done）

满足以下全部条件才算 Phase 1 完成：

```
1) 本地：一条命令成功生成覆盖率
   bash scripts/ci/test_all.sh

2) 产物存在：
   target/coverage/
     ├── html/index.html
     ├── coverage.json
     └── lcov.info

3) CI：
   - coverage job < 15 min
   - 每次 PR 都能产出 coverage.json 与 HTML（artifact）

4) 稳定性：
   - 连续运行 3 次结果一致（±1% 以内）
   - 无随机失败（flaky）

5) 不引入外部负载：
   - 不依赖 server / sysbench / tpch
```

---

# 1. 范围（In Scope）

* 使用 `cargo-llvm-cov` 采集 **workspace 内 unit + integration tests**
* 统一生成：
  * HTML 报告
  * JSON 报告
  * LCOV（用于后续平台接入）
* CI 中上传 artifacts

---

# 2. 非范围（Out of Scope，禁止实现）

```
- ❌ 不启动数据库 server
- ❌ 不接入 sysbench / TPC-H
- ❌ 不做 SQL corpus 自动化
- ❌ 不做 Dashboard UI（HTML 之外）
- ❌ 不做 coverage gate（阈值校验）
- ❌ 不做按 crate 拆分并行
```

---

# 3. 交付物（Deliverables）

```
scripts/ci/test_all.sh
scripts/ci/merge_coverage.sh
.github/workflows/coverage.yml   (或 Gitea 等效配置)
```

---

# 4. 实施步骤（按 PR 拆分，禁止合并）

## PR1：引入覆盖率工具（最小接入）

### 验收

```bash
cargo llvm-cov --no-report
```

✔ 成功退出（即通过）

---

## PR2：最小测试入口脚本

### 新增文件

`scripts/ci/test_all.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

# 覆盖率环境
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="coverage-%p-%m.profraw"

echo "[1/3] clean"
cargo llvm-cov clean --workspace || true
rm -f coverage-*.profraw || true

echo "[2/3] run tests (no report)"
cargo llvm-cov --workspace --no-report

echo "[3/3] generate report"
bash scripts/ci/merge_coverage.sh

echo "[done]"
```

---

`scripts/ci/merge_coverage.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

mkdir -p target/coverage

echo "[report] HTML"
cargo llvm-cov report \
  --workspace \
  --html \
  --output-dir target/coverage/html

echo "[report] JSON"
cargo llvm-cov report \
  --workspace \
  --json > target/coverage/coverage.json

echo "[report] LCOV"
cargo llvm-cov report \
  --workspace \
  --lcov > target/coverage/lcov.info

echo "coverage ready:"
echo "  - target/coverage/html/index.html"
echo "  - target/coverage/coverage.json"
echo "  - target/coverage/lcov.info"
```

### 验收

```bash
bash scripts/ci/test_all.sh
```

必须生成：

```
target/coverage/html/index.html
target/coverage/coverage.json
target/coverage/lcov.info
```

---

## PR3：CI 集成（最小版本）

Gitea Actions workflow: `.gitea/workflows/coverage.yml`

```yaml
name: Coverage (Phase1)

on:
  pull_request:
    branches: [develop/v2.9.0]
  push:
    branches: [develop/v2.9.0]

jobs:
  coverage:
    runs-on: [sqlrustgo-runner]
    timeout-minutes: 15

    steps:
      - name: Checkout
        run: |
          REPO="openclaw/sqlrustgo"
          git clone --depth=1 "http://192.168.0.252:3000/$REPO" repo
          cd repo && git fetch --depth=1 origin "$GITEA_SHA"

      - name: Install deps
        run: |
          cargo install cargo-llvm-cov
          rustup component add llvm-tools-preview

      - name: Run coverage
        run: bash scripts/ci/test_all.sh

      - name: Upload HTML
        run: |
          tar czf coverage-html.tar.gz -C target/coverage html

      - name: Upload JSON
        run: |
          cp target/coverage/coverage.json coverage.json
```

### 验收

* CI 总耗时 < 15 分钟
* coverage.json 和 html 产物存在

---

# 5. 验收步骤（Reviewer Checklist）

```
[ ] 本地运行 test_all.sh 成功
[ ] 覆盖率三类文件存在
[ ] CI 能稳定运行
[ ] 重复运行结果一致（±1%）
[ ] 未引入 server / sysbench / tpch
```

---

# 6. 回滚策略

如 CI 超时或失败：

```bash
git revert <PR>
```

或临时关闭 workflow：

```yaml
# .gitea/workflows/coverage.yml
if: false
```

---

# 7. 强制约束

```
- 不允许扩大范围
- 不允许合并 PR（必须逐个独立）
- 不允许提前实现 Phase2 内容
- 每个 PR 必须可独立运行
```

---

# 8. 下一阶段入口（Phase 2）

```
- SQL corpus 接入
- server 覆盖率
- dashboard（模块级）
- coverage gate
```

---

# ✅ 结论

Phase 1 的目标只有一个：

> **让覆盖率"稳定产出"，而不是"完整覆盖系统"**

---

# 7. 里程碑输出

完成后你应能看到：

```text
Coverage: 24.0%
Stability: Δ 0.0% (2 runs)
Artifacts:
  - coverage.json: 8.8MB
  - html/index.html: OK
  - lcov.info: OK
```
