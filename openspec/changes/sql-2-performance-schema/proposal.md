## Why

Performance Schema provides runtime metrics for monitoring database performance, diagnosing bottlenecks, and understanding query execution characteristics. This enables production-grade performance monitoring and tuning for v3.2.0.

## What Changes

- **New**: `setup_actors` table for configuring monitoring filters
- **New**: `setup_instruments` table for enabling/disabling instrumentation
- **New**: `events_statements_*` tables for statement execution metrics
- **New**: `events_waits_*` tables for wait event metrics
- **New**: `global_events` table for aggregated event statistics
- **Modified**: PerformanceSchema struct with data collection and query APIs

## Capabilities

### New Capabilities

- `ps-setup-actors`: Configure which accounts are monitored by Performance Schema
- `ps-setup-instruments`: Enable/disable specific instrumentation points
- `ps-events-statements`: Record statement execution events with timing and resource usage
- `ps-events-waits`: Record wait events (lock, io, mutex) with timing
- `ps-global-events`: Aggregate and summarize events globally

### Modified Capabilities

- `performance-schema`: Existing PerformanceSchema struct will be extended with new tables and query methods

## Impact

- **Information Schema crate**: New tables in `crates/information-schema/src/performance_schema.rs`
- **Observability crate**: Enhanced event collection in `crates/observability/`
- **Tests**: New PS-specific tests in `tests/ps/`
