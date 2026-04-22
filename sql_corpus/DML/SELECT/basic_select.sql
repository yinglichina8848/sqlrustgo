-- === SETUP ===
CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(100), email VARCHAR(100), age INT, country VARCHAR(50));
CREATE TABLE products (id INT PRIMARY KEY, name VARCHAR(100), price DECIMAL(10,2), category VARCHAR(50));
CREATE TABLE orders (id INT PRIMARY KEY, user_id INT, product_id INT, quantity INT, order_date DATE);
CREATE TABLE employees (id INT PRIMARY KEY, name VARCHAR(100), salary DECIMAL(10,2), department VARCHAR(50), manager_id INT);
CREATE TABLE departments (id INT PRIMARY KEY, name VARCHAR(100), budget DECIMAL(15,2));
CREATE TABLE countries (id INT PRIMARY KEY, name VARCHAR(100), continent VARCHAR(50));
CREATE TABLE cities (id INT PRIMARY KEY, name VARCHAR(100), country_id INT, population INT);
CREATE TABLE scores (id INT PRIMARY KEY, student_name VARCHAR(100), subject VARCHAR(50), score INT);
CREATE TABLE accounts (id INT PRIMARY KEY, account_holder VARCHAR(100), balance DECIMAL(15,2), account_type VARCHAR(20));
CREATE TABLE transactions (id INT PRIMARY KEY, account_id INT, amount DECIMAL(10,2), transaction_type VARCHAR(20), transaction_date DATE);

INSERT INTO users VALUES (1, 'Alice', 'alice@email.com', 30, 'USA');
INSERT INTO users VALUES (2, 'Bob', 'bob@email.com', 25, 'UK');
INSERT INTO users VALUES (3, 'Charlie', 'charlie@email.com', 35, 'Canada');
INSERT INTO users VALUES (4, 'Diana', 'diana@email.com', 28, 'USA');
INSERT INTO users VALUES (5, 'Eve', 'eve@email.com', 32, 'France');
INSERT INTO users VALUES (6, 'Frank', 'frank@email.com', 45, 'Germany');
INSERT INTO users VALUES (7, 'Grace', 'grace@email.com', 29, 'Spain');
INSERT INTO users VALUES (8, 'Henry', 'henry@email.com', 38, 'Italy');
INSERT INTO users VALUES (9, 'Ivy', 'ivy@email.com', 27, 'Portugal');
INSERT INTO users VALUES (10, 'Jack', 'jack@email.com', 33, 'Netherlands');

INSERT INTO products VALUES (1, 'Laptop', 999.99, 'Electronics');
INSERT INTO products VALUES (2, 'Mouse', 29.99, 'Electronics');
INSERT INTO products VALUES (3, 'Keyboard', 79.99, 'Electronics');
INSERT INTO products VALUES (4, 'Chair', 199.99, 'Furniture');
INSERT INTO products VALUES (5, 'Desk', 299.99, 'Furniture');
INSERT INTO products VALUES (6, 'Monitor', 399.99, 'Electronics');
INSERT INTO products VALUES (7, 'Headphones', 149.99, 'Electronics');
INSERT INTO products VALUES (8, 'Webcam', 89.99, 'Electronics');
INSERT INTO products VALUES (9, 'Microphone', 129.99, 'Electronics');
INSERT INTO products VALUES (10, 'Speaker', 199.99, 'Electronics');

INSERT INTO orders VALUES (1, 1, 1, 1, '2024-01-15');
INSERT INTO orders VALUES (2, 1, 2, 2, '2024-01-16');
INSERT INTO orders VALUES (3, 2, 3, 1, '2024-01-17');
INSERT INTO orders VALUES (4, 3, 1, 1, '2024-01-18');
INSERT INTO orders VALUES (5, 4, 4, 2, '2024-01-19');
INSERT INTO orders VALUES (6, 5, 5, 1, '2024-01-20');
INSERT INTO orders VALUES (7, 1, 6, 1, '2024-01-21');
INSERT INTO orders VALUES (8, 2, 7, 3, '2024-01-22');
INSERT INTO orders VALUES (9, 3, 8, 1, '2024-01-23');
INSERT INTO orders VALUES (10, 4, 9, 2, '2024-01-24');

INSERT INTO employees VALUES (1, 'John Smith', 75000.00, 'Engineering', NULL);
INSERT INTO employees VALUES (2, 'Sarah Johnson', 65000.00, 'Engineering', 1);
INSERT INTO employees VALUES (3, 'Michael Brown', 70000.00, 'Engineering', 1);
INSERT INTO employees VALUES (4, 'Emily Davis', 55000.00, 'Marketing', NULL);
INSERT INTO employees VALUES (5, 'David Wilson', 60000.00, 'Marketing', 4);
INSERT INTO employees VALUES (6, 'Jessica Taylor', 80000.00, 'Sales', NULL);
INSERT INTO employees VALUES (7, 'Daniel Anderson', 45000.00, 'Sales', 6);
INSERT INTO employees VALUES (8, 'Amanda Thomas', 50000.00, 'Sales', 6);
INSERT INTO employees VALUES (9, 'Christopher Lee', 85000.00, 'Engineering', 1);
INSERT INTO employees VALUES (10, 'Michelle Martin', 55000.00, 'HR', NULL);

INSERT INTO departments VALUES (1, 'Engineering', 500000.00);
INSERT INTO departments VALUES (2, 'Marketing', 200000.00);
INSERT INTO departments VALUES (3, 'Sales', 300000.00);
INSERT INTO departments VALUES (4, 'HR', 100000.00);
INSERT INTO departments VALUES (5, 'Finance', 250000.00);

INSERT INTO countries VALUES (1, 'USA', 'North America');
INSERT INTO countries VALUES (2, 'UK', 'Europe');
INSERT INTO countries VALUES (3, 'Canada', 'North America');
INSERT INTO countries VALUES (4, 'France', 'Europe');
INSERT INTO countries VALUES (5, 'Germany', 'Europe');
INSERT INTO countries VALUES (6, 'Spain', 'Europe');
INSERT INTO countries VALUES (7, 'Italy', 'Europe');
INSERT INTO countries VALUES (8, 'Portugal', 'Europe');
INSERT INTO countries VALUES (9, 'Netherlands', 'Europe');
INSERT INTO countries VALUES (10, 'Japan', 'Asia');

