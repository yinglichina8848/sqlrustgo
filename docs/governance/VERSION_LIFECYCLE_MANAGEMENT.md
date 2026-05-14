# SQLRustGo 版本生命周期管理

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **用途**: 定义 SQLRustGo 版本的完整生命周期管理规范，包括版本命名、阶段转换、发布流程和版本延续
> **SSOT**: 版本生命周期规范以本文档为权威来源

> **关联文档**:
> - `RELEASE_LIFECYCLE.md` — 四级门禁模型详解
> - `GATE_SPEC_MASTER.md` — 门禁规范 SSOT
> - `GATE_PHASES_AND_TRACKING.md` — 分阶段门禁追踪
> - `GOVERNANCE_STANDARD.md` — 治理标准总纲

---

## 一、版本生命周期概述

### 1.1 四级门禁模型

SQLRustGo 采用四级门禁发布生命周期：

```
A-Gate ──▶ B-Gate ──▶ R-Gate ──▶ G-Gate
 (α入口)    (β入口)    (RC入口)    (GA入口)
```

| 门禁 | 名称 | 阶段目标 | 覆盖率目标 | 性能目标 |
|------|------|----------|------------|----------|
| **A-Gate** | Alpha Gate | 开发完成，可运行原型 | ≥50% | 基线建立 |
| **B-Gate** | Beta Gate | 功能冻结，进入稳定期 | ≥75% | TPC-H SF=0.1 22/22 |
| **R-Gate** | RC Gate | 发布候选，性能优化完成 | ≥85% | TPC-H SF=1 22/22 + QPS 基线 |
| **G-Gate** | GA Gate | 正式发布，所有门槛达标 | ≥85% | Point Select ≥10K QPS |

### 1.2 版本阶段流程图

```
                    ┌─────────────────────────────────────────────┐
                    │           develop/v{X}.{Y}.{Z}               │
                    │                  开发阶段                      │
                    └─────────────────────────────────────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────────┐
                    │              A-Gate (Alpha)                  │
                    │  入口: 开发任务完成，代码已提交               │
                    │  出口: 可运行原型，测试≥80%                  │
                    └─────────────────────────────────────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────────┐
                    │              B-Gate (Beta)                   │
                    │  入口: A-Gate PASS，无 P0/P1 Bug           │
                    │  出口: 功能冻结，覆盖率≥75%                  │
                    └─────────────────────────────────────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────────┐
                    │              R-Gate (RC)                     │
                    │  入口: B-Gate PASS，TPC-H SF=1 可运行       │
                    │  出口: 发布候选，QPS 退化≤5%                 │
                    └─────────────────────────────────────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────────┐
                    │              G-Gate (GA)                    │
                    │  入口: R-Gate PASS，性能达标                │
                    │  出口: 正式发布，Point Select ≥10K QPS      │
                    └─────────────────────────────────────────────┘
                                        │
                                        ▼
                    ┌─────────────────────────────────────────────┐
                    │               v{X}.{Y}.{Z} Tag              │
                    │          Merge → main, Create release/{Z}  │
                    └─────────────────────────────────────────────┘
```

---

## 二、版本号规则

### 2.1 语义化版本 (Semantic Versioning)

SQLRustGo 遵循语义化版本规范：

```
v{MAJOR}.{MINOR}.{PATCH}
 │      │      │
 │      │      └── PATCH: Bug 修复（向后兼容）
 │      └──────── MINOR: 新功能（向后兼容）
 └─────────────── MAJOR: 破坏性变更
```

| 版本类型 | 规则 | 示例 |
|----------|------|------|
| MAJOR | 破坏性 API 变更，需要 MAJOR 版本升级 | v2.9.0 → v3.0.0 |
| MINOR | 新功能，向后兼容 | v3.0.0 → v3.1.0 |
| PATCH | Bug 修复，向后兼容 | v3.1.0 → v3.1.1 |

### 2.2 Tag 命名规范

```
v{MAJOR}.{MINOR}.{PATCH}-{phase}{number}
                                    │
                                    ├── alpha{N}: Alpha 阶段
                                    ├── beta{N}: Beta 阶段
                                    ├── rc{N}: RC 阶段
                                    └── (无后缀): GA 正式版
```

**Tag 命名示例**：

