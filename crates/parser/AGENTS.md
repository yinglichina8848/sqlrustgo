# crates/parser - SQL Parser

> SQL 解析器。词法分析 → Token → AST。

## 核心文件

| 文件 | 作用 |
|------|------|
| `src/lexer.rs` | 词法分析 (16k 行) |
| `src/parser.rs` | 解析器 (5.5k 行，最大文件) |
| `src/token.rs` | Token 定义 (26k 行) |

## 常用命令

```bash
# 测试 parser
cargo test -p sqlrustgo-parser

# 单 crate 构建
cargo build -p sqlrustgo-parser --all-features

# 运行词法/语法测试
cargo test --package sqlrustgo-parser token
cargo test --package sqlrustgo-parser parser
```

## 约定

- Token 定义使用枚举，所有 SQL 关键字
- Parser 生成 AST 节点，定义在 `sqlrustgo-types` crate
- 词法错误 → `LexError`，语法错误 → `ParseError`
