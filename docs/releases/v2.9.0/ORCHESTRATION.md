# v2.9.0 多 Agent 协作架构与启动指令

> **版本**: v2.9.0
> **日期**: 2026-05-02
> **原则**: Hermes 不参与写代码 — 只负责测试验证和门禁检查

---

## 一、AI 分工

```
┌──────────────────────────────────────────────────────────────────┐
│                     Harness 治理体系                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Hermes (Mac Mini, 192.168.3.6)                         │   │
│  │  ├─ 测试验证: cargo test, clippy, fmt, gate check       │   │
│  │  ├─ PR 门禁: R1-R10 检查                                 │   │
│  │  ├─ 证明验证: Proof Registry 一致性检查                   │   │
│  │  ├─ 回归测试: SQL Corpus 进度追踪                        │   │
│  │  └─ 发布门禁: G-Gate/P-Gate/T-Gate/S-Gate/R-Gate       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ OpenCode Z6G4│  │ OpenCode Z440│  │ OpenCode     │          │
│  │ Phase G + S  │  │ Phase C + D  │  │ Phase E      │          │
│  │ (治理+证明)  │  │ (SQL+分布式) │  │ (生产就绪)   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         │                  │                  │                  │
│         └──────────────────┴──────────────────┘                  │
│                            │                                     │
│                    Gitea develop/v2.9.0                          │
│                    PR → 合并 → Hermes 验证                       │
└──────────────────────────────────────────────────────────────────┘
```

### Hermes 职责（裁判员，不写代码）

| 职责 | 内容 | 频率 |
|------|------|------|
| 测试验证 | 合并前运行完整 `cargo test --all-features` | 每次 PR 前 |
| 代码质量 | `cargo clippy --all-features -- -D warnings` | 每次 PR 前 |
| 代码格式 | `cargo fmt --check --all` | 每次 PR 前 |
| 回归测试 | `cargo test -p sql-corpus` 追踪通过率 | 每日 |
| 门禁检查 | R1-R10 规则执行 | 每次合并前 |
| 证明验证 | Proof Registry 一致性验证 | 每周 |
| 报告产出 | 开发进度报告、测试报告 | 每周 |

### OpenCode 职责（运动员，只写代码）

| 平台 | 分工 | Issues | 工作目录 |
|------|------|--------|---------|
| **Z6G4 OpenCode** | Phase G (治理体系) + Phase S (SQL 证明) | #116, #117 | `~/workspace/dev/yinglichina163/sqlrustgo` |
| **Z440 OpenCode** | Phase C (SQL 兼容性) + Phase D (分布式) | #118, #119 | `~/workspace/dev/sqlrustgo` |
| **OpenCode 3** | Phase E (生产就绪) | #120 | `~/workspace/dev/openheart/sqlrustgo` |

---

## 二、OpenCode 启动指令

以下是指令，每个 OpenCode 实例执行前读取此段：

```
## SQLRustGo v2.9.0 开发启动指令

### 环境准备
1. 拉取最新代码:
   git fetch origin develop/v2.9.0
   git checkout develop/v2.9.0
   git pull origin develop/v2.9.0

2. 阅读开发计划:
   cat docs/releases/v2.9.0/DEVELOPMENT_PLAN.md

3. 阅读你的分工 Issue:
   - Z6G4: Gitea Issues #116, #117
   - Z440: Gitea Issues #118, #119
   - OpenCode3: Gitea Issue #120

### 工作流程
1. 从 develop/v2.9.0 创建 feature branch:
   git checkout -b feature/<task-id>-<description>

2. 实现功能 + 写测试 (TDD 模式)
   - 写测试 → 运行确认失败 (RED)
   - 写代码 → 运行确认通过 (GREEN)
   - 重构 → 测试仍然通过 (REFACTOR)

3. 提交:
   git add <files>
   git commit -m "feat(v2.9.0): <task-id> - description"
   
4. 创建 PR 到 develop/v2.9.0:
   git push origin feature/<task-id>-<description>
   → 在 Gitea Web UI 创建 PR
   → 等待 Hermes 验证后合并

### 开发规范
- 所有代码必须通过: cargo clippy --all-features -- -D warnings
- 所有代码必须通过: cargo fmt --check --all
- 新增功能必须有测试
- 使用 git worktree 隔离不同 feature 分支

### Gitea 提交配置
git config user.name "opencode"
git config user.email "openheart@gaoyuanyiyao.com"
git remote set-url origin ssh://git@192.168.0.252:222/openclaw/sqlrustgo.git
```

---

## 三、各 Phase 具体任务分配

### Phase G: 可信任治理体系 → Z6G4 OpenCode

最大优先级: G-01 (R门禁扩展)

```
G-01: R门禁 R8-R10
  R8: SQL 兼容性门禁 — 检查 PR 是否降低 Corpus 通过率
  R9: 性能退化门禁 — 检查 PR 是否导致性能退化
  R10: 形式化证明门禁 — 检查 PR 相关的证明是否注册
  
G-02: 证明注册表系统升级
  文件: proof-registry/src/lib.rs
  注册表格式: Formulog/Dafny/TLA+ 证明路径
  自动验证: cargo test 时验证已注册证明

G-03: 攻击面验证 AV10
  新增 AV10: 形式化模型检查
  工具: 集成 TLA+ 模型检查

G-04/G-05: CI/CD 集成 + 告警
  scripts/gate/ 目录新增门禁脚本
  Gitea Webhook 配置
```

