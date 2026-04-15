-- SQLCorpus: Subquery Tests
-- Tests for EXISTS, IN, ANY/ALL subqueries

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie');

CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount INTEGER);
INSERT INTO orders VALUES (1, 1, 100), (2, 1, 200), (3, 2, 150), (4, 3, 50);

-- === CASE: exists_subquery ===
SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id);
-- EXPECT: 2 rows

-- === CASE: in_subquery ===
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE amount > 100);
-- EXPECT: 2 rows

-- === CASE: not_in_subquery ===
SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM orders WHERE amount > 100);
-- EXPECT: 1 rows

-- === CASE: any_subquery ===
SELECT * FROM users WHERE id = ANY (SELECT user_id FROM orders);
-- EXPECT: 3 rows

-- === CASE: all_subquery ===
SELECT * FROM users WHERE id = ALL (SELECT user_id FROM orders WHERE user_id IS NOT NULL);
-- EXPECT: 0 rows
