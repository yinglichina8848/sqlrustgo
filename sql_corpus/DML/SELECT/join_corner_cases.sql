-- === SKIP ===

-- SQLCorpus: JOIN Corner Cases
-- Edge cases and stress tests for JOIN operations

-- === SETUP ===
CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, dept_id INTEGER, manager_id INTEGER);
INSERT INTO employees VALUES (1, 'CEO', NULL, NULL);
INSERT INTO employees VALUES (2, 'Alice', 1, 1);
INSERT INTO employees VALUES (3, 'Bob', 1, 1);
INSERT INTO employees VALUES (4, 'Charlie', 2, 2);
INSERT INTO employees VALUES (5, 'David', 2, 2);
INSERT INTO employees VALUES (6, 'Eve', 3, 3);

CREATE TABLE departments (id INTEGER PRIMARY KEY, name TEXT, budget INTEGER);
INSERT INTO departments VALUES (1, 'Engineering', 100000);
INSERT INTO departments VALUES (2, 'Sales', 50000);
INSERT INTO departments VALUES (3, 'Marketing', 30000);

-- === CASE: self_join ===
SELECT e1.name AS employee, e2.name AS manager FROM employees e1 JOIN employees e2 ON e1.manager_id = e2.id;
-- EXPECT: 4 rows

-- === CASE: self_join_left ===
SELECT e1.name AS employee, e2.name AS manager FROM employees e1 LEFT JOIN employees e2 ON e1.manager_id = e2.id;
-- EXPECT: 6 rows

-- === CASE: multiple_join ===
SELECT e.name, d.name, d.budget FROM employees e JOIN departments d ON e.dept_id = d.id;
-- EXPECT: 4 rows

-- === CASE: join_with_expression ===
SELECT e.name, d.budget * 2 AS doubled_budget FROM employees e JOIN departments d ON e.dept_id = d.id;
-- EXPECT: 4 rows

-- === CASE: join_with_aggregate ===
SELECT d.name, COUNT(e.id) FROM departments d LEFT JOIN employees e ON d.id = e.dept_id GROUP BY d.name;
-- EXPECT: 3 rows

-- === CASE: three_table_join ===
CREATE TABLE projects (id INTEGER PRIMARY KEY, dept_id INTEGER, name TEXT);
INSERT INTO projects VALUES (1, 1, 'Project A'), (2, 2, 'Project B');
SELECT e.name, d.name, p.name FROM employees e JOIN departments d ON e.dept_id = d.id JOIN projects p ON d.id = p.dept_id;
-- EXPECT: 2 rows

-- === CASE: non_equi_join ===
CREATE TABLE sales (id INTEGER, region TEXT, amount INTEGER);
CREATE TABLE targets (id INTEGER, region TEXT, target INTEGER);
INSERT INTO sales VALUES (1, 'North', 100), (2, 'South', 150), (3, 'East', 80);
INSERT INTO targets VALUES (1, 'North', 120), (2, 'South', 140), (3, 'West', 100);
SELECT s.region, s.amount, t.target FROM sales s JOIN targets t ON s.region = t.region AND s.amount > t.target * 0.5;
-- EXPECT: 2 rows

-- === CASE: join_with_or_condition ===
SELECT e.name, d.name FROM employees e LEFT JOIN departments d ON e.dept_id = d.id OR e.id = 1;
-- EXPECT: 6 rows

-- === CASE: nested_join ===
SELECT e.name, d.name, p.name FROM (employees e JOIN departments d ON e.dept_id = d.id) JOIN projects p ON d.id = p.dept_id WHERE e.id < 5;
-- EXPECT: 2 rows

-- === CASE: full_outer_join ===
CREATE TABLE t1 (id INTEGER, val TEXT);
CREATE TABLE t2 (id INTEGER, val TEXT);
INSERT INTO t1 VALUES (1, 'A'), (2, 'B'), (3, 'C');
INSERT INTO t2 VALUES (2, 'B'), (3, 'C'), (4, 'D');
SELECT COALESCE(t1.id, t2.id) AS id FROM t1 FULL OUTER JOIN t2 ON t1.id = t2.id;
-- EXPECT: 4 rows

-- === CASE: cross_join_with_limit ===
CREATE TABLE users (id INTEGER, name TEXT);
CREATE TABLE orders (id INTEGER, amount INTEGER);
INSERT INTO users VALUES (1, 'A'), (2, 'B'), (3, 'C');
INSERT INTO orders VALUES (1, 100), (2, 200);
SELECT u.name, o.amount FROM users u CROSS JOIN orders o LIMIT 5;
-- EXPECT: 5 rows

-- === CASE: join_to_empty ===
CREATE TABLE empty_table (id INTEGER);
SELECT u.name, e.id FROM users u LEFT JOIN empty_table e ON u.id = e.id;
-- EXPECT: 3 rows

-- === CASE: join_with_null_on_condition ===
CREATE TABLE null_test (id INTEGER, val TEXT);
INSERT INTO null_test VALUES (1, 'A'), (2, NULL), (3, 'C');
SELECT u.name, n.val FROM users u LEFT JOIN null_test n ON u.id = n.id;
-- EXPECT: 3 rows

-- === CASE: multi_column_join ===
CREATE TABLE a (x INTEGER, y INTEGER, val TEXT);
CREATE TABLE b (x INTEGER, y INTEGER, val TEXT);
INSERT INTO a VALUES (1, 1, 'A1'), (1, 2, 'A2'), (2, 1, 'A3');
INSERT INTO b VALUES (1, 1, 'B1'), (1, 2, 'B2');
SELECT a.val, b.val FROM a JOIN b ON a.x = b.x AND a.y = b.y;
-- EXPECT: 2 rows
