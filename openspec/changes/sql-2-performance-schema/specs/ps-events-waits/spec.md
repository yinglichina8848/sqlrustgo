# Performance Schema Events Waits

## ADDED Requirements

### Requirement: events_waits_current table
The system SHALL provide events_waits_current to track currently executing wait events.

#### Scenario: Query current waits
- **WHEN** user queries SELECT * FROM performance_schema.events_waits_current
- **THEN** the system returns rows for waits currently in progress
- **AND** columns include: THREAD_ID, EVENT_ID, EVENT_NAME, SOURCE, TIMER_START, TIMER_END, OBJECT_TYPE, OBJECT_SCHEMA, OBJECT_NAME

### Requirement: events_waits_history table
The system SHALL provide events_waits_history to track completed wait events.

#### Scenario: Query wait history
- **WHEN** user queries SELECT * FROM performance_schema.events_waits_history
- **THEN** the system returns recently completed wait events
- **AND** rows are in ring buffer (overwrites oldest when full)
