-- UNION and LIMIT Test Cases
-- Compatibility: MySQL 5.7+

SELECT name FROM users WHERE status = 'active'
UNION
SELECT name FROM users WHERE age > 65;

SELECT name FROM products WHERE price > 100
UNION ALL
SELECT name FROM products WHERE category_id = 1;

SELECT name, email FROM users WHERE status = 'active'
UNION
SELECT name, email FROM users WHERE status = 'vip';

SELECT name FROM users WHERE age < 18
UNION
SELECT name FROM users WHERE age > 65
UNION
SELECT name FROM users WHERE status = 'inactive';

SELECT name, 'user' AS type FROM users
UNION ALL
SELECT name, 'admin' AS type FROM admins;

SELECT id, name FROM products WHERE price > 50
ORDER BY price DESC
LIMIT 10;

SELECT id, name FROM products
ORDER BY id
LIMIT 5 OFFSET 10;

SELECT name FROM users WHERE status = 'active'
UNION
SELECT name FROM admins
ORDER BY name
LIMIT 20;

SELECT name FROM products WHERE stock < 10
UNION ALL
SELECT name FROM products WHERE stock = 0
LIMIT 5;

SELECT id, name, price FROM products
WHERE price > 100
UNION
SELECT id, name, price FROM products
WHERE category_id = 1
ORDER BY price DESC
LIMIT 10;

SELECT name FROM users WHERE country = 'USA'
UNION
SELECT name FROM users WHERE country = 'Canada'
ORDER BY name
LIMIT 50;

SELECT name, price FROM products WHERE price > 200
UNION ALL
SELECT name, price FROM products WHERE price < 50
LIMIT 20;

SELECT name, email FROM users WHERE email LIKE '%@company.com'
UNION
SELECT name, email FROM users WHERE email LIKE '%@partner.com'
ORDER BY email
LIMIT 100;

SELECT name FROM categories WHERE id IN (SELECT category_id FROM products WHERE stock > 0)
UNION
SELECT name FROM categories WHERE id IN (SELECT category_id FROM products GROUP BY category_id HAVING SUM(stock) > 1000);

SELECT name FROM products WHERE name LIKE '%pro%'
UNION
SELECT name FROM products WHERE name LIKE '%premium%'
UNION
SELECT name FROM products WHERE name LIKE '%elite%'
LIMIT 50;

SELECT id, name, price FROM products
ORDER BY price
LIMIT 1;

SELECT id, name, price FROM products
ORDER BY price
LIMIT 0, 5;

SELECT name FROM users WHERE status = 'pending'
UNION ALL
SELECT name FROM users WHERE status = 'inactive'
UNION ALL
SELECT name FROM users WHERE status = 'banned'
ORDER BY name
LIMIT 100;

SELECT product_id, SUM(quantity) FROM order_items GROUP BY product_id
UNION
SELECT id AS product_id, stock AS sum_quantity FROM products
LIMIT 20;

SELECT name, price, 'high price' AS category FROM products WHERE price > 150
UNION
SELECT name, price, 'medium price' FROM products WHERE price BETWEEN 50 AND 150
UNION
SELECT name, price, 'low price' FROM products WHERE price < 50
ORDER BY price DESC
LIMIT 100;

SELECT name FROM users WHERE YEAR(created_at) = 2024
UNION
SELECT name FROM users WHERE status = 'vip'
ORDER BY name
LIMIT 30;

SELECT name, price FROM products WHERE category_id = 1
UNION
SELECT name, price FROM products WHERE category_id = 2
UNION
SELECT name, price FROM products WHERE category_id = 3
ORDER BY price DESC
LIMIT 20;

SELECT name FROM users WHERE LENGTH(name) > 20
UNION
SELECT name FROM users WHERE LENGTH(name) < 5
ORDER BY LENGTH(name)
LIMIT 50;

SELECT name, price, stock FROM products WHERE stock > 0
ORDER BY price DESC
LIMIT 10;

SELECT name FROM users WHERE email LIKE '%@gmail.com'
UNION
SELECT name FROM users WHERE email LIKE '%@yahoo.com'
UNION
SELECT name FROM users WHERE email LIKE '%@hotmail.com'
ORDER BY name
LIMIT 100;

SELECT id, name, price FROM products
ORDER BY id
LIMIT 100, 50;

SELECT name, price FROM products WHERE name LIKE '%widget%'
UNION ALL
SELECT name, price FROM products WHERE name LIKE '%gadget%'
UNION ALL
SELECT name, price FROM products WHERE name LIKE '%tool%'
ORDER BY price
LIMIT 50;