### Phase S: SQL 可证明性 → Z6G4 OpenCode

```
S-01: Parser 正确性证明 (Formulog)
  证明 SQL SELECT 解析生成的 AST 不丢失信息
  文件: proof-registry/proofs/parser_soundness.flg

S-02: 类型系统安全性证明 (Dafny)
  证明类型推断对所有表达式终止且唯一
  文件: proof-registry/proofs/type_safety.dfy

S-03: 事务 ACID 性质证明 (TLA+)
  证明 WAL 重放后 = 崩溃前已提交
  文件: proof-registry/proofs/wal_recovery.tla
```

### Phase C: SQL 兼容性 → Z440 OpenCode

```
C-01: SQL Corpus 40.8% → 80%
  对 sql-corpus 中的 252 失败用例分组修复
  每组修复后运行: cargo test -p sql-corpus

C-02: CTE (WITH/Recursive)
  parser: WITH ... AS (...) SELECT ... 
  executor: CTE 执行支持

C-03: JSON 函数 (P2)
  JSON_EXTRACT, JSON_OBJECT 解析和执行

C-04: 窗口函数补全 (P1)
  LEAD, LAG, NTILE, FIRST_VALUE, LAST_VALUE
```

### Phase D: 分布式增强 → Z440 OpenCode

```
D-01: 半同步复制
  crates/distributed/src/replication.rs
  AFTER_SYNC, AFTER_COMMIT 模式

D-02: 并行复制 (MTS)
  从库并行回放 WAL entry
```

### Phase E: 生产就绪 → OpenCode 3

```
E-01: Sysbench OLTP 基准
  crates/bench/ 新增 oltp 基准工作负载
  目标: oltp_read_write ≥ 10K QPS

E-03: 消除 33 个 #[ignore] 测试
  逐一检查并修复

E-04: GRANT/REVOKE 列级权限
  parser: GRANT column ON table TO user
  executor: ColumnMasker 集成

E-06: AES-256 存储加密
  crates/security/src/encryption.rs → FileStorage 集成
```

---

## 四、Hermes 验证流程

每次 OpenCode 提交 PR 后，Hermes 执行:

```bash
# Step 1: 拉取 PR 分支
git fetch origin pull/<PR>/head:verify/<PR>
git checkout verify/<PR>

# Step 2: 编译检查
cargo check --all-features

# Step 3: 代码质量
cargo clippy --all-features -- -D warnings
cargo fmt --check --all

# Step 4: 测试
cargo test --all-features 2>&1 | tee test_result.log
grep "FAILED" test_result.log && echo "❌ TESTS FAILED" || echo "✅ TESTS PASS"

# Step 5: R 门禁检查
# R1-R7 (基础) + R8-R10 (新增)
bash scripts/gate/check_gate.sh || echo "❌ GATE FAILED"

# Step 6: 报告
# 通过 → 合并 PR
# 失败 → 在 PR 中评论失败原因，打回
```

---

## 五、开发启动命令

### 给 Z6G4 OpenCode 的初始指令

```bash
cd ~/workspace/dev/yinglichina163/sqlrustgo
git fetch origin develop/v2.9.0
git checkout develop/v2.9.0
git pull origin develop/v2.9.0

# 阅读 Issue #116: 可信任治理体系
# 阅读 Issue #117: SQL 可证明性

# 启动 G-01: R门禁扩展
git checkout -b feature/g01-r-gate-extend

# 开始开发...
```

### 给 Z440 OpenCode 的初始指令

```bash
cd ~/workspace/dev/sqlrustgo
git fetch origin develop/v2.9.0
git checkout develop/v2.9.0
git pull origin develop/v2.9.0

# 阅读 Issue #118: SQL 兼容性
# 阅读 Issue #119: 分布式增强

# 启动 C-01: SQL Corpus 提升
git checkout -b feature/c01-corpus-80pct

# 开始开发...
```

---

## 六、Harness 规则（不可违反）

| 规则 | 内容 | 违反后果 |
|------|------|---------|
| H-01 | **Hermes 不写生产代码** — 只做测试、验证、门禁 | 治理失败 |
| H-02 | **OpenCode 必须走 PR 流程** — 禁止直接 push develop | 分支保护阻止 |
| H-03 | **每次 PR 前必须跑完整测试** — cargo test --all-features | Hermes 拒绝合并 |
| H-04 | **TDD 模式** — 测试先于代码 | 质量门禁不通过 |
| H-05 | **PR 必须有 Issue 引用** — Closes #116 等 | 治理审计不通过 |
| H-06 | **代码质量门禁** — clippy + fmt 必须通过 | Hermes 拒绝合并 |
| H-07 | **SQL Corpus 不能退化** — 每次 PR 不降低通过率 | 兼容性门禁不通过 |

---

*Harness 规则版本: v1.0 // 最后更新: 2026-05-02*
