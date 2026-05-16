## ADDED Requirements

### Requirement: MERGE Statement Implementation
The system SHALL support MERGE statement as defined in SQL:2003.

#### Scenario: MERGE WHEN MATCHED
- **WHEN** executing MERGE INTO target USING source ON condition WHEN MATCHED THEN UPDATE
- **THEN** matching rows are updated

#### Scenario: MERGE WHEN NOT MATCHED
- **WHEN** executing MERGE INTO target USING source ON condition WHEN NOT MATCHED THEN INSERT
- **THEN** non-matching rows are inserted

### Requirement: Event Scheduler
The system SHALL support scheduled event execution.

#### Scenario: Create event
- **WHEN** creating an event with schedule
- **THEN** event is stored in the system

#### Scenario: Event execution
- **WHEN** scheduled time arrives
- **THEN** event body is executed