SELECT name, age FROM users WHERE age BETWEEN 18 AND 25
UNION
SELECT name, age FROM users WHERE age BETWEEN 26 AND 35
UNION
SELECT name, age FROM users WHERE age BETWEEN 36 AND 45
ORDER BY age
LIMIT 100;

SELECT name FROM products WHERE price = (SELECT MIN(price) FROM products WHERE price > 0)
UNION
SELECT name FROM products WHERE price = (SELECT MAX(price) FROM products)
LIMIT 10;

SELECT name FROM categories WHERE id NOT IN (SELECT category_id FROM products)
UNION
SELECT name FROM categories WHERE id IN (SELECT category_id FROM products GROUP BY category_id HAVING COUNT(*) > 10)
ORDER BY name
LIMIT 50;

SELECT name, total FROM (SELECT name, total FROM users u JOIN orders o ON u.id = o.user_id ORDER BY total DESC LIMIT 10) t
UNION
SELECT name, 0 AS total FROM users WHERE id NOT IN (SELECT user_id FROM orders)
ORDER BY total DESC
LIMIT 20;

SELECT name FROM products WHERE category_id = 1 AND price > 100
UNION
SELECT name FROM products WHERE category_id = 2 AND price > 100
UNION
SELECT name FROM products WHERE category_id = 3 AND price > 100
ORDER BY name
LIMIT 50;

SELECT id, name FROM (SELECT id, name FROM users ORDER BY created_at DESC LIMIT 100) t
UNION
SELECT id, name FROM users WHERE status = 'admin'
ORDER BY name
LIMIT 50;

SELECT name, price FROM products WHERE price > 0 AND price <= 50
UNION
SELECT name, price FROM products WHERE price > 50 AND price <= 100
UNION
SELECT name, price FROM products WHERE price > 100 AND price <= 200
UNION
SELECT name, price FROM products WHERE price > 200
ORDER BY price
LIMIT 100;

SELECT name FROM users WHERE status = 'active' AND age > 30
UNION ALL
SELECT name FROM users WHERE status = 'vip'
ORDER BY name
LIMIT 30;

SELECT name FROM products WHERE name LIKE '%a%' AND name LIKE '%e%'
UNION
SELECT name FROM products WHERE name LIKE '%i%' AND name LIKE '%o%'
UNION
SELECT name FROM products WHERE name LIKE '%u%'
ORDER BY name
LIMIT 50;

SELECT name, price FROM products ORDER BY price DESC LIMIT 5
UNION
SELECT name, price FROM products ORDER BY price ASC LIMIT 5;

SELECT name FROM users WHERE country = 'USA' AND status = 'active'
UNION
SELECT name FROM users WHERE country = 'UK' AND status = 'active'
UNION
SELECT name FROM users WHERE country = 'Canada' AND status = 'active'
ORDER BY name
LIMIT 100;

SELECT id, name, price FROM products WHERE id IN (SELECT product_id FROM order_items GROUP BY product_id HAVING SUM(quantity) > 100)
UNION
SELECT id, name, price FROM products WHERE stock > 500
ORDER BY price DESC
LIMIT 50;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'completed')
UNION
SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'shipped')
ORDER BY name
LIMIT 100;

SELECT name, price FROM products WHERE price > (SELECT AVG(price) FROM products)
UNION
SELECT name, price FROM products WHERE price < (SELECT MIN(price) FROM products WHERE price > 0)
ORDER BY price
LIMIT 50;

SELECT name, total FROM (SELECT u.name, SUM(o.total) AS total FROM users u JOIN orders o ON u.id = o.user_id GROUP BY u.id, u.name ORDER BY total DESC LIMIT 20) t
UNION
SELECT name, 0 AS total FROM users WHERE id NOT IN (SELECT user_id FROM orders)
ORDER BY total DESC
LIMIT 30;

SELECT name FROM categories WHERE id IN (1, 2, 3)
UNION
SELECT name FROM categories WHERE id IN (4, 5, 6)
UNION
SELECT name FROM categories WHERE id IN (7, 8, 9)
ORDER BY name
LIMIT 100;

SELECT name, stock FROM products WHERE stock > 0 ORDER BY stock DESC LIMIT 10
UNION
SELECT name, 0 AS stock FROM products WHERE stock = 0
ORDER BY stock DESC
LIMIT 20;

SELECT name FROM users WHERE LENGTH(email) > 30
UNION
SELECT name FROM users WHERE email NOT LIKE '%@%.%'
UNION
SELECT name FROM users WHERE email LIKE '%@%@%'
ORDER BY name
LIMIT 50;

SELECT name FROM products WHERE category_id = 1 AND price > 50
UNION ALL
SELECT name FROM products WHERE category_id = 2 AND price > 50
UNION ALL
SELECT name FROM products WHERE category_id = 3 AND price > 50
UNION ALL
SELECT name FROM products WHERE category_id = 4 AND price > 50
UNION ALL
SELECT name FROM products WHERE category_id = 5 AND price > 50
ORDER BY name
LIMIT 100;

