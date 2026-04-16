-- SQLCorpus: INSERT Variations
-- Tests for various INSERT patterns

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER, email TEXT);

-- === CASE: insert_single ===
INSERT INTO users (id, name, age, email) VALUES (1, 'Alice', 30, 'alice@example.com');
SELECT COUNT(*) FROM users;
-- EXPECT: 1 rows

-- === CASE: insert_multiple_values ===
INSERT INTO users (id, name, age, email) VALUES (2, 'Bob', 25, 'bob@example.com'), (3, 'Charlie', 35, 'charlie@example.com');
SELECT COUNT(*) FROM users;
-- EXPECT: 3 rows

-- === CASE: insert_without_columns ===
INSERT INTO users VALUES (4, 'Diana', 28, 'diana@example.com');
SELECT COUNT(*) FROM users;
-- EXPECT: 4 rows

-- === CASE: insert_partial ===
INSERT INTO users (id, name) VALUES (5, 'Eve');
SELECT id, name FROM users WHERE id = 5;
-- EXPECT: 1 rows

-- === CASE: insert_with_null ===
INSERT INTO users (id, name, age, email) VALUES (6, 'Frank', NULL, NULL);
SELECT age FROM users WHERE id = 6;
-- EXPECT: 1 rows

-- === CASE: insert_text_with_quotes ===
INSERT INTO users (id, name, age, email) VALUES (7, 'G''H', 40, 'gh@example.com');
SELECT name FROM users WHERE id = 7;
-- EXPECT: 1 rows

-- === CASE: insert_update_conflict ===
INSERT OR REPLACE INTO users (id, name, age, email) VALUES (1, 'Alice Updated', 31, 'alice.new@example.com');
SELECT name FROM users WHERE id = 1;
-- EXPECT: 1 rows

-- === CASE: insert_default_values ===
INSERT INTO users DEFAULT VALUES;
SELECT COUNT(*) FROM users;
-- EXPECT: 6 rows

-- === CASE: insert_expression ===
INSERT INTO users (id, name, age, email) VALUES (8, 'Heidi', 20 + 10, 'heidi@example.com');
SELECT age FROM users WHERE id = 8;
-- EXPECT: 1 rows

-- === CASE: insert_string_concat ===
INSERT INTO users (id, name, age, email) VALUES (9, 'Ivan' || ' Jr.', 45, 'ivan@example.com');
SELECT name FROM users WHERE id = 9;
-- EXPECT: 1 rows