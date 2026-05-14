-- === DIFFERENTIAL TEST: Basic SELECT Comparison ===
-- Purpose: Compare basic SELECT results between SQLRustGo and MySQL 5.7
-- Engine: Both engines should produce identical results

-- === SETUP ===
CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100), email VARCHAR(100), age INT, country VARCHAR(50));
CREATE TABLE products (id INT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category VARCHAR(50));

INSERT INTO users VALUES (1, 'Alice', 'alice@email.com', 30, 'USA');
INSERT INTO users VALUES (2, 'Bob', 'bob@email.com', 25, 'UK');
INSERT INTO users VALUES (3, 'Charlie', 'charlie@email.com', 35, 'Canada');
INSERT INTO users VALUES (4, 'Diana', 'diana@email.com', 28, 'USA');
INSERT INTO users VALUES (5, 'Eve', 'eve@email.com', 32, 'France');

INSERT INTO products VALUES (1, 'Laptop', 999.99, 'Electronics');
INSERT INTO products VALUES (2, 'Mouse', 29.99, 'Electronics');
INSERT INTO products VALUES (3, 'Chair', 199.99, 'Furniture');
INSERT INTO products VALUES (4, 'Desk', 299.99, 'Furniture');
INSERT INTO products VALUES (5, 'Keyboard', 79.99, 'Electronics');

-- === CASE: Select all users
SELECT * FROM users;
-- EXPECT: rows 5

-- === CASE: Select all products
SELECT * FROM products;
-- EXPECT: rows 5

-- === CASE: Select specific columns
SELECT name, email FROM users;
-- EXPECT: rows 5

-- === CASE: Select with WHERE clause (integer)
SELECT * FROM users WHERE age > 30;
-- EXPECT: rows 2

-- === CASE: Select with WHERE clause (string equality)
SELECT * FROM users WHERE country = 'USA';
-- EXPECT: rows 2

-- === CASE: Select with WHERE clause (string inequality)
SELECT * FROM users WHERE country != 'USA';
-- EXPECT: rows 3

-- === CASE: Select with ORDER BY ASC
SELECT * FROM users ORDER BY age ASC;
-- EXPECT: rows 5

-- === CASE: Select with ORDER BY DESC
SELECT * FROM users ORDER BY age DESC;
-- EXPECT: rows 5

-- === CASE: Select with LIMIT
SELECT * FROM users ORDER BY age ASC LIMIT 3;
-- EXPECT: rows 3

-- === CASE: Select with LIMIT OFFSET
SELECT * FROM users ORDER BY age ASC LIMIT 3 OFFSET 2;
-- EXPECT: rows 3

-- === CASE: Select with DISTINCT
SELECT DISTINCT country FROM users;
-- EXPECT: rows 5

-- === CASE: Select with LIKE pattern (starts with)
SELECT * FROM users WHERE name LIKE 'A%';
-- EXPECT: rows 1

-- === CASE: Select with LIKE pattern (contains)
SELECT * FROM users WHERE name LIKE '%e%';
-- EXPECT: rows 3

-- === CASE: Select with IN clause
SELECT * FROM users WHERE age IN (25, 30, 35);
-- EXPECT: rows 3

-- === CASE: Select with BETWEEN clause
SELECT * FROM users WHERE age BETWEEN 25 AND 35;
-- EXPECT: rows 5

-- === CASE: Select with AND condition
SELECT * FROM users WHERE age > 25 AND country = 'USA';
-- EXPECT: rows 2

-- === CASE: Select with OR condition
SELECT * FROM users WHERE age < 28 OR country = 'France';
-- EXPECT: rows 3

-- === CASE: Select with NOT condition
SELECT * FROM users WHERE NOT country = 'USA';
-- EXPECT: rows 3

-- === CASE: Select with IS NULL
SELECT * FROM users WHERE email IS NULL;
-- EXPECT: rows 0

-- === CASE: Select with IS NOT NULL
SELECT * FROM users WHERE email IS NOT NULL;
-- EXPECT: rows 5

-- === CASE: Select with aggregate COUNT
SELECT COUNT(*) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with aggregate SUM
SELECT SUM(age) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with aggregate AVG
SELECT AVG(age) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with aggregate MAX
SELECT MAX(age) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with aggregate MIN
SELECT MIN(age) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with GROUP BY
SELECT country, COUNT(*) FROM users GROUP BY country;
-- EXPECT: rows 5

-- === CASE: Select with GROUP BY and HAVING
SELECT country, COUNT(*) as cnt FROM users GROUP BY country HAVING cnt > 1;
-- EXPECT: rows 1

-- === CASE: Select with column alias
SELECT name AS username, age AS user_age FROM users;
-- EXPECT: rows 5

-- === CASE: Select with arithmetic expression
SELECT price * 1.1 AS increased_price FROM products;
-- EXPECT: rows 5

-- === CASE: Select with CASE simple
SELECT name, CASE WHEN age > 30 THEN 'Senior' ELSE 'Junior' END AS category FROM users;
-- EXPECT: rows 5

-- === CASE: Select with CASE search
SELECT name, CASE WHEN age >= 35 THEN 'Old'
                  WHEN age >= 30 THEN 'Middle'
                  ELSE 'Young' END AS age_group FROM users;
-- EXPECT: rows 5

-- === CASE: Select with COALESCE
SELECT name, COALESCE(email, 'no-email') AS email_fallback FROM users;
-- EXPECT: rows 5

-- === CASE: Select with NULLIF (age 0 would become NULL)
SELECT NULLIF(age, 0) FROM users;
-- EXPECT: rows 5
