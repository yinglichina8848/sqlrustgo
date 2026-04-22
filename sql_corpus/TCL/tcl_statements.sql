-- TCL (Transaction Control) Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Basic Transaction
-- ============================================

START TRANSACTION;
INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com');
INSERT INTO orders (user_id, total) VALUES (LAST_INSERT_ID(), 100);
COMMIT;

START TRANSACTION;
UPDATE accounts SET balance = balance - 100 WHERE id = 1;
UPDATE accounts SET balance = balance + 100 WHERE id = 2;
COMMIT;

-- ============================================
-- 2. Transaction with ROLLBACK
-- ============================================

START TRANSACTION;
INSERT INTO products (name, price) VALUES ('Test Product', 50);
ROLLBACK;

START TRANSACTION;
DELETE FROM users WHERE id = 999;
ROLLBACK;

START TRANSACTION;
UPDATE inventory SET stock = stock - 1 WHERE id = 1;
UPDATE inventory SET stock = stock - 1 WHERE id = 1;
UPDATE inventory SET stock = stock - 1 WHERE id = 1;
IF (SELECT stock FROM inventory WHERE id = 1) < 0 THEN
    ROLLBACK;
ELSE
    COMMIT;
END IF;

-- ============================================
-- 3. SAVEPOINT and ROLLBACK TO
-- ============================================

START TRANSACTION;
INSERT INTO users (name) VALUES ('User1');
SAVEPOINT sp1;
INSERT INTO users (name) VALUES ('User2');
SAVEPOINT sp2;
INSERT INTO users (name) VALUES ('User3');
ROLLBACK TO sp2;
INSERT INTO users (name) VALUES ('User4');
COMMIT;

START TRANSACTION;
INSERT INTO orders (total) VALUES (100);
SAVEPOINT sv1;
INSERT INTO orders (total) VALUES (200);
ROLLBACK TO sv1;
INSERT INTO orders (total) VALUES (150);
COMMIT;

-- ============================================
-- 4. RELEASE SAVEPOINT
-- ============================================

START TRANSACTION;
INSERT INTO users (name) VALUES ('TempUser');
SAVEPOINT my_savepoint;
INSERT INTO orders (total) VALUES (100);
RELEASE SAVEPOINT my_savepoint;
COMMIT;

-- ============================================
-- 5. AUTOCOMMIT
-- ============================================

SET AUTOCOMMIT = 0;
INSERT INTO users (name) VALUES ('UserA');
INSERT INTO users (name) VALUES ('UserB');
COMMIT;
SET AUTOCOMMIT = 1;

SET AUTOCOMMIT = 0;
UPDATE products SET price = price * 1.1;
ROLLBACK;
SET AUTOCOMMIT = 1;

-- ============================================
-- 6. COMMIT with conditions
-- ============================================

START TRANSACTION;
INSERT INTO orders (user_id, total) VALUES (1, 500);
UPDATE inventory SET reserved = reserved + 1 WHERE product_id = 1;
COMMIT;

START TRANSACTION;
UPDATE account SET balance = balance - 1000 WHERE id = 1 AND balance >= 1000;
IF ROW_COUNT() = 0 THEN
    ROLLBACK;
ELSE
    UPDATE account SET balance = balance + 1000 WHERE id = 2;
    COMMIT;
END IF;

-- ============================================
-- 7. Nested transactions (MySQL simulated)
-- ============================================

START TRANSACTION;
INSERT INTO logs (action) VALUES ('TX_START');
    START TRANSACTION;
    INSERT INTO logs (action) VALUES ('NESTED_TX');
    COMMIT;
INSERT INTO logs (action) VALUES ('TX_END');
COMMIT;

-- ============================================
-- 8. Transaction isolation levels
-- ============================================

SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;
START TRANSACTION;
SELECT * FROM products;
SELECT * FROM products;
COMMIT;

SET TRANSACTION ISOLATION LEVEL READ COMMITTED;
START TRANSACTION;
SELECT * FROM orders;
COMMIT;

SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;
START TRANSACTION;
SELECT * FROM users;
COMMIT;

SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
START TRANSACTION;
SELECT * FROM categories;
COMMIT;

-- ============================================
-- 9. Consistent snapshot (REPEATABLE READ)
-- ============================================

SET SESSION TRANSACTION ISOLATION LEVEL REPEATABLE READ;
START TRANSACTION;
SELECT * FROM products;
-- Another session inserts here
SELECT * FROM products;
COMMIT;

-- ============================================
-- 10. LOCK IN SHARE MODE
-- ============================================

START TRANSACTION;
SELECT * FROM products WHERE id = 1 LOCK IN SHARE MODE;
SELECT * FROM products WHERE id = 1;
COMMIT;

START TRANSACTION;
SELECT name, price FROM products WHERE price > 100 LOCK IN SHARE MODE;
COMMIT;

-- ============================================
-- 11. FOR UPDATE
-- ============================================

START TRANSACTION;
SELECT * FROM inventory WHERE stock < 10 FOR UPDATE;
UPDATE inventory SET stock = stock + 50 WHERE stock < 10;
COMMIT;

START TRANSACTION;
SELECT * FROM orders WHERE status = 'pending' FOR UPDATE;
UPDATE orders SET status = 'processing' WHERE status = 'pending';
COMMIT;

-- ============================================
-- 12. NOWAIT (MySQL 8.0+)
-- ============================================

