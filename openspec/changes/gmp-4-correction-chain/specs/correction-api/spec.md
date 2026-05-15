## ADDED Requirements

### Requirement: Create Correction API
The system SHALL provide an API to create correction records.

#### Scenario: Create correction via API
- **WHEN** POST /api/corrections is called with valid correction data and approvals
- **THEN** system creates correction record and returns correction_id

### Requirement: Query Corrections API
The system SHALL provide an API to query corrections for a record.

#### Scenario: Query corrections for original record
- **WHEN** GET /api/corrections?original_id=12345 is called
- **THEN** system returns list of correction records for id=12345

### Requirement: Verify Correction Chain API
The system SHALL provide an API to verify correction chain integrity.

#### Scenario: Verify chain via API
- **WHEN** GET /api/corrections/{original_id}/verify is called
- **THEN** system returns verification result with details
