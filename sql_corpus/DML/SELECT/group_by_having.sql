-- SQLCorpus: GROUP BY and HAVING
-- Tests for aggregation with grouping

-- === SETUP ===
CREATE TABLE orders (id INTEGER PRIMARY KEY, customer TEXT, product TEXT, quantity INTEGER, price INTEGER);
INSERT INTO orders VALUES (1, 'Alice', 'Apple', 5, 100);
INSERT INTO orders VALUES (2, 'Alice', 'Banana', 3, 50);
INSERT INTO orders VALUES (3, 'Bob', 'Apple', 2, 100);
INSERT INTO orders VALUES (4, 'Bob', 'Carrot', 4, 30);
INSERT INTO orders VALUES (5, 'Charlie', 'Banana', 6, 50);
INSERT INTO orders VALUES (6, 'Charlie', 'Apple', 1, 100);
INSERT INTO orders VALUES (7, 'Alice', 'Carrot', 2, 30);
INSERT INTO orders VALUES (8, 'Bob', 'Banana', 3, 50);

-- === CASE: group_by_single ===
SELECT customer, COUNT(*) FROM orders GROUP BY customer;
-- EXPECT: 3 rows

-- === CASE: group_by_with_sum ===
SELECT customer, SUM(quantity) FROM orders GROUP BY customer;
-- EXPECT: 3 rows

-- === CASE: group_by_with_avg ===
SELECT customer, AVG(price) FROM orders GROUP BY customer;
-- EXPECT: 3 rows

-- === CASE: group_by_with_min_max ===
SELECT product, MIN(price), MAX(price) FROM orders GROUP BY product;
-- EXPECT: 3 rows

-- === CASE: group_by_multiple_columns ===
SELECT customer, product, SUM(quantity) FROM orders GROUP BY customer, product;
-- EXPECT: 6 rows

-- === CASE: having_count ===
SELECT customer FROM orders GROUP BY customer HAVING COUNT(*) >= 3;
-- EXPECT: 2 rows

-- === CASE: having_sum ===
SELECT customer FROM orders GROUP BY customer HAVING SUM(quantity) > 5;
-- EXPECT: 2 rows

-- === CASE: having_avg ===
SELECT product FROM orders GROUP BY product HAVING AVG(price) > 60;
-- EXPECT: 2 rows

-- === CASE: group_by_with_order ===
SELECT customer, SUM(quantity) FROM orders GROUP BY customer ORDER BY SUM(quantity) DESC;
-- EXPECT: 3 rows

-- === CASE: group_by_count_order ===
SELECT customer, COUNT(*) as cnt FROM orders GROUP BY customer ORDER BY cnt;
-- EXPECT: 3 rows

-- === CASE: having_with_and ===
SELECT customer FROM orders GROUP BY customer HAVING COUNT(*) > 2 AND SUM(quantity) > 5;
-- EXPECT: 2 rows

-- === CASE: having_with_or ===
SELECT customer FROM orders GROUP BY customer HAVING COUNT(*) > 3 OR SUM(quantity) > 10;
-- EXPECT: 2 rows

-- === CASE: group_by_distinct_agg ===
SELECT COUNT(DISTINCT customer) FROM orders;
-- EXPECT: 1 rows

-- === CASE: group_by_sum_multiply ===
SELECT customer, SUM(quantity * price) FROM orders GROUP BY customer;
-- EXPECT: 3 rows

-- === CASE: group_by_alias ===
SELECT customer, SUM(quantity) as total_qty FROM orders GROUP BY customer;
-- EXPECT: 3 rows

-- === CASE: having_on_alias ===
SELECT customer, SUM(quantity) as total_qty FROM orders GROUP BY customer HAVING total_qty > 5;
-- EXPECT: 3 rows

-- === CASE: group_by_null ===
SELECT COALESCE(customer, 'Unknown'), COUNT(*) FROM orders GROUP BY COALESCE(customer, 'Unknown');
-- EXPECT: 3 rows