-- START TRANSACTION;
-- SELECT * FROM products WHERE id = 1 FOR UPDATE NOWAIT;
-- COMMIT;

-- START TRANSACTION;
-- SELECT * FROM inventory WHERE id = 1 FOR UPDATE NOWAIT;
-- ROLLBACK;

-- ============================================
-- 13. SKIP LOCKED (MySQL 8.0+)
-- ============================================

-- START TRANSACTION;
-- SELECT * FROM inventory WHERE stock > 0 FOR UPDATE SKIP LOCKED;
-- UPDATE inventory SET stock = stock - 1 WHERE id = ?;
-- COMMIT;

-- ============================================
-- 14. XA transactions
-- ============================================

-- XA START 'tx1';
-- INSERT INTO users (name) VALUES ('XA_User');
-- XA END 'tx1';
-- XA PREPARE 'tx1';
-- XA COMMIT 'tx1';

-- XA START 'tx2';
-- UPDATE accounts SET balance = balance - 100 WHERE id = 1;
-- XA END 'tx2';
-- XA PREPARE 'tx2';
-- XA COMMIT 'tx2';

-- XA START 'tx3';
-- DELETE FROM temp_data WHERE id > 100;
-- XA END 'tx3';
-- XA PREPARE 'tx3';
-- XA ROLLBACK 'tx3';

-- ============================================
-- 15. Transaction with locking reads
-- ============================================

START TRANSACTION;
SELECT * FROM products WHERE id = 1 FOR UPDATE;
UPDATE products SET price = price + 10 WHERE id = 1;
COMMIT;

START TRANSACTION;
SELECT SUM(balance) INTO @total FROM accounts FOR UPDATE;
UPDATE accounts SET balance = balance / @total * 100 WHERE id > 0;
COMMIT;

-- ============================================
-- 16. Multi-statement transaction
-- ============================================

START TRANSACTION;
INSERT INTO users (name, email) VALUES ('Bob', 'bob@test.com');
INSERT INTO profiles (user_id, bio) VALUES (LAST_INSERT_ID(), 'New user profile');
INSERT INTO preferences (user_id, theme) VALUES (LAST_INSERT_ID(), 'dark');
UPDATE stats SET user_count = user_count + 1;
COMMIT;

-- ============================================
-- 17. Rollback on error (MySQL behavior)
-- ============================================

START TRANSACTION;
INSERT INTO products (name, price) VALUES ('Item1', 10);
INSERT INTO products (name, price) VALUES ('Item2', 20);
-- If next statement fails due to constraint, transaction auto-rollbacks
-- INSERT INTO products (id, name) VALUES (1, 'Duplicate'); -- Would cause rollback
COMMIT;

-- ============================================
-- 18. Transaction and foreign key
-- ============================================

START TRANSACTION;
INSERT INTO users (name) VALUES ('ParentUser');
SET @uid = LAST_INSERT_ID();
INSERT INTO orders (user_id, total) VALUES (@uid, 100);
COMMIT;

START TRANSACTION;
DELETE FROM users WHERE id = @uid;
-- Fails if FK constraint prevents deletion of user with orders
ROLLBACK;

-- ============================================
-- 19. Savepoint naming
-- ============================================

START TRANSACTION;
INSERT INTO logs (level, message) VALUES ('INFO', 'Step 1');
SAVEPOINT step1;
INSERT INTO logs (level, message) VALUES ('INFO', 'Step 2');
SAVEPOINT step2;
INSERT INTO logs (level, message) VALUES ('INFO', 'Step 3');
SAVEPOINT step3;
ROLLBACK TO step2;
INSERT INTO logs (level, message) VALUES ('INFO', 'Step 2b');
COMMIT;

-- ============================================
-- 20. Transaction timeout
-- ============================================

-- SET TRANSACTION_TIMEOUT = 30;
-- START TRANSACTION;
-- -- Long running operation
-- SELECT * FROM large_table;
-- COMMIT;

-- ============================================
-- 21. Readonly transactions
-- ============================================

SET TRANSACTION READ ONLY;
START TRANSACTION;
SELECT COUNT(*) FROM users;
SELECT SUM(total) FROM orders;
COMMIT;

-- ============================================
-- 22. Readwrite transactions
-- ============================================

SET TRANSACTION READ WRITE;
START TRANSACTION;
INSERT INTO audit_log (action) VALUES ('User created');
COMMIT;

-- ============================================
-- 23. Two-phase commit simulation
-- ============================================

START TRANSACTION;
INSERT INTO accounts (id, balance) VALUES (1, 1000);
INSERT INTO accounts (id, balance) VALUES (2, 1000);
-- Prepare phase
-- UPDATE accounts SET status = 'prepared' WHERE id IN (1, 2);
-- Commit phase
COMMIT;

-- ============================================
-- 24. Transaction with IFNULL
-- ============================================

START TRANSACTION;
UPDATE products SET stock = IFNULL(stock, 0) - 1 WHERE id = 1;
INSERT INTO purchase_history (product_id, quantity) VALUES (1, 1);
COMMIT;

-- ============================================
-- 25. Transaction with CASE
-- ============================================

START TRANSACTION;
UPDATE orders SET
    status = CASE
        WHEN status = 'pending' AND quantity > 10 THEN 'approved'
        WHEN status = 'pending' AND quantity <= 10 THEN 'manual_review'
        ELSE status
    END
WHERE id = 1;
COMMIT;