| 阶段 | Tag 格式 | 示例 |
|------|----------|------|
| Alpha | vX.Y.Z-alpha{N} | v3.1.0-alpha1 |
| Alpha | vX.Y.Z-alpha{N} | v3.1.0-alpha2 |
| Beta | vX.Y.Z-beta{N} | v3.1.0-beta1 |
| RC | vX.Y.Z-rc{N} | v3.1.0-rc1 |
| RC | vX.Y.Z-rc{N} | v3.1.0-rc2 |
| GA | vX.Y.Z | v3.1.0 |

### 2.3 版本分支规范

```
main
  │
  ├── release/3.0  (GA 分支，维护 v3.0.x)
  │
  ├── release/2.9  (GA 分支，维护 v2.9.x)
  │
  └── develop/v3.1  (开发分支，v3.1.0 开发)
           │
           ├── feature/xxx  (功能分支)
           └── fix/xxx      (修复分支)
```

| 分支类型 | 命名规范 | 用途 |
|----------|----------|------|
| main | main | GA 正式版本 |
| release/X.Y | release/X.Y | 维护已发布版本的补丁 |
| develop/vX.Y.Z | develop/vX.Y.Z | 当前版本开发 |
| feature | feature/{feature-name} | 新功能开发 |
| fix | fix/{issue-number}-{short-desc} | Bug 修复 |

---

## 三、阶段定义

### 3.1 Alpha 阶段 (A-Gate)

**阶段说明**: 开发完成阶段

| 属性 | 值 |
|------|-----|
| 入口条件 | 开发任务完成，代码已提交 |
| 重点 | 核心功能实现，架构设计 |
| 特点 | API 可能变化，不发布正式 Release |
| 分支 | `develop/v{X}.{Y}.{Z}` |
| Tag | 无正式 Tag |

**门禁要求 (A-Gate)**：

| 检查项 | 通过标准 | 命令 |
|--------|----------|------|
| A1 编译 | 无错误 | `cargo build --workspace` |
| A2 测试 | ≥80% | `cargo test --workspace` |
| A3 Clippy | 零警告 | `cargo clippy --all-features -- -D warnings` |
| A4 格式化 | 无格式错误 | `cargo fmt --all -- --check` |
| A5 文档链接 | 无死链 | `bash scripts/gate/check_docs_links.sh` |
| A6 覆盖率 | ≥50% | `cargo llvm-cov --all-features --lcov` |
| A7 安全扫描 | 无高危漏洞 | `cargo audit` |

### 3.2 Beta 阶段 (B-Gate)

**阶段说明**: 功能冻结阶段

| 属性 | 值 |
|------|-----|
| 入口条件 | A-Gate PASS，无 P0/P1 Bug |
| 重点 | Bug 修复，稳定化 |
| 特点 | API 基本稳定，只允许 Bug Fix |
| 分支 | `develop/v{X}.{Y}.{Z}` |
| Tag | `v{X}.{Y}.{Z}-alpha{N}` |

**门禁要求 (B-Gate)**：

| 检查项 | 通过标准 | 命令 |
|--------|----------|------|
| B1 编译 | 无错误 | `cargo build --release --workspace` |
| B2 测试 | ≥90% | `cargo test --all-features` |
| B3 Clippy | 零警告 | `cargo clippy --all-features -- -D warnings` |
| B4 格式化 | 无格式错误 | `cargo fmt --all -- --check` |
| B5 覆盖率 | ≥75% | `cargo llvm-cov --all-features --lcov` |
| B6 安全扫描 | 无高危漏洞 | `cargo audit` |
| B7 文档链接 | 无死链 | `bash scripts/gate/check_docs_links.sh` |
| B8 TPC-H SF=0.1 | 22/22 通过，无 OOM | `scripts/gate/check_tpch.sh sf=0.1` |
| B9 SQL Corpus | ≥85% | `cargo test -p sqlrustgo-sql-corpus` |
| B-S1~B-S6 稳定性 | 全部 PASS | `cargo test --test {test_name}` |

### 3.3 RC 阶段 (R-Gate)

**阶段说明**: 发布候选阶段

| 属性 | 值 |
|------|-----|
| 入口条件 | B-Gate PASS，TPC-H SF=1 可运行，SQL Corpus ≥95% |
| 重点 | 性能优化，文档完善 |
| 特点 | 只允许严重 Bug 修复，功能完全冻结 |
| 分支 | `develop/v{X}.{Y}.{Z}` |
| Tag | `v{X}.{Y}.{Z}-rc{N}` |

