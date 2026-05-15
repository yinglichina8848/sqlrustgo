## ADDED Requirements

### Requirement: Point SELECT QPS ≥30K
The system SHALL achieve ≥30,000 QPS on point SELECT operations.

#### Scenario: Point select benchmark
- **WHEN** running sysbench point_select with 8 threads
- **THEN** QPS is at least 30,000

### Requirement: PERF-01 MySQL Protocol Flush Optimization
The system SHALL optimize MySQL protocol flush behavior.

#### Scenario: Protocol flush optimization
- **WHEN** handling MySQL client requests
- **THEN** flush operations are batched efficiently
