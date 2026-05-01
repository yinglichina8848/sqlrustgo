# 第12周：性能优化与重构

> 实验时间：2学时（🕹️ 手动档第5周）
> 实验类型：设计性+验证性

---

## 一、实验目标

- [ ] 掌握性能分析方法
- [ ] 能够编写基准测试
- [ ] 能够识别和消除代码异味
- [ ] 能够使用AI辅助重构

---

## 二、实验环境

| 项目 | 要求 |
|------|------|
| Rust工具链 | 1.75+ |
| 项目代码 | SQLRustGo |

---

## 三、操作步骤

### 步骤1：编写基准测试（30分钟）

#### 1.1 创建基准测试文件

创建 `crates/executor/benches/executor_bench.rs`：

```rust
//! 执行器基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlrustgo_executor::*;

fn setup_test_database() -> Database {
    // 创建测试数据库
    Database::new(":memory:").unwrap()
}

fn bench_simple_select(c: &mut Criterion) {
    let db = setup_test_database();
    db.execute("CREATE TABLE users (id INT, name TEXT)").unwrap();
    db.execute("INSERT INTO users VALUES (1, 'test')").unwrap();

    c.bench_function("simple_select", |b| {
        b.iter(|| {
            db.execute(black_box("SELECT * FROM users WHERE id = 1"))
                .unwrap()
        })
    });
}

fn bench_where_clause(c: &mut Criterion) {
    let db = setup_test_database();
    // ... 插入测试数据

    c.bench_function("where_clause", |b| {
        b.iter(|| {
            db.execute(black_box("SELECT * FROM users WHERE age > 18"))
                .unwrap()
        })
    });
}

criterion_group!( benches, bench_simple_select, bench_where_clause );
criterion_main!(benches);
```

#### 1.2 运行基准测试

```bash
# 安装criterion
cargo install cargo-criterion

# 运行基准测试
cargo criterion
```

#### ✅ 检查点1：记录基准测试结果

---

### 步骤2：性能分析（25分钟）

#### 2.1 安装flamegraph

```bash
cargo install cargo-flamegraph
```

#### 2.2 生成火焰图

```bash
# 为sqlrustgo生成火焰图
cargo flamegraph --bin sqlrustgo -- "SELECT * FROM users WHERE id = 1"

# 或者使用具体测试
cargo flamegraph --test executor_tests -- "test_name"
```

#### 2.3 分析火焰图

打开生成的 `flamegraph.svg`，分析：

- 哪些函数占用最多时间？
- 是否有可以优化的热点代码？
- 内存分配是否过多？

#### ✅ 检查点2：保存火焰图和分析

---

### 步骤3：识别代码异味（20分钟）

#### 3.1 使用Clippy识别

```bash
# 运行Clippy
cargo clippy --all-features -- -W clippy::all
```

#### 3.2 使用AI识别代码问题

```
提示词示例：

分析以下SQLRustGo代码，识别代码异味：
[粘贴lexer.rs或parser.rs的代码]

请识别：
1. 重复代码
2. 过长函数
3. 命名问题
4. 错误处理问题
5. 建议的重构方案
```

#### 3.3 记录代码异味

| 位置 | 问题类型 | 描述 | 建议方案 |
|------|---------|------|---------|
| lexer.rs:100 | 重复代码 | token类型判断重复 | 提取方法 |
| parser.rs:200 | 过长函数 | parse_expression超过100行 | 拆分为多个函数 |

#### ✅ 检查点3：记录代码异味清单

---

### 步骤4：执行重构（25分钟）

#### 4.1 选择重构目标

从代码异味清单中选择一个进行重构。

#### 4.2 重构原则

- **小步重构**：每次只改一点
- **测试先行**：确保测试通过
- **频繁提交**：每个小改动都提交

#### 4.3 重构示例

```rust
// 重构前
fn parse_expression(&mut self) -> Result<Expr, Error> {
    let mut result = self.parse_primary()?;
    while self.check(TokenType::Plus) || self.check(TokenType::Minus) {
        let op = self.bump().unwrap();
        let right = self.parse_primary()?;
        result = match op.token_type {
            TokenType::Plus => Expr::Add(Box::new(result), Box::new(right)),
            TokenType::Minus => Expr::Sub(Box::new(result), Box::new(right)),
            _ => unreachable!(),
        };
    }
    Ok(result)
}

// 重构后
fn parse_expression(&mut self) -> Result<Expr, Error> {
    let mut result = self.parse_primary()?;
    while self.is_binary_operator() {
        let op = self.bump().unwrap();
        let right = self.parse_primary()?;
        result = self.create_binary_expr(result, op, right)?;
    }
    Ok(result)
}

fn is_binary_operator(&self) -> bool {
    matches!(self.current.token_type, TokenType::Plus | TokenType::Minus)
}

fn create_binary_expr(&self, left: Expr, op: Token, right: Expr) -> Result<Expr, Error> {
    match op.token_type {
        TokenType::Plus => Ok(Expr::Add(Box::new(left), Box::new(right))),
        TokenType::Minus => Ok(Expr::Sub(Box::new(left), Box::new(right))),
        _ => Err(Error::unexpected_token(op)),
    }
}
```

#### 4.4 验证重构

```bash
# 确保所有测试通过
cargo test

# 确保Clippy通过
cargo clippy --all-features -- -D warnings

# 确保格式化通过
cargo fmt --check --all
```

#### ✅ 检查点4：记录重构前后对比

---

## 四、实验报告

### 4.1 报告内容

1. **基准测试代码**
2. **性能分析报告**（火焰图）
3. **代码异味识别和修复记录**
4. **重构前后的对比**

### 4.2 提交方式

```bash
git checkout -b experiment/week-12-你的学号
mkdir -p reports/week-12
# 放入报告
git add reports/week-12/
git commit -m "experiment: submit week-12 report"
git push origin experiment/week-12-你的学号
```

---

## 五、思考与反思

### 🎯 本周能力阶段：Harness Engineering（规则约束工程）

**本周重点**：用规则约束AI的行为，确保代码质量

### 问题1：优化与改进
- 我是否开始学会用"明确的规则"来指导AI的代码生成？
- 如何平衡"让AI自由发挥"和"让AI遵守规则"？

### 问题2：AI不可替代的能力
- AI知道"这段代码在未来可能有问题"吗？（如性能隐患）
- 人类的"经验"和"直觉"在代码审查中起什么作用？

### 问题3：自我提升
- 我是否开始学会评估AI建议的"价值"，而不是照单全收？
- 我是否能区分"AI能帮我做的"和"必须我自己做的"？

---

## 六、评分标准

| 检查项 | 分值 |
|--------|------|
| 基准测试编写 | 20分 |
| 性能分析报告 | 20分 |
| 代码异味识别 | 20分 |
| 重构实践 | 25分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-04-19*
*🕹️ 手动档第5周 - 性能优化与重构*
