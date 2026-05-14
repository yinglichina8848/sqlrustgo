# SQLRustGo 测试系统

> 双测试系统架构 - 本地开发 + Nomad CI

## 概述

```
┌─────────────────────────────────────────────────────────────────┐
│                    SQLRustGo 测试基础设施                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────┐    ┌──────────────────────────────┐  │
│  │   SYSTEM 1: LOCAL    │    │     SYSTEM 2: NOMAD CI       │  │
│  │   本地开发测试系统      │    │      Nomad CI 门禁系统       │  │
│  ├──────────────────────┤    ├──────────────────────────────┤  │
│  │ 触发: 开发者手动       │    │ 触发: PR/合并/定时           │  │
│  │ 目标: <10min 反馈     │    │ 目标: 100% 覆盖率门禁         │  │
│  │ 环境: Mac Mini/Z6G4   │    │ 环境: Nomad Cluster          │  │
│  └──────────────────────┘    └──────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 快速开始

### 本地测试 (System 1)

```bash
# L0 冒烟测试 (<5min)
./scripts/test/local/l0_smoke.sh

# L1 单元测试 (<10min)
./scripts/test/local/l1_unit.sh

# L2 集成测试 (<15min)
./scripts/test/local/l2_integration.sh

# 全部本地测试
./scripts/test/local/run_local.sh all
```

### CI 门禁 (System 2)

```bash
# PR 门禁
./scripts/test/nomad/pr_gate.sh

# Beta 门禁
./scripts/test/nomad/b_gate.sh

# RC 门禁
./scripts/test/nomad/r_gate.sh
```

## 分层结构

| 层级 | 名称 | 执行时间 | 触发方式 |
|------|------|----------|----------|
| L0 | 冒烟测试 | <5min | 每次 push 前强制 |
| L1 | 单元测试 | <10min | 每次 push 前强制 |
| L2 | 集成测试 | <15min | 每次 PR |
| L3 | 快速回归 | <20min | 每日 / 关键 PR |

## 门禁系统

| Gate | 名称 | 触发条件 | 执行位置 |
|------|------|----------|----------|
| PR-Gate | PR 合并门禁 | 每个 PR | Nomad runner |
| B-Gate | Beta 门禁 | Alpha→Beta | Nomad runner |
| R-Gate | RC 门禁 | Beta→RC | Nomad runner |
| G-Gate | GA 门禁 | RC→GA | Nomad runner |

## 文件结构

```
scripts/test/
├── local/
│   ├── l0_smoke.sh          # L0 冒烟测试
│   ├── l1_unit.sh           # L1 单元测试
│   ├── l2_integration.sh    # L2 集成测试
│   └── run_local.sh         # 本地测试入口
├── nomad/
│   ├── pr_gate.sh           # PR 门禁
│   ├── b_gate.sh            # Beta 门禁
│   └── r_gate.sh            # RC 门禁
└── manifest/
    └── test_manifest.yaml   # 测试清单

.gitea/workflows/
├── pr-gate.yml              # PR Gate CI
├── beta-gate.yml            # Beta Gate CI
└── rc-gate.yml              # RC Gate CI
```

## 测试分类

| 测试类型 | L0 | L1 | L2 | L3 | PR-Gate | B-Gate | R-Gate |
|----------|----|----|----|----|---------|--------|--------|
| 冒烟测试 | ✅ | | | | ✅ | ✅ | ✅ |
| 单元测试 | | ✅ | | | ✅ | ✅ | ✅ |
| 集成测试 | | | ✅ | | ✅ | ✅ | ✅ |
| 性能测试 | | | | ✅ | | ✅ | ✅ |
| TPC-H | | | | ✅ | | ✅ | ✅ |
| 崩溃恢复 | | | | ✅ | | ✅ | ✅ |
| 安全审计 | | | | | | ✅ | ✅ |

## 报告输出

```bash
# 本地测试报告
./scripts/test/local/l0_smoke.sh
# 输出: JSON + Console

# CI 门禁报告
./scripts/test/nomad/b_gate.sh
# 输出: /tmp/b_gate_artifacts/b_gate_report.json
```

## 故障处理

| 层级 | 失败处理 |
|------|----------|
| L0 | 阻塞 push |
| L1 | 阻塞 push |
| L2 | 允许 push，标记 WARN |
| PR-Gate | 阻塞合并 |

## 下一步

- [ ] 实现 L3 回归测试脚本
- [ ] 实现 E2E 测试框架
- [ ] 部署 Nomad CI jobs
- [ ] 设置定时触发器