**门禁要求 (R-Gate)**：

| 检查项 | 通过标准 | 命令 |
|--------|----------|------|
| R1 编译 | 无错误 | `cargo build --release --workspace` |
| R2 测试 | 100% | `cargo test --all-features` |
| R3 Clippy | 零警告 | `cargo clippy --all-features -- -D warnings` |
| R4 格式化 | 无格式错误 | `cargo fmt --all -- --check` |
| R5 覆盖率 | ≥85% | `cargo llvm-cov --all-features --lcov` |
| R6 安全扫描 | 无高危漏洞 | `cargo audit` |
| R7 文档 | R7a~R7d 全部 PASS | 见 GATE_SPEC_MASTER.md |
| R8 SQL Corpus | ≥95% | `cargo test -p sqlrustgo-sql-corpus` |
| R9 TPC-H SF=1 | 22/22 可运行 | `scripts/gate/check_tpch.sh sf=1` |
| R10 性能基线 | QPS 退化≤5% | `cargo bench && check_perf_baseline.sh` |
| R11 Sysbench | Point/UPDATE/INSERT 达标 | `scripts/gate/check_sysbench.sh` |
| R12 MySQL 协议 | 连接成功 | mysql:5.7 容器测试 |

### 3.4 GA 阶段 (G-Gate)

**阶段说明**: 正式发布阶段

| 属性 | 值 |
|------|-----|
| 入口条件 | R-Gate PASS，性能达标 |
| 重点 | 发布准备，最终验证 |
| 特点 | 所有质量门槛达标，可以发布 |
| 分支 | `main` (merge 后) |
| Tag | `v{X}.{Y}.{Z}` |

**门禁要求 (G-Gate)**：

| 检查项 | 通过标准 | 命令 |
|--------|----------|------|
| G1 编译 | 无错误 | `cargo build --release --workspace` |
| G2 测试 | 100% | `cargo test --all-features` |
| G3 Clippy | 零警告 | `cargo clippy --all-features -- -D warnings` |
| G4 格式化 | 无格式错误 | `cargo fmt --all -- --check` |
| G5 覆盖率 | ≥85% | `cargo llvm-cov --all-features --lcov` |
| G6 安全扫描 | 无高危漏洞 | `cargo audit` |
| G7 Point Select QPS | **≥10,000 ops/s** | `cargo bench -- point_select` |
| G8 UPDATE QPS | **≥5,000 ops/s** | `cargo bench -- update_simple` |
| G9 DELETE QPS | **≥2,000 ops/s** | `cargo bench -- delete_simple` |
| G10 TPC-H SF=1 | 22/22 通过，无 OOM | `scripts/gate/check_tpch.sh sf=1` |
| G11 SQL Corpus | **≥98%** | `cargo test -p sqlrustgo-sql-corpus` |
| G12 稳定性测试 | B-S1~B-S6 全部 PASS | `cargo test --test {test_name}` |
| G13 MySQL 协议 | 连接成功 | mysql:5.7 容器测试 |

---

## 四、阶段转换规则

### 4.1 转换条件

```
阶段转换 = 前置门禁 PASS + 所有 FAIL 项有 Issue/PR + 所有豁免项已审批
```

| 转换 | 前置条件 | 触发条件 |
|------|----------|----------|
| → A-Gate | 开发任务完成 | 开发者/AI 主动触发 |
| A-Gate → B-Gate | A-Gate 全部检查 PASS | CI 自动或人工触发 |
| B-Gate → R-Gate | B-Gate 全部检查 PASS，TPC-H SF=0.1 22/22 | CI 自动或人工触发 |
| R-Gate → G-Gate | R-Gate 全部检查 PASS，性能达标 | CI 自动或人工触发 |
| → GA | G-Gate 全部检查 PASS，所有 Issue 已关闭 | Human Architect 审批 |

### 4.2 阶段转换流程图

