## ADDED Requirements

### Requirement: TPC-H SF=10 All 22 Queries
The system SHALL execute all 22 TPC-H queries at SF=10 without OOM.

#### Scenario: SF=10 Q1
- **WHEN** executing TPC-H Q1 at SF=10
- **THEN** query completes without OOM

#### Scenario: SF-10 Q22
- **WHEN** executing TPC-H Q22 at SF=10
- **THEN** query completes without OOM