INSERT INTO cities VALUES (1, 'New York', 1, 8336817);
INSERT INTO cities VALUES (2, 'Los Angeles', 1, 3979576);
INSERT INTO cities VALUES (3, 'Chicago', 1, 2693976);
INSERT INTO cities VALUES (4, 'London', 2, 8982000);
INSERT INTO cities VALUES (5, 'Manchester', 2, 547627);
INSERT INTO cities VALUES (6, 'Toronto', 3, 2731571);
INSERT INTO cities VALUES (7, 'Vancouver', 3, 631486);
INSERT INTO cities VALUES (8, 'Paris', 4, 2161000);
INSERT INTO cities VALUES (9, 'Lyon', 4, 515695);
INSERT INTO cities VALUES (10, 'Berlin', 5, 3644826);

INSERT INTO scores VALUES (1, 'Alice', 'Math', 95);
INSERT INTO scores VALUES (2, 'Alice', 'English', 88);
INSERT INTO scores VALUES (3, 'Alice', 'Science', 92);
INSERT INTO scores VALUES (4, 'Bob', 'Math', 78);
INSERT INTO scores VALUES (5, 'Bob', 'English', 85);
INSERT INTO scores VALUES (6, 'Bob', 'Science', 80);
INSERT INTO scores VALUES (7, 'Charlie', 'Math', 92);
INSERT INTO scores VALUES (8, 'Charlie', 'English', 90);
INSERT INTO scores VALUES (9, 'Charlie', 'Science', 88);
INSERT INTO scores VALUES (10, 'Diana', 'Math', 85);
INSERT INTO scores VALUES (11, 'Diana', 'English', 91);
INSERT INTO scores VALUES (12, 'Diana', 'Science', 87);
INSERT INTO scores VALUES (13, 'Eve', 'Math', 73);
INSERT INTO scores VALUES (14, 'Eve', 'English', 79);
INSERT INTO scores VALUES (15, 'Eve', 'Science', 76);

INSERT INTO accounts VALUES (1, 'Alice', 10000.00, 'Checking');
INSERT INTO accounts VALUES (2, 'Bob', 15000.00, 'Savings');
INSERT INTO accounts VALUES (3, 'Charlie', 8000.00, 'Checking');
INSERT INTO accounts VALUES (4, 'Diana', 20000.00, 'Savings');
INSERT INTO accounts VALUES (5, 'Eve', 12000.00, 'Checking');
INSERT INTO accounts VALUES (6, 'Frank', 5000.00, 'Savings');
INSERT INTO accounts VALUES (7, 'Grace', 18000.00, 'Checking');
INSERT INTO accounts VALUES (8, 'Henry', 9000.00, 'Savings');
INSERT INTO accounts VALUES (9, 'Ivy', 14000.00, 'Checking');
INSERT INTO accounts VALUES (10, 'Jack', 11000.00, 'Savings');

INSERT INTO transactions VALUES (1, 1, 500.00, 'Deposit', '2024-01-01');
INSERT INTO transactions VALUES (2, 1, -200.00, 'Withdrawal', '2024-01-02');
INSERT INTO transactions VALUES (3, 2, 1000.00, 'Deposit', '2024-01-03');
INSERT INTO transactions VALUES (4, 3, -300.00, 'Withdrawal', '2024-01-04');
INSERT INTO transactions VALUES (5, 4, 2000.00, 'Deposit', '2024-01-05');
INSERT INTO transactions VALUES (6, 5, -500.00, 'Withdrawal', '2024-01-06');
INSERT INTO transactions VALUES (7, 6, 250.00, 'Deposit', '2024-01-07');
INSERT INTO transactions VALUES (8, 7, -1000.00, 'Withdrawal', '2024-01-08');
INSERT INTO transactions VALUES (9, 8, 750.00, 'Deposit', '2024-01-09');
INSERT INTO transactions VALUES (10, 9, -400.00, 'Withdrawal', '2024-01-10');

-- === CASE: Select all users
SELECT * FROM users;
-- EXPECT: rows 10

-- === CASE: Select users where age > 30
SELECT * FROM users WHERE age > 30;
-- EXPECT: rows 5

-- === CASE: Select users where age < 30
SELECT * FROM users WHERE age < 30;
-- EXPECT: rows 5

-- === CASE: Select users where age = 30
SELECT * FROM users WHERE age = 30;
-- EXPECT: rows 1

-- === CASE: Select users where age >= 30
SELECT * FROM users WHERE age >= 30;
-- EXPECT: rows 6

-- === CASE: Select users where age <= 30
SELECT * FROM users WHERE age <= 30;
-- EXPECT: rows 5

-- === CASE: Select users where age != 30
SELECT * FROM users WHERE age != 30;
-- EXPECT: rows 9

-- === CASE: Select users where country = 'USA'
SELECT * FROM users WHERE country = 'USA';
-- EXPECT: rows 2

-- === CASE: Select users where country != 'USA'
SELECT * FROM users WHERE country != 'USA';
-- EXPECT: rows 8

-- === CASE: Select users with name starting with A
SELECT * FROM users WHERE name LIKE 'A%';
-- EXPECT: rows 1

-- === CASE: Select users with name ending with e
SELECT * FROM users WHERE name LIKE '%e';
-- EXPECT: rows 4

-- === CASE: Select users with name containing a
SELECT * FROM users WHERE name LIKE '%a%';
-- EXPECT: rows 6

-- === CASE: Select users where age > 25 AND country = 'USA'
SELECT * FROM users WHERE age > 25 AND country = 'USA';
-- EXPECT: rows 2

-- === CASE: Select users where age > 30 OR country = 'UK'
SELECT * FROM users WHERE age > 30 OR country = 'UK';
-- EXPECT: rows 6

