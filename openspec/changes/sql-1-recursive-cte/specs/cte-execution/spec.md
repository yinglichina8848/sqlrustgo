# CTE Execution

## MODIFIED Requirements

### Requirement: Non-recursive CTE materialization
The system SHALL materialize CTE definitions into temporary tables before executing the main query that references them.

#### Scenario: Single non-recursive CTE
- **WHEN** executing `WITH cte AS (SELECT 1) SELECT * FROM cte`
- **THEN** the CTE subquery is executed first, producing a temp table
- **AND** the main SELECT reads from that temp table

#### Scenario: Multiple CTEs with dependencies
- **WHEN** executing `WITH cte1 AS (...), cte2 AS (...) SELECT ... FROM cte1 JOIN cte2`
- **THEN** all CTEs are materialized in order
- **AND** the main SELECT can reference any CTE

### Requirement: CTE name resolution
The system SHALL allow CTE definitions to be referenced by name in the main query and in subsequent CTE definitions.

#### Scenario: CTE referenced in main query
- **WHEN** executing `WITH cte AS (SELECT 1 AS id) SELECT * FROM cte WHERE id = 1`
- **THEN** the CTE name `cte` resolves to the materialized CTE result
