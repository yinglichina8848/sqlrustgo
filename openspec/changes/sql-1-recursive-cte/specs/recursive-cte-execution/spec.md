# Recursive CTE Execution

## ADDED Requirements

### Requirement: Recursive CTE anchor member execution
The system SHALL execute the anchor member (first SELECT in UNION ALL) exactly once as the initial working set for a recursive CTE.

#### Scenario: Factorial recursive CTE anchor
- **WHEN** executing `WITH RECURSIVE factorial(n, fact) AS (SELECT 0, 1 UNION ALL SELECT n+1, (n+1)*fact FROM factorial WHERE n < 5) SELECT * FROM factorial`
- **THEN** the anchor member `SELECT 0, 1` is executed first
- **AND** produces row (0, 1)

#### Scenario: Fibonacci recursive CTE anchor
- **WHEN** executing `WITH RECURSIVE fib(a, b) AS (SELECT 0, 1 UNION ALL SELECT b, a+b FROM fib WHERE b < 50) SELECT a FROM fib`
- **THEN** the anchor member `SELECT 0, 1` is executed first
- **AND** produces row (0, 1)

### Requirement: Recursive CTE iterative execution
The system SHALL iteratively execute the recursive member (SELECT after UNION ALL that references the CTE by name), feeding the results of each iteration as input to the next iteration.

#### Scenario: Factorial iteration
- **WHEN** executing recursive CTE with working set containing row (n=0, fact=1)
- **THEN** the recursive member `SELECT n+1, (n+1)*fact FROM factorial WHERE n < 5` executes with that working set
- **AND** produces row (1, 1)
- **AND** iteration continues with new working set (0,1), (1,1)

#### Scenario: Fibonacci iteration
- **WHEN** executing recursive CTE with working set containing row (a=0, b=1)
- **THEN** the recursive member `SELECT b, a+b FROM fib WHERE b < 50` executes
- **AND** produces row (1, 1)
- **AND** next iteration uses (a=1, b=1) to produce (1, 2)

### Requirement: Recursive CTE termination
The system SHALL terminate recursive CTE execution when the recursive member produces no rows.

#### Scenario: Termination by limit clause
- **WHEN** executing `WITH RECURSIVE factorial(n, fact) AS (SELECT 0, 1 UNION ALL SELECT n+1, (n+1)*fact FROM factorial WHERE n < 5) SELECT * FROM factorial`
- **THEN** iteration stops when `n < 5` evaluates to false
- **AND** no more rows are produced

#### Scenario: Termination by empty result
- **WHEN** executing `WITH RECURSIVE fib(a, b) AS (SELECT 0, 1 UNION ALL SELECT b, a+b FROM fib WHERE b < 50) SELECT a FROM fib`
- **THEN** iteration stops when `b < 50` evaluates to false
- **AND** final result is all rows produced across all iterations

### Requirement: Recursive CTE UNION ALL aggregation
The system SHALL aggregate all rows produced by the anchor member and all recursive iterations using UNION ALL semantics.

#### Scenario: UNION ALL accumulates rows
- **WHEN** executing `WITH RECURSIVE counter(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM counter WHERE n < 3) SELECT * FROM counter`
- **THEN** anchor produces: (1)
- **AND** iteration 1 produces: (2)
- **AND** iteration 2 produces: (3)
- **AND** final result is rows: (1), (2), (3)

### Requirement: Recursive CTE max iteration safety
The system SHALL enforce a maximum iteration limit (default 1000) to prevent infinite loops.

#### Scenario: Max iterations reached
- **WHEN** recursive CTE would iterate indefinitely
- **THEN** system stops after 1000 iterations
- **AND** returns all rows produced before limit

### Requirement: Recursive CTE column type inheritance
The system SHALL use column names and types from the anchor member for the CTE result set.

#### Scenario: Column names from anchor
- **WHEN** executing `WITH RECURSIVE fib(a, b) AS (SELECT 0, 1 UNION ALL SELECT b, a+b FROM fib WHERE b < 50) SELECT a FROM fib`
- **THEN** column names `a` and `b` are taken from anchor SELECT