```
                    A-Gate 通过
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 阶段转换: Alpha → Beta                               │
│                                                                         │
│ 检查清单:                                                              │
│ [ ] 所有 A-Gate 检查项 PASS                                           │
│ [ ] 无 OPEN 的 P0/P1 Bug                                             │
│ [ ] TPC-H SF=0.1 数据已生成                                          │
│ [ ] SQL Corpus 已准备                                                 │
│                                                                         │
│ 执行操作:                                                              │
│ 1. 创建 milestone: v{X}.{Y}.{Z}-beta                                │
│ 2. 创建 alpha Tag: v{X}.{Y}.{Z}-alpha{N}                            │
│ 3. 通知团队进入 Beta 阶段                                             │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
                    B-Gate 通过
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 阶段转换: Beta → RC                                   │
│                                                                         │
│ 检查清单:                                                              │
│ [ ] 所有 B-Gate 检查项 PASS                                           │
│ [ ] 无 OPEN 的 P0/P1 Bug                                             │
│ [ ] TPC-H SF=1 可运行，无 OOM                                        │
│ [ ] SQL Corpus ≥95%                                                  │
│                                                                         │
│ 执行操作:                                                              │
│ 1. 创建 milestone: v{X}.{Y}.{Z}-rc                                  │
│ 2. 创建 beta Tag: v{X}.{Y}.{Z}-beta{N}                              │
│ 3. 通知团队进入 RC 阶段                                               │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
                    R-Gate 通过
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 阶段转换: RC → GA                                     │
│                                                                         │
│ 检查清单:                                                              │
│ [ ] 所有 R-Gate 检查项 PASS                                           │
│ [ ] Point Select QPS ≥10,000                                         │
│ [ ] UPDATE QPS ≥5,000                                                │
│ [ ] DELETE QPS ≥2,000                                                │
│ [ ] 所有 milestone Issue 已关闭                                       │
│                                                                         │
│ 执行操作:                                                              │
│ 1. Human Architect 审批发布                                          │
│ 2. 创建 rc Tag: v{X}.{Y}.{Z}-rc{N}                                  │
│ 3. 合并到 main 分支                                                   │
│ 4. 创建 GA Tag: v{X}.{Y}.{Z}                                        │
│ 5. 创建 release/X.Y 分支                                              │
│ 6. 发布 Release Notes                                                │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.3 门禁时间要求

| 门禁 | 预计时间 | 触发时机 |
|------|----------|----------|
| A-Gate | ~30min | Alpha→Beta |
| B-Gate | ~2h | Beta→RC |
| R-Gate | ~6h | RC→GA |
| G-Gate | ~12h | GA 发布前 |

---

## 五、版本发布流程

### 5.1 GA 发布流程图

```
                    R-Gate 通过
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    发布准备阶段                                       │
│                                                                         │
│ 1. 最终代码冻结                                                       │
│    [ ] 代码已审查                                                     │
│    [ ] 无未合并的 PR                                                  │
│    [ ] 分支保护已启用                                                 │
│                                                                         │
│ 2. 文档准备                                                           │
│    [ ] CHANGELOG 已更新                                               │
│    [ ] Release Notes 已创建                                           │
│    [ ] 用户文档已更新                                                  │
│    [ ] API 文档已生成                                                  │
│                                                                         │
│ 3. 发布审批                                                           │
│    [ ] Human Architect 审批发布                                       │
│    [ ] 确认发布时间表                                                  │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    发布执行阶段                                       │
│                                                                         │
│ 1. 创建 RC Tag                                                        │
│    git tag -a v{X}.{Y}.{Z}-rc1 -m "Release candidate 1"             │
│    git push origin v{X}.{Y}.{Z}-rc1                                 │
│                                                                         │
│ 2. 最终测试                                                           │
│    [ ] 执行 G-Gate 检查                                               │
│    [ ] 确认所有检查 PASS                                              │
│                                                                         │
│ 3. 合并到 main                                                        │
│    git checkout main                                                  │
│    git merge develop/v{X}.{Y}.{Z}                                   │
│    git push origin main                                               │
│                                                                         │
│ 4. 创建 GA Tag                                                        │
│    git tag -a v{X}.{Y}.{Z} -m "Version {X}.{Y}.{Z}"                 │
│    git push origin v{X}.{Y}.{Z}                                      │
│                                                                         │
│ 5. 创建 release 分支                                                  │
│    git checkout -b release/{X}.{Y}                                  │
│    git push origin release/{X}.{Y}                                   │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    发布后阶段                                         │
│                                                                         │
│ 1. 通知                                                               │
│    [ ] 团队通知                                                       │
│    [ ] 用户公告                                                       │
│    [ ] 文档更新                                                       │
│                                                                         │
│ 2. 清理                                                               │
│    [ ] 关闭 milestone                                                 │
│    [ ] 归档文档                                                       │
│    [ ] 清理旧分支                                                     │
│                                                                         │
│ 3. 回顾                                                               │
│    [ ] 版本回顾记录                                                    │
│    [ ] 治理审计更新                                                   │
│    [ ] 下版本计划启动                                                 │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 发布检查清单

