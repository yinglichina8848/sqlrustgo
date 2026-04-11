# Parser 重构与测试覆盖率改进计划

## 目标

1. **提升性能** - 优化解析速度
2. **改善架构** - 模块化拆分，更清晰的设计
3. **提升测试覆盖率** - 从 45.76% 提升至 80%

## 约束

- 保持现有 SQL 语法支持不变
- 保持 API 兼容
- 优先单元测试

## 当前状态

| 文件 | 行数 | 覆盖率 |
|------|------|--------|
| parser.rs | 6466 | 45.76% |
| lexer.rs | ~1000 | 95.96% |
| token.rs | ~500 | 74.36% |

## 实施阶段

### 阶段 1: 提取 expression.rs (最高优先级)

**目标**: 将表达式解析逻辑分离到独立模块

**文件变更**:
```
parser/src/
├── lib.rs           # 更新重导出
├── lexer.rs         # 不变
├── token.rs         # 不变
├── parser.rs        # 移除 expression 解析逻辑 (~600行)
├── expression.rs    # 新文件: Expression 类型 + 解析器
└── error.rs        # 新文件: 统一错误类型
```

**关键内容提取**:
- `Expression` enum 定义 (parser.rs:597-645)
- `parse_expression()` 解析链:
  - `parse_or_expression()`
  - `parse_and_expression()`
  - `parse_comparison_expression()`
  - `parse_arithmetic_expression()`
  - `parse_primary_expression()`
  - `parse_case_when_expression()`
  - `parse_extract_expression()`
  - `parse_substring_expression()`
- `WindowFrameInfo`, `FrameBoundInfo`, `OrderByItem` 类型

**步骤**:
1. 创建 `expression.rs`，包含 `Expression` enum 和解析方法
2. 创建 `error.rs`，定义 `ParseError` 类型
3. 更新 `parser.rs` 使用新的 `expression.rs` 模块
4. 更新 `lib.rs` 重导出
5. 运行测试确保功能不变

**验收标准**:
- 表达式解析功能完全正常
- 现有测试全部通过
- Expression 模块可独立测试

---

### 阶段 2: 错误处理统一

**目标**: 提取统一的错误类型

**新增内容**:
```rust
// error.rs
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: Option<usize>,
    pub expected: Vec<String>,
    pub found: Option<Token>,
}
```

**收益**:
- 统一的错误格式
- 更容易测试错误情况
- 更好的错误信息

---

### 阶段 3: 提取 statement.rs

**目标**: 将语句解析分离

**文件变更**:
```
parser/src/
├── lib.rs
├── lexer.rs
├── token.rs
├── parser.rs        # 简化为调度器 (~300行)
├── expression.rs    # 来自阶段 1
├── statement.rs     # 新文件: Statement 类型
├── error.rs        # 来自阶段 2
└── tests/
    └── expression_test.rs  # 新测试文件
```

**关键内容提取**:
- `Statement` enum 定义
- 各种 Statement 类型定义
- `parse_statement()` 路由方法
- 各 `parse_xxx()` 方法

---

### 阶段 4: 测试基础设施改进

**目标**: 建立更好的测试模式

**新增测试策略**:

1. **表驱动测试** - 对表达式解析
```rust
#[test]
fn test_expression_parsing() {
    let cases = vec![
        ("1 + 2", Expression::BinaryOp(...)),
        ("a > b", Expression::BinaryOp(...)),
        // ...
    ];
    for (sql, expected) in cases {
        let result = parse_expression(sql);
        assert_eq!(result, expected);
    }
}
```

2. **错误路径测试** - 对边界情况
```rust
#[test]
fn test_parse_error_invalid_syntax() {
    let result = parse("SELECT FROM");
    assert!(result.is_err());
}
```

3. **Property-based 测试** - 对复杂表达式
- 使用 `quickcheck` 或 `proptest`

**覆盖率目标分解**:
| 模块 | 当前 | 目标 |
|------|------|------|
| expression.rs | N/A | 85% |
| statement.rs | N/A | 80% |
| parser.rs | 45% | 70% |
| lexer.rs | 96% | 96% |
| token.rs | 74% | 80% |

---

## 风险与缓解

| 风险 | 缓解措施 |
|------|----------|
| 重构破坏现有功能 | 每阶段完成后运行完整测试 |
| 覆盖率提升困难 | 通过表驱动测试批量覆盖边界情况 |
| 性能下降 | 使用 benchmarks 监控每阶段性能 |

## 时间估算

| 阶段 | 工作量 | 风险 |
|------|--------|------|
| 阶段 1: expression.rs | 高 | 中 |
| 阶段 2: error.rs | 低 | 低 |
| 阶段 3: statement.rs | 高 | 中 |
| 阶段 4: 测试改进 | 中 | 低 |

## 实施方式

**Subagent 执行模式**:
- 每个阶段由独立 subagent 执行
- 主 session 协调和审查
- 每阶段完成后代码审查

---

## 下一步

1. 确认设计文档
2. 使用 writing-plans skill 创建详细实施计划
3. 使用 subagent-driven-development 执行