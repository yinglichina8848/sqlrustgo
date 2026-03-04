# SQLRustGo 教学实践材料

> 本目录包含 SQLRustGo 项目的学生实践资源和课堂材料

---

## 一、目录结构

```
docs/教学实践/
├── v1.1.0-beta/              # v1.1.0-beta 阶段实践材料
│   ├── README.md             # 阶段概述
│   ├── developer/            # 开发者指南
│   ├── testing/              # 测试指南
│   ├── user/                 # 用户指南
│   ├── handbook-student.md   # 学生执行手册
│   ├── handbook-ta.md        # 助教执行手册
│   ├── task-board.md         # 任务看板
│   └── pr-evidence.md        # PR 证据链
│
└── templates/                # 通用模板
    └── daily-template.md     # 日报模板
```

---

## 二、与教学计划的区别

| 目录 | 内容 | 使用者 |
|------|------|--------|
| [教学计划](../教学计划/) | PPT、教学进度、大纲、实验指导书 | 教师 |
| **教学实践** (本目录) | 学生手册、任务看板、日报模板 | 学生 |

---

## 三、实践材料索引

### v1.1.0-beta 阶段

| 文档 | 描述 | 适用角色 |
|------|------|----------|
| [用户指南](./v1.1.0-beta/user/README.md) | REPL 使用、SQL 支持、服务器模式 | 用户 |
| [开发指南](./v1.1.0-beta/developer/README.md) | 开发环境搭建、代码规范、调试 | 开发者 |
| [测试指南](./v1.1.0-beta/testing/README.md) | 单元测试、集成测试、性能测试 | 测试人员 |
| [学生执行手册](./v1.1.0-beta/handbook-student.md) | 学生可复现步骤 | 学生 |
| [助教执行手册](./v1.1.0-beta/handbook-ta.md) | PR 证据链示例 | 助教 |
| [任务看板](./v1.1.0-beta/task-board.md) | Beta 阶段任务追踪 | 全员 |
| [PR 证据链](./v1.1.0-beta/pr-evidence.md) | PR 审核与风险摘要 | 开发者 |

### 通用模板

| 文档 | 描述 |
|------|------|
| [日报模板](./templates/daily-template.md) | 课堂用日报模板 |

---

## 四、门禁检查标准

| 检查项 | 命令 | 要求 |
|--------|------|------|
| 编译 | `cargo build --all-features` | 通过 |
| 测试 | `cargo test --all-features` | 全部通过 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt --check` | 通过 |
| 覆盖率 | `cargo tarpaulin` 或 `cargo llvm-cov` | ≥ 80% |

---

## 五、相关链接

- [教学计划](../教学计划/) - 教师准备材料
- [发布文档](../releases/) - 正式版本发布文档
- [v1.0 开发文档](../v1.0/) - v1.0.x 开发过程文档
- [AI 增强软件工程](../AI增强软件工程/) - AI 协作开发教程

---

*本文档由 yinglichina8848 创建*
*最后更新: 2026-03-05*
