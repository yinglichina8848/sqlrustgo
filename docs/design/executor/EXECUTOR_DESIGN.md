# Executor Design

## Iterator Model

The executor uses a **Volcano-style iterator model** (also known as the "pipeline" or "pull" model) for processing query results. This model provides a clean abstraction for composing query operators.

### VolcanoExecutor Trait

```rust
pub trait VolcanoExecutor: Send + Sync {
    fn open(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn close(&mut self) -> SqlResult<()>;
}
```

**Key characteristics:**
- **open()**: Initializes the executor and prepares to produce rows
- **next()**: Returns the next row, or `None` when exhausted
- **close()**: Releases resources

This design allows operators to be composed in a chain, where each operator pulls rows from its child operator. The model supports:
- Lazy evaluation (rows are only computed when needed)
- Pipeline breakers for operations that require all input (e.g., sorting)
- Resource management through explicit close() calls

### Batch Model (LocalExecutor)

In contrast to the Volcano pull model, the `LocalExecutor` uses a batch model where results are computed upfront and returned as a `Vec<Vec<Value>>`:

```rust
pub struct ExecutorResult {
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: usize,
}
```

The `LocalExecutorAdapter` bridges these two models by wrapping batch results into the Volcano interface.

## Execution Operators

### 1. Scan Operator

The **Scan operator** reads rows from a table (either from memory or disk storage).

```rust
pub trait ScanExecutor: Send {
    fn init(&mut self) -> SqlResult<()>;
    fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
    fn schema(&self) -> &Schema;
    fn name(&self) -> &str;
    fn close(&mut self) -> SqlResult<()>;
}
```

**Types of scans:**
- **Sequential Scan**: Full table scan reading all rows
- **Index Scan**: Uses an index to find matching rows

**Statistics tracked:**
```rust
pub struct ScanStats {
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub used_index: bool,
}
```

### 2. Filter Operator

The **Filter operator** applies a predicate to filter rows from its child operator.

```rust
pub struct FilterVolcanoExecutor {
    child: Box<dyn VolcanoExecutor>,
    predicate: Expr,
    schema: Schema,
    input_schema: Schema,
    initialized: bool,
}
```

**Behavior:**
- Evaluates the predicate expression for each row from the child
- Returns only rows where predicate evaluates to `true`
- Rows where predicate evaluates to `NULL` are excluded (unless predicate contains subquery)
- Short-circuits evaluation

### 3. Project Operator

The **Project operator** (selection) extracts specific columns from rows.

- Takes a schema describing the output columns
- Maps input rows to output rows by selecting specific column positions
- Used for SELECT column_list queries

### 4. Join Operator

The **Join operator** combines rows from two tables based on a join condition.

**Supported join types:**
- `INNER`: Only matching rows from both tables
- `LEFT`: All rows from left table, matching rows from right (NULL for non-matches)
- `RIGHT`: Matching rows from left, all rows from right table
- `FULL`: All rows from both tables
- `CROSS`: Cartesian product of both tables

**Join execution strategies:**
- **Nested Loop Join**: O(n*m) complexity, used for small tables
- **Hash Join**: Builds hash table on smaller table, probes with larger
- **Merge Join**: Requires sorted inputs on join keys

### 5. Aggregate Operator

The **Aggregate operator** performs aggregation computations.

```rust
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

pub struct AggregateCall {
    pub func: AggregateFunction,
    pub args: Vec<Expression>,
    pub distinct: bool,
}
```

**Supported aggregates:**
- COUNT, SUM, AVG, MIN, MAX
- DISTINCT modifier support (e.g., COUNT(DISTINCT col))
- GROUP BY with multiple columns
- HAVING clause support for filtering on aggregates

### 6. Sort Operator

The **Sort operator** orders rows based on ORDER BY expressions.

```rust
pub struct OrderByExpression {
    pub expression: Expression,
    pub ascending: bool,
    pub nulls_first: Option<bool>,
}
```

**Implementation considerations:**
- External sorting for datasets larger than memory
- Uses `Vec::sort()` for in-memory sorting
- Tracks sort statistics for optimization

## MemoryExecutionEngine

