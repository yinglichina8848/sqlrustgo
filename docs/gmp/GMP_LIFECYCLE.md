# GMP_LIFECYCLE.md - 生命周期管理

**文件路径**: `/Users/liying/workspace/dev/yinglichina163/sqlrustgo/docs/gmp/GMP_LIFECYCLE.md`  
**版本**: v1.0.0  
**状态**: ACTIVE  
**创建日期**: 2026-05-14  
**最后更新**: 2026-05-14  

---

## 1. 概述

本文档定义了 SQLRustGo GMP 可信可证明系统的全生命周期管理流程，从需求捕获到系统退役，确保每个阶段都有完整的证据链和可追溯性。

**核心目标**: 每一阶段的可交付物必须满足"可信、可证明"标准，才能进入下一阶段。

---

## 2. 生命周期阶段总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        GMP 生命周期阶段                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│  │  需求   │───▶│  设计   │───▶│  实现   │───▶│  测试   │───▶│  发布   │  │
│  │  阶段   │    │  阶段   │    │  阶段   │    │  阶段   │    │  阶段   │  │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘  │
│       │              │              │              │              │        │
│       ▼              ▼              ▼              ▼              ▼        │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│  │ REQ-XXX │    │ DES-XXX │    │ PR/CODE │    │ TEST-XX │    │ REL-XXX │  │
│  │ 文档    │    │ 文档    │    │ 证据包  │    │ 报告   │    │ 证据包  │  │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘  │
│       │              │              │              │              │        │
│       │              │              │              │              │        │
│       ▼              ▼              ▼              ▼              ▼        │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐  │
│  │ 运营    │◀───│ 监控    │◀───│ 集成    │◀───│ 验证    │◀───│ 阶段    │  │
│  │ 阶段    │    │ 阶段    │    │ 阶段    │    │ 阶段    │    │ 评审    │  │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. 阶段详细定义

## 3.1 需求阶段 (Requirements Phase)

**阶段目标**: 捕获、定义和锁定系统需求，建立完整的可追溯性基线。

### 3.1.1 需求分类

| 类型 | 标识 | 说明 | 证据等级 |
|------|------|------|----------|
| 功能需求 | REQ-FUNC | 系统必须提供的功能 | L3 |
| 性能需求 | REQ-PERF | 性能指标要求 | L3 |
| 安全需求 | REQ-SEC | 安全属性要求 | L2 |
| 接口需求 | REQ-INTF | 外部接口规范 | L3 |
| 可靠性需求 | REQ-RELY | 可用性、故障处理 | L2 |
| 形式化验证需求 | REQ-FORM | 必须形式化证明的属性 | L1 |

### 3.1.2 需求文档模板

```json
{
  "requirement": {
    "id": "REQ-FUNC-001",
    "title": "SQL SELECT 语句支持",
    "version": "1.0",
    "status": "APPROVED",
    "classification": "FUNCTIONAL",
    "priority": "MUST_HAVE",
    "description": "系统必须支持符合 SQL92 标准的 SELECT 语句",
    "specification": {
      "syntax": "SELECT [DISTINCT] column_list FROM table_list [WHERE condition] [GROUP BY column] [HAVING condition] [ORDER BY column [ASC|DESC]]",
      "support_level": "FULL",
      "dialect": "SQL92"
    },
    "acceptance_criteria": [
      {
        "id": "AC-001",
        "description": "可正确解析基本 SELECT 语句",
        "verification_method": "TEST",
        "evidence_required": "TST-EVD-001"
      },
      {
        "id": "AC-002", 
        "description": "可正确执行 JOIN 操作",
        "verification_method": "TEST",
        "evidence_required": "TST-EVD-002"
      },
      {
        "id": "AC-003",
        "description": "可正确执行子查询",
        "verification_method": "TEST",
        "evidence_required": "TST-EVD-003"
      }
    ],
    "traceability": {
      "derived_from": [],
      "related_requirements": ["REQ-FUNC-002", "REQ-FUNC-003"],
      "satisfied_by_design": ["DES-ARCH-001"],
      "implemented_by": [],
      "tested_by": ["TEST-INT-001", "TEST-INT-002"]
    },
    "signatures": {
      "author": { "name": "XXX", "timestamp": "2026-01-01T00:00:00Z" },
      "reviewer": { "name": "YYY", "timestamp": "2026-01-02T00:00:00Z" },
      "approver": { "name": "ZZZ", "timestamp": "2026-01-03T00:00:00Z" }
    }
  }
}
```

