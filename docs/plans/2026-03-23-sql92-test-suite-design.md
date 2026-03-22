# SQL-92 符合度测试套件设计

## 一、设计目标

- 全面覆盖：涵盖 SQL-92 标准的核心特性（数据类型、DDL、DML、查询、约束、事务、视图等）
- 自动化执行：支持一键运行所有测试，生成报告
- 结果可验证：通过与预期输出对比，精确判断每个特性的支持状态
- 可扩展：易于添加新测试用例，适应后续版本迭代
- CI 集成：可在 GitHub Actions 等 CI 环境中自动运行，确保回归

---

## 二、测试套件架构

```
test/sql92/
├── Cargo.toml                 # 独立测试项目
├── runner/
│   ├── main.rs               # 主入口：解析命令行，执行测试
│   ├── executor.rs            # SQL 执行器（连接数据库，执行 SQL）
│   ├── comparator.rs         # 结果比较器（支持精确/近似匹配）
│   └── reporter.rs            # 报告生成器（Markdown/JSON）
├── cases/                    # 测试用例目录
│   ├── ddl/                  # DDL 测试
│   ├── dml/                  # DML 测试
│   ├── queries/              # 查询测试
│   ├── types/                # 数据类型测试
│   ├── constraints/          # 约束测试
│   ├── transactions/         # 事务测试
│   ├── views/                # 视图测试
│   └── functions/           # 函数测试
├── utils/                    # 辅助工具
│   └── schema.sql            # 公共数据定义
└── README.md                 # 使用说明
```

---

## 三、测试用例格式

每个测试用例由两个文件组成：

1. `<name>.sql` - SQL 语句
2. `<name>.expected` - 预期输出

示例：`ddl/create_table.sql`
```sql
CREATE TABLE person (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    birth DATE
);
INSERT INTO person VALUES (1, 'Alice', '1990-01-01');
SELECT * FROM person;
```

预期输出 `create_table.expected`：
```
1 Alice 1990-01-01
```

---

## 四、测试执行器设计

### 4.1 运行方式

```bash
cd test/sql92
cargo run -- --db-url "sqlrustgo://localhost:3306" --output report.md
```

### 4.2 执行器模块 (executor.rs)

```rust
use sqlx::{Connection, Executor, Row};

pub struct SqlExecutor {
    connection: Box<dyn Connection>,
}

impl SqlExecutor {
    pub async fn new(url: &str) -> Result<Self> {
        let conn = sqlx::any::AnyConnection::connect(url).await?;
        Ok(Self { connection: Box::new(conn) })
    }

    pub async fn execute_file(&mut self, path: &PathBuf) -> Result<Vec<Vec<Vec<String>>>> {
        let content = std::fs::read_to_string(path)?;
        let statements = self.split_statements(&content);
        let mut all_results = Vec::new();
        for stmt in statements {
            if stmt.trim().is_empty() { continue; }
            let result = self.execute_sql(stmt).await?;
            all_results.push(result);
        }
        Ok(all_results)
    }

    async fn execute_sql(&mut self, sql: &str) -> Result<Vec<Vec<String>>> {
        let rows = self.connection.fetch_all(sql).await?;
        let mut result = Vec::new();
        for row in rows {
            let mut row_data = Vec::new();
            for i in 0..row.len() {
                row_data.push(row.get(i).unwrap_or_else(|| "NULL".to_string()));
            }
            result.push(row_data);
        }
        Ok(result)
    }

    fn split_statements(&self, content: &str) -> Vec<String> { ... }
}
```

### 4.3 结果比较器 (comparator.rs)

```rust
#[derive(Clone)]
pub enum CompareMode {
    Exact,
    Approximate(f64),  // 相对误差
}

pub struct ResultComparator;

impl ResultComparator {
    pub fn compare(expected: &str, actual: &Vec<Vec<String>>, mode: CompareMode) -> bool {
        let expected_rows = Self::parse_expected(expected);
        if expected_rows.len() != actual.len() { return false; }
        
        for (i, exp_row) in expected_rows.iter().enumerate() {
            let act_row = &actual[i];
            if exp_row.len() != act_row.len() { return false; }
            for (j, exp_val) in exp_row.iter().enumerate() {
                if !Self::value_equal(exp_val, &act_row[j], &mode) {
                    return false;
                }
            }
        }
        true
    }

    fn value_equal(exp: &str, act: &str, mode: &CompareMode) -> bool {
        match mode {
            CompareMode::Exact => exp == act,
            CompareMode::Approximate(tol) => {
                let exp_num: f64 = exp.parse().unwrap_or(f64::NAN);
                let act_num: f64 = act.parse().unwrap_or(f64::NAN);
                if exp_num.is_nan() || act_num.is_nan() {
                    exp == act
                } else {
                    (exp_num - act_num).abs() <= exp_num.abs() * tol
                }
            }
        }
    }

    fn parse_expected(content: &str) -> Vec<Vec<String>> { ... }
}
```

---

