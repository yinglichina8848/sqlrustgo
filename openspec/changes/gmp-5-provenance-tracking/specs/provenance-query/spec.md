## ADDED Requirements

### Requirement: Query by Record ID
The system SHALL provide API to query provenance by record ID.

#### Scenario: Query provenance for specific record
- **WHEN** GET /api/provenance?record_id=12345 is called
- **THEN** system returns provenance records for record 12345

### Requirement: Query by Time Range
The system SHALL provide API to query provenance by time range.

#### Scenario: Query provenance in date range
- **WHEN** GET /api/provenance?from=2026-01-01&to=2026-01-31 is called
- **THEN** system returns provenance records created in that range

### Requirement: Query by Creator
The system SHALL provide API to query provenance by creator/responsible user.

#### Scenario: Query records created by user
- **WHEN** GET /api/provenance?creator_id=user123 is called
- **THEN** system returns all records created by user123

### Requirement: Lineage Traversal Query
The system SHALL provide API to query full lineage path.

#### Scenario: Query full lineage path
- **WHEN** GET /api/provenance/12345/lineage is called
- **THEN** system returns complete lineage path from origin to record 12345
