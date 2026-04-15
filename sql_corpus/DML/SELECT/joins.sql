-- SQLCorpus: JOIN Tests
-- Tests for various JOIN types

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie');

CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount INTEGER);
INSERT INTO orders VALUES (1, 1, 100), (2, 1, 200), (3, 2, 150), (4, 4, 50);

-- === CASE: inner_join ===
SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id;
-- EXPECT: 3 rows

-- === CASE: inner_join_no_match ===
SELECT u.name, o.amount FROM users u JOIN orders o ON u.id = o.user_id WHERE o.amount > 150;
-- EXPECT: 1 rows

-- === CASE: left_join ===
SELECT u.name, o.amount FROM users u LEFT JOIN orders o ON u.id = o.user_id;
-- EXPECT: 3 rows

-- === CASE: left_join_with_null ===
SELECT u.name, o.amount FROM users u LEFT JOIN orders o ON u.id = o.user_id WHERE o.amount IS NULL;
-- EXPECT: 1 rows

-- === CASE: right_join ===
SELECT u.name, o.amount FROM users u RIGHT JOIN orders o ON u.id = o.user_id;
-- EXPECT: 4 rows

-- === CASE: cross_join ===
SELECT u.name, o.amount FROM users u CROSS JOIN orders o WHERE u.id = 1;
-- EXPECT: 3 rows