### 3.1.3 阶段入口/出口准则

**入口准则 (Entry Criteria)**:
- [ ] 业务需求文档已提供
- [ ] 利益相关者已识别
- [ ] 初始需求草案已完成

**出口准则 (Exit Criteria)**:
- [ ] 所有需求已分配唯一标识符 (REQ-XXXX)
- [ ] 每个需求有明确的验收标准
- [ ] 需求评审已通过并签字
- [ ] 需求追踪矩阵已建立
- [ ] 形式化验证需求已识别

### 3.1.4 需求阶段检查清单

- [ ] 需求可测试
- [ ] 需求无歧义
- [ ] 需求可追溯到业务目标
- [ ] 需求优先级已确定
- [ ] 安全需求已明确
- [ ] 性能需求有量化指标
- [ ] 接口需求有明确定义

---

## 3.2 设计阶段 (Design Phase)

**阶段目标**: 将需求转化为系统架构和详细设计，每项设计决策都有明确依据。

### 3.2.1 设计工件

| 工件 | 标识 | 说明 | 必需性 |
|------|------|------|--------|
| 架构设计 | DES-ARCH | 系统高层架构 | 必须 |
| 接口设计 | DES-INTF | 组件间接口定义 | 必须 |
| 数据模型 | DES-DATA | 数据结构定义 | 必须 |
| 算法规格 | DES-ALGO | 关键算法规格说明 | 必须 |
| 安全设计 | DES-SEC | 安全架构 | 必须 |
| 形式化规格 | DES-FORM | 形式化验证对象 | 条件必需 |

### 3.2.2 架构设计模板

```json
{
  "architecture_design": {
    "id": "DES-ARCH-001",
    "title": "SQLRustGo 核心架构",
    "version": "1.0",
    "status": "APPROVED",
    "overview": "描述系统高层架构...",
    "components": [
      {
        "name": "sql_parser",
        "responsibility": "SQL 解析",
        "interface": "Parser::parse(sql: &str) -> Result<AST>",
        "dependencies": [],
        "design_decisions": [
          {
            "id": "DD-001",
            "decision": "使用手写解析器而非生成器",
            "rationale": "更好的错误报告和控制",
            "alternatives_considered": ["LALRPOP", "pest"],
            "evidence": "PERF-EVD-001"
          }
        ]
      }
    ],
    "design_rules": [
      {
        "rule": "所有组件间通信通过定义好的接口",
        "rationale": "可测试性和可替换性",
        "enforcement": "接口边界静态分析"
      }
    ],
    "traceability": {
      "satisfies_requirements": ["REQ-FUNC-001", "REQ-FUNC-002"],
      "derived_from_architecture_decisions": ["DD-001", "DD-002"]
    },
    "signatures": {
      "architect": { "name": "XXX", "timestamp": "2026-01-10T00:00:00Z" },
      "reviewers": [
        { "name": "YYY", "timestamp": "2026-01-11T00:00:00Z" }
      ],
      "approver": { "name": "ZZZ", "timestamp": "2026-01-12T00:00:00Z" }
    }
  }
}
```

### 3.2.3 阶段入口/出口准则

**入口准则**:
- [ ] 需求已锁定 (REQ 状态为 APPROVED)
- [ ] 架构师已指派
- [ ] 设计计划已批准

**出口准则**:
- [ ] 架构设计文档已完成并评审
- [ ] 所有接口已定义
- [ ] 关键算法有规格说明
- [ ] 形式化验证对象已识别并有验证计划
- [ ] 设计评审通过并签字

### 3.2.4 设计阶段检查清单

