-- SQLCorpus: FOREIGN KEY CASCADE Tests

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob');

CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount INTEGER);
CREATE INDEX idx_orders_user_id ON orders(user_id);

-- === CASE: fk_insert_valid ===
INSERT INTO orders VALUES (1, 1, 100);
-- EXPECT: 1 rows

-- === CASE: fk_insert_invalid ===
INSERT INTO orders VALUES (2, 999, 200);
-- EXPECT: ERROR

-- === CASE: fk_delete_parent ===
INSERT INTO orders VALUES (3, 2, 150);
DELETE FROM users WHERE id = 2;
-- EXPECT: 1 rows (order should be deleted via CASCADE)
