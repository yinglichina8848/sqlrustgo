-- === DIFFERENTIAL TEST: Aggregate Functions Comparison ===
-- Purpose: Compare aggregate function results between SQLRustGo and MySQL 5.7
-- Critical: Aggregate behaviors (especially with NULLs) may differ

-- === SETUP ===
CREATE TABLE sales (id INT PRIMARY KEY, product VARCHAR(50), category VARCHAR(50), amount DECIMAL(10,2), quantity INT, sale_date DATE);
CREATE TABLE employees (id INT PRIMARY KEY, name VARCHAR(100), department VARCHAR(50), salary DECIMAL(10,2), bonus DECIMAL(10,2));

INSERT INTO sales VALUES (1, 'Laptop', 'Electronics', 999.99, 1, '2024-01-15');
INSERT INTO sales VALUES (2, 'Mouse', 'Electronics', 29.99, 5, '2024-01-16');
INSERT INTO sales VALUES (3, 'Chair', 'Furniture', 199.99, 2, '2024-01-17');
INSERT INTO sales VALUES (4, 'Desk', 'Furniture', 299.99, 1, '2024-01-18');
INSERT INTO sales VALUES (5, 'Keyboard', 'Electronics', 79.99, 3, '2024-01-19');
INSERT INTO sales VALUES (6, 'Monitor', 'Electronics', 399.99, 2, '2024-01-20');
INSERT INTO sales VALUES (7, 'Table', 'Furniture', 249.99, 1, '2024-01-21');
INSERT INTO sales VALUES (8, 'Webcam', 'Electronics', 89.99, 4, '2024-01-22');
INSERT INTO sales VALUES (9, 'Headphones', 'Electronics', 149.99, 6, '2024-01-23');
INSERT INTO sales VALUES (10, 'Lamp', 'Furniture', 49.99, 10, '2024-01-24');

INSERT INTO employees VALUES (1, 'Alice', 'Engineering', 80000, 5000);
INSERT INTO employees VALUES (2, 'Bob', 'Engineering', 75000, NULL);
INSERT INTO employees VALUES (3, 'Charlie', 'Sales', 65000, 3000);
INSERT INTO employees VALUES (4, 'Diana', 'Sales', 70000, 4000);
INSERT INTO employees VALUES (5, 'Eve', 'HR', 55000, 2000);
INSERT INTO employees VALUES (6, 'Frank', 'Engineering', 90000, NULL);
INSERT INTO employees VALUES (7, 'Grace', 'HR', 60000, 2500);
INSERT INTO employees VALUES (8, 'Henry', 'Sales', 85000, NULL);
INSERT INTO employees VALUES (9, 'Ivy', 'Engineering', 82000, 6000);
INSERT INTO employees VALUES (10, 'Jack', 'HR', 58000, 3000);

-- === CASE: COUNT(*)
SELECT COUNT(*) FROM sales;
-- EXPECT: rows 1 (should be 10)

-- === CASE: COUNT(column) ignores NULLs
SELECT COUNT(bonus) FROM employees;
-- EXPECT: rows 1 (should be 7, not 10)

-- === CASE: COUNT(DISTINCT)
SELECT COUNT(DISTINCT department) FROM employees;
-- EXPECT: rows 1 (should be 3)