```markdown
## GA 发布检查清单

### 发布前 48 小时
- [ ] G-Gate 全部检查 PASS
- [ ] Human Architect 审批发布
- [ ] CHANGELOG 已更新
- [ ] Release Notes 已起草

### 发布前 24 小时
- [ ] 最终代码冻结
- [ ] 分支保护已启用
- [ ] 所有 milestone Issue 已关闭
- [ ] 文档已更新

### 发布日
- [ ] 创建 RC Tag
- [ ] 最终验证
- [ ] 合并到 main
- [ ] 创建 GA Tag
- [ ] 创建 release 分支
- [ ] 推送所有更改

### 发布后
- [ ] 团队通知
- [ ] 用户公告
- [ ] 清理旧分支
- [ ] 版本回顾
```

---

## 六、版本延续机制

### 6.1 延续触发条件

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 说明 |
|------|------|
| 修复需要 3 人周以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖完成 | 如 CBO 需要先完成索引选择 |

### 6.2 延续流程

```
v3.0.0 B-Gate FAIL (SQL Corpus 20%)
         │
         ├── Issue #451 创建 (milestone: v3.0.0-beta)
         │
         ├── 评估: 修复需要 3 人周以上 → 触发版本延续
         │
         ▼
v3.1.0 DEVELOPMENT_PLAN.md §6 建立映射
         │
         ▼
v3.1.0 开发过程中
         │
         ▼
Issue #451 修复 → PR #XXX → 验证 PASS
         │
         ▼
Issue #451 关闭（需 PR 证据）
```

### 6.3 延续格式

```markdown
## v{NEXT_VERSION} 延续任务（来自 v{CURRENT_VERSION} 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | v{NEXT_VERSION} 目标 | 验收条件 | 优先级 |
|----------|----------|------------|---------------------|----------|--------|
| #451 | SQL Operations 语法支持 | 20% (11/55) | ≥80% (44/55) | test_sql_corpus_operations ≥80% | P0 |
```

---

## 七、版本维护

### 7.1 Patch 版本发布

当发现已发布版本的 Bug 时，发布 Patch 版本：

```
main
  │
  ├── release/3.0 ──────────────────────────────────▶ v3.0.1 (patch)
  │       │                                              │
  │       └── 从 main 或 develop/v3.0  cherry-pick 修复   │
  │                                                       │
  └── develop/v3.0 继续开发 v3.1.0                        │
```

**Patch 版本规则**：
- 只修复 Bug，不引入新功能
- 不改变 API
- 遵循语义化版本

### 7.2 版本维护周期

| 版本类型 | 维护周期 | 说明 |
|----------|----------|------|
| GA 正式版 | 12 个月 | 持续修复 Bug 和安全漏洞 |
| Minor 版本 | 与当前 GA 同期 | 新功能开发 |
| Patch 版本 | 需要时发布 | Bug 修复 |

### 7.3 版本支持状态

```
┌─────────────────────────────────────────────────────────────────────┐
│ 版本支持状态示例                                                    │
├─────────────────────────────────────────────────────────────────────┤
│ v3.1.0 (当前 GA)    │ ✅ 全面支持                                  │
│ v3.0.x (上版本)     │ ✅ 安全更新                                  │
│ v2.9.x (更早版本)   │ ⚠️ 延长支持 (安全更新)                       │
│ v2.8.x 及更早       │ ❌ 不再支持                                   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 八、版本目录结构

### 8.1 版本目录规范

```
docs/releases/
├── v3.1.0/
│   ├── DEVELOPMENT_PLAN.md           # 版本开发计划
│   ├── ALPHA_GATE_CHECKLIST.md      # Alpha 门禁清单
│   ├── ALPHA_TEST_PLAN.md           # Alpha 测试计划
│   ├── BETA_GATE_CHECKLIST.md       # Beta 门禁清单
│   ├── BETA_TEST_PLAN.md            # Beta 测试计划
│   ├── RC_GATE_CHECKLIST.md         # RC 门禁清单
│   ├── RC_TEST_PLAN.md              # RC 测试计划
│   ├── GA_GATE_CHECKLIST.md        # GA 门禁清单
│   ├── GOVERNANCE_AUDIT.md          # 治理审计报告
│   └── COMPREHENSIVE_STATUS_REPORT.md  # 综合状态报告
│
├── v3.0.0/
│   └── ... (同上结构)
│
└── v2.9.0/
    └── ... (同上结构)
