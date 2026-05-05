# Parser Design

## Overview

The parser module (`sqlrustgo-parser`) converts raw SQL text into an Abstract Syntax Tree (AST) that represents the semantic structure of the query. It uses a **recursive descent parsing** approach with a separate **lexer/tokenizer** phase.

```
SQL Text -> Lexer (Tokenizer) -> Tokens -> Parser -> AST (Statement)
```

## Lexer/Tokenizer

### Lexer Structure

```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}
```

The lexer performs character-by-character scanning to convert SQL text into a sequence of tokens.

### Token Types

```rust
pub enum Token {
    // Keywords
    Select, From, Where, Insert, Into, Values, Update, Set, Delete,
    Create, Table, Drop, Alter, Index, Join, Left, Right, Inner, Outer,
    Group, By, Having, Order, Limit, Offset, Distinct, Union, And, Or, Not,
    // ... many more
    
    // Literals
    StringLiteral(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
    Null,
    
    // Identifiers and operators
    Identifier(String),
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    Plus, Minus, Star, Slash, Percent,
    // ... and more
}
```

### Lexer Implementation

The lexer processes input with the following strategies:

1. **Whitespace Skipping**: Continuously skips whitespace characters
2. **Keyword Recognition**: Checks if alphanumeric sequences match SQL keywords (case-insensitive)
3. **String Handling**: Properly handles single-quoted strings with escape sequences (`''`)
4. **Number Parsing**: Supports both integers and floating-point numbers
5. **Multi-character Operators**: Recognizes `>=`, `<=`, `<>`, `!=`

### Key Methods

```rust
impl Lexer {
    pub fn new(input: &'a str) -> Self;
    pub fn next_token(&mut self) -> Token;
    pub fn tokenize(&mut self) -> Vec<Token>;
}
```

### Usage Example

```rust
let sql = "SELECT id, name FROM users WHERE id = 1";
let tokens = Lexer::new(sql).tokenize();
// Returns: [Select, Identifier("id"), Comma, Identifier("name"), 
//           From, Identifier("users"), Where, Identifier("id"), 
//           Equal, NumberLiteral("1"), Eof]
```

## AST (Abstract Syntax Tree)

### Statement Hierarchy

The parsed SQL is represented as a `Statement` enum with variants for each SQL statement type:

```rust
pub enum Statement {
    Select(SelectStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    DropTable(DropTableStatement),
    CreateIndex(CreateIndexStatement),
    CreateView(CreateViewStatement),
    CreateTrigger(CreateTriggerStatement),
    CreateProcedure(CreateProcedureStatement),
    AlterTable(AlterTableStatement),
    Call(CallStatement),
    Union(UnionStatement),
    Transaction(TransactionStatement),
    Grant(GrantStatement),
    Revoke(RevokeStatement),
    // ... and more
}
```

### Core AST Nodes

#### SelectStatement

```rust
pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,
    pub table: String,                    // Backward compatibility
    pub from: Option<FromClause>,         // Full FROM clause
    pub where_clause: Option<Expression>,
    pub join_clause: Option<JoinClause>,
    pub aggregates: Vec<AggregateCall>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderByExpression>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub distinct: bool,
}
```

#### FromClause

```rust
pub struct FromClause {
    pub tables: Vec<FromTable>,          // Comma-separated tables
    pub join_clauses: Vec<JoinClause>,    // Explicit JOINs
}

pub struct FromTable {
    pub name: String,
    pub alias: Option<String>,
}
```

#### JoinClause

```rust
pub enum JoinType {
    Inner, Left, Right, Full, Cross,
}

pub struct JoinClause {
    pub join_type: JoinType,
    pub table: String,
    pub on_clause: Expression,
}
```

#### Expression

```rust
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    UnaryOp(String, Box<Expression>),
    Like(Box<Expression>, Box<Expression>, Option<char>),
    Between(Box<Expression>, Box<Expression>, Box<Expression>),
    In(Box<Expression>, Box<SelectStatement>),
    InList(Box<Expression>, Vec<Expression>),
    Exists(Box<SelectStatement>),
    Aggregate(AggregateCall),
    FunctionCall(String, Vec<Expression>),
    WindowCall(WindowCall),
    CaseWhen(Vec<WhenClause>, Option<Box<Expression>>),
    // ... more variants
}
```

#### Other Statement Types

```rust
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,
    pub select: Option<Box<SelectStatement>>,
    pub is_replace: bool,
    pub on_duplicate_key_update: Option<Vec<(String, Expression)>>,
}

pub struct UpdateStatement {
    pub table: String,
    pub set_clauses: Vec<(String, Expression)>,
    pub where_clause: Option<Expression>,
}

pub struct DeleteStatement {
    pub table: String,
    pub where_clause: Option<Expression>,
}

pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub constraints: Vec<TableConstraint>,
    pub if_not_exists: bool,
}

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
    pub default_value: Option<String>,
    pub references: Option<ForeignKeyRef>,
}
```

## SQL-92 Support

The parser implements support for the SQL-92 standard with extensions for MySQL compatibility.

### Supported SQL-92 Features

#### Data Types
- INTEGER, INT
- TEXT, VARCHAR, CHAR
- FLOAT, DOUBLE, REAL, DECIMAL, NUMERIC
- BOOLEAN, BOOL
- BLOB
- NULL

#### Query Features
- **SELECT**: Full projection, selection, joins
- **WHERE**: Comparison operators, AND, OR, NOT, BETWEEN, LIKE, IN, EXISTS
- **GROUP BY**: With HAVING clause
- **ORDER BY**: ASC/DESC, NULLS FIRST/LAST
- **LIMIT/OFFSET**: Pagination
- **DISTINCT**: Duplicate elimination
- **UNION**: Set operations (UNION, UNION ALL)

