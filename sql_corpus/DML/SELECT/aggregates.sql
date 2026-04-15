-- SQLCorpus: Aggregate Functions
-- Tests for COUNT, SUM, AVG, MIN, MAX

-- === SETUP ===
CREATE TABLE sales (id INTEGER PRIMARY KEY, product TEXT, amount INTEGER);
INSERT INTO sales VALUES (1, 'Apple', 100), (2, 'Banana', 200), (3, 'Apple', 150), (4, 'Banana', 50), (5, 'Cherry', 300);

-- === CASE: count_all ===
SELECT COUNT(*) FROM sales;
-- EXPECT: 1 rows

-- === CASE: sum_amount ===
SELECT SUM(amount) FROM sales;
-- EXPECT: 1 rows

-- === CASE: avg_amount ===
SELECT AVG(amount) FROM sales;
-- EXPECT: 1 rows

-- === CASE: min_amount ===
SELECT MIN(amount) FROM sales;
-- EXPECT: 1 rows

-- === CASE: max_amount ===
SELECT MAX(amount) FROM sales;
-- EXPECT: 1 rows

-- === CASE: count_by_product ===
SELECT product, COUNT(*) FROM sales GROUP BY product;
-- EXPECT: 3 rows

-- === CASE: sum_by_product ===
SELECT product, SUM(amount) FROM sales GROUP BY product;
-- EXPECT: 3 rows
