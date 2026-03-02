# AI-CLI 协同开发指南

> 版本：v1.0
> 日期：2026-03-02
> 目标：指导 AI-CLI 工具协同开发 SQLRustGo 2.0

---

## 一、概述

本文档定义了使用 AI-CLI 工具（如 TRAE、Claude、Cursor 等）协同开发 SQLRustGo 2.0 的规范和流程。

### 1.1 AI-CLI 角色

```
AI-CLI 角色:
├── 实现者 (Implementer)
│   └── 根据设计文档编写代码
│
├── 测试者 (Tester)
│   └── 编写单元测试和集成测试
│
├── 审查者 (Reviewer)
│   └── 代码审查和优化建议
│
└── 文档者 (Documenter)
    └── 编写和更新文档
```

### 1.2 适用场景

| 场景 | AI-CLI 能力 | 人工介入 |
|------|-------------|----------|
| 代码实现 | ✅ 完全自主 | 审查确认 |
| 测试编写 | ✅ 完全自主 | 运行验证 |
| 文档编写 | ✅ 完全自主 | 内容审核 |
| 架构设计 | ⚠️ 辅助建议 | 最终决策 |
| 需求分析 | ⚠️ 辅助分析 | 最终确认 |

---

## 二、开发流程

### 2.1 标准开发流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AI-CLI 开发流程                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 任务领取                                                                │
│      ├── 从 Issue 中获取任务详情                                             │
│      ├── 确认任务依赖关系                                                    │
│      └── 创建任务分支                                                        │
│                                                                              │
│   2. 代码实现                                                                │
│      ├── 阅读相关设计文档                                                    │
│      ├── 实现功能代码                                                        │
│      └── 编写单元测试                                                        │
│                                                                              │
│   3. 测试验证                                                                │
│      ├── 运行单元测试                                                        │
│      ├── 运行集成测试                                                        │
│      └── 性能基准测试（如适用）                                              │
│                                                                              │
│   4. 代码审查                                                                │
│      ├── AI-CLI 自我审查                                                     │
│      ├── 生成审查报告                                                        │
│      └── 人工审查确认                                                        │
│                                                                              │
│   5. 提交合并                                                                │
│      ├── 提交代码                                                            │
│      ├── 创建 Pull Request                                                   │
│      └── 合并到主分支                                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 任务分支命名规范

```
feature/<模块>-<功能>      # 新功能
fix/<模块>-<问题>          # Bug 修复
refactor/<模块>-<内容>     # 重构
docs/<内容>                # 文档更新
test/<模块>-<内容>         # 测试相关

示例:
feature/network-server
feature/executor-plugin
fix/network-connection-leak
refactor/logical-plan
docs/api-reference
```

---

## 三、任务执行规范

### 3.1 任务模板

每个任务应包含以下信息：

```markdown
## 任务: [TASK-ID] 任务名称

### 描述
简要描述任务目标

### 输入
- 设计文档链接
- 相关代码文件
- 依赖任务

### 输出
- 新增/修改的文件列表
- 测试文件
- 文档更新

### 验收标准
- [ ] 功能实现完成
- [ ] 单元测试通过
- [ ] 代码审查通过
- [ ] 文档已更新

### 预估时间
X 小时
```

### 3.2 代码实现规范

#### 3.2.1 代码风格

```rust
// 遵循 Rust 标准命名规范
// 函数名: snake_case
// 类型名: PascalCase
// 常量: SCREAMING_SNAKE_CASE
// 模块: snake_case

// 使用有意义的变量名
let connection_count = 0;  // 好
let cnt = 0;               // 不好

// 添加必要的注释
/// 计算两个数的和
/// 
/// # Arguments
/// * `a` - 第一个数
/// * `b` - 第二个数
/// 
/// # Returns
/// 两数之和
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

#### 3.2.2 错误处理

```rust
// 使用 thiserror 定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Authentication failed")]
    AuthFailed,
}

