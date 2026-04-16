-- SQLCorpus: INNER JOIN
-- Tests for INNER JOIN patterns

-- === SETUP ===
CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT);
CREATE TABLE orders (id INTEGER PRIMARY KEY, customer_id INTEGER, product TEXT);
INSERT INTO customers VALUES (1, 'Alice'), (2, 'Bob'), (3, 'Charlie');
INSERT INTO orders VALUES (1, 1, 'Apple'), (2, 1, 'Banana'), (3, 2, 'Carrot'), (4, 4, 'Date');

-- === CASE: inner_join ===
SELECT c.name, o.product FROM customers c INNER JOIN orders o ON c.id = o.customer_id;
-- EXPECT: 3 rows

-- === CASE: inner_join_with_filter ===
SELECT c.name, o.product FROM customers c INNER JOIN orders o ON c.id = o.customer_id WHERE c.id = 1;
-- EXPECT: 2 rows

-- === CASE: inner_join_multiple ===
SELECT c.name, o.product FROM customers c JOIN orders o ON c.id = o.customer_id;
-- EXPECT: 3 rows

-- === CASE: inner_join_count ===
SELECT COUNT(*) FROM customers c INNER JOIN orders o ON c.id = o.customer_id;
-- EXPECT: 1 rows

-- === CASE: inner_join_group ===
SELECT c.name, COUNT(o.id) as order_count FROM customers c INNER JOIN orders o ON c.id = o.customer_id GROUP BY c.name;
-- EXPECT: 2 rows

-- === CASE: inner_join_order ===
SELECT c.name, o.product FROM customers c INNER JOIN orders o ON c.id = o.customer_id ORDER BY c.name;
-- EXPECT: 3 rows

-- === CASE: implicit_inner_join ===
SELECT c.name, o.product FROM customers c, orders o WHERE c.id = o.customer_id;
-- EXPECT: 3 rows

-- === CASE: self_join ===
CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, manager_id INTEGER);
INSERT INTO employees VALUES (1, 'CEO', NULL), (2, 'Manager', 1), (3, 'Worker', 2);
SELECT e.name as emp, m.name as manager FROM employees e INNER JOIN employees m ON e.manager_id = m.id;
-- EXPECT: 2 rows