SELECT name, price FROM (SELECT name, price FROM products ORDER BY price DESC LIMIT 5) t
UNION
SELECT 'AVERAGE', (SELECT AVG(price) FROM products)
UNION
SELECT 'MIN', MIN(price) FROM products
UNION
SELECT 'MAX', MAX(price) FROM products;

SELECT name FROM users WHERE DAYOFWEEK(created_at) = 1
UNION
SELECT name FROM users WHERE DAYOFWEEK(created_at) = 7
ORDER BY name
LIMIT 50;

SELECT name, price FROM products WHERE price BETWEEN 10 AND 50
UNION
SELECT name, price FROM products WHERE price BETWEEN 51 AND 100
UNION
SELECT name, price FROM products WHERE price BETWEEN 101 AND 200
UNION
SELECT name, price FROM products WHERE price > 200
ORDER BY price
LIMIT 100;

SELECT name FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders GROUP BY user_id HAVING SUM(total) > 1000)
UNION
SELECT name FROM users WHERE status = 'vip'
ORDER BY name
LIMIT 50;

SELECT name, created_at FROM (SELECT name, created_at FROM users ORDER BY created_at DESC LIMIT 10) t
UNION
SELECT name, created_at FROM users WHERE status = 'new'
ORDER BY created_at DESC
LIMIT 20;

SELECT name FROM products WHERE id IN (SELECT product_id FROM order_items WHERE quantity > 10 GROUP BY product_id)
UNION
SELECT name FROM products WHERE stock > 200
ORDER BY name
LIMIT 50;

SELECT name, price FROM products ORDER BY RAND() LIMIT 5
UNION
SELECT name, price FROM products ORDER BY RAND() LIMIT 5
UNION
SELECT name, price FROM products ORDER BY RAND() LIMIT 5;

SELECT name FROM categories WHERE name LIKE '%a%'
UNION
SELECT name FROM categories WHERE name LIKE '%e%'
UNION
SELECT name FROM categories WHERE name LIKE '%i%'
UNION
SELECT name FROM categories WHERE name LIKE '%o%'
UNION
SELECT name FROM categories WHERE name LIKE '%u%'
ORDER BY name
LIMIT 100;

SELECT name FROM users WHERE created_at > (SELECT MIN(created_at) FROM users WHERE status = 'active')
UNION
SELECT name FROM users WHERE status = 'inactive'
ORDER BY name
LIMIT 50;

SELECT name, stock FROM products WHERE stock = 0
UNION ALL
SELECT name, stock FROM products WHERE stock < 10
UNION ALL
SELECT name, stock FROM products WHERE stock BETWEEN 10 AND 50
UNION ALL
SELECT name, stock FROM products WHERE stock > 50
ORDER BY stock DESC
LIMIT 100;

SELECT name FROM users WHERE country IN ('USA', 'Canada', 'Mexico')
UNION
SELECT name FROM users WHERE country IN ('UK', 'France', 'Germany')
UNION
SELECT name FROM users WHERE country IN ('Japan', 'China', 'Korea')
ORDER BY name
LIMIT 100;

SELECT name, price FROM products WHERE name LIKE '%new%'
UNION
SELECT name, price FROM products WHERE name LIKE '%latest%'
UNION
SELECT name, price FROM products WHERE name LIKE '%2024%'
UNION
SELECT name, price FROM products WHERE name LIKE '%2025%'
ORDER BY name
LIMIT 50;

SELECT name FROM users WHERE id NOT IN (SELECT DISTINCT user_id FROM orders)
UNION
SELECT name FROM users WHERE status = 'vip'
ORDER BY name
LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 1)
UNION
SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 2)
UNION
SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 3)
ORDER BY price DESC
LIMIT 20;

SELECT name FROM users WHERE YEAR(created_at) = 2024 AND MONTH(created_at) BETWEEN 1 AND 6
UNION
SELECT name FROM users WHERE YEAR(created_at) = 2024 AND MONTH(created_at) BETWEEN 7 AND 12
ORDER BY name
LIMIT 100;

SELECT name FROM products WHERE category_id = 1 ORDER BY name LIMIT 10
UNION
SELECT name FROM products WHERE category_id = 2 ORDER BY name LIMIT 10
UNION
SELECT name FROM products WHERE category_id = 3 ORDER BY name LIMIT 10
UNION
SELECT name FROM products WHERE category_id = 4 ORDER BY name LIMIT 10
UNION
SELECT name FROM products WHERE category_id = 5 ORDER BY name LIMIT 10;

