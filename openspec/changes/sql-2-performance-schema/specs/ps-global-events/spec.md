# Performance Schema Global Events

## ADDED Requirements

### Requirement: global_events table
The system SHALL provide a global_events table for aggregated event statistics.

#### Scenario: Query global events
- **WHEN** user queries SELECT * FROM performance_schema.global_events
- **THEN** the system returns aggregated event counts across all threads
- **AND** columns include: EVENT_NAME, COUNT_STAR, SUM_TIMER_WAIT, MIN_TIMER_WAIT, AVG_TIMER_WAIT, MAX_TIMER_WAIT

### Requirement: Global event aggregation
The system SHALL aggregate events from all threads into global counters.

#### Scenario: Event aggregation
- **WHEN** a statement completes execution
- **THEN** the system updates global_events with the event statistics
- **AND** increments COUNT_STAR and adds to SUM_TIMER_WAIT
