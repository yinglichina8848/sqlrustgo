-- SQLCorpus: Transaction Tests
-- Tests for transaction behavior

-- === SETUP ===
CREATE TABLE accounts (id INTEGER PRIMARY KEY, name TEXT, balance INTEGER);

-- === CASE: transaction_commit ===
BEGIN TRANSACTION;
INSERT INTO accounts VALUES (1, 'Alice', 100);
INSERT INTO accounts VALUES (2, 'Bob', 50);
COMMIT;
SELECT SUM(balance) FROM accounts;
-- EXPECT: 1 rows

-- === CASE: transaction_rollback ===
BEGIN TRANSACTION;
INSERT INTO accounts VALUES (3, 'Charlie', 200);
ROLLBACK;
SELECT COUNT(*) FROM accounts WHERE name = 'Charlie';
-- EXPECT: 0 rows

-- === CASE: transaction_update ===
BEGIN TRANSACTION;
UPDATE accounts SET balance = balance + 50 WHERE id = 1;
COMMIT;
SELECT balance FROM accounts WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: savepoint_basic ===
BEGIN TRANSACTION;
INSERT INTO accounts VALUES (4, 'David', 300);
SAVEPOINT sp1;
INSERT INTO accounts VALUES (5, 'Eve', 400);
ROLLBACK TO SAVEPOINT sp1;
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 5 rows

-- === CASE: transaction_delete ===
BEGIN TRANSACTION;
DELETE FROM accounts WHERE id = 1;
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 4 rows

-- === CASE: begin_transaction ===
BEGIN;
INSERT INTO accounts VALUES (6, 'Frank', 600);
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 6 rows

-- === CASE: deferred_transaction ===
BEGIN DEFERRED;
INSERT INTO accounts VALUES (7, 'Grace', 700);
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 7 rows

-- === CASE: immediate_transaction ===
BEGIN IMMEDIATE;
INSERT INTO accounts VALUES (8, 'Henry', 800);
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 8 rows

-- === CASE: exclusive_transaction ===
BEGIN EXCLUSIVE;
INSERT INTO accounts VALUES (9, 'Ivy', 900);
COMMIT;
SELECT COUNT(*) FROM accounts;
-- EXPECT: 9 rows