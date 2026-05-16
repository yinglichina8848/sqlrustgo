# Performance Schema Setup Tables

## ADDED Requirements

### Requirement: setup_actors table
The system SHALL provide a setup_actors table to configure which user/host combinations are monitored.

#### Scenario: Query setup_actors
- **WHEN** user queries SELECT * FROM performance_schema.setup_actors
- **THEN** the system returns rows with columns: MID, NAME, ENABLED, HISTORY, PROPERTIES

#### Scenario: Default setup_actors
- **WHEN** Performance Schema is initialized
- **THEN** setup_actors contains default entries for all users ('%', '%')

### Requirement: setup_instruments table
The system SHALL provide a setup_instruments table to enable/disable instrumentation.

#### Scenario: Query setup_instruments
- **WHEN** user queries SELECT * FROM performance_schema.setup_instruments
- **THEN** the system returns rows with columns: NAME, ENABLED, TIMED, PROPERTIES

#### Scenario: Disable instrument
- **WHEN** user executes UPDATE performance_schema.setup_instruments SET ENABLED='NO' WHERE NAME='statement/sql/select'
- **THEN** SELECT statements are not instrumented
