-- === SQLRustGo Table Maintenance Test Suite ===
-- Tests for ANALYZE, CHECK, OPTIMIZE, VACUUM commands

-- === SETUP ===
CREATE TABLE t1 (id INTEGER PRIMARY KEY, val TEXT);
INSERT INTO t1 VALUES (1, 'a'), (2, 'b'), (3, 'c');

CREATE TABLE t2 (id INTEGER, data TEXT);
INSERT INTO t2 VALUES (1, 'x'), (2, 'y'), (3, 'z');

-- === CASE: Analyze Table Statistics ===
-- EXPECT: success
ANALYZE TABLE t1;

-- === CASE: Check Table Integrity ===
-- EXPECT: success
CHECK TABLE t1;

-- === CASE: Check Table with Quick Option ===
-- EXPECT: success
CHECK TABLE t1 QUICK;

-- === CASE: Check Table with Extended Option ===
-- EXPECT: success
CHECK TABLE t1 EXTENDED;

-- === CASE: Optimize Table (Defragment) ===
-- EXPECT: success
OPTIMIZE TABLE t1;

-- === CASE: Vacuum Table (PostgreSQL style) ===
-- EXPECT: success
VACUUM TABLE t1;

-- === CASE: Vacuum Full ===
-- EXPECT: success
VACUUM TABLE t1 FULL;

-- === CASE: Vacuum Analyze ===
-- EXPECT: success
VACUUM ANALYZE t1;

-- === CASE: Repair Table ===
-- EXPECT: success
REPAIR TABLE t1;

-- === CASE: Multiple Table Maintenance ===
-- EXPECT: success
ANALYZE TABLE t1, t2;

-- === CASE: Table Statistics After Insert ===
-- EXPECT: 3 rows
INSERT INTO t1 VALUES (4, 'd');
SELECT COUNT(*) FROM t1 WHERE id > 3;

-- === TEARDOWN ===
DROP TABLE t1;
DROP TABLE t2;
