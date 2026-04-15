-- SQLCorpus: Aggregate Function Tests
-- Extended tests for aggregate functions

-- === SETUP ===
CREATE TABLE sales_summary (id INTEGER, region TEXT, amount INTEGER, quantity INTEGER);
INSERT INTO sales_summary VALUES (1, 'North', 100, 10), (2, 'South', 200, 15), (3, 'North', 150, 12), (4, 'South', 50, 5), (5, 'East', 300, 25), (6, 'West', 75, 8);

-- === CASE: count_with_group ===
SELECT region, COUNT(*) FROM sales_summary GROUP BY region;
-- EXPECT: 4 rows

-- === CASE: sum_with_group ===
SELECT region, SUM(amount) FROM sales_summary GROUP BY region;
-- EXPECT: 4 rows

-- === CASE: avg_with_group ===
SELECT region, AVG(amount) FROM sales_summary GROUP BY region;
-- EXPECT: 4 rows

-- === CASE: min_with_group ===
SELECT region, MIN(amount) FROM sales_summary GROUP BY region;
-- EXPECT: 4 rows

-- === CASE: max_with_group ===
SELECT region, MAX(amount) FROM sales_summary GROUP BY region;
-- EXPECT: 4 rows

-- === CASE: having_clause ===
SELECT region, SUM(amount) FROM sales_summary GROUP BY region HAVING SUM(amount) > 150;
-- EXPECT: 2 rows

-- === CASE: order_by_aggregate ===
SELECT region, SUM(amount) FROM sales_summary GROUP BY region ORDER BY SUM(amount) DESC;
-- EXPECT: 4 rows

-- === CASE: count_distinct ===
SELECT COUNT(DISTINCT region) FROM sales_summary;
-- EXPECT: 1 rows

-- === CASE: sum_no_rows ===
SELECT SUM(amount) FROM sales_summary WHERE amount > 10000;
-- EXPECT: 1 rows

-- === CASE: count_no_rows ===
SELECT COUNT(*) FROM sales_summary WHERE amount > 10000;
-- EXPECT: 1 rows
