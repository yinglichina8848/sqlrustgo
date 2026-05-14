# MERGE Executor Implementation Design

**Date**: 2026-05-14
**Issue**: #900
**Status**: Approved

## Overview

Implement MERGE executor for SQLRustGo v3.1.0 GA target. MERGE combines INSERT + UPDATE + DELETE operations based on join conditions.

## Requirements Summary

| Requirement | Value |
|-------------|-------|
| SOURCE | Table name OR subquery |
| WHEN MATCHED | Multiple, with optional WHERE |
| WHEN NOT MATCHED | Multiple, with optional WHERE |
| Operations | UPDATE, INSERT, DELETE |
| Coverage Target | ≥90% |

## Architecture

**Approach**: Composition (组合实现)
- Reuse existing HashJoin, UpdateExec, InsertExec
- New MergeExecutor handles state machine orchestration

```
MERGE INTO target USING source ON condition
    WHEN MATCHED THEN UPDATE SET ... WHERE ...
    WHEN MATCHED THEN DELETE WHERE ...
    WHEN NOT MATCHED THEN INSERT (...) VALUES (...) WHERE ...
```

## Components

| Component | File | Responsibility |
|-----------|------|----------------|
| MergeExecutor | `executor/src/merge_executor.rs` (new) | State machine orchestration |
| DeleteExec | `executor/src/ddl_executor.rs` (add) | DELETE operation |

## State Machine

```
INIT → SOURCE_SCAN → HASH_BUILD → PROBE → (MATCHED|NOT_MATCHED)* → DONE
```

## Flow

1. **SOURCE_SCAN**: Execute source query/scan
2. **HASH_BUILD**: Build hash table from target table
3. **PROBE**: For each source row:
   - Find match in hash table via ON condition
   - MATCHED → execute matched_actions (in order)
   - NOT_MATCHED → execute not_matched_actions (in order)

## Files

| Action | File |
|--------|------|
| Create | `crates/executor/src/merge_executor.rs` |
| Modify | `crates/executor/src/lib.rs` |
| Modify | `crates/executor/src/ddl_executor.rs` |

## Testing

| Test | Description |
|------|-------------|
| MERGE-1 | Basic UPDATE on match |
| MERGE-2 | Basic INSERT on not match |
| MERGE-3 | UPDATE with WHERE |
| MERGE-4 | INSERT with WHERE |
| MERGE-5 | Multiple WHEN MATCHED |
| MERGE-6 | Subquery as SOURCE |
| MERGE-7 | Bulk merge (1000 rows) |
| MERGE-8 | DELETE operation |
| MERGE-9 | Concurrent MERGE |

## Verification

- [ ] MERGE INTO executes correctly
- [ ] Coverage ≥90%
- [ ] Integrated into Beta Gate