`MemoryExecutionEngine` is a type alias for `ExecutionEngine<MemoryStorage>`:

```rust
pub type MemoryExecutionEngine = ExecutionEngine<MemoryStorage>;
```

### Architecture

```
┌─────────────────────────────────────────────┐
│           ExecutionEngine<S: StorageEngine>  │
├─────────────────────────────────────────────┤
│  storage: Arc<RwLock<S>>                    │
│  catalog: Option<Arc<RwLock<Catalog>>>      │
│  stats: Arc<RwLock<ExecutionStats>>         │
│  cbo_enabled: bool                          │
│  transaction_manager: TransactionManager    │
│  query_cache: Arc<RwLock<QueryCache>>       │
└─────────────────────────────────────────────┘
```

### Key Features

1. **Cost-Based Optimization (CBO)**
   - Table-level statistics (row count, column statistics)
   - Selectivity estimation for predicates
   - Cost models for sequential scan, index scan, and joins
   - Join order optimization using greedy algorithm

2. **Query Cache**
   - Caches query results to avoid re-execution
   - Configurable cache size and TTL
   - Cache invalidation on table modifications

3. **Transaction Support**
   - Transaction manager for ACID properties
   - Isolation level support (READ_COMMITTED, SERIALIZABLE, etc.)
   - Savepoint support

4. **Catalog Integration**
   - Schema information storage
   - Stored procedure registration
   - Trigger management

### Execution Flow

```rust
pub fn execute(&mut self, sql: &str) -> SqlResult<ExecutorResult> {
    // 1. Check query cache
    // 2. Parse SQL into Statement AST
    // 3. Execute statement based on type
    // 4. Cache result if cacheable
    // 5. Return result
}
```

## DiskExecutionEngine

Disk-based execution extends the execution engine to handle data that exceeds memory capacity. While the primary repository shows `MemoryExecutionEngine` as the concrete implementation, the `ExecutionEngine<S: StorageEngine>` generic design allows for storage backends that may involve disk I/O.

### Design Principles

1. **Storage Abstraction**: The `StorageEngine` trait abstracts storage details:
   ```rust
   pub trait StorageEngine {
       fn scan(&self, table: &str) -> SqlResult<Vec<Vec<Value>>>;
       fn get_table_info(&self, table: &str) -> SqlResult<TableInfo>;
       // ... other methods
   }
   ```

2. **Memory Management**: When processing large datasets:
   - Results are streamed in batches rather than loaded entirely into memory
   - External sorting algorithms handle data larger than available memory
   - Buffer management minimizes disk I/O

3. **I/O Optimization**:
   - Sequential vs random I/O cost estimation
   - Prefetching strategies
   - Write coalescing for updates

### Execution Engine Statistics

```rust
pub struct ExecutionStats {
    pub table_stats: HashMap<String, TableStatistics>,
}

pub struct TableStatistics {
    pub row_count: u64,
    pub column_stats: HashMap<String, ColumnStatistics>,
}

pub struct ColumnStatistics {
    pub null_count: u64,
    pub distinct_count: u64,
    pub min_value: Option<SqlValue>,
    pub max_value: Option<SqlValue>,
}
```

These statistics enable the CBO to make informed decisions about:
- Sequential scan vs index scan selection
- Join method selection (hash, merge, nested loop)
- Join order optimization

## Cost-Based Optimization (CBO)

The execution engine includes cost-based optimization when `cbo_enabled` is true:

### Cost Models

1. **Sequential Scan Cost**:
   ```
   cost = row_count * 1.0
   ```

2. **Index Scan Cost**:
   ```
   cost = index_lookup_fixed_cost + (matching_rows * random_io_cost_per_row)
   ```

3. **Hash Join Cost**:
   ```
   cost = build_side_rows * 0.8 + probe_side_rows * 0.8
   ```

4. **Merge Join Cost**:
   ```
   cost = left_sort_cost + right_sort_cost + merge_cost
   ```

### Join Order Optimization

Uses a greedy algorithm that:
1. Starts with the smallest table
2. iteratively joins the remaining table with the lowest incremental join cost
3. Returns tables in optimal join order
