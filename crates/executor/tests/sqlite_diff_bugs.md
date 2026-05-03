# SQLite Differential Testing — Known Bug Log
#
# Format:
#   SQL | SQLite | SQLRustGo | Bug Description
#
# When a bug is FIXED: remove the entry and add a proper test case.
# When a NEW bug is found: add an entry here.
#
# This file drives the regression tracking system.

## ACTIVE BUGS (unfixed)

# --- Parser ---

SELECT 1 AS a, 2 AS b
  → SQLite: "1\t2"
  → SQLRustGo: ParseError("Expected FROM or column name")
  → Bug: Scalar SELECT without FROM not supported
  → Severity: HIGH

SELECT 1 AS x UNION ALL SELECT 2 UNION ALL SELECT 3 ORDER BY x
  → SQLite: "1\n2\n3"
  → SQLRustGo: ParseError("Expected FROM or column name")
  → Bug: UNION ALL not supported
  → Severity: HIGH

# --- Executor: DISTINCT ---

SELECT DISTINCT x FROM (VALUES ...):
  → SQLite: distinct values only
  → SQLRustGo: all values (DISTINCT ignored)
  → Bug: DISTINCT keyword not implemented
  → Severity: HIGH

# --- Executor: WHERE filter ---

WHERE x = 10 with NULL rows:
  → SQLite: correctly filters to rows where x=10
  → SQLRustGo: returns wrong rows, includes non-matching
  → Bug: WHERE condition not applied correctly
  → Severity: HIGH

# --- Executor: GROUP BY ---

GROUP BY with COUNT/SUM aggregates:
  → SQLite: grouped aggregation result
  → SQLRustGo: empty result or wrong rows
  → Bug: GROUP BY not implemented
  → Severity: CRITICAL

# --- Executor: JOIN ---

INNER JOIN / LEFT JOIN:
  → SQLite: correct join result
  → SQLRustGo: empty result
  → Bug: JOIN execution broken
  → Severity: CRITICAL

# --- Executor: COUNT ---

COUNT(*) vs COUNT(column):
  → SQLite: correct counts
  → SQLRustGo: wrong result
  → Bug: aggregate execution broken
  → Severity: CRITICAL

# --- Known Intentional Differences (do NOT fix) ---

NULL = NULL → SQLite=0, SQL standard=NULL
  → Mark #[ignore] in test

ORDER BY x NULLS LAST → SQLRustGo may not support
  → Mark #[ignore] in test

IN (1,2,NULL) semantics → differs across engines
  → Mark #[ignore] in test
