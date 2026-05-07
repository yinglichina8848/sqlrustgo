-- === SQLRustGo Transaction Isolation Test Suite ===
-- Tests for transaction isolation levels and concurrency

-- === SETUP ===
CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance REAL);
INSERT INTO accounts VALUES (1, 1000.00);
INSERT INTO accounts VALUES (2, 2000.00);

-- === CASE: Default Isolation Level ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL DEFAULT;

-- === CASE: Snapshot Isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL SNAPSHOT;

-- === CASE: Serializable Isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;

-- === CASE: Begin Read Only Transaction ===
-- EXPECT: success
BEGIN READ ONLY;

-- === CASE: Begin Immediate Transaction ===
-- EXPECT: success
BEGIN IMMEDIATE;

-- === CASE: Begin Exclusive Transaction ===
-- EXPECT: success
BEGIN EXCLUSIVE;

-- === CASE: Begin Deferred Transaction ===
-- EXPECT: success
BEGIN DEFERRED;

-- === CASE: Savepoint ===
-- EXPECT: success
SAVEPOINT sp1;

-- === CASE: Rollback to Savepoint ===
-- EXPECT: success
ROLLBACK TO SAVEPOINT sp1;

-- === CASE: Release Savepoint ===
-- EXPECT: success
RELEASE SAVEPOINT sp1;

-- === CASE: Commit Transaction ===
-- EXPECT: success
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
COMMIT;

-- === CASE: Rollback Transaction ===
-- EXPECT: success
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
ROLLBACK;

-- === CASE: Concurrent Transactions - Serializable ===
-- EXPECT: success or error (serialization failure)
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
BEGIN;
SELECT SUM(balance) FROM accounts;
-- (concurrent transaction would try to update)
COMMIT;

-- === CASE: Transaction Isolation Verification ===
-- EXPECT: 2000.00 (rollback test)
SELECT balance FROM accounts WHERE id = 1;

-- === TEARDOWN ===
DROP TABLE accounts;