#### Joins
- INNER JOIN
- LEFT [OUTER] JOIN
- RIGHT [OUTER] JOIN
- FULL [OUTER] JOIN
- CROSS JOIN
- NATURAL JOIN

#### Data Manipulation
- INSERT INTO ... VALUES ...
- INSERT INTO ... SELECT ...
- UPDATE ... SET ... WHERE ...
- DELETE FROM ... WHERE ...
- REPLACE INTO ... (MySQL extension)

#### Data Definition
- CREATE TABLE
- DROP TABLE
- CREATE INDEX
- DROP INDEX
- ALTER TABLE (ADD COLUMN, DROP COLUMN, etc.)
- CREATE VIEW
- DROP VIEW

#### Transactions
- BEGIN / BEGIN WORK
- COMMIT / COMMIT WORK
- ROLLBACK / ROLLBACK WORK
- START TRANSACTION
- SAVEPOINT
- SET TRANSACTION ISOLATION LEVEL

#### Security
- GRANT
- REVOKE
- CREATE ROLE
- DROP ROLE
- GRANT ROLE
- REVOKE ROLE
- SET ROLE

#### Other
- Common Table Expressions (CTE) with WITH
- Stored Procedures (CREATE PROCEDURE, CALL)
- Triggers (CREATE TRIGGER)
- ANALYZE (statistics collection)

### Parser Structure

```rust
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self;
    pub fn parse_statement(&mut self) -> Result<Statement, String>;
    
    // Statement-specific parsers
    fn parse_select(&mut self) -> Result<Statement, String>;
    fn parse_insert(&mut self) -> Result<Statement, String>;
    fn parse_update(&mut self) -> Result<Statement, String>;
    fn parse_delete(&mut self) -> Result<Statement, String>;
    fn parse_create(&mut self) -> Result<Statement, String>;
    // ... more parsers
}
```

### Top-Level Parse Entry Point

```rust
pub fn parse(sql: &str) -> Result<Statement, ParseError> {
    let tokens = Lexer::new(sql).tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}
```

## Expression Parsing

The parser handles complex expressions with the following precedence (from lowest to highest):

1. **OR**
2. **AND**
3. **NOT**
4. **Comparison**: `=`, `!=`, `<>`, `<`, `>`, `<=`, `>=`, `IS`, `IS NOT`, `IN`, `BETWEEN`
5. **Addition**: `+`, `-`
6. **Multiplication**: `*`, `/`, `%`
7. **Unary**: `-` (negation), `NOT`
8. **Primary**: literals, identifiers, function calls, subqueries

### Expression Parsing Methods

```rust
impl Parser {
    fn parse_expression(&mut self) -> Result<Expression, String>;
    fn parse_or_expression(&mut self) -> Result<Expression, String>;
    fn parse_and_expression(&mut self) -> Result<Expression, String>;
    fn parse_not_expression(&mut self) -> Result<Expression, String>;
    fn parse_comparison(&mut self) -> Result<Expression, String>;
    fn parse_additive(&mut self) -> Result<Expression, String>;
    fn parse_multiplicative(&mut self) -> Result<Expression, String>;
    fn parse_unary(&mut self) -> Result<Expression, String>;
    fn parse_primary(&mut self) -> Result<Expression, String>;
}
```

### Aggregate Parsing

Aggregates are parsed with the following structure:

```rust
pub enum AggregateFunction {
    Count, Sum, Avg, Min, Max,
}

pub struct AggregateCall {
    pub func: AggregateFunction,
    pub args: Vec<Expression>,
    pub distinct: bool,
}

// Example: COUNT(DISTINCT col) parses as:
// AggregateCall { func: Count, args: [Identifier("col")], distinct: true }
```

### Window Function Support

```rust
pub struct WindowSpecification {
    pub partition_by: Vec<Expression>,
    pub order_by: Vec<(Expression, bool)>,
}

pub struct WindowCall {
    pub func_name: String,
    pub args: Vec<Expression>,
    pub window_spec: WindowSpecification,
}

// Example: ROW_NUMBER() OVER (PARTITION BY col ORDER BY id)
```

### Subquery Support

Expressions can contain subqueries:

```rust
// SELECT * FROM t WHERE id IN (SELECT id FROM t2)
Expression::In(
    Box::new(Identifier("id".to_string())),
    Box::new(SelectStatement { ... })
)

// SELECT * FROM t WHERE EXISTS (SELECT 1 FROM t2 WHERE t.id = t2.id)
Expression::Exists(Box::new(SelectStatement { ... }))
```

### CASE Expression

```rust
// CASE WHEN cond1 THEN val1 WHEN cond2 THEN val2 ELSE val3 END
Expression::CaseWhen(
    vec![
        WhenClause { condition: cond1, result: val1 },
        WhenClause { condition: cond2, result: val2 },
    ],
    Some(Box::new(val3))  // ELSE clause
)
```

## Error Handling

The parser returns descriptive error messages:

```rust
fn expect(&mut self, expected: Token) -> Result<Token, String> {
    match self.current() {
        Some(t) if t == &expected => Ok(self.next().unwrap()),
        Some(t) => Err(format!("Expected {:?}, got {:?}", expected, t)),
        None => Err("Unexpected end of input".to_string()),
    }
}
```

## Performance Considerations

1. **Tokenization**: Single pass through input, O(n) where n = input length
2. **Parsing**: Recursive descent with no backtracking for most constructs, O(n)
3. **Memory**: Tokens and AST nodes are owned values; no heap allocation for small constructs
4. **Error Recovery**: Minimal - first error stops parsing