**架构检查**:
- [ ] 模块划分清晰，职责单一
- [ ] 接口定义完整、无歧义
- [ ] 依赖关系明确，无循环依赖
- [ ] 关键路径有备选方案

**接口检查**:
- [ ] 所有接口有输入/输出规格
- [ ] 错误处理策略明确
- [ ] 接口变更管理机制建立

**可证明性检查**:
- [ ] 关键属性已形式化规格化
- [ ] 证明方法已选定
- [ ] 证明工具已配置

---

## 3.3 实现阶段 (Implementation Phase)

**阶段目标**: 按照设计实现代码，确保代码质量、可测试性和可追溯性。

### 3.3.1 代码组织

```
src/
├── main.rs                 # 应用入口
├── lib.rs                  # 库入口
├── sql/
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── lexer.rs        # 词法分析
│   │   ├── parser.rs       # 语法分析
│   │   ├── ast.rs          # AST 定义
│   │   └── error.rs        # 错误类型
│   ├── planner/
│   │   ├── mod.rs
│   │   ├── optimizer.rs    # 查询优化
│   │   └── planner.rs      # 执行计划生成
│   └── executor/
│       ├── mod.rs
│       ├── hash_join.rs    # Hash Join 实现
│       └── ...
├── storage/
│   └── ...
└── protocol/
    └── ...
```

### 3.3.2 代码模板

```rust
//! Hash Join 模块
//!
//! # 形式化规格
//!
//! ## 不变量 INV1: 结果正确性
//! 对于任意输入 R1, R2 和键 k:
//! ```text
//! (r1 ∈ R1 ∧ r2 ∈ R2 ∧ r1.key = r2.key = k) ⇔ (r1 ⨝ r2 ∈ Result)
//! ```
//!
//! ## 不变量 INV2: 无丢失连接
//! ```text
//! ∀r1 ∈ R1, r2 ∈ R2: r1.key = r2.key ⇒ ∃g ∈ Result: g.key = k ∧ g.r1_data = r1.data
//! ```
//!
//! # 证据要求
//!
//! - 单元测试: TST-EVD-010 (分支覆盖 ≥ 95%)
//! - 集成测试: TST-EVD-011 (与其它操作符协作正确)
//! - 形式化证明: PRF-EVD-001 (CBMC 模型检查通过)
//!
//! # 变更历史
//!
//! | 版本 | 日期 | 变更 | 作者 |
//! |------|------|------|------|
//! | 1.0 | 2026-01-01 | 初始实现 | XXX |

use std::collections::HashMap;

/// Hash Join 构建阶段的错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashJoinError {
    /// 内存耗尽
    OutOfMemory { requested: usize, available: usize },
    /// 无效键类型
    InvalidKeyType { expected: &'static str, actual: &'static str },
    /// 构建失败
    BuildFailed { reason: &'static str },
}

