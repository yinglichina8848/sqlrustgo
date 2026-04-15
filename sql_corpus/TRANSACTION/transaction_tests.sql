-- SQLCorpus: Transaction Tests
-- Tests for transaction isolation and behavior

-- === SETUP ===
CREATE TABLE trans_test (id INTEGER PRIMARY KEY, value INTEGER);
INSERT INTO trans_test VALUES (1, 100), (2, 200);

-- === CASE: commit_rollback ===
BEGIN TRANSACTION;
UPDATE trans_test SET value = 999 WHERE id = 1;
SELECT value FROM trans_test WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: rollback ===
BEGIN TRANSACTION;
UPDATE trans_test SET value = 111 WHERE id = 1;
ROLLBACK;
SELECT value FROM trans_test WHERE id = 1;
-- EXPECT: 1 rows
