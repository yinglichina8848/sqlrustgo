# SQLRustGo 分支治理规范

> **版本**: 1.0
> **更新日期**: 2026-03-06
> **维护人**: yinglichina8848

---

## 一、分支结构

```
main
 │
 ├─ develop                    # 下一个版本开发
 │
 ├─ develop-1.2.0              # 当前版本开发
 │    ├─ feature/*             # 功能分支
 │    └─ bugfix/*              # 修复分支
 │
 └─ release/1.x                # 维护分支
```

---

## 二、分支类型

| 分支类型 | 命名规则 | 用途 | 生命周期 |
|----------|----------|------|----------|
| `main` | `main` | 稳定版本 | 永久 |
| `develop` | `develop` | 下一版本开发 | 永久 |
| `develop-x.x.x` | `develop-{version}` | 版本开发 | 版本发布后转 release |
| `feature` | `feature/{name}` | 功能开发 | 合并后删除 |
| `bugfix` | `bugfix/{name}` | Bug 修复 | 合并后删除 |
| `release` | `release/{version}` | 版本维护 | 长期 |

---

## 三、版本阶段演进

**核心原则**: 分支不关，但"门禁"越来越严格

| 阶段 | 允许提交 | 禁止提交 | 门禁要求 |
|------|----------|----------|----------|
| Draft | 架构、目录、接口设计 | 无 | 编译通过 |
| Alpha | 新功能、新模块 | 架构变化 | 测试 ≥ 80% |
| Beta | Bug 修复、性能优化 | 架构/API 变化 | 测试 ≥ 95% |
| RC | 仅 Critical Bug 修复 | 新代码、重构 | 测试 100% |
| GA | 禁止修改 | 所有提交 | 全部门禁 |

详见: [docs/releases/v1.2.0/BRANCH_STAGE_GOVERNANCE.md](docs/releases/v1.2.0/BRANCH_STAGE_GOVERNANCE.md)

---

## 四、分支保护规则

### 4.1 保护级别

| 分支 | 直接 Push | PR 审核 | CI 要求 |
|------|-----------|---------|---------|
| `main` | ❌ 禁止 | 2 人 | 全绿 |
| `develop-x.x.x` | ❌ 禁止 | 1 人 | 编译通过 |
| `feature/*` | ✅ 允许 | 可选 | 无 |
| `release/*` | ❌ 禁止 | 2 人 | 全绿 |

### 4.2 AI 协作开发特别规则

由于采用 AI 协作开发模式，所有版本分支必须：

1. **禁止直接 push**
2. **必须通过 PR**
3. **必须通过 CI**
4. **必须有人工审核**

---

## 五、版本发布流程

```
develop-x.x.x
    │
    ├── Draft
    ├── Alpha
    ├── Beta
    ├── RC
    │
    └── GA
        ├── git tag v1.x.x
        ├── merge → main
        └── rename → release/x.x
```

---

## 六、相关文档

- [BRANCH_STAGE_GOVERNANCE.md](docs/releases/v1.2.0/BRANCH_STAGE_GOVERNANCE.md) - 阶段演进详细规范
- [VERSION_PLAN.md](docs/releases/v1.2.0/VERSION_PLAN.md) - 版本计划
- [RELEASE_GATE_CHECKLIST.md](docs/releases/v1.2.0/RELEASE_GATE_CHECKLIST.md) - 门禁清单

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本 |

---

*本文档由 yinglichina8848 维护*