## 五、报告生成

### 5.1 报告格式

生成 Markdown 文件，包含：
- 测试环境信息（数据库版本、时间）
- 按类别统计（通过/失败/跳过）
- 每个测试用例的详细结果（含错误信息）

### 5.2 报告生成器 (reporter.rs)

```rust
pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub details: Vec<TestCaseResult>,
}

pub struct TestCaseResult {
    pub name: String,
    pub category: String,
    pub status: Status,
    pub message: Option<String>,
}

pub enum Status {
    Passed,
    Failed,
    Skipped,
}

impl TestReport {
    pub fn to_markdown(&self) -> String { ... }
    pub fn to_json(&self) -> String { ... }
}
```

---

## 六、测试用例设计

### 6.1 分类与优先级

| 类别 | 子类 | 优先级 | 用例数量 |
|------|------|--------|----------|
| 数据类型 | INTEGER, DECIMAL, VARCHAR, DATE, TIMESTAMP, BOOLEAN, JSON | P0 | 15 |
| DDL | CREATE/DROP TABLE, ALTER TABLE, CREATE/DROP INDEX, CREATE/DROP VIEW | P0 | 12 |
| DML | INSERT, UPDATE, DELETE | P0 | 6 |
| 查询 | SELECT, WHERE, GROUP BY, HAVING, ORDER BY, LIMIT/OFFSET | P0 | 10 |
| 连接 | INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL JOIN | P1 | 8 |
| 子查询 | 标量子查询, 行子查询, 表子查询, IN/EXISTS | P0 | 6 |
| 集合操作 | UNION, INTERSECT, EXCEPT | P1 | 3 |
| 约束 | PRIMARY KEY, UNIQUE, NOT NULL, CHECK, FOREIGN KEY | P0 | 6 |
| 事务 | BEGIN, COMMIT, ROLLBACK, SAVEPOINT | P0 | 5 |
| 视图 | CREATE VIEW, 查询视图, 更新视图 | P1 | 4 |
| 函数 | 聚合, 字符串, 时间, 数学 | P0 | 12 |
| 存储过程 | CREATE PROCEDURE, CALL, IN/OUT 参数, 控制流 | P1 | 8 |
| 触发器 | CREATE TRIGGER, BEFORE/AFTER, NEW/OLD | P1 | 6 |

**总计约 100+ 个测试用例**

### 6.2 公共数据集

```sql
CREATE TABLE employee (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100),
    salary DECIMAL(10,2),
    dept_id INTEGER,
    hire_date DATE
);

CREATE TABLE department (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100)
);

INSERT INTO department VALUES (1, 'HR'), (2, 'IT');
INSERT INTO employee VALUES (1, 'Alice', 5000.00, 1, '2020-01-01');
INSERT INTO employee VALUES (2, 'Bob', 6000.00, 2, '2019-05-15');
```

---

## 七、CI 集成

```yaml
sql92-compliance:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
    - name: Build test runner
      working-directory: test/sql92
      run: cargo build --release
    - name: Start SQLRustGo server
      run: cargo run --bin sqlrustgo-server -- --port 3306 &
    - name: Run SQL-92 tests
      working-directory: test/sql92
      run: cargo run --release -- --output report.md
    - name: Upload report
      uses: actions/upload-artifact@v4
      with:
        name: sql92-compliance-report
        path: test/sql92/report.md
    - name: Check test success
      run: |
        if grep -q "Failed" test/sql92/report.md; then
          exit 1
        fi
```

---

## 八、可维护性设计

- 测试用例命名规范：`<category>/<subcategory>/<feature>.sql`
- 结果快照更新：提供 `--update-expected` 选项
- 选择性运行：支持按类别或文件名过滤（`--filter ddl/`）
- 并行执行：使用 rayon 并行执行独立测试用例
- 日志记录：每个测试用例的 SQL 和输出记录到临时文件

---

## 九、扩展性

- 新特性测试：只需添加新的 `.sql` 和 `.expected` 文件
- 新数据库支持：测试执行器使用 sqlx 的 Any 连接，可轻松切换后端
- 自定义比较器：可通过配置文件指定特定测试用例的比较模式

---

## 十、示例运行输出

```
SQL-92 Compliance Test Suite
Running 102 tests...

[PASS] ddl/create_table
[PASS] ddl/alter_table
[FAIL] queries/left_join
  - Expected rows: 3, got 2
  - Missing row: "2 Bob IT"
[PASS] types/decimal
...

Summary:
  Passed: 99
  Failed: 3
  Skipped: 0
  Pass rate: 97.06%

Report written to report.md
```

---

## 十一、总结

本测试套件设计以自动化、可扩展、易维护为核心，通过标准化测试用例与结果验证，能够精确衡量 SQLRustGo 对 SQL-92 的符合度。它既可作为版本发布的质量门禁，也能引导开发团队补齐缺失特性。随着 SQLRustGo 的演进，该套件将不断扩充，确保未来版本的功能稳定性。
