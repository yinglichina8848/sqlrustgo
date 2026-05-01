-- === SKIP ===

-- === Complex Transaction Test Suite ===

-- === CASE: Basic transaction with COMMIT ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (100, 'Test User', 'test@example.com');
COMMIT;

-- === CASE: Basic transaction with ROLLBACK ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (101, 'Rollback User', 'rollback@example.com');
ROLLBACK;

-- === CASE: Transaction with SAVEPOINT ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (102, 'Savepoint User', 'savepoint@example.com');
SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (103, 'Savepoint User 2', 'savepoint2@example.com');
ROLLBACK TO SAVEPOINT sp1;
COMMIT;

-- === CASE: Read committed isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
BEGIN TRANSACTION;
SELECT * FROM users WHERE id < 10;
COMMIT;

-- === CASE: Read uncommitted isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
BEGIN TRANSACTION;
SELECT * FROM users WHERE id < 10;
COMMIT;

-- === CASE: Repeatable read isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
BEGIN TRANSACTION;
SELECT * FROM users WHERE id < 10;
COMMIT;

-- === CASE: Serializable isolation ===
-- EXPECT: success
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
BEGIN TRANSACTION;
SELECT * FROM users WHERE id < 10;
COMMIT;

-- === CASE: Multiple inserts in transaction ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (104, 'Multi1', 'multi1@example.com');
INSERT INTO users (id, name, email) VALUES (105, 'Multi2', 'multi2@example.com');
INSERT INTO users (id, name, email) VALUES (106, 'Multi3', 'multi3@example.com');
COMMIT;

-- === CASE: Update in transaction ===
-- EXPECT: 5 rows affected
BEGIN TRANSACTION;
UPDATE users SET email = 'updated_' || email WHERE id BETWEEN 1 AND 5;
COMMIT;

-- === CASE: Delete in transaction ===
-- EXPECT: 2 rows affected
BEGIN TRANSACTION;
DELETE FROM users WHERE id BETWEEN 95 AND 99;
COMMIT;

-- === CASE: Mixed DML in transaction ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (107, 'Mixed1', 'mixed1@example.com');
UPDATE users SET email = 'mixed_updated@example.com' WHERE id = 1;
DELETE FROM users WHERE id = 2;
COMMIT;

-- === CASE: Rollback to savepoint ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (108, 'SP1', 'sp1@example.com');
SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (109, 'SP2', 'sp2@example.com');
ROLLBACK TO SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (110, 'SP3', 'sp3@example.com');
COMMIT;

-- === CASE: Release savepoint ===
-- EXPECT: success
BEGIN TRANSACTION;
INSERT INTO users (id, name, email) VALUES (111, 'Release1', 'release1@example.com');
SAVEPOINT sp1;
INSERT INTO users (id, name, email) VALUES (112, 'Release2', 'release2@example.com');
RELEASE SAVEPOINT sp1;
COMMIT;

-- === CASE: Transaction with locking ===
-- EXPECT: success
BEGIN TRANSACTION;
SELECT * FROM users WHERE id = 1 FOR UPDATE;
UPDATE users SET email = 'locked@example.com' WHERE id = 1;
COMMIT;

-- === CASE: Deferrable transaction ===
-- EXPECT: success
BEGIN TRANSACTION DEFERRED;
SELECT * FROM users LIMIT 5;
COMMIT;

-- === CASE: Immediate transaction ===
-- EXPECT: success
BEGIN TRANSACTION IMMEDIATE;
SELECT COUNT(*) FROM users;
COMMIT;
