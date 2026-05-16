## Context

Performance Schema (PS) is a MySQL-compatible performance monitoring system. It collects runtime metrics about query execution, wait events, and resource usage. Existing infrastructure in `information_schema` provides basic metrics, but full PS tables are needed.

**Current State**:
- Basic PerformanceSchema struct exists in `crates/information-schema/src/performance_schema.rs`
- Provides transaction_history, lock_wait, recovery_history, wal_stats
- No setup_* tables or events_* tables

**Requirements**:
- MySQL-compatible Performance Schema table structure
- Configurable instrumentation via setup tables
- Statement and wait event recording
- Efficient data collection without overhead

## Goals / Non-Goals

**Goals:**
- Implement setup_actors and setup_instruments tables
- Implement events_statements_current/history/summary tables
- Implement events_waits_current/history/summary tables
- Query APIs for each table

**Non-Goals:**
- Full MySQL PS compatibility (all 100+ tables)
- Performance impact on production workloads
- Automatic tuning based on PS data

## Decisions

### Decision 1: Event Collection Strategy

**Option A: Continuous collection (chosen)**
- All events collected continuously
- Higher overhead but complete data

**Option B: Sampling
- Sample a percentage of events
- Lower overhead but incomplete

**Decision**: Continuous collection for Beta Gate - correctness over optimization.

### Decision 2: Data Retention

**Option A: Ring buffer (chosen)**
- Fixed-size buffers per event type
- Overwrites oldest when full

**Option B: Unlimited history
- Store all events
- Memory concerns for high-throughput

**Decision**: Ring buffer with configurable size.

### Decision 3: Table Implementation

**Option A: Separate tables per event type**
- Clean separation, follows MySQL
- More complex queries

**Option B: Single events table with type column
- Simpler schema
- Less MySQL-compatible

**Decision**: Separate tables following MySQL structure.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Performance overhead | Medium | Allow disabling via setup_instruments |
| Memory usage | Medium | Ring buffer size limits |
| Incomplete MySQL compatibility | Low | Document known differences |

## Open Questions

1. **Ring buffer size**: What default size? (Configurable)
2. **Event attribute granularity**: Full attribute set or minimal? (Minimal for v3.2.0)
3. **Consumer/filter tables**: Required? (Future enhancement)