SELECT name, price FROM products WHERE price > (SELECT AVG(price) FROM products WHERE category_id = 1)
UNION
SELECT name, price FROM products WHERE price > (SELECT AVG(price) FROM products WHERE category_id = 2)
ORDER BY price
LIMIT 50;

SELECT name FROM users WHERE email LIKE '%@company.com' AND status = 'active'
UNION
SELECT name FROM users WHERE email LIKE '%@partner.com' AND status = 'active'
UNION
SELECT name FROM users WHERE email LIKE '%@vendor.com' AND status = 'active'
ORDER BY name
LIMIT 100;

SELECT name, total FROM (SELECT u.name, SUM(o.total) AS total FROM users u JOIN orders o ON u.id = o.user_id WHERE o.status = 'completed' GROUP BY u.id, u.name ORDER BY total DESC LIMIT 10) t
UNION
SELECT name, 0 AS total FROM users WHERE status = 'new'
ORDER BY total DESC
LIMIT 20;

SELECT name FROM products WHERE name LIKE '%pro%' AND price > 100
UNION
SELECT name FROM products WHERE name LIKE '%ultra%' AND price > 100
UNION
SELECT name FROM products WHERE name LIKE '%premium%' AND price > 100
UNION
SELECT name FROM products WHERE name LIKE '%elite%' AND price > 100
ORDER BY name
LIMIT 50;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 10
UNION
SELECT name FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 10
UNION
SELECT name FROM users WHERE status = 'pending' ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY price LIMIT 20
UNION
SELECT 'TOTAL AVERAGE', (SELECT AVG(price) FROM products WHERE price > 0);

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE YEAR(created_at) = 2024 GROUP BY user_id HAVING COUNT(*) > 10)
UNION
SELECT name FROM users WHERE status = 'vip'
ORDER BY name
LIMIT 50;

SELECT name, stock FROM products WHERE stock > 500 ORDER BY stock DESC LIMIT 10
UNION
SELECT name, stock FROM products WHERE stock = 0 ORDER BY name LIMIT 10
UNION
SELECT name, stock FROM products WHERE stock BETWEEN 1 AND 10 ORDER BY stock LIMIT 10;

SELECT name FROM categories WHERE id IN (SELECT category_id FROM products WHERE price > 100 GROUP BY category_id)
UNION
SELECT name FROM categories WHERE id IN (SELECT category_id FROM products WHERE stock > 100 GROUP BY category_id)
ORDER BY name
LIMIT 50;

SELECT name FROM products WHERE name = (SELECT name FROM products ORDER BY created_at DESC LIMIT 1)
UNION
SELECT name FROM products WHERE name = (SELECT name FROM products ORDER BY created_at ASC LIMIT 1)
UNION
SELECT name FROM products WHERE name = (SELECT name FROM products ORDER BY price DESC LIMIT 1)
UNION
SELECT name FROM products WHERE name = (SELECT name FROM products ORDER BY price ASC LIMIT 1);

SELECT name FROM users WHERE country = 'USA' AND status = 'active'
UNION ALL
SELECT name FROM users WHERE country = 'USA' AND status = 'inactive'
UNION ALL
SELECT name FROM users WHERE country = 'Canada' AND status = 'active'
UNION ALL
SELECT name FROM users WHERE country = 'Canada' AND status = 'inactive'
ORDER BY country, status, name
LIMIT 100;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 5
UNION
SELECT name, price FROM products WHERE category_id = 2 ORDER BY price DESC LIMIT 5
UNION
SELECT name, price FROM products WHERE category_id = 3 ORDER BY price DESC LIMIT 5
UNION
SELECT name, price FROM products WHERE category_id = 4 ORDER BY price DESC LIMIT 5
UNION
SELECT name, price FROM products WHERE category_id = 5 ORDER BY price DESC LIMIT 5;

SELECT name FROM users WHERE age < 18
UNION
SELECT name FROM users WHERE age BETWEEN 18 AND 25
UNION
SELECT name FROM users WHERE age BETWEEN 26 AND 35
UNION
SELECT name FROM users WHERE age BETWEEN 36 AND 50
UNION
SELECT name FROM users WHERE age > 50
ORDER BY age
LIMIT 100;

SELECT name, price FROM products WHERE name LIKE '%' || 'a' || '%' AND price > 50
UNION
SELECT name, price FROM products WHERE name LIKE '%' || 'e' || '%' AND price > 50
UNION
SELECT name, price FROM products WHERE name LIKE '%' || 'i' || '%' AND price > 50
UNION
SELECT name, price FROM products WHERE name LIKE '%' || 'o' || '%' AND price > 50
UNION
SELECT name, price FROM products WHERE name LIKE '%' || 'u' || '%' AND price > 50
ORDER BY name
LIMIT 100;