// 使用 Result 传播错误
pub fn connect(addr: &str) -> Result<Connection, NetworkError> {
    // ...
}
```

#### 3.2.3 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name_scenario() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "expected");
    }
    
    #[tokio::test]
    async fn test_async_function() {
        // 异步测试
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

---

## 四、提交规范

### 4.1 Commit Message 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Type 类型

| Type | 说明 |
|------|------|
| feat | 新功能 |
| fix | Bug 修复 |
| refactor | 重构 |
| docs | 文档更新 |
| test | 测试相关 |
| chore | 构建/工具相关 |

#### Scope 范围

| Scope | 说明 |
|-------|------|
| network | 网络模块 |
| executor | 执行引擎 |
| planner | 计划器 |
| storage | 存储引擎 |
| parser | 解析器 |

#### 示例

```
feat(network): add async server implementation

- Implement AsyncServer with Tokio
- Add connection handling
- Support multiple concurrent connections

Task: N-011
Issue: #86
```

### 4.2 PR 模板

```markdown
## 变更说明

简要描述本次变更的内容和目的。

## 变更类型

- [ ] 新功能 (feat)
- [ ] Bug 修复 (fix)
- [ ] 重构 (refactor)
- [ ] 文档更新 (docs)
- [ ] 测试相关 (test)

## 变更内容

- 变更点 1
- 变更点 2
- 变更点 3

## 测试

- [ ] 单元测试已通过
- [ ] 集成测试已通过
- [ ] 手动测试已完成

## 相关 Issue

Closes #XXX

## 任务 ID

Task: XXX-XXX
```

---

## 五、AI-CLI 指令模板

### 5.1 任务启动指令

```
请执行任务 N-001: 创建 server.rs 基础框架

设计文档: docs/v2.0/网络设计/NETWORK_ENHANCEMENT_PLAN.md
参考文件: src/network/mod.rs

要求:
1. 创建 src/bin/server.rs 文件
2. 实现命令行参数解析
3. 集成存储引擎初始化
4. 添加必要的错误处理

完成后:
- 运行 cargo build 确保编译通过
- 提交代码并注明 Task: N-001
```

### 5.2 代码审查指令

```
请审查以下代码变更:

文件: src/bin/server.rs

审查要点:
1. 代码风格是否符合规范
2. 错误处理是否完善
3. 是否有潜在的安全问题
4. 性能是否有优化空间

请生成审查报告，包括:
- 发现的问题
- 改进建议
- 总体评价
```

### 5.3 测试生成指令

```
请为以下代码生成测试:

文件: src/network/server.rs
函数: AsyncServer::handle_connection

测试场景:
1. 正常连接处理
2. 连接超时
3. 认证失败
4. 查询执行

要求:
- 使用 tokio::test 进行异步测试
- 覆盖正常和异常路径
- 添加必要的注释
```

---

## 六、质量检查清单

### 6.1 代码提交前检查

- [ ] 代码编译通过 (`cargo build`)
- [ ] 单元测试通过 (`cargo test`)
- [ ] 代码格式正确 (`cargo fmt --check`)
- [ ] 无 Clippy 警告 (`cargo clippy`)
- [ ] 文档已更新
- [ ] Commit Message 符合规范

### 6.2 PR 合并前检查

- [ ] 所有 CI 检查通过
- [ ] 代码审查已完成
- [ ] 测试覆盖率达标
- [ ] 文档已更新
- [ ] 变更日志已更新

---

## 七、常见问题

### Q1: AI-CLI 生成的代码不符合预期怎么办？

**解决方案**:
1. 提供更详细的需求描述
2. 提供参考代码示例
3. 分步骤实现，逐步验证
4. 人工介入修改

### Q2: 如何处理任务依赖？

**解决方案**:
1. 查看任务列表中的依赖关系
2. 按顺序完成任务
3. 如果依赖任务未完成，先完成依赖任务

### Q3: 测试失败如何处理？

**解决方案**:
1. 分析错误信息
2. 检查相关代码
3. 修复问题
4. 重新运行测试

---

## 八、附录

### 8.1 常用命令

```bash
# 编译
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# Clippy 检查
cargo clippy

# 运行服务器
cargo run --bin sqlrustgo-server

# 运行客户端
cargo run --bin sqlrustgo-client
```

### 8.2 文件结构

```
docs/v2.0/
├── SQLRUSTGO_2_0_ROADMAP.md      # 路线图
├── 网络设计/
│   └── NETWORK_ENHANCEMENT_PLAN.md
├── 架构设计/
│   ├── PLUGIN_ARCHITECTURE.md
│   └── ...
└── AI_CLI_GUIDE.md               # 本文档
```

---

*本文档由 TRAE (GLM-5.0) 创建*
