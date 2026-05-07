-- === DIFFERENTIAL TEST: NULL Handling Comparison ===
-- Purpose: Compare NULL handling between SQLRustGo and MySQL 5.7
-- Critical: NULL semantics differ between engines, these tests expose compatibility issues

-- === SETUP ===
CREATE TABLE null_test (id INT PRIMARY KEY, val INT, name VARCHAR(100), amount DECIMAL(10,2));
CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100), email VARCHAR(100), age INT);

INSERT INTO null_test VALUES (1, NULL, 'Alpha', NULL);
INSERT INTO null_test VALUES (2, 10, NULL, 100.00);
INSERT INTO null_test VALUES (3, 20, 'Gamma', 200.00);
INSERT INTO null_test VALUES (4, NULL, 'Delta', NULL);
INSERT INTO null_test VALUES (5, 50, 'Epsilon', 500.00);

INSERT INTO users VALUES (1, 'Alice', 'alice@email.com', 30);
INSERT INTO users VALUES (2, 'Bob', NULL, 25);
INSERT INTO users VALUES (3, 'Charlie', 'charlie@email.com', NULL);
INSERT INTO users VALUES (4, 'Diana', 'diana@email.com', 28);
INSERT INTO users VALUES (5, 'Eve', NULL, NULL);

-- === CASE: Select rows where val IS NULL
SELECT * FROM null_test WHERE val IS NULL;
-- EXPECT: rows 2

-- === CASE: Select rows where val IS NOT NULL
SELECT * FROM null_test WHERE val IS NOT NULL;
-- EXPECT: rows 3

-- === CASE: Select with COALESCE on INT
SELECT id, COALESCE(val, 0) AS val_coalesced FROM null_test;
-- EXPECT: rows 5

-- === CASE: Select with COALESCE on VARCHAR
SELECT id, COALESCE(name, 'unnamed') AS name_coalesced FROM null_test;
-- EXPECT: rows 5

-- === CASE: Select with COALESCE on DECIMAL
SELECT id, COALESCE(amount, 0.00) AS amount_coalesced FROM null_test;
-- EXPECT: rows 5

-- === CASE: NULLIF when values are equal (should return NULL)
SELECT NULLIF(10, 10) AS nullif_result;
-- EXPECT: rows 1

-- === CASE: NULLIF when values are different (should return first value)
SELECT NULLIF(10, 20) AS nullif_result;
-- EXPECT: rows 1

-- === CASE: NULL comparison with equals (NULL = NULL)
-- MySQL: NULL (unknown), SQLRustGo behavior may differ
SELECT * FROM null_test WHERE val = val;
-- EXPECT: rows 0 (both engines should return 0 rows for NULL = NULL)

-- === CASE: NULL comparison with not equals (NULL != NULL)
-- MySQL: NULL (unknown), SQLRustGo behavior may differ
SELECT * FROM null_test WHERE val != val;
-- EXPECT: rows 0

-- === CASE: NULL IN clause with single NULL
SELECT * FROM null_test WHERE val IN (NULL);
-- EXPECT: rows 0

-- === CASE: NULL NOT IN clause with single NULL
SELECT * FROM null_test WHERE val NOT IN (NULL);
-- EXPECT: rows 0

-- === CASE: NULL IN clause with mixed values
SELECT * FROM null_test WHERE val IN (10, NULL, 20);
-- EXPECT: rows 2 (only non-NULL matches)

-- === CASE: NULL NOT IN clause with mixed values
SELECT * FROM null_test WHERE val NOT IN (10, NULL, 20);
-- EXPECT: rows 0 (NULL NOT IN is never true for NULL values)

-- === CASE: ISNULL function
SELECT id, ISNULL(val) AS is_null FROM null_test WHERE val IS NULL;
-- EXPECT: rows 2

-- === CASE: IFNULL function
SELECT id, IFNULL(val, -1) AS val_ifnull FROM null_test;
-- EXPECT: rows 5

-- === CASE: NVL alias (Oracle compatibility)
SELECT id, NVL(val, -1) AS val_nvl FROM null_test;
-- EXPECT: rows 5

-- === CASE: String concatenation with NULL
SELECT CONCAT('Hello', NULL, 'World') AS concat_null;
-- EXPECT: rows 1

-- === CASE: Arithmetic with NULL (addition)
SELECT NULL + 10 AS null_arith;
-- EXPECT: rows 1

-- === CASE: Arithmetic with NULL (multiplication)
SELECT NULL * 10 AS null_arith;
-- EXPECT: rows 1

-- === CASE: Comparison NULL and numeric
SELECT * FROM null_test WHERE val > 30;
-- EXPECT: rows 1

-- === CASE: Comparison NULL and string
SELECT * FROM null_test WHERE name > 'Beta';
-- EXPECT: rows 3

-- === CASE: ORDER BY with NULLs first
SELECT * FROM null_test ORDER BY val ASC NULLS FIRST;
-- EXPECT: rows 5

-- === CASE: ORDER BY with NULLs last
SELECT * FROM null_test ORDER BY val DESC NULLS LAST;
-- EXPECT: rows 5

-- === CASE: Aggregate functions ignore NULLs
SELECT COUNT(val), SUM(val), AVG(val) FROM null_test;
-- EXPECT: rows 1

-- === CASE: COUNT with asterisk includes rows with NULL
SELECT COUNT(*) FROM null_test;
-- EXPECT: rows 5

-- === CASE: COUNT with column ignores NULLs
SELECT COUNT(val) FROM null_test;
-- EXPECT: rows 3

-- === CASE: DISTINCT with NULL
SELECT DISTINCT val FROM null_test;
-- EXPECT: rows 3

-- === CASE: GROUP BY with NULL values
SELECT val, COUNT(*) FROM null_test GROUP BY val;
-- EXPECT: rows 3

-- === CASE: HAVING with NULL comparison
SELECT val, COUNT(*) FROM null_test GROUP BY val HAVING val IS NOT NULL;
-- EXPECT: rows 2

-- === CASE: JOIN with NULL keys
CREATE TABLE orders (id INT PRIMARY KEY, user_id INT, amount DECIMAL(10,2));
CREATE TABLE users_null (id INT PRIMARY KEY, name VARCHAR(100));

INSERT INTO users_null VALUES (1, 'Alice');
INSERT INTO users_null VALUES (2, 'Bob');
INSERT INTO users_null VALUES (3, 'Charlie');

INSERT INTO orders VALUES (1, NULL, 100.00);
INSERT INTO orders VALUES (2, 1, 200.00);
INSERT INTO orders VALUES (3, NULL, 300.00);

SELECT u.name, o.amount FROM users_null u LEFT JOIN orders o ON u.id = o.user_id;
-- EXPECT: rows 3

-- === CASE: Subquery returning NULL
SELECT * FROM null_test WHERE val = (SELECT val FROM null_test WHERE id = 1);
-- EXPECT: rows 0

-- === CASE: EXISTS with NULL
SELECT EXISTS (SELECT * FROM null_test WHERE val = NULL) AS exists_null;
-- EXPECT: rows 1

-- === CASE: NOT EXISTS with NULL
SELECT NOT EXISTS (SELECT * FROM null_test WHERE val = NULL) AS not_exists_null;
-- EXPECT: rows 1