impl std::fmt::Display for HashJoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfMemory { requested, available } => {
                write!(f, "Out of memory: requested {} bytes, available {}", requested, available)
            }
            Self::InvalidKeyType { expected, actual } => {
                write!(f, "Invalid key type: expected {}, got {}", expected, actual)
            }
            Self::BuildFailed { reason } => {
                write!(f, "Build failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for HashJoinError {}

// ==================== 公开 API ====================

/// 执行 Hash Join
///
/// # 参数
/// - `left`: 左关系 (build side)
/// - `right`: 右关系 (probe side)
/// - `left_key`: 左关系用于 join 的键位置
/// - `right_key`: 右关系用于 join 的键位置
///
/// # 返回
/// - `Ok(Vec<Row>)`: 连接结果
/// - `Err(HashJoinError)`: 错误信息
///
/// # 形式化规格
/// - **PRE**: left 和 right 非空，且键位置有效
/// - **POST**: 返回所有匹配的元组对
pub fn hash_join(
    left: &[Row],
    right: &[Row],
    left_key: usize,
    right_key: usize,
) -> Result<Vec<Row>, HashJoinError> {
    // 前置条件检查
    assert!(!left.is_empty(), "PRE: left must be non-empty");
    assert!(!right.is_empty(), "PRE: right must be non-empty");
    
    // 构建哈希表
    let mut hash_table: HashMap<&HashableValue, Vec<&Row>> = HashMap::new();
    
    for row in left {
        let key = row.key_at(left_key)?;
        hash_table
            .entry(key)
            .or_insert_with(Vec::new)
            .push(row);
    }
    
    // 探测阶段
    let mut results = Vec::new();
    for row in right {
        let key = row.key_at(right_key)?;
        if let Some(matches) = hash_table.get(&key) {
            for left_row in matches {
                results.push(row.join(left_row));
            }
        }
    }
    
    Ok(results)
}
```

### 3.3.3 阶段入口/出口准则

**入口准则**:
- [ ] 设计文档已冻结 (DES 状态为 APPROVED)
- [ ] 开发环境已配置
- [ ] 代码规范已确定
- [ ] PR 流程已建立

**出口准则**:
- [ ] 所有模块实现完成
- [ ] 代码审查通过 (至少 1 人审查)
- [ ] 静态分析通过 (0 warnings)
- [ ] 单元测试通过 (覆盖率达标)
- [ ] 形式化证明完成 (如需要)
- [ ] 提交信息规范 (含证据 ID)

### 3.3.4 实现阶段检查清单

**代码质量**:
- [ ] 所有公共 API 有文档注释
- [ ] 错误类型定义完整
- [ ] 无未处理的 Result/Option
- [ ] 无硬编码配置/凭证
- [ ] 无同步阻塞 I/O (在 async 上下文中)

**可追溯性**:
- [ ] 每个文件关联 REQ
- [ ] 每个函数有前置/后置条件
- [ ] 提交信息包含证据 ID

**禁止的模式**:
- [ ] `unsafe` 块无文档
- [ ] `expect`/`unwrap` 无合理理由
- [ ] 魔法数字
- [ ] 重复代码

---

## 3.4 测试阶段 (Test Phase)

**阶段目标**: 通过系统化测试验证实现满足需求，产生完整的测试证据。

### 3.4.1 测试计划模板

```json
{
  "test_plan": {
    "id": "TEST-PLAN-001",
    "title": "SQLRustGo v2.1.0 测试计划",
    "version": "1.0",
    "status": "APPROVED",
    "scope": {
      "components": ["sql_parser", "query_planner", "hash_join"],
      "excluded": []
    },
    "test_levels": {
      "L1_FORMAL": {
        "required": true,
        "coverage": "关键算法 100%",
        "tools": ["TLA+", "CBMC"]
      },
      "L2_INTEGRATION": {
        "required": true,
        "test_count_minimum": 100,
        "coverage_minimum": "90%"
      },
      "L3_UNIT": {
        "required": true,
        "test_count_minimum": 500,
        "coverage_minimum": "85%"
      },
      "L4_STATIC": {
        "required": true,
        "clippy_level": "deny",
        "warnings_as_errors": true
      }
    },
    "entry_criteria": [
      "代码实现完成",
      "CI 配置完成"
    ],
    "exit_criteria": [
      "所有测试通过",
      "覆盖率达标",
      "无 HIGH/CRITICAL 缺陷",
      "证据包完整"
    ]
  }
}
```

### 3.4.2 阶段入口/出口准则

**入口准则**:
- [ ] 代码实现冻结
- [ ] 测试环境已搭建
- [ ] 测试工具已配置

**出口准则**:
- [ ] 所有测试用例执行完成
- [ ] 测试覆盖率达标
- [ ] 所有缺陷已记录或修复
- [ ] 测试报告已生成
- [ ] 证据包已收集

### 3.4.3 测试阶段检查清单

**L1 形式化证明** (如需要):
- [ ] 证明模型已建立
- [ ] 不变量已定义
- [ ] 证明已完成
- [ ] 证明验证通过

**L2 集成测试**:
- [ ] 组件接口测试覆盖
- [ ] 端到端场景测试覆盖
- [ ] 故障注入测试通过

**L3 单元测试**:
- [ ] 公共 API 全覆盖
- [ ] 边界条件覆盖
- [ ] 错误路径覆盖
- [ ] 分支覆盖达标

**L4 静态分析**:
- [ ] Clippy 无警告
- [ ] 安全扫描通过
- [ ] 格式检查通过

---

## 3.5 验证阶段 (Verification Phase)

**阶段目标**: 独立验证实现满足需求，产生活动证据。

### 3.5.1 验证活动

| 活动 | 方法 | 证据 |
|------|------|------|
| 需求验证 | 需求追踪矩阵审查 | REQ-EVD |
| 设计验证 | 设计评审 | DES-EVD |
| 代码验证 | 静态分析 + 审查 | COD-EVD |
| 测试验证 | 测试执行 + 审查 | TST-EVD |
| 安全验证 | 渗透测试 + 扫描 | SEC-EVD |
| 性能验证 | 基准测试 | PRF-EVD |

### 3.5.2 阶段入口/出口准则

**入口准则**:
- [ ] 测试阶段完成
- [ ] 所有证据已收集
- [ ] 验证环境已准备

**出口准则**:
- [ ] 需求验证通过
- [ ] 验证报告已生成
- [ ] 所有问题已关闭或接受风险
- [ ] QA 签字确认

---

## 3.6 发布阶段 (Release Phase)

**阶段目标**: 产生可部署的发布包和完整证据包。

### 3.6.1 发布检查清单

**预发布检查**:
- [ ] 版本号已分配 (semver)
- [ ] CHANGELOG 已更新
- [ ] 发布说明已编写
- [ ] 证据包已归档
- [ ] 安全扫描通过
- [ ] 性能基准达标
- [ ] 所有签名收集完成

**发布批准**:
```json
{
  "release_approval": {
    "version": "2.1.0",
    "artifacts": [
      { "type": "binary", "path": "target/release/sqlrustgo", "checksum": "sha256:xxxxx" },
      { "type": "evidence", "path": "evidence/2.1.0/MANIFEST.json", "checksum": "sha256:yyyyy" }
    ],
    "signatures": [
      { "role": "Technical Lead", "name": "XXX", "timestamp": "2026-05-14T12:00:00Z" },
      { "role": "QA Lead", "name": "YYY", "timestamp": "2026-05-14T12:01:00Z" },
      { "role": "Release Manager", "name": "ZZZ", "timestamp": "2026-05-14T12:02:00Z" }
    ],
    "status": "APPROVED"
  }
}
```

### 3.6.2 阶段入口/出口准则

**入口准则**:
- [ ] 验证阶段完成
- [ ] 发布候选版本已构建
- [ ] 发布说明已准备

**出口准则**:
- [ ] 发布批准已签字
- [ ] 发布包已分发
- [ ] 证据包已归档
- [ ] 部署验证通过

---

## 3.7 运营阶段 (Operation Phase)

**阶段目标**: 监控系统运行，收集运营证据，支持问题追溯。

### 3.7.1 监控指标

| 指标 | 目标 | 告警阈值 |
|------|------|----------|
| 可用性 | 99.9% | < 99.5% |
| 响应时间 P99 | < 100ms | > 200ms |
| 错误率 | < 0.1% | > 1% |
| CPU 使用率 | < 70% | > 90% |
| 内存使用率 | < 80% | > 95% |

### 3.7.2 事件响应流程

```
事件检测 ──▶ 事件分类 ──▶ 应急响应 ──▶ 问题修复 ──▶ 复盘总结
                  │                                     │
                  ▼                                     ▼
            P1/P2: 立即处理                    证据归档
            P3/P4: 计划修复                    流程改进
```

### 3.7.3 阶段检查清单

- [ ] 监控系统正常运行
- [ ] 告警配置正确
- [ ] 事件响应流程已定义
- [ ] 问题追溯机制已建立

---

## 4. 阶段转换流程

### 4.1 阶段门控

```
┌──────────────────────────────────────────────────────────────┐
│                    阶段门控检查                               │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  阶段 N 完成 ──▶ 门控检查 ──┬── PASS ──▶ 阶段 N+1 开始     │
│                             │                                │
│                             └── FAIL ──▶ 返回阶段 N 修复    │
│                                                              │
│  门控检查内容:                                               │
│  1. 必需工件存在且完整                                      │
│  2. 必需签名收集完成                                        │
│  3. 必需证据包已归档                                        │
│  4. 开放问题已接受或解决                                    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### 4.2 门控检查清单模板

```json
{
  "gate_check": {
    "phase": "TEST",
    "gate": "GATE-TEST-TO-RELEASE",
    "check_date": "2026-05-14T12:00:00Z",
    "checks": [
      {
        "id": "GC-001",
        "description": "所有 L3 单元测试通过",
        "status": "PASS",
        "evidence": "TST-EVD-001"
      },
      {
        "id": "GC-002",
        "description": "分支覆盖率 ≥ 85%",
        "status": "PASS",
        "evidence": "COV-EVD-001"
      },
      {
        "id": "GC-003",
        "description": "无 HIGH/CRITICAL 缺陷",
        "status": "PASS",
        "evidence": "DEF-EVD-001"
      },
      {
        "id": "GC-004",
        "description": "静态分析通过",
        "status": "PASS",
        "evidence": "STA-EVD-001"
      }
    ],
    "overall_status": "PASS",
    "signatures": {
      "gate_checker": { "name": "QA", "timestamp": "2026-05-14T12:30:00Z" },
      "technical_lead": { "name": "TL", "timestamp": "2026-05-14T12:31:00Z" }
    }
  }
}
```

---

## 5. 变更管理

### 5.1 变更分类

| 分类 | 影响范围 | 处理流程 |
|------|----------|----------|
| PATCH | 缺陷修复，向后兼容 | 快速通道 (2 人批准) |
| MINOR | 新功能，向后兼容 | 标准流程 (完整测试) |
| MAJOR | 破坏性变更 | 完整流程 (架构评审) |

### 5.2 变更请求模板

```json
{
  "change_request": {
    "id": "CR-001",
    "title": "Hash Join 内存优化",
    "classification": "MINOR",
    "rationale": "减少大表连接内存占用",
    "affected_components": ["sql/executor/hash_join.rs"],
    "affected_requirements": ["REQ-PERF-001"],
    "impact_assessment": {
      "scope": "内部实现优化",
      "backward_compatibility": "完全兼容",
      "risk": "LOW"
    },
    "implementation_plan": {
      "phase": "IMPLEMENTATION",
      "tasks": ["优化哈希表实现", "更新单元测试"],
      "test_strategy": "回归测试 + 性能基准"
    },
    "approval": {
      "technical_lead": { "status": "APPROVED", "timestamp": "2026-05-10T00:00:00Z" },
      "product_owner": { "status": "APPROVED", "timestamp": "2026-05-10T00:01:00Z" }
    }
  }
}
```

---

## 6. 问题管理

### 6.1 缺陷分类

| 严重级别 | 定义 | 响应时间 | 处理优先级 |
|----------|------|----------|------------|
| P1 (Critical) | 系统崩溃、数据损坏 | 立即 | 必须立即修复 |
| P2 (High) | 核心功能不可用 | 24h | 高优先级 |
| P3 (Medium) | 功能缺陷，性能下降 | 72h | 正常流程 |
| P4 (Low) | 表面问题，优化建议 | 1 周 | 低优先级 |

### 6.2 缺陷模板

```json
{
  "defect": {
    "id": "DEF-001",
    "title": "Hash Join 处理 NULL 键时产生错误结果",
    "severity": "P1",
    "status": "OPEN",
    "discovery": {
      "source": "automated_test",
      "test_case": "test_hash_join_null_key",
      "date": "2026-05-14T12:00:00Z"
    },
    "affected_requirements": ["REQ-FUNC-010"],
    "root_cause": "NULL 键未正确处理",
    "fix": {
      "task_id": "TASK-001",
      "assignee": "Developer XXX",
      "planned_date": "2026-05-15",
      "evidence_required": "回归测试通过"
    },
    "signatures": {
      "reporter": { "name": "Tester", "timestamp": "2026-05-14T12:00:00Z" },
      "triager": { "name": "TL", "timestamp": "2026-05-14T12:30:00Z" }
    }
  }
}
```

---

## 7. 证据归档

### 7.1 证据归档结构

```
evidence/
├── v2.1.0/
│   ├── MANIFEST.json           # 证据清单
│   ├── 01_REQUIREMENTS/         # 需求阶段证据
│   │   ├── REQ-APPROVAL.json
│   │   └── REQ-TRACKING-MATRIX.json
│   ├── 02_DESIGN/              # 设计阶段证据
│   │   ├── DES-APPROVAL.json
│   │   └── DESIGN-REVIEW.json
│   ├── 03_IMPLEMENTATION/      # 实现阶段证据
│   │   ├── CODE-REVIEW.json
│   │   └── STATIC-ANALYSIS.json
│   ├── 04_TEST/                # 测试阶段证据
│   │   ├── TEST-RESULTS.json
│   │   ├── COVERAGE.json
│   │   └── FORMAL-PROOF/
│   │       └── *.tla
│   ├── 05_VERIFICATION/        # 验证阶段证据
│   │   └── VERIFICATION-REPORT.json
│   ├── 06_RELEASE/             # 发布阶段证据
│   │   ├── APPROVAL.json
│   │   └── ARTIFACTS.json
│   └── SIGNATURES.json         # 所有签名
```

### 7.2 证据保留策略

| 证据类型 | 保留期限 | 存储位置 |
|----------|----------|----------|
| 发布证据包 | 永久 | 归档存储 |
| 测试结果 | 永久 | 归档存储 |
| 代码审查记录 | 永久 | 归档存储 |
| 性能基准 | 5 年 | 归档存储 |
| 安全扫描报告 | 5 年 | 归档存储 |
| 形式化证明 | 永久 | 归档存储 |

---

## 8. 流程图

### 8.1 完整生命周期流程

```
                              ┌─────────────────┐
                              │     开始        │
                              └────────┬────────┘
                                       │
                                       ▼
                        ┌──────────────────────────┐
                        │      需求阶段             │
                        │  (REQ → 需求文档)        │
                        └────────────┬─────────────┘
                                     │ 需求冻结
                                     ▼
                        ┌──────────────────────────┐
                        │      设计阶段             │
                        │  (DES → 架构设计)        │
                        └────────────┬─────────────┘
                                     │ 设计冻结
                                     ▼
                        ┌──────────────────────────┐
                        │      实现阶段             │
                        │  (CODE → PR审查通过)     │
                        └────────────┬─────────────┘
                                     │ 代码完成
                                     ▼
                        ┌──────────────────────────┐
                        │      测试阶段             │
                        │  (TEST → 测试通过)       │
                        └────────────┬─────────────┘
                                     │ 测试通过
                                     ▼
                        ┌──────────────────────────┐
                        │      验证阶段             │
                        │  (VERIFICATION → QA通)   │
                        └────────────┬─────────────┘
                                     │ 验证通过
                                     ▼
                        ┌──────────────────────────┐
                        │      发布阶段             │
                        │  (RELEASE → 发布批准)    │
                        └────────────┬─────────────┘
                                     │ 发布完成
                                     ▼
                        ┌──────────────────────────┐
                        │      运营阶段             │
                        │  (监控 → 问题处理)       │
                        └────────────┬─────────────┘
                                     │
                                     │ 重大变更
                                     ▼
                        ┌──────────────────────────┐
                        │      变更请求流程         │
                        └────────────┬─────────────┘
                                     │
                                     │ 生命周期结束
                                     ▼
                              ┌─────────────────┐
                              │      结束        │
                              └─────────────────┘
```

---

## 9. 参考文档

- GMP_STANDARD.md - GMP 规范总纲
- GMP_TEST_FRAMEWORK.md - 测试体系框架
- GMP_PROVABLE_SYSTEM.md - 可证明系统分析

---

## 10. 修订历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0.0 | 2026-05-14 | 初始版本 | GMP Agent |
