# Gate OS Lite — B-Gate CI 规范

> **版本**: v2.9.0
> **文件**: `.gitea/workflows/gate-ci.yml`
> **PR**: #242
> **状态**: Beta 分支已启用

---

## 一、架构概览

```
git push → Gitea Actions → hp-z6g4 nomad runner → gate_report.json → Hermes 读取
```

**设计原则**:
- CI 不需要"可观测"（无 API 查询）
- CI 只生成 `gate_report.json` 作为单一事实来源
- Hermes 通过读取 artifact 观察结果，而非查询 CI 系统

---

## 二、触发条件

| 事件 | 分支 | 说明 |
|------|------|------|
| push | `develop/v2.9.0` | 主开发分支 |
| push | `beta/v2.9.0` | Beta 发布分支 |
| push | `ci/v2.9.0-*` | CI 测试分支 |
| PR | `develop/v2.9.0` | PR 审查 |
| PR | `beta/v2.9.0` | Beta PR |

---

## 三、执行阶段

### 3.1 Build
```
cargo build --all-features
```
目标: 全量编译，确保代码可构建。

### 3.2 Test
```
cargo test --all-features
```
目标: 运行所有测试套件，统计通过/失败数量。

### 3.3 Clippy
```
cargo clippy --all-features -- -D warnings
```
目标: 静态分析，零警告策略。

### 3.4 Format
```
cargo fmt --check --all
```
目标: 代码风格一致性检查。

### 3.5 Coverage
```
cargo tarpaulin --all-features --out Json
```
目标: 行覆盖率计算，阈值 ≥75%。

### 3.6 SQL Corpus
```
cargo test -p sqlrustgo-sql-corpus
```
目标: SQL 语法兼容性测试。

---

## 四、Gate Report 格式

执行完成后生成 `/tmp/gate-workspace/gate_report.json`:

```json
{
  "gate": "B-GATE-v2.9.0",
  "version": "v2.9.0",
  "commit": "abc123def456",
  "status": "PASS",
  "coverage": 78.5,
  "test_count": 294,
  "timestamp": "2026-05-03T16:50:49.000Z",
  "stages": {
    "build": "PASS",
    "test": "PASS",
    "clippy": "PASS",
    "fmt": "PASS",
    "coverage": "PASS",
    "sql_corpus": "PASS"
  }
}
```

**状态枚举**: `PASS` | `FAIL` | `ERROR`

---

## 五、Artifact 上传

使用 `actions/upload-artifact@v4`:

```
name: gate-report-{run_number}
path: /tmp/gate-workspace/gate_report.json
retention-days: 30
```

Hermes 读取方式: 通过 Gitea API 下载 artifact。

---

## 六、Runner 配置

| 属性 | 值 |
|------|---|
| Runner 标签 | `hp-z6g4` |
| 平台 | Nomad |
| 容器 | `nomad-runner` |
| CPU | 40 cores |
| 内存 | 高性能配置 |

---

## 七、Env vars

| 变量 | 值 |
|------|---|
| `GATE_VERSION` | `v2.9.0` |
| `RUST_VERSION` | `1.75` |

---

## 八、与 Hermes Gate 的关系

| 维度 | Hermes Gate (`gate/hermes_gate.sh`) | B-Gate CI |
|------|------------------------------------|-----------|
| 执行时机 | 本地 pre-commit | CI push 时 |
| 覆盖范围 | fmt, clippy, shell | fmt, clippy, test, coverage |
| 事实来源 | 本地执行结果 | gate_report.json |
| 阻断能力 | 建议性 | 强制性（CI required status） |

**策略**: Hermes Gate 作为快速反馈（本地），B-Gate CI 作为唯一裁决源（远程）。

---

## 九、使用流程

### 9.1 开发者推送代码
```bash
git push origin develop/v2.9.0
# 或
git push origin beta/v2.9.0
```

### 9.2 CI 自动执行
Gitea Actions 调度 B-Gate job 到 hp-z6g4 runner。

### 9.3 Hermes 读取结果
```bash
# 下载最新 artifact
curl -s "https://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/actions/artifacts"
# 解析 gate_report.json
```

### 9.4 决策
- `status: PASS` → 允许合并
- `status: FAIL` → 阻止合并，通知开发者

---

*文档生成: 2026-05-04*