-- === CASE: SUM
SELECT SUM(amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: SUM with DISTINCT
SELECT SUM(DISTINCT amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: AVG
SELECT AVG(amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: AVG with DISTINCT
SELECT AVG(DISTINCT amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: MAX
SELECT MAX(amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: MIN
SELECT MIN(amount) FROM sales;
-- EXPECT: rows 1

-- === CASE: MIN with VARCHAR
SELECT MIN(product) FROM sales;
-- EXPECT: rows 1

-- === CASE: MAX with VARCHAR
SELECT MAX(product) FROM sales;
-- EXPECT: rows 1

-- === CASE: GROUP BY single column
SELECT department, SUM(salary) FROM employees GROUP BY department;
-- EXPECT: rows 3

-- === CASE: GROUP BY multiple columns
SELECT department, COUNT(*) FROM employees GROUP BY department, salary;
-- EXPECT: rows 10

-- === CASE: GROUP BY with HAVING
SELECT department, AVG(salary) as avg_sal FROM employees GROUP BY department HAVING avg_sal > 65000;
-- EXPECT: rows 2

-- === CASE: GROUP BY with HAVING and COUNT
SELECT department, COUNT(*) as cnt FROM employees GROUP BY department HAVING cnt > 2;
-- EXPECT: rows 2

-- === CASE: SUM with HAVING
SELECT department, SUM(salary) as total FROM employees GROUP BY department HAVING total > 200000;
-- EXPECT: rows 2

-- === CASE: Multiple aggregates
SELECT COUNT(*) as cnt, SUM(amount) as total, AVG(amount) as average, MAX(amount) as max_amt, MIN(amount) as min_amt FROM sales;
-- EXPECT: rows 1

-- === CASE: Aggregate with arithmetic
SELECT SUM(amount * quantity) AS total_revenue FROM sales;
-- EXPECT: rows 1

-- === CASE: Aggregate with WHERE
SELECT SUM(amount) FROM sales WHERE category = 'Electronics';
-- EXPECT: rows 1

-- === CASE: Aggregate with WHERE and GROUP BY
SELECT category, SUM(amount) FROM sales WHERE quantity > 2 GROUP BY category;
-- EXPECT: rows 2

-- === CASE: COUNT with WHERE (no matches)
SELECT COUNT(*) FROM sales WHERE amount > 10000;
-- EXPECT: rows 1 (returns 0, not empty)

-- === CASE: SUM with WHERE (no matches)
SELECT SUM(amount) FROM sales WHERE amount > 10000;
-- EXPECT: rows 1 (returns NULL or 0)

-- === CASE: AVG with WHERE (no matches)
SELECT AVG(amount) FROM sales WHERE amount > 10000;
-- EXPECT: rows 1 (returns NULL)

-- === CASE: Scalar subquery with aggregate
SELECT * FROM employees WHERE salary > (SELECT AVG(salary) FROM employees);
-- EXPECT: rows 4

-- === CASE: Aggregate in HAVING without GROUP BY
SELECT SUM(salary) FROM employees HAVING SUM(salary) > 200000;
-- EXPECT: rows 1

-- === CASE: GROUP_CONCAT (MySQL specific)
SELECT GROUP_CONCAT(name) FROM employees WHERE department = 'Engineering';
-- EXPECT: rows 1

-- === CASE: GROUP_CONCAT with DISTINCT
SELECT GROUP_CONCAT(DISTINCT department) FROM employees;
-- EXPECT: rows 1

-- === CASE: GROUP_CONCAT with ORDER BY
SELECT department, GROUP_CONCAT(name ORDER BY name) FROM employees GROUP BY department;
-- EXPECT: rows 3

-- === CASE: GROUP_CONCAT with separator
SELECT GROUP_CONCAT(name SEPARATOR ';') FROM employees;
-- EXPECT: rows 1

-- === CASE: String aggregation with NULL handling
SELECT GROUP_CONCAT(bonus) FROM employees;
-- EXPECT: rows 1 (NULLs are ignored)

-- === CASE: STDDEV_POP (population standard deviation)
SELECT STDDEV_POP(salary) FROM employees;
-- EXPECT: rows 1

-- === CASE: STDDEV_SAMP (sample standard deviation)
SELECT STDDEV_SAMP(salary) FROM employees;
-- EXPECT: rows 1

-- === CASE: VAR_POP (population variance)
SELECT VAR_POP(salary) FROM employees;
-- EXPECT: rows 1

-- === CASE: VAR_SAMP (sample variance)
SELECT VAR_SAMP(salary) FROM employees;
-- EXPECT: rows 1

-- === CASE: BIT_AND
SELECT BIT_AND(id) FROM employees;
-- EXPECT: rows 1

-- === CASE: BIT_OR
SELECT BIT_OR(id) FROM employees;
-- EXPECT: rows 1

-- === CASE: BIT_XOR
SELECT BIT_XOR(id) FROM employees;
-- EXPECT: rows 1

-- === CASE: Aggregate with ROLLUP
SELECT department, SUM(salary) FROM employees GROUP BY ROLLUP(department);
-- EXPECT: rows 4 (3 departments + 1 grand total)

-- === CASE: Aggregate with CUBE
SELECT department, SUM(salary) FROM employees GROUP BY CUBE(department);
-- EXPECT: rows 7 (all combinations)

-- === CASE: COUNT with CASE in aggregate
SELECT SUM(CASE WHEN amount > 100 THEN 1 ELSE 0 END) FROM sales;
-- EXPECT: rows 1

-- === CASE: COUNT with DECODE (Oracle compatibility)
SELECT COUNT(DECODE(amount, 999.99, 1)) FROM sales;
-- EXPECT: rows 1

-- === CASE: Aggregate on expression
SELECT SUM(COALESCE(bonus, 0)) FROM employees;
-- EXPECT: rows 1

-- === CASE: Multiple aggregates with GROUP BY
SELECT category, COUNT(*) as cnt, SUM(amount) as total, AVG(amount) as avg_amt FROM sales GROUP BY category;
-- EXPECT: rows 2
