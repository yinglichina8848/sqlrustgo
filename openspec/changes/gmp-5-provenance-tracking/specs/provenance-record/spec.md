## ADDED Requirements

### Requirement: Record Provenance Creation
The system SHALL create a ProvenanceRecord when data is created or modified.

#### Scenario: Create provenance for new record
- **WHEN** a new record is inserted
- **THEN** system creates ProvenanceRecord with source_type=INSERT and lineage_path pointing to new record

### Requirement: Provenance Record Fields
Each ProvenanceRecord SHALL contain: source_id, lineage_path, creator_id, create_time, operation_type.

#### Scenario: Verify provenance record completeness
- **WHEN** a provenance record is retrieved
- **THEN** all required fields are present and valid

### Requirement: Source Tracking
The system SHALL track the source of imported or external data.

#### Scenario: Track imported data source
- **WHEN** data is imported from batch #12345
- **THEN** provenance record has source_type=IMPORT and source_batch_id=12345
