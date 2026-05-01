-- === SKIP ===

-- === Transaction Isolation Levels Test Suite ===

-- === CASE: READ UNCOMMITTED ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
BEGIN;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: READ COMMITTED ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
BEGIN;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: REPEATABLE READ ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
BEGIN;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: SERIALIZABLE ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
BEGIN;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: Immediate transaction ===
-- EXPECT: success
BEGIN IMMEDIATE;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: Exclusive transaction ===
-- EXPECT: success
BEGIN EXCLUSIVE;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: Deferred transaction ===
-- EXPECT: success
BEGIN DEFERRED;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: Transaction with savepoint ===
-- EXPECT: success
BEGIN;
SAVEPOINT sp1;
SELECT COUNT(*) FROM users;
ROLLBACK TO SAVEPOINT sp1;
COMMIT;

-- === CASE: Transaction with multiple savepoints ===
-- EXPECT: success
BEGIN;
SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (9001, 'TX1', 'tx1@test.com');
SAVEPOINT sp2;
INSERT INTO users (id, name, email) VALUES (9002, 'TX2', 'tx2@test.com');
ROLLBACK TO SAVEPOINT sp1;
COMMIT;

-- === CASE: Release savepoint ===
-- EXPECT: success
BEGIN;
SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (9003, 'TX3', 'tx3@test.com');
RELEASE SAVEPOINT sp1;
COMMIT;

-- === CASE: Readonly transaction ===
-- EXPECT: success
BEGIN READ ONLY;
SELECT COUNT(*) FROM users;
COMMIT;

-- === CASE: Read write transaction ===
-- EXPECT: success
BEGIN READ WRITE;
SELECT COUNT(*) FROM users;
COMMIT;