-- === CASE: Select users where NOT country = 'USA'
SELECT * FROM users WHERE NOT country = 'USA';
-- EXPECT: rows 8

-- === CASE: Select users where age IN (25, 30, 35)
SELECT * FROM users WHERE age IN (25, 30, 35);
-- EXPECT: rows 3

-- === CASE: Select users where age BETWEEN 25 AND 35
SELECT * FROM users WHERE age BETWEEN 25 AND 35;
-- EXPECT: rows 10

-- === CASE: Select users where name IS NULL
SELECT * FROM users WHERE name IS NULL;
-- EXPECT: rows 0

-- === CASE: Select users where name IS NOT NULL
SELECT * FROM users WHERE name IS NOT NULL;
-- EXPECT: rows 10

-- === CASE: Select all products
SELECT * FROM products;
-- EXPECT: rows 10

-- === CASE: Select products where price > 100
SELECT * FROM products WHERE price > 100;
-- EXPECT: rows 7

-- === CASE: Select products where price < 100
SELECT * FROM products WHERE price < 100;
-- EXPECT: rows 3

-- === CASE: Select products where category = 'Electronics'
SELECT * FROM products WHERE category = 'Electronics';
-- EXPECT: rows 7

-- === CASE: Select products where category != 'Electronics'
SELECT * FROM products WHERE category != 'Electronics';
-- EXPECT: rows 3

-- === CASE: Select products ordered by price ASC
SELECT * FROM products ORDER BY price ASC;
-- EXPECT: rows 10

-- === CASE: Select products ordered by price DESC
SELECT * FROM products ORDER BY price DESC;
-- EXPECT: rows 10

-- === CASE: Select products ordered by name ASC
SELECT * FROM products ORDER BY name ASC;
-- EXPECT: rows 10

-- === CASE: Select products ordered by category ASC, price DESC
SELECT * FROM products ORDER BY category ASC, price DESC;
-- EXPECT: rows 10

-- === CASE: Select first 5 products
SELECT * FROM products LIMIT 5;
-- EXPECT: rows 5

-- === CASE: Select products with offset
SELECT * FROM products LIMIT 5 OFFSET 5;
-- EXPECT: rows 5

-- === CASE: Select distinct countries
SELECT DISTINCT country FROM users;
-- EXPECT: rows 10

-- === CASE: Select count of users
SELECT COUNT(*) FROM users;
-- EXPECT: rows 1

-- === CASE: Select sum of product prices
SELECT SUM(price) FROM products;
-- EXPECT: rows 1

-- === CASE: Select avg of product prices
SELECT AVG(price) FROM products;
-- EXPECT: rows 1

-- === CASE: Select max price
SELECT MAX(price) FROM products;
-- EXPECT: rows 1

-- === CASE: Select min price
SELECT MIN(price) FROM products;
-- EXPECT: rows 1

-- === CASE: Select all orders
SELECT * FROM orders;
-- EXPECT: rows 10

-- === CASE: Select orders with quantity > 1
SELECT * FROM orders WHERE quantity > 1;
-- EXPECT: rows 4

-- === CASE: Select orders joined with users
SELECT orders.*, users.name FROM orders JOIN users ON orders.user_id = users.id;
-- EXPECT: rows 10

-- === CASE: Select orders with user names using JOIN
SELECT orders.id, users.name, products.name FROM orders JOIN users ON orders.user_id = users.id JOIN products ON orders.product_id = products.id;
-- EXPECT: rows 10

-- === CASE: Count users per country
SELECT country, COUNT(*) FROM users GROUP BY country;
-- EXPECT: rows 10

-- === CASE: Select products grouped by category
SELECT category, COUNT(*) FROM products GROUP BY category;
-- EXPECT: rows 2

-- === CASE: Select employees
SELECT * FROM employees;
-- EXPECT: rows 10

-- === CASE: Select employees where salary > 60000
SELECT * FROM employees WHERE salary > 60000;
-- EXPECT: rows 5

-- === CASE: Select employees where department = 'Engineering'
SELECT * FROM employees WHERE department = 'Engineering';
-- EXPECT: rows 4

-- === CASE: Select employees ordered by salary DESC
SELECT * FROM employees ORDER BY salary DESC;
-- EXPECT: rows 10

-- === CASE: Select employees with LIMIT
SELECT * FROM employees ORDER BY salary DESC LIMIT 3;
-- EXPECT: rows 3

-- === CASE: Select departments
SELECT * FROM departments;
-- EXPECT: rows 5

-- === CASE: Select countries
SELECT * FROM countries;
-- EXPECT: rows 10

-- === CASE: Select cities
SELECT * FROM cities;
-- EXPECT: rows 10

-- === CASE: Select cities joined with countries
SELECT cities.name, countries.name FROM cities JOIN countries ON cities.country_id = countries.id;
-- EXPECT: rows 10

-- === CASE: Select scores
SELECT * FROM scores;
-- EXPECT: rows 15

-- === CASE: Select scores where subject = 'Math'
SELECT * FROM scores WHERE subject = 'Math';
-- EXPECT: rows 5

-- === CASE: Select scores grouped by student
SELECT student_name, AVG(score) FROM scores GROUP BY student_name;
-- EXPECT: rows 5

-- === CASE: Select scores grouped by subject
SELECT subject, AVG(score) FROM scores GROUP BY subject;
-- EXPECT: rows 3

-- === CASE: Select accounts
SELECT * FROM accounts;
-- EXPECT: rows 10

-- === CASE: Select accounts where balance > 10000
SELECT * FROM accounts WHERE balance > 10000;
-- EXPECT: rows 5

-- === CASE: Select accounts where account_type = 'Checking'
SELECT * FROM accounts WHERE account_type = 'Checking';
-- EXPECT: rows 5

-- === CASE: Select transactions
SELECT * FROM transactions;
-- EXPECT: rows 10

-- === CASE: Select transactions where amount > 0
SELECT * FROM transactions WHERE amount > 0;
-- EXPECT: rows 5

-- === CASE: Select transactions where transaction_type = 'Deposit'
SELECT * FROM transactions WHERE transaction_type = 'Deposit';
-- EXPECT: rows 5

