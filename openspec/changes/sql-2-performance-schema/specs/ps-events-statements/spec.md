# Performance Schema Events Statements

## ADDED Requirements

### Requirement: events_statements_current table
The system SHALL provide events_statements_current to track currently executing statements.

#### Scenario: Query current statements
- **WHEN** user queries SELECT * FROM performance_schema.events_statements_current
- **THEN** the system returns rows for statements currently executing
- **AND** columns include: THREAD_ID, EVENT_ID, EVENT_NAME, SOURCE, TIMER_START, TIMER_END, LOCK_TIME, SQL_TEXT

### Requirement: events_statements_history table
The system SHALL provide events_statements_history to track completed statements.

#### Scenario: Query statement history
- **WHEN** user queries SELECT * FROM performance_schema.events_statements_history
- **THEN** the system returns recently completed statements
- **AND** rows are in ring buffer (overwrites oldest when full)

### Requirement: events_statements_summary_by_digest
The system SHALL provide statement digest aggregation for repeated queries.

#### Scenario: Query digest summary
- **WHEN** user queries SELECT * FROM performance_schema.events_statements_summary_by_digest
- **THEN** the system returns aggregated stats per query digest
- **AND** includes: SCHEMA_NAME, DIGEST, DIGEST_TEXT, COUNT_STAR, SUM_TIMER_WAIT