```

### 8.2 必选文档清单

| 文档 | 用途 | 强制级别 |
|------|------|----------|
| DEVELOPMENT_PLAN.md | 版本开发计划 | 强制 |
| {PHASE}_GATE_CHECKLIST.md | 各阶段门禁清单 | 强制 |
| {PHASE}_TEST_PLAN.md | 各阶段测试计划 | 参考 |
| GOVERNANCE_AUDIT.md | 版本治理审计 | 强制 |

---

## 九、相关文档

| 文档 | 作用 | 路径 |
|------|------|------|
| RELEASE_LIFECYCLE.md | 四级门禁模型详解 | `docs/governance/RELEASE_LIFECYCLE.md` |
| GATE_SPEC_MASTER.md | 门禁规范 SSOT | `docs/governance/GATE_SPEC_MASTER.md` |
| GATE_PHASES_AND_TRACKING.md | 分阶段门禁追踪 | `docs/governance/GATE_PHASES_AND_TRACKING.md` |
| gate_lifecycle_tracking.md | Issue 追踪闭环 | `docs/governance/gate_lifecycle_tracking.md` |
| GOVERNANCE_STANDARD.md | 治理标准总纲 | `docs/governance/GOVERNANCE_STANDARD.md` |
| RELEASE_POLICY.md | 发布策略 | `docs/governance/RELEASE_POLICY.md` |

---

## 十、变更历史

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本，建立版本生命周期管理规范 | hermes-z6g4 |

---

## 附录 A: 版本号速查

```text
语义化版本: v{MAJOR}.{MINOR}.{PATCH}
示例: v3.1.0

Tag 格式:
- Alpha: vX.Y.Z-alpha{N}     示例: v3.1.0-alpha1
- Beta:  vX.Y.Z-beta{N}     示例: v3.1.0-beta1
- RC:    vX.Y.Z-rc{N}        示例: v3.1.0-rc1
- GA:    vX.Y.Z              示例: v3.1.0
```

## 附录 B: 分支命名速查

```text
main                          # GA 正式版本
release/X.Y                   # 维护已发布版本
develop/vX.Y.Z                # 当前版本开发
feature/{feature-name}        # 新功能
fix/{issue}-{short-desc}     # Bug 修复
```

## 附录 C: 门禁检查命令速查

```bash
# A-Gate
cargo build --workspace
cargo test --workspace
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
bash scripts/gate/check_docs_links.sh
cargo llvm-cov --all-features --lcov
cargo audit

# B-Gate (A-Gate +)
cargo build --release --workspace
cargo test --all-features
cargo llvm-cov --all-features --lcov
bash scripts/gate/check_tpch.sh sf=0.1
cargo test -p sqlrustgo-sql-corpus

# R-Gate (B-Gate +)
cargo test --all-features  # 100%
cargo llvm-cov --all-features --lcov  # ≥85%
bash scripts/gate/check_tpch.sh sf=1
cargo bench && scripts/gate/check_perf_baseline.sh
bash scripts/gate/check_sysbench.sh

# G-Gate (R-Gate +)
cargo bench -- point_select  # ≥10K QPS
cargo bench -- update_simple  # ≥5K QPS
cargo bench -- delete_simple  # ≥2K QPS
cargo test -p sqlrustgo-sql-corpus  # ≥98%
```

---

*本文档是 SQLRustGo 版本生命周期管理的唯一权威来源。所有版本的开发、发布和维护必须遵循本文档的规定。*

*维护人: hermes-z6g4*
*最后更新: 2026-05-14*