-- === CASE: Select users with country and city
SELECT users.name, cities.name FROM users JOIN cities ON users.country = cities.country_id;
-- EXPECT: rows 10

-- === CASE: Select employees with department
SELECT employees.name, departments.name FROM employees JOIN departments ON employees.department = departments.name;
-- EXPECT: rows 10

-- === CASE: Select users using subquery
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders);
-- EXPECT: rows 4

-- === CASE: Select products using subquery
SELECT * FROM products WHERE id IN (SELECT product_id FROM orders WHERE quantity > 1);
-- EXPECT: rows 3

-- === CASE: Select users with aggregate in subquery
SELECT * FROM users WHERE age > (SELECT AVG(age) FROM users);
-- EXPECT: rows 4

-- === CASE: Select products where price > avg price
SELECT * FROM products WHERE price > (SELECT AVG(price) FROM products);
-- EXPECT: rows 4

-- === CASE: Select users with exists subquery
SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id);
-- EXPECT: rows 4

-- === CASE: Select users with not exists subquery
SELECT * FROM users WHERE NOT EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id);
-- EXPECT: rows 6

-- === CASE: Select employees with any department
SELECT * FROM employees WHERE department = ANY (SELECT name FROM departments);
-- EXPECT: rows 10

-- === CASE: Select users with ALL condition
SELECT * FROM users WHERE age > ALL (SELECT age FROM users WHERE country = 'USA');
-- EXPECT: rows 8

-- === CASE: Select with UNION
SELECT name FROM users UNION SELECT name FROM employees;
-- EXPECT: rows 15

-- === CASE: Select with UNION ALL
SELECT name FROM users UNION ALL SELECT name FROM employees;
-- EXPECT: rows 20

-- === CASE: Select with INTERSECT
SELECT country FROM users INTERSECT SELECT name FROM departments;
-- EXPECT: rows 0

-- === CASE: Select with EXCEPT
SELECT country FROM users EXCEPT SELECT name FROM departments;
-- EXPECT: rows 10

-- === CASE: Select with CASE simple
SELECT name, CASE WHEN age > 30 THEN 'Senior' ELSE 'Junior' END as category FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CASE search
SELECT name, CASE WHEN salary > 70000 THEN 'High' WHEN salary > 50000 THEN 'Medium' ELSE 'Low' END as salary_category FROM employees;
-- EXPECT: rows 10

-- === CASE: Select with COALESCE
SELECT name, COALESCE(email, 'noemail') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with NULLIF
SELECT NULLIF(age, 0) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CAST
SELECT CAST(price AS CHAR) FROM products;
-- EXPECT: rows 10

-- === CASE: Select with CONVERT
SELECT CONVERT(price, CHAR) FROM products;
-- EXPECT: rows 10

