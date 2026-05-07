-- === DIFFERENTIAL TEST: Transaction Isolation Comparison ===
-- Purpose: Compare transaction behavior between SQLRustGo and MySQL 5.7
-- Critical: Transaction isolation levels may be implemented differently

-- === SETUP ===
CREATE TABLE accounts (id INT PRIMARY KEY, name VARCHAR(100), balance DECIMAL(15,2));
CREATE TABLE inventory (id INT PRIMARY KEY, product VARCHAR(100), quantity INT);

INSERT INTO accounts VALUES (1, 'Alice', 1000.00);
INSERT INTO accounts VALUES (2, 'Bob', 500.00);
INSERT INTO accounts VALUES (3, 'Charlie', 2000.00);

INSERT INTO inventory VALUES (1, 'Laptop', 10);
INSERT INTO inventory VALUES (2, 'Mouse', 50);
INSERT INTO inventory VALUES (3, 'Keyboard', 30);

-- === CASE: Basic transaction (BEGIN/COMMIT)
BEGIN;
UPDATE accounts SET balance = balance + 100 WHERE id = 1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1 (balance should be 1100.00)

-- === CASE: Basic transaction (BEGIN/ROLLBACK)
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
ROLLBACK;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1 (balance should be 1000.00, rolled back)

-- === CASE: Default isolation level
SELECT @@tx_isolation;
-- EXPECT: rows 1

-- === CASE: Set READ UNCOMMITTED
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
SELECT @@tx_isolation;
-- EXPECT: rows 1

-- === CASE: Set READ COMMITTED
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
SELECT @@tx_isolation;
-- EXPECT: rows 1

-- === CASE: Set REPEATABLE READ
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
SELECT @@tx_isolation;
-- EXPECT: rows 1

-- === CASE: Set SERIALIZABLE
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
SELECT @@tx_isolation;
-- EXPECT: rows 1

-- === CASE: START TRANSACTION
START TRANSACTION;
SELECT balance FROM accounts WHERE id = 1;
UPDATE accounts SET balance = balance + 50 WHERE id = 1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1 (balance should be 1050.00)

-- === CASE: SAVEPOINT creation
BEGIN;
UPDATE accounts SET balance = balance + 100 WHERE id = 1;
SAVEPOINT sp1;
UPDATE accounts SET balance = balance + 50 WHERE id = 2;
ROLLBACK TO SAVEPOINT sp1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
SELECT balance FROM accounts WHERE id = 2;
-- EXPECT: rows 2 (id=1: 1100.00, id=2: 500.00 - only first update committed)

-- === CASE: Multiple savepoints
BEGIN;
UPDATE accounts SET balance = 1500 WHERE id = 1;
SAVEPOINT sp1;
UPDATE accounts SET balance = 2000 WHERE id = 1;
SAVEPOINT sp2;
UPDATE accounts SET balance = 2500 WHERE id = 1;
ROLLBACK TO SAVEPOINT sp1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1 (balance should be 1500.00)

-- === CASE: RELEASE SAVEPOINT
BEGIN;
UPDATE accounts SET balance = balance + 100 WHERE id = 1;
SAVEPOINT sp1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
RELEASE SAVEPOINT sp1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
SELECT balance FROM accounts WHERE id = 2;
-- EXPECT: rows 2 (both updates should be committed)

-- === CASE: LOCK IN SHARE MODE
BEGIN;
SELECT * FROM accounts WHERE id = 1 LOCK IN SHARE MODE;
-- This should not block - just testing lock syntax
COMMIT;

-- === CASE: FOR UPDATE
BEGIN;
SELECT * FROM accounts WHERE id = 1 FOR UPDATE;
-- This should not block - just testing lock syntax
COMMIT;

-- === CASE: Autocommit behavior
SET autocommit = 0;
SELECT @@autocommit;
SET autocommit = 1;
SELECT @@autocommit;
-- EXPECT: rows 2

-- === CASE: Transaction with aggregate
BEGIN;
SELECT SUM(balance) FROM accounts;
UPDATE accounts SET balance = balance + 100 WHERE id = 1;
SELECT SUM(balance) FROM accounts;
COMMIT;
-- EXPECT: rows 3

-- === CASE: Nested transactions (simulated with savepoints)
BEGIN;
UPDATE accounts SET balance = 999.99 WHERE id = 1;
BEGIN;
UPDATE accounts SET balance = 888.88 WHERE id = 1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1 (nested BEGIN is ignored, balance should be 888.88)

-- === CASE: Rollback on table creation
BEGIN;
CREATE TABLE temp_test (id INT PRIMARY KEY);
INSERT INTO temp_test VALUES (1);
ROLLBACK;
-- Table should not exist after rollback
SELECT COUNT(*) FROM temp_test;
-- EXPECT: error or rows 0

-- === CASE: Transaction isolation: dirty read test
-- In READ UNCOMMITTED, uncommitted data may be visible
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
BEGIN;
UPDATE accounts SET balance = 9999.99 WHERE id = 1;
-- Without COMMIT, other transactions might see this value in READ UNCOMMITTED
-- EXPECT: behavior varies by isolation level

-- === CASE: SELECT without transaction
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: rows 1

-- === CASE: INSERT within transaction
BEGIN;
INSERT INTO accounts VALUES (4, 'Diana', 1500.00);
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: rows 1 (count should be 4)

-- === CASE: DELETE within transaction
BEGIN;
DELETE FROM accounts WHERE id = 4;
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: rows 1 (count should be back to 3)

-- === CASE: UPDATE affecting multiple rows
BEGIN;
UPDATE accounts SET balance = balance + 10;
COMMIT;
SELECT SUM(balance) FROM accounts;
-- EXPECT: rows 1

-- === CASE: DELETE affecting multiple rows
BEGIN;
DELETE FROM accounts WHERE balance > 1500;
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: rows 1

-- === CASE: ROLLBACK of DELETE
BEGIN;
DELETE FROM accounts;
ROLLBACK;
SELECT COUNT(*) FROM accounts;
-- EXPECT: rows 1 (should be original count after rollback)