-- === CASE: Select with CHAR_LENGTH
SELECT name, CHAR_LENGTH(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CONCAT
SELECT CONCAT(name, ' - ', email) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UPPER
SELECT UPPER(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LOWER
SELECT LOWER(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with TRIM
SELECT TRIM(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SUBSTRING
SELECT SUBSTRING(name, 1, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REPLACE
SELECT REPLACE(name, 'a', '@') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LENGTH
SELECT name, LENGTH(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ABS
SELECT ABS(age - 30) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CEIL
SELECT CEIL(price) FROM products;
-- EXPECT: rows 10

-- === CASE: Select with FLOOR
SELECT FLOOR(price) FROM products;
-- EXPECT: rows 10

-- === CASE: Select with ROUND
SELECT ROUND(price, 0) FROM products;
-- EXPECT: rows 10

-- === CASE: Select with MOD
SELECT MOD(age, 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with POW
SELECT POW(age, 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SQRT
SELECT SQRT(age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with RAND
SELECT RAND() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with NOW
SELECT NOW() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CURDATE
SELECT CURDATE() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DATE
SELECT DATE(transaction_date) FROM transactions;
-- EXPECT: rows 10

-- === CASE: Select with YEAR
SELECT YEAR(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with MONTH
SELECT MONTH(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DAY
SELECT DAY(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DAYNAME
SELECT DAYNAME(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with MONTHNAME
SELECT MONTHNAME(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DATEDIFF
SELECT DATEDIFF(order_date, '2024-01-01') FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DATE_ADD
SELECT DATE_ADD(order_date, INTERVAL 1 DAY) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DATE_SUB
SELECT DATE_SUB(order_date, INTERVAL 1 DAY) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with COUNT with DISTINCT
SELECT COUNT(DISTINCT country) FROM users;
-- EXPECT: rows 1

-- === CASE: Select with SUM with DISTINCT
SELECT SUM(DISTINCT age) FROM users;
-- EXPECT: rows 1

-- === CASE: Select users with column alias
SELECT name AS username FROM users;
-- EXPECT: rows 10

-- === CASE: Select with table alias
SELECT u.name FROM users AS u;
-- EXPECT: rows 10

-- === CASE: Select with multiple columns
SELECT id, name, email FROM users;
-- EXPECT: rows 10

-- === CASE: Select with arithmetic
SELECT price * 1.1 FROM products;
-- EXPECT: rows 10

-- === CASE: Select with arithmetic addition
SELECT price + 10 FROM products;
-- EXPECT: rows 10

-- === CASE: Select with arithmetic subtraction
SELECT price - 10 FROM products;
-- EXPECT: rows 10

-- === CASE: Select with arithmetic division
SELECT price / 2 FROM products;
-- EXPECT: rows 10

-- === CASE: Select with LIKE NOT
SELECT * FROM users WHERE name NOT LIKE '%a%';
-- EXPECT: rows 4

-- === CASE: Select with IN subquery
SELECT * FROM users WHERE country IN (SELECT name FROM countries);
-- EXPECT: rows 10

-- === CASE: Select with NOT IN subquery
SELECT * FROM users WHERE country NOT IN (SELECT name FROM countries);
-- EXPECT: rows 0

-- === CASE: Select employees with HAVING
SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) > 2;
-- EXPECT: rows 2

-- === CASE: Select with COUNT HAVING
SELECT department, COUNT(*) FROM employees GROUP BY department HAVING COUNT(*) >= 3;
-- EXPECT: rows 1

-- === CASE: Select with AVG HAVING
SELECT department, AVG(salary) FROM employees GROUP BY department HAVING AVG(salary) > 60000;
-- EXPECT: rows 2

-- === CASE: Select with SUM HAVING
SELECT department, SUM(salary) FROM employees GROUP BY department HAVING SUM(salary) > 200000;
-- EXPECT: rows 1

-- === CASE: Select with nested subquery
SELECT * FROM (SELECT * FROM users WHERE age > 25) AS temp;
-- EXPECT: rows 7

-- === CASE: Select with correlated subquery
SELECT * FROM employees e WHERE salary > (SELECT AVG(salary) FROM employees WHERE department = e.department);
-- EXPECT: rows 7

-- === CASE: Select with ANY
SELECT * FROM users WHERE age = ANY (SELECT age FROM users WHERE country = 'USA');
-- EXPECT: rows 2

-- === CASE: Select LEFT JOIN
SELECT users.name, orders.id FROM users LEFT JOIN orders ON users.id = orders.user_id;
-- EXPECT: rows 10

-- === CASE: Select RIGHT JOIN
SELECT users.name, orders.id FROM users RIGHT JOIN orders ON users.id = orders.user_id;
-- EXPECT: rows 10

-- === CASE: Select CROSS JOIN
SELECT users.name, products.name FROM users CROSS JOIN products LIMIT 10;
-- EXPECT: rows 10

-- === CASE: Select self JOIN
SELECT e.name AS employee, m.name AS manager FROM employees e LEFT JOIN employees m ON e.manager_id = m.id;
-- EXPECT: rows 10

-- === CASE: Select NATURAL JOIN
SELECT name, department FROM employees NATURAL JOIN departments;
-- EXPECT: rows 0

-- === CASE: Select with GROUP BY ROLLUP
SELECT department, COUNT(*) FROM employees GROUP BY ROLLUP(department);
-- EXPECT: rows 6

-- === CASE: Select with GROUP BY CUBE
SELECT department, COUNT(*) FROM employees GROUP BY CUBE(department);
-- EXPECT: rows 11

-- === CASE: Select with LIMIT OFFSET
SELECT * FROM users ORDER BY id LIMIT 3 OFFSET 2;
-- EXPECT: rows 3

-- === CASE: Select with LIMIT and OFFSET using OFFSET keyword
SELECT * FROM users ORDER BY id LIMIT 3 OFFSET 5;
-- EXPECT: rows 3

-- === CASE: Select with FOR UPDATE
SELECT * FROM users WHERE id = 1 FOR UPDATE;
-- EXPECT: rows 1

-- === CASE: Select with LOCK IN SHARE MODE
SELECT * FROM users WHERE id = 1 LOCK IN SHARE MODE;
-- EXPECT: rows 1

-- === CASE: Select with STRAIGHT_JOIN
SELECT users.name, orders.id FROM users STRAIGHT_JOIN orders ON users.id = orders.user_id;
-- EXPECT: rows 10

-- === CASE: Select with IGNORE INDEX
SELECT * FROM users USE INDEX (PRIMARY) WHERE id > 0;
-- EXPECT: rows 10

-- === CASE: Select with FORCE INDEX
SELECT * FROM users FORCE INDEX (PRIMARY) WHERE id > 0;
-- EXPECT: rows 10

-- === CASE: Select with HIGH_PRIORITY
SELECT HIGH_PRIORITY * FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SQL_CACHE
SELECT SQL_CACHE * FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SQL_NO_CACHE
SELECT SQL_NO_CACHE * FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SQL_CALC_FOUND_ROWS
SELECT SQL_CALC_FOUND_ROWS * FROM users LIMIT 5;
-- EXPECT: rows 5

-- === CASE: Select FOUND_ROWS after SQL_CALC_FOUND_ROWS
SELECT FOUND_ROWS();
-- EXPECT: rows 1

-- === CASE: Select with JOIN and WHERE
SELECT users.name, orders.id FROM users JOIN orders ON users.id = orders.user_id WHERE orders.quantity > 1;
-- EXPECT: rows 4

-- === CASE: Select with multiple JOIN conditions
SELECT users.name, products.name, orders.quantity FROM users JOIN orders ON users.id = orders.user_id JOIN products ON orders.product_id = products.id WHERE orders.quantity > 1;
-- EXPECT: rows 4

-- === CASE: Select with BETWEEN with NOT
SELECT * FROM users WHERE age NOT BETWEEN 25 AND 35;
-- EXPECT: rows 0

-- === CASE: Select with LIKE with ESCAPE
SELECT * FROM users WHERE name LIKE '%\%' ESCAPE '\\';
-- EXPECT: rows 0

-- === CASE: Select with REGEXP
SELECT * FROM users WHERE name REGEXP '^A';
-- EXPECT: rows 1

-- === CASE: Select with REGEXP NOT
SELECT * FROM users WHERE name NOT REGEXP '^A';
-- EXPECT: rows 9

-- === CASE: Select with RLIKE
SELECT * FROM users WHERE name RLIKE '^A';
-- EXPECT: rows 1

-- === CASE: Select with SOUNDS LIKE
SELECT * FROM users WHERE name SOUNDS LIKE 'Alice';
-- EXPECT: rows 1

-- === CASE: Select with IF function
SELECT name, IF(age > 30, 'Old', 'Young') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with IFNULL
SELECT IFNULL(email, 'no-email') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ISNULL
SELECT ISNULL(email) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with NVL (alias for IFNULL)
SELECT NVL(email, 'no-email') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DECODE
SELECT DECODE(age, 30, 'Thirty', 'Other') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with GREATEST
SELECT GREATEST(1, 2, 3, 4, 5) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LEAST
SELECT LEAST(1, 2, 3, 4, 5) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with IF with AND
SELECT name FROM users WHERE IF(age > 25, country = 'USA', 1=1);
-- EXPECT: rows 2

-- === CASE: Select with nested IF
SELECT name, IF(age > 35, 'Very Old', IF(age > 30, 'Old', IF(age > 25, 'Young', 'Very Young')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DATEDIFF in HAVING
SELECT order_date, COUNT(*) FROM orders GROUP BY order_date HAVING DATEDIFF(order_date, '2024-01-15') > 0;
-- EXPECT: rows 9

-- === CASE: Select with YEAR in GROUP BY
SELECT YEAR(order_date), COUNT(*) FROM orders GROUP BY YEAR(order_date);
-- EXPECT: rows 1

-- === CASE: Select with MONTH in GROUP BY
SELECT MONTH(order_date), COUNT(*) FROM orders GROUP BY MONTH(order_date);
-- EXPECT: rows 1

-- === CASE: Select with DAY in GROUP BY
SELECT DAY(order_date), COUNT(*) FROM orders GROUP BY DAY(order_date);
-- EXPECT: rows 10

-- === CASE: Select with WEEK
SELECT WEEK(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with MONTHNAME
SELECT DISTINCT MONTHNAME(order_date) FROM orders;
-- EXPECT: rows 1

-- === CASE: Select with DAYOFWEEK
SELECT DAYOFWEEK(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DAYOFMONTH
SELECT DAYOFMONTH(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DAYOFYEAR
SELECT DAYOFYEAR(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with HOUR
SELECT HOUR(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with MINUTE
SELECT MINUTE(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with SECOND
SELECT SECOND(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with TIME
SELECT TIME(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with TIMESTAMP
SELECT TIMESTAMP(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with ADDDATE
SELECT ADDDATE(order_date, 5) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with SUBDATE
SELECT SUBDATE(order_date, 5) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with DATE_FORMAT
SELECT DATE_FORMAT(order_date, '%Y-%m-%d') FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with TIME_FORMAT
SELECT TIME_FORMAT(order_date, '%H:%i:%s') FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with STR_TO_DATE
SELECT STR_TO_DATE('2024-01-15', '%Y-%m-%d') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UTC_DATE
SELECT UTC_DATE() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UTC_TIME
SELECT UTC_TIME() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UTC_TIMESTAMP
SELECT UTC_TIMESTAMP() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UNIX_TIMESTAMP
SELECT UNIX_TIMESTAMP(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with FROM_UNIXTIME
SELECT FROM_UNIXTIME(1705315200) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MAKEDATE
SELECT MAKEDATE(2024, 15) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MAKETIME
SELECT MAKETIME(10, 30, 45) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with PERIOD_ADD
SELECT PERIOD_ADD(202401, 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with PERIOD_DIFF
SELECT PERIOD_DIFF(202401, 202310) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with QUARTER
SELECT QUARTER(order_date) FROM orders;
-- EXPECT: rows 10

-- === CASE: Select with TIME_TO_SEC
SELECT TIME_TO_SEC('10:30:45') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SEC_TO_TIME
SELECT SEC_TO_TIME(37845) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with WEIGHT_STRING
SELECT WEIGHT_STRING('test') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LPAD
SELECT LPAD(name, 20, '*') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with RPAD
SELECT RPAD(name, 20, '*') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LEFT
SELECT LEFT(name, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with RIGHT
SELECT RIGHT(name, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MID (alias for SUBSTRING)
SELECT MID(name, 2, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SUBSTRING_INDEX
SELECT SUBSTRING_INDEX('www.example.com', '.', 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LOCATE
SELECT LOCATE('a', name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with POSITION
SELECT POSITION('a' IN name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with INSTR
SELECT INSTR(name, 'a') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REVERSE
SELECT REVERSE(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SPACE
SELECT CONCAT(name, SPACE(10), email) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with STRCMP
SELECT STRCMP('abc', 'def') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CHAR
SELECT CHAR(65, 66, 67) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CONCAT_WS
SELECT CONCAT_WS('-', name, email) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ELT
SELECT ELT(2, 'a', 'b', 'c') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with EXPORT_SET
SELECT EXPORT_SET(5, 'Y', 'N', ',', 4) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with FORMAT
SELECT FORMAT(12345.6789, 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with HEX
SELECT HEX(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UNHEX
SELECT UNHEX('416C696365') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with INSERT function
SELECT INSERT(name, 2, 3, 'XXX') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LOAD_FILE
SELECT LOAD_FILE('/tmp/test.txt') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LOCATE with position
SELECT LOCATE('a', name, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MAKE_SET
SELECT MAKE_SET(5, 'a', 'b', 'c', 'd', 'e') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with OCTET_LENGTH
SELECT OCTET_LENGTH(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with QUOTE
SELECT QUOTE(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REGEXP_INSTR
SELECT REGEXP_INSTR(name, '^A') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REGEXP_LIKE
SELECT REGEXP_LIKE(name, '^A') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REGEXP_REPLACE
SELECT REGEXP_REPLACE(name, 'a', '@') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with REGEXP_SUBSTR
SELECT REGEXP_SUBSTR(name, 'a') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SOUNDEX
SELECT SOUNDEX(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SUBSTRING using FROM FOR syntax
SELECT SUBSTRING(name FROM 2 FOR 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with TRIM with specific characters
SELECT TRIM(BOTH '*' FROM name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with TRIM LEADING
SELECT TRIM(LEADING '*' FROM name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with TRIM TRAILING
SELECT TRIM(TRAILING '*' FROM name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LTRIM
SELECT LTRIM(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with RTRIM
SELECT RTRIM(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ASCII
SELECT ASCII(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ORD
SELECT ORD(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with BIN
SELECT BIN(age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with BIT_COUNT
SELECT BIT_COUNT(age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with BIT_LENGTH
SELECT BIT_LENGTH(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with INET_ATON
SELECT INET_ATON('192.168.1.1') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with INET_NTOA
SELECT INET_NTOA(3232235777) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with IS_IPV4
SELECT IS_IPV4('192.168.1.1') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with IS_IPV6
SELECT IS_IPV6('::1') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with PASSWORD
SELECT PASSWORD('test') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SHA1
SELECT SHA1(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SHA2
SELECT SHA2(name, 256) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MD5
SELECT MD5(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CRC32
SELECT CRC32(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with COMPRESS
SELECT COMPRESS(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with UNCOMPRESS
SELECT UNCOMPRESS(COMPRESS(name)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ENCRYPT
SELECT ENCRYPT('test', 'salt') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with VALIDATE_PASSWORD_STRENGTH
SELECT VALIDATE_PASSWORD_STRENGTH('Test1234') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ROW_COUNT
SELECT ROW_COUNT() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SCHEMA
SELECT SCHEMA() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DATABASE
SELECT DATABASE() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with USER
SELECT USER() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SESSION_USER
SELECT SESSION_USER() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SYSTEM_USER
SELECT SYSTEM_USER() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with VERSION
SELECT VERSION() FROM users;
-- EXPECT: rows 10

-- === CASE: Select with BENCHMARK
SELECT BENCHMARK(1000000, SHA1('test')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CHARSET
SELECT CHARSET(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with COLLATION
SELECT COLLATION(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with COERCIBILITY
SELECT COERCIBILITY(name) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with INTERVAL
SELECT INTERVAL(5, 1, 3, 5, 7, 9) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ROW_NUMBER (if window functions supported)
SELECT ROW_NUMBER() OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with RANK (if window functions supported)
SELECT RANK() OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DENSE_RANK (if window functions supported)
SELECT DENSE_RANK() OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with PERCENT_RANK (if window functions supported)
SELECT PERCENT_RANK() OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with CUME_DIST (if window functions supported)
SELECT CUME_DIST() OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with FIRST_VALUE (if window functions supported)
SELECT FIRST_VALUE(name) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LAST_VALUE (if window functions supported)
SELECT LAST_VALUE(name) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with NTH_VALUE (if window functions supported)
SELECT NTH_VALUE(name, 3) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LAG (if window functions supported)
SELECT LAG(name) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with LEAD (if window functions supported)
SELECT LEAD(name) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with NTILE (if window functions supported)
SELECT NTILE(3) OVER (ORDER BY age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with COUNT with OVER
SELECT COUNT(*) OVER (PARTITION BY country) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with SUM with OVER
SELECT SUM(age) OVER (PARTITION BY country) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with AVG with OVER
SELECT AVG(age) OVER (PARTITION BY country) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MAX with OVER
SELECT MAX(age) OVER (PARTITION BY country) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MIN with OVER
SELECT MIN(age) OVER (PARTITION BY country) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_ARRAY
SELECT JSON_ARRAY(1, 2, 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_OBJECT
SELECT JSON_OBJECT('name', name, 'age', age) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_QUOTE
SELECT JSON_QUOTE('test') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_UNQUOTE
SELECT JSON_UNQUOTE('"test"') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_TYPE
SELECT JSON_TYPE('[1,2,3]') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_VALID
SELECT JSON_VALID('{"a":1}') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_LENGTH
SELECT JSON_LENGTH('{"a":1}') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_DEPTH
SELECT JSON_DEPTH('[1,[2,3]]') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_KEYS
SELECT JSON_KEYS('{"a":1,"b":2}') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_EXTRACT
SELECT JSON_EXTRACT('{"a":1,"b":2}', '$.a') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_SET
SELECT JSON_SET('{"a":1}', '$.a', 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_INSERT
SELECT JSON_INSERT('{"a":1}', '$.b', 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_REPLACE
SELECT JSON_REPLACE('{"a":1}', '$.a', 2) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_REMOVE
SELECT JSON_REMOVE('{"a":1,"b":2}', '$.b') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_MERGE
SELECT JSON_MERGE('[1,2]', '[3,4]') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_MERGE_PATCH
SELECT JSON_MERGE_PATCH('{"a":1}', '{"b":2}') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_MERGE_PRESERVE
SELECT JSON_MERGE_PRESERVE('{"a":1}', '{"b":2}') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_APPEND
SELECT JSON_APPEND('[1,2]', '$', 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_ARRAY_APPEND
SELECT JSON_ARRAY_APPEND('[1,2]', '$', 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with JSON_ARRAY_INSERT
SELECT JSON_ARRAY_INSERT('[1,2]', '$[0]', 3) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MATCH AGAINST basic
SELECT * FROM users WHERE MATCH(name) AGAINST('Alice');
-- EXPECT: rows 1

-- === CASE: Select with MATCH AGAINST boolean
SELECT * FROM users WHERE MATCH(name) AGAINST('+Alice -Bob' IN BOOLEAN MODE);
-- EXPECT: rows 1

-- === CASE: Select with MATCH AGAINST with wildcards
SELECT * FROM users WHERE MATCH(name) AGAINST('Ali*' IN BOOLEAN MODE);
-- EXPECT: rows 1

-- === CASE: Select with MATCH AGAINST natural language
SELECT * FROM users WHERE MATCH(name) AGAINST('Alice' IN NATURAL LANGUAGE MODE);
-- EXPECT: rows 1

-- === CASE: Select with MATCH AGAINST query expansion
SELECT * FROM users WHERE MATCH(name) AGAINST('Alice' WITH QUERY EXPANSION);
-- EXPECT: rows 1

-- === CASE: Select with SOUNDEX
SELECT SOUNDEX('Smith') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with DIFFERENCE
SELECT DIFFERENCE('Smith', 'Smythe') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MATCH in multiple columns
SELECT * FROM users WHERE MATCH(name, email) AGAINST('Alice');
-- EXPECT: rows 1

-- === CASE: Select with AGAINST in boolean mode with quotes
SELECT * FROM users WHERE MATCH(name) AGAINST('"Alice"' IN BOOLEAN MODE);
-- EXPECT: rows 1

-- === CASE: Select with AGAINST with no noise words
SELECT * FROM users WHERE MATCH(name) AGAINST('the' IN BOOLEAN MODE);
-- EXPECT: rows 0

-- === CASE: Select with ST_distance
SELECT ST_distance(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geomfromtext
SELECT ST_geomfromtext('POINT(1 1)') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_astext
SELECT ST_astext(ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_x
SELECT ST_x(ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_y
SELECT ST_y(ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_srid
SELECT ST_srid(ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_dimension
SELECT ST_dimension(ST_geomfromtext('LINESTRING(0 0,1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_equals
SELECT ST_equals(ST_geomfromtext('POINT(1 1)'), ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_disjoint
SELECT ST_disjoint(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_intersects
SELECT ST_intersects(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_touches
SELECT ST_touches(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('LINESTRING(0 0,0 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_crosses
SELECT ST_crosses(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('LINESTRING(0 0,1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_within
SELECT ST_within(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_contains
SELECT ST_contains(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))'), ST_geomfromtext('POINT(0.5 0.5)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_overlaps
SELECT ST_overlaps(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))'), ST_geomfromtext('POLYGON((0.5 0.5,1.5 0.5,1.5 1.5,0.5 1.5,0.5 0.5))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_buffer
SELECT ST_astext(ST_buffer(ST_geomfromtext('POINT(0 0)'), 1)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_centroid
SELECT ST_astext(ST_centroid(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_area
SELECT ST_area(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_length
SELECT ST_length(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_exteriorring
SELECT ST_astext(ST_exteriorring(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_interiorringn
SELECT ST_astext(ST_interiorringn(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))'), 0)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geometryn
SELECT ST_astext(ST_geometryn(ST_geomfromtext('MULTIPOINT(0 0,1 1,2 2)'), 1)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_numgeometries
SELECT ST_numgeometries(ST_geomfromtext('MULTIPOINT(0 0,1 1,2 2)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_numinteriorring
SELECT ST_numinteriorring(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_numpoints
SELECT ST_numpoints(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_pointn
SELECT ST_astext(ST_pointn(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)'), 1)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_startpoint
SELECT ST_astext(ST_startpoint(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_endpoint
SELECT ST_astext(ST_endpoint(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_polygon
SELECT ST_astext(ST_polygon(ST_geomfromtext('LINESTRING(0 0,1 0,1 1,0 1,0 0)'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geomcollfromtext
SELECT ST_geomcollfromtext('GEOMETRYCOLLECTION(POINT(0 0),LINESTRING(0 0,1 1))') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geomfromwkb
SELECT ST_geomfromwkb(ST_aswkb(ST_geomfromtext('POINT(1 1)'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_linefromwkb
SELECT ST_astext(ST_linefromwkb(ST_aswkb(ST_geomfromtext('LINESTRING(0 0,1 1)')))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_pointfromwkb
SELECT ST_astext(ST_pointfromwkb(ST_aswkb(ST_geomfromtext('POINT(1 1)')))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_polyfromwkb
SELECT ST_astext(ST_polyfromwkb(ST_aswkb(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_mlinefromwkb
SELECT ST_astext(ST_mlinefromwkb(ST_aswkb(ST_geomfromtext('MULTILINESTRING((0 0,1 1),(2 2,3 3)')))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_mpointfromwkb
SELECT ST_astext(ST_mpointfromwkb(ST_aswkb(ST_geomfromtext('MULTIPOINT(0 0,1 1)')))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_mpolyfromwkb
SELECT ST_astext(ST_mpolyfromwkb(ST_aswkb(ST_geomfromtext('MULTIPOLYGON(((0 0,1 0,1 1,0 1,0 0)))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_wkbtosql
SELECT ST_wkbtosql(ST_aswkb(ST_geomfromtext('POINT(1 1)'))) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_wktosql
SELECT ST_wktosql(ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with GeometryCollection
SELECT ST_astext(ST_geomfromtext('GEOMETRYCOLLECTION(POINT(0 0),LINESTRING(0 0,1 1))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MultiPoint
SELECT ST_astext(ST_geomfromtext('MULTIPOINT(0 0,1 1,2 2)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MultiLineString
SELECT ST_astext(ST_geomfromtext('MULTILINESTRING((0 0,1 1),(2 2,3 3))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with MultiPolygon
SELECT ST_astext(ST_geomfromtext('MULTIPOLYGON(((0 0,1 0,1 1,0 1,0 0)))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_isclosed
SELECT ST_isclosed(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_isring
SELECT ST_isring(ST_geomfromtext('LINESTRING(0 0,1 1,0 0)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_issimple
SELECT ST_issimple(ST_geomfromtext('POINT(0 0)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_overunder
SELECT ST_overunder(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POINT(1 1)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_orderingequals
SELECT ST_orderingequals(ST_geomfromtext('POINT(0 0)'), ST_geomfromtext('POINT(0 0)')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_snap
SELECT ST_astext(ST_snap(ST_geomfromtext('LINESTRING(0 0,2 2)'), ST_geomfromtext('POINT(1 1)'), 1)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_transform
SELECT ST_astext(ST_transform(ST_geomfromtext('POINT(0 0)'), 4326)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geohash
SELECT ST_geohash(ST_geomfromtext('POINT(0 0)'), 10) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_geomfromgeohash
SELECT ST_astext(ST_geomfromgeohash('s09uni00')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_latfromgeohash
SELECT ST_latfromgeohash('s09uni00') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_longfromgeohash
SELECT ST_longfromgeohash('s09uni00') FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_pointfromgeohash
SELECT ST_astext(ST_pointfromgeohash('s09uni00')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_sroverlaps
SELECT ST_sroverlaps(ST_geomfromtext('POINT(0 0)'), 4326, ST_geomfromtext('POINT(0 0)'), 4326) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_isvalid
SELECT ST_isvalid(ST_geomfromtext('POLYGON((0 0,1 0,1 1,0 1,0 0))')) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_makeenvelope
SELECT ST_astext(ST_makeenvelope(-180, -90, 180, 90)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_dump
SELECT ST_astext((ST_dump(ST_geomfromtext('MULTIPOINT(0 0,1 1)')).geom)) FROM users;
-- EXPECT: rows 10

-- === CASE: Select with ST_dumppoints
SELECT (ST_dumppoints(ST_geomfromtext('LINESTRING(0 0,1 1,2 2)')).pt) FROM users;
-- EXPECT: rows 10
