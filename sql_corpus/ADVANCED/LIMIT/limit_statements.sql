-- LIMIT and OFFSET Test Cases
-- Compatibility: MySQL 5.7+

SELECT * FROM users LIMIT 10;

SELECT * FROM products ORDER BY price DESC LIMIT 10;

SELECT * FROM orders ORDER BY created_at DESC LIMIT 20;

SELECT name, price FROM products LIMIT 5;

SELECT id, name FROM users ORDER BY created_at DESC LIMIT 100;

SELECT * FROM products WHERE price > 100 ORDER BY price LIMIT 50;

SELECT name, email FROM users LIMIT 0, 10;

SELECT * FROM products LIMIT 10, 20;

SELECT * FROM orders ORDER BY created_at LIMIT 0, 25;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY stock DESC LIMIT 5;

SELECT id, name, price FROM products ORDER BY id LIMIT 100 OFFSET 200;

SELECT name, age FROM users ORDER BY age ASC LIMIT 10;

SELECT * FROM products WHERE name LIKE '%widget%' ORDER BY price LIMIT 20;

SELECT id, name, total FROM orders ORDER BY total DESC LIMIT 15;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT name FROM categories LIMIT 10 OFFSET 0;

SELECT id, name, stock FROM products WHERE stock < 10 ORDER BY stock ASC LIMIT 20;

SELECT * FROM orders WHERE status = 'completed' ORDER BY created_at DESC LIMIT 100;

SELECT name, price FROM products WHERE price BETWEEN 50 AND 200 ORDER BY price LIMIT 30;

SELECT * FROM products ORDER BY created_at DESC LIMIT 5 OFFSET 10;

SELECT name, email FROM users WHERE email LIKE '%@company.com' LIMIT 50;

SELECT id, name, price FROM products ORDER BY price ASC LIMIT 10 OFFSET 100;

SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at DESC LIMIT 10;

SELECT name FROM users ORDER BY name LIMIT 100 OFFSET 0;

SELECT * FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 25;

SELECT id, name, age FROM users WHERE age > 30 ORDER BY age ASC LIMIT 20;

SELECT * FROM orders ORDER BY total DESC LIMIT 5 OFFSET 0;

SELECT name, price, stock FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 100;

SELECT * FROM users WHERE created_at > '2024-01-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE price > 0 ORDER BY price LIMIT 50 OFFSET 50;

SELECT name, created_at FROM users ORDER BY created_at LIMIT 25 OFFSET 0;

SELECT id, name FROM products WHERE name LIKE '%pro%' ORDER BY name LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 100;

SELECT name, price FROM products ORDER BY CHAR_LENGTH(name) DESC LIMIT 10;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at DESC LIMIT 25;

SELECT id, name, email FROM users WHERE email NOT LIKE '%@spam.com' LIMIT 100;

SELECT name, stock FROM products WHERE stock BETWEEN 10 AND 100 ORDER BY stock DESC LIMIT 30;

SELECT * FROM products ORDER BY RAND() LIMIT 5;

SELECT id, name, total FROM orders WHERE total > 100 ORDER BY total ASC LIMIT 50;

SELECT name, age FROM users WHERE age < 25 ORDER BY age DESC LIMIT 15;

SELECT * FROM products WHERE category_id IN (1, 2, 3) ORDER BY category_id, price DESC LIMIT 60;

SELECT name FROM users WHERE name LIKE 'A%' ORDER BY name LIMIT 100;

SELECT * FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) ORDER BY created_at DESC LIMIT 20;

SELECT id, name, price FROM products WHERE price = (SELECT MAX(price) FROM products) LIMIT 1;

SELECT name, created_at FROM users ORDER BY created_at ASC LIMIT 1;

SELECT * FROM products ORDER BY id DESC LIMIT 1;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 10 OFFSET 90;

SELECT * FROM orders WHERE status IN ('pending', 'processing') ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE name LIKE '%sale%' ORDER BY price ASC LIMIT 25;

SELECT * FROM users WHERE LENGTH(name) > 20 ORDER BY LENGTH(name) DESC LIMIT 20;

SELECT name, total FROM orders WHERE user_id = 1 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock = 0 ORDER BY name ASC LIMIT 30;

SELECT name FROM categories ORDER BY name ASC LIMIT ALL;

SELECT id, name, price FROM products WHERE price > 100 ORDER BY price DESC LIMIT 0, 10;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 100;

SELECT name, stock FROM products WHERE stock > 500 ORDER BY stock DESC LIMIT 10;

SELECT id, name, email FROM users WHERE email LIKE '%@%.com' LIMIT 100 OFFSET 0;

SELECT * FROM orders ORDER BY created_at DESC LIMIT 10 OFFSET 1000;

SELECT name, price FROM products WHERE price < 50 ORDER BY price ASC LIMIT 20;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50 OFFSET 0;

SELECT id, name, total FROM orders WHERE total > 500 ORDER BY total DESC LIMIT 25;

SELECT name, age FROM users WHERE age BETWEEN 18 AND 65 ORDER BY age ASC LIMIT 100;

SELECT * FROM products WHERE category_id = 1 ORDER BY created_at DESC LIMIT 20;

SELECT name FROM users ORDER BY name ASC LIMIT 1 OFFSET 0;

SELECT * FROM products ORDER BY name ASC LIMIT 50 OFFSET 0;

SELECT id, name, created_at FROM users WHERE created_at > '2024-06-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 10 OFFSET 5;

SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip') ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock < 50 ORDER BY stock ASC LIMIT 30;

SELECT id, name, email FROM users WHERE email IS NOT NULL ORDER BY email LIMIT 100;

SELECT * FROM products WHERE name LIKE '%premium%' ORDER BY price DESC LIMIT 20;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at DESC LIMIT 25;

SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM orders) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 50 AND price < 150 ORDER BY price ASC LIMIT 40;

SELECT name, created_at FROM orders ORDER BY created_at DESC LIMIT 10 OFFSET 0;

SELECT * FROM products WHERE category_id = (SELECT id FROM categories WHERE name = 'Featured') ORDER BY name LIMIT 30;

SELECT name, price FROM products WHERE stock > 0 AND price > 0 ORDER BY price / stock DESC LIMIT 20;

SELECT * FROM users WHERE created_at BETWEEN '2024-01-01' AND '2024-06-30' ORDER BY created_at DESC LIMIT 100;

SELECT name, total FROM orders WHERE YEAR(created_at) = 2024 ORDER BY total DESC LIMIT 10;

SELECT id, name, stock FROM products ORDER BY stock DESC LIMIT 20 OFFSET 10;

SELECT * FROM users WHERE status = 'active' AND age > 30 ORDER BY created_at DESC LIMIT 50;

SELECT name FROM users WHERE DAYOFWEEK(created_at) = 1 ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE name LIKE '%' || 'a' || '%' AND price > 100 ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE MONTH(created_at) = MONTH(CURDATE()) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products ORDER BY id LIMIT 100 OFFSET 0;

SELECT * FROM users WHERE id IN (SELECT user_id FROM orders GROUP BY user_id HAVING COUNT(*) > 5) ORDER BY created_at DESC LIMIT 30;

SELECT name, stock FROM products WHERE stock > 100 ORDER BY stock DESC LIMIT 10 OFFSET 0;

SELECT id, name, email FROM users WHERE email LIKE '%@business.com' ORDER BY name LIMIT 50;

SELECT * FROM products WHERE price > (SELECT AVG(price) FROM products) ORDER BY price ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 20 OFFSET 0;

SELECT * FROM users ORDER BY created_at DESC LIMIT 1 OFFSET 999;

SELECT id, name, price FROM products WHERE category_id = 1 ORDER BY price ASC LIMIT 10;

SELECT name FROM users WHERE status = 'active' ORDER BY name ASC LIMIT 100 OFFSET 0;

SELECT * FROM products WHERE stock BETWEEN 1 AND 50 ORDER BY stock ASC LIMIT 30;

SELECT name, created_at FROM orders WHERE user_id = 1 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM categories ORDER BY name ASC LIMIT 50;

SELECT id, name, price FROM products WHERE name LIKE '%widget%' OR name LIKE '%gadget%' ORDER BY price LIMIT 30;

SELECT name, age FROM users WHERE age > 0 ORDER BY age DESC LIMIT 20 OFFSET 0;

SELECT * FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 1;

SELECT * FROM users WHERE status = 'pending' ORDER BY created_at DESC LIMIT 25;

SELECT id, name, total FROM orders WHERE total < 100 ORDER BY total ASC LIMIT 30;

SELECT name FROM products WHERE category_id = 1 UNION SELECT name FROM products WHERE category_id = 2 ORDER BY name LIMIT 50;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 100;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE price > 0) ORDER BY name LIMIT 10;

SELECT * FROM orders WHERE status = 'cancelled' ORDER BY created_at DESC LIMIT 20;

SELECT id, name, age FROM users WHERE age IS NOT NULL ORDER BY age ASC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%new%' ORDER BY created_at DESC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock != 0 ORDER BY stock DESC LIMIT 25;

SELECT id, name, email FROM users WHERE email REGEXP '^[a-z]' ORDER BY name LIMIT 50;

SELECT * FROM products ORDER BY created_at DESC LIMIT 5 OFFSET 5;

SELECT name, total FROM orders WHERE status = 'refunded' ORDER BY total DESC LIMIT 20;

SELECT * FROM users WHERE LENGTH(name) > 0 AND LENGTH(name) < 10 ORDER BY LENGTH(name) DESC LIMIT 30;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 5 OFFSET 0;

SELECT * FROM orders WHERE user_id = 1 AND status = 'completed' ORDER BY created_at DESC LIMIT 10;

SELECT name FROM users WHERE status = 'active' ORDER BY RAND() LIMIT 10;

SELECT id, name, stock FROM products WHERE stock > 0 ORDER BY stock ASC, name ASC LIMIT 50;

SELECT * FROM users WHERE created_at < '2024-01-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%' ORDER BY name ASC LIMIT 100 OFFSET 0;

SELECT * FROM orders WHERE total > (SELECT AVG(total) FROM orders) ORDER BY total DESC LIMIT 50;

SELECT name, created_at FROM users WHERE status = 'vip' ORDER BY created_at DESC LIMIT 30;

SELECT id, name FROM products WHERE stock = (SELECT MAX(stock) FROM products) LIMIT 10;

SELECT * FROM orders ORDER BY id DESC LIMIT 20 OFFSET 0;

SELECT name, age FROM users WHERE age BETWEEN 20 AND 30 ORDER BY age ASC LIMIT 50;

SELECT name FROM users WHERE name REGEXP '^[AEIOU]' ORDER BY name LIMIT 50;

SELECT * FROM products WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 100;

SELECT name, price FROM products WHERE price < 50 ORDER BY price DESC LIMIT 30;

SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM orders WHERE status = 'completed') ORDER BY created_at DESC LIMIT 50;

SELECT id, name, total FROM orders WHERE user_id = 1 ORDER BY created_at ASC LIMIT 10;

SELECT name, stock FROM products ORDER BY stock DESC LIMIT 100 OFFSET 50;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name FROM categories WHERE id IN (SELECT category_id FROM products GROUP BY category_id ORDER BY COUNT(*) DESC LIMIT 5) ORDER BY name;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 0, 1;

SELECT * FROM orders WHERE QUARTER(created_at) = QUARTER(CURDATE()) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%basic%' OR name LIKE '%standard%' ORDER BY price LIMIT 30;

SELECT * FROM users WHERE MONTH(created_at) = 1 ORDER BY created_at DESC LIMIT 50;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders GROUP BY user_id HAVING SUM(total) > 1000) ORDER BY name LIMIT 50;

SELECT * FROM products WHERE category_id = 1 AND stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT name, total FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) ORDER BY total DESC LIMIT 10;

SELECT id, name, price FROM products ORDER BY id LIMIT 10 OFFSET 0;

SELECT * FROM users WHERE status = 'inactive' AND created_at > '2024-01-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE price BETWEEN (SELECT AVG(price) FROM products) * 0.9 AND (SELECT AVG(price) FROM products) * 1.1 ORDER BY price LIMIT 50;

SELECT * FROM orders WHERE DAY(created_at) = DAY(CURDATE()) ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock < (SELECT AVG(stock) FROM products) ORDER BY stock ASC LIMIT 30;

SELECT id, name, email FROM users WHERE email IS NOT NULL AND email != '' ORDER BY email ASC LIMIT 100;

SELECT * FROM products WHERE stock > 0 AND category_id = 1 ORDER BY price DESC LIMIT 20;

SELECT name, created_at FROM users ORDER BY created_at ASC LIMIT 5 OFFSET 0;

SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'active') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' AND price > 100 ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) >= 2024 ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 10;

SELECT id, name FROM products WHERE stock = 0 ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 MONTH) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 2 ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND age > 25 ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 200 ORDER BY price ASC LIMIT 20;

SELECT * FROM products WHERE name LIKE '%premium%' ORDER BY created_at DESC LIMIT 30;

SELECT name, total FROM orders WHERE user_id = (SELECT id FROM users WHERE status = 'vip' LIMIT 1) ORDER BY created_at DESC LIMIT 10;

SELECT id, name, stock FROM products ORDER BY stock ASC LIMIT 50 OFFSET 0;

SELECT * FROM users WHERE created_at > (SELECT created_at FROM users WHERE status = 'admin' LIMIT 1) ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 1) LIMIT 10;

SELECT * FROM orders WHERE status = 'completed' AND total > 0 ORDER BY created_at DESC LIMIT 100;

SELECT name FROM users WHERE LENGTH(email) > 20 ORDER BY LENGTH(email) DESC LIMIT 30;

SELECT * FROM products WHERE category_id = (SELECT id FROM categories ORDER BY name LIMIT 1) ORDER BY name LIMIT 30;

SELECT name, price FROM products WHERE price < 100 ORDER BY price DESC LIMIT 30;

SELECT id, name, created_at FROM users WHERE YEAR(created_at) = 2023 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id IN (SELECT user_id FROM orders GROUP BY user_id HAVING COUNT(*) > 10) ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products ORDER BY price DESC LIMIT 10 OFFSET 10;

SELECT * FROM users WHERE status = 'active' AND country = 'USA' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 100 AND 500 ORDER BY stock DESC LIMIT 30;

SELECT id, name, email FROM users WHERE email LIKE '%@company.com' ORDER BY name LIMIT 50;

SELECT * FROM products WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = 1 ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE total > 1000 ORDER BY total ASC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-01-01' AND created_at < '2024-07-01' ORDER BY created_at DESC LIMIT 100;

SELECT name, price FROM products WHERE name LIKE '%' || 'pro' || '%' ORDER BY name LIMIT 30;

SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at ASC LIMIT 10 OFFSET 0;

SELECT name FROM categories ORDER BY CHAR_LENGTH(name) DESC LIMIT 20;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY price ASC, name ASC LIMIT 50;

SELECT * FROM users WHERE status = 'pending' ORDER BY created_at ASC LIMIT 30;

SELECT name, total FROM orders WHERE status IN ('completed', 'shipped') ORDER BY total DESC LIMIT 50;

SELECT * FROM products WHERE category_id = 1 OR category_id = 2 OR category_id = 3 ORDER BY category_id, price DESC LIMIT 60;

SELECT id, name, created_at FROM users WHERE created_at < '2024-01-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > (SELECT MIN(price) FROM products WHERE price > 0) ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) - 1 ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock > 0 AND stock < 100 ORDER BY stock ASC LIMIT 30;

SELECT id, name, email FROM users WHERE email LIKE '%@business.com' OR email LIKE '%@corp.com' ORDER BY name LIMIT 50;

SELECT * FROM products WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 3 ORDER BY price DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND LENGTH(name) > 5 ORDER BY name ASC LIMIT 50;

SELECT name, total FROM orders WHERE total > 0 ORDER BY total ASC LIMIT 1;

SELECT * FROM products WHERE name LIKE '%elite%' ORDER BY price DESC LIMIT 20;

SELECT id, name, created_at FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 YEAR) ORDER BY created_at DESC LIMIT 100;

SELECT name, price FROM products WHERE price BETWEEN 50 AND 100 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE country = 'USA') ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products ORDER BY stock DESC LIMIT 10 OFFSET 5;

SELECT * FROM users WHERE status = 'vip' AND created_at > '2024-01-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE MONTHNAME(created_at) = 'January' ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY id ASC LIMIT 100 OFFSET 0;

SELECT name FROM users WHERE name = (SELECT name FROM users GROUP BY name HAVING COUNT(*) = 1 LIMIT 1) LIMIT 10;

SELECT * FROM products WHERE category_id = 1 AND stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE created_at < (SELECT created_at FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 1) ORDER BY created_at DESC LIMIT 30;

SELECT id, name, price FROM products WHERE stock = (SELECT MAX(stock) FROM products WHERE stock < 1000) ORDER BY name LIMIT 30;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT name, price FROM products WHERE name LIKE '%ultra%' ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'completed' GROUP BY user_id HAVING COUNT(*) > 5) ORDER BY created_at DESC LIMIT 30;

SELECT name, total FROM orders WHERE total BETWEEN 100 AND 500 ORDER BY total DESC LIMIT 30;

SELECT * FROM products ORDER BY created_at ASC LIMIT 5 OFFSET 0;

SELECT id, name, email FROM users WHERE email NOT LIKE '%@spam%' AND email NOT LIKE '%@temp%' ORDER BY name LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE stock > 0) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND QUARTER(created_at) = 1 ORDER BY created_at DESC LIMIT 100;

SELECT name, stock FROM products WHERE stock > 200 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 4 ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE created_at BETWEEN '2024-01-01' AND '2024-06-30' ORDER BY created_at DESC LIMIT 100;

SELECT name, total FROM orders WHERE user_id = 1 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 AND stock < 10 ORDER BY stock ASC LIMIT 20;

SELECT id, name, created_at FROM users WHERE created_at >= '2024-01-01' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%plus%' ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY created_at DESC LIMIT 50;

SELECT name FROM users WHERE LENGTH(name) = (SELECT MAX(LENGTH(name)) FROM users) LIMIT 10;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 0, 5;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, total FROM orders WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = MONTH(CURDATE()) ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE name LIKE '%compact%' ORDER BY name ASC LIMIT 30;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE YEAR(created_at) = 2024) ORDER BY name LIMIT 50;

SELECT * FROM products WHERE category_id = 2 ORDER BY created_at DESC LIMIT 20;

SELECT id, name, price FROM products WHERE price < (SELECT AVG(price) FROM products) ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 2 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name NOT IN (SELECT name FROM products WHERE price > 100) ORDER BY name LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 60 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND YEAR(created_at) = 2024 ORDER BY total DESC LIMIT 50;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 10 OFFSET 0;

SELECT id, name, stock FROM products WHERE stock > 100 ORDER BY stock DESC, name ASC LIMIT 30;

SELECT * FROM users WHERE email LIKE '%@' || 'company' || '.com' ORDER BY name LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE price > 100) ORDER BY name LIMIT 30;

SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%deluxe%' ORDER BY price DESC LIMIT 30;

SELECT * FROM products WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT name FROM users WHERE id = (SELECT user_id FROM orders ORDER BY total DESC LIMIT 1) LIMIT 10;

SELECT * FROM orders WHERE status = 'pending' AND user_id = 1 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products ORDER BY price DESC LIMIT 1 OFFSET 0;

SELECT * FROM users WHERE created_at > '2024-06-01' ORDER BY created_at ASC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at DESC LIMIT 20;

SELECT id, name, price FROM products WHERE category_id = 1 AND stock > 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders GROUP BY user_id HAVING COUNT(*) >= 3) ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE price BETWEEN (SELECT MIN(price) FROM products WHERE price > 0) AND (SELECT MAX(price) FROM products) ORDER BY price ASC LIMIT 50;

SELECT * FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 14 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock > 50 AND stock < 200 ORDER BY stock DESC LIMIT 30;

SELECT id, name, email FROM users WHERE email IS NOT NULL ORDER BY email DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%standard%' ORDER BY name ASC LIMIT 30;

SELECT name FROM users WHERE YEAR(created_at) = 2024 ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'cancelled' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 10 OFFSET 0;

SELECT * FROM users WHERE LENGTH(name) > 10 ORDER BY LENGTH(name) DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY id ASC LIMIT 50 OFFSET 100;

SELECT * FROM orders WHERE user_id = 3 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%mini%' ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at DESC LIMIT 30;

SELECT name, total FROM orders WHERE total > (SELECT AVG(total) FROM orders WHERE status = 'completed') ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT id, name, created_at FROM users WHERE created_at >= '2024-01-01' AND created_at < '2024-04-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' AND user_id = 1 ORDER BY created_at ASC LIMIT 10;

SELECT name, stock FROM products WHERE stock = (SELECT MIN(stock) FROM products WHERE stock > 0) ORDER BY name LIMIT 30;

SELECT * FROM users WHERE id NOT IN (SELECT user_id FROM orders WHERE status = 'completed') AND status = 'active' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE price < 100 ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 3 ORDER BY name ASC LIMIT 30;

SELECT name FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 20;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY price DESC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total ASC LIMIT 10;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 3 MONTH) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 150 ORDER BY price ASC LIMIT 30;

SELECT name, stock FROM products ORDER BY stock DESC LIMIT 5;

SELECT * FROM orders WHERE user_id = 4 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%luxury%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND created_at >= '2024-01-01' ORDER BY created_at ASC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 2 ORDER BY created_at DESC LIMIT 20;

SELECT name FROM orders WHERE user_id = 1 ORDER BY created_at ASC LIMIT 10;

SELECT * FROM products WHERE stock = 0 OR stock IS NULL ORDER BY name ASC LIMIT 30;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 2) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE status = 'pending' AND total > 0 ORDER BY total DESC LIMIT 30;

SELECT name FROM users WHERE country = 'USA' ORDER BY name ASC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%special%' ORDER BY name ASC LIMIT 30;

SELECT id, name, total FROM orders WHERE YEAR(created_at) = 2024 ORDER BY total DESC LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) AND status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price BETWEEN 75 AND 125 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip') ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 10 AND 100 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY created_at DESC LIMIT 50;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'pending') ORDER BY name LIMIT 50;

SELECT * FROM products WHERE category_id = 4 ORDER BY name ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' AND user_id = 1 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE LENGTH(email) < 20 ORDER BY name ASC LIMIT 30;

SELECT id, name, price FROM products WHERE name LIKE '%' ORDER BY id ASC LIMIT 100 OFFSET 50;

SELECT name, price FROM products WHERE price > (SELECT AVG(price) FROM products WHERE category_id = 1) ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) - 1 ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock < (SELECT AVG(stock) FROM products) ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 1;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY price DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 5 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%' || 'new' || '%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at > '2024-03-01' ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price < 30 ORDER BY price ASC LIMIT 30;

SELECT name FROM users WHERE id = (SELECT user_id FROM orders WHERE total = (SELECT MAX(total) FROM orders WHERE status = 'completed')) LIMIT 10;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 2 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE YEAR(created_at) = 2024 AND status = 'active' ORDER BY created_at ASC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 30;

SELECT id, name, price FROM products WHERE category_id = 1 ORDER BY created_at ASC LIMIT 20;

SELECT * FROM users WHERE email LIKE '%@' || 'company.com' ORDER BY name LIMIT 50;

SELECT name, stock FROM products WHERE stock > 300 ORDER BY stock DESC LIMIT 30;

SELECT * FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 WEEK) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%pro%' AND price > 100 ORDER BY price DESC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at ASC LIMIT 30;

SELECT id, name, price FROM products WHERE price BETWEEN 25 AND 75 ORDER BY price ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at ASC LIMIT 10;

SELECT * FROM products WHERE category_id = 3 AND stock > 0 ORDER BY name ASC LIMIT 30;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 5 OFFSET 95;

SELECT * FROM orders WHERE user_id = 6 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%ultra%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at BETWEEN '2024-01-01' AND '2024-12-31' ORDER BY created_at DESC LIMIT 100;

SELECT id, name, stock FROM products WHERE stock = 100 ORDER BY name ASC LIMIT 30;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE price > 50) ORDER BY name LIMIT 30;

SELECT * FROM orders WHERE status = 'cancelled' ORDER BY created_at DESC LIMIT 20;

SELECT name, total FROM orders WHERE user_id = 3 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at ASC LIMIT 20;

SELECT * FROM users WHERE status = 'vip' AND created_at >= '2024-01-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE category_id = 2 ORDER BY price ASC LIMIT 20;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, email FROM users WHERE email NOT LIKE '%@temp%' ORDER BY name LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 1 AND 100 ORDER BY stock ASC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%deluxe%' ORDER BY name ASC LIMIT 30;

SELECT name, total FROM orders WHERE user_id = 4 ORDER BY created_at ASC LIMIT 10;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 6 MONTH) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 75 ORDER BY price DESC LIMIT 30;

SELECT name FROM users WHERE status = 'active' ORDER BY name ASC LIMIT 50 OFFSET 50;

SELECT * FROM orders WHERE status = 'pending' AND user_id = 5 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%compact%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE YEAR(created_at) = 2024) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 4 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 6 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 0, 10;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at DESC LIMIT 30;

SELECT * FROM products WHERE stock > 0 AND stock < 25 ORDER BY stock ASC LIMIT 30;

SELECT name FROM users WHERE LENGTH(name) > 15 ORDER BY LENGTH(name) DESC LIMIT 20;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = 1 ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 5 ORDER BY name ASC LIMIT 30;

SELECT name, total FROM orders WHERE user_id = 7 ORDER BY created_at ASC LIMIT 10;

SELECT * FROM users WHERE created_at >= '2024-02-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%elite%' ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND total > 500 ORDER BY total DESC LIMIT 30;

SELECT id, name, stock FROM products WHERE stock > 150 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND country = 'UK' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 3) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 8 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 45 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 1 AND price > 50 ORDER BY price DESC LIMIT 20;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE name LIKE '%plus%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY name ASC LIMIT 50;

SELECT name, price FROM products WHERE price BETWEEN (SELECT AVG(price) FROM products) * 0.8 AND (SELECT AVG(price) FROM products) * 1.2 ORDER BY price ASC LIMIT 50;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY total DESC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock > 0 AND stock <= 50 ORDER BY stock DESC LIMIT 30;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'shipped') ORDER BY name LIMIT 50;

SELECT * FROM orders WHERE user_id = 9 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 2 ORDER BY created_at ASC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-05-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at ASC LIMIT 10;

SELECT * FROM products WHERE stock = (SELECT MAX(stock) FROM products WHERE category_id = 1) ORDER BY name LIMIT 30;

SELECT id, name, email FROM users WHERE email LIKE '%@business.com' ORDER BY name ASC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY id ASC LIMIT 25 OFFSET 75;

SELECT * FROM orders WHERE user_id = 10 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%standard%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price < (SELECT MIN(price) FROM products WHERE price > 100) ORDER BY name LIMIT 30;

SELECT name, total FROM orders WHERE status = 'cancelled' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE category_id = 3 AND stock > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 21 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%new%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 11 ORDER BY created_at ASC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'completed' AND user_id = 7 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 4 ORDER BY price ASC LIMIT 20;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%premium%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'completed' ORDER BY created_at ASC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 200 AND 400 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'completed') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 4) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 12 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price > 50 ORDER BY price DESC LIMIT 30;

SELECT name FROM users WHERE status = 'active' ORDER BY name ASC LIMIT 50 OFFSET 0;

SELECT * FROM orders WHERE status = 'pending' AND total > 100 ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= '2024-04-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 13 ORDER BY created_at ASC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at DESC LIMIT 30;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY name ASC LIMIT 50;

SELECT name FROM users WHERE id IN (SELECT user_id FROM orders WHERE status = 'pending') ORDER BY name LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 8 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%mini%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 2 MONTH) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 1 ORDER BY stock DESC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 75 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock = (SELECT MIN(stock) FROM products WHERE stock > 0) ORDER BY name LIMIT 30;

SELECT * FROM users WHERE status = 'vip' AND created_at >= '2024-01-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE category_id = 2 ORDER BY price ASC LIMIT 20;

SELECT * FROM orders WHERE user_id = 14 ORDER BY created_at ASC LIMIT 10;

SELECT * FROM users WHERE created_at >= '2024-03-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%ultra%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 9 ORDER BY created_at DESC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total ASC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at ASC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 3 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at ASC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%deluxe%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'shipped' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 50 AND 150 ORDER BY stock DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND country = 'Canada' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 5) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 15 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 25 ORDER BY price ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total ASC LIMIT 10;

SELECT * FROM products WHERE category_id = 4 AND stock > 0 ORDER BY price DESC LIMIT 20;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%elite%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock > 0 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'pending') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 2) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 16 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-07-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock = 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%plus%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 10 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY id DESC LIMIT 50;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50 OFFSET 100;

SELECT * FROM orders WHERE status = 'pending' AND total < 100 ORDER BY total ASC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%standard%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 75 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY price DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 17 ORDER BY created_at ASC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'completed' AND user_id = 11 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 2 ORDER BY created_at ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%compact%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY total ASC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 100 AND 300 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND created_at >= '2024-01-01' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 4) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 18 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY price DESC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 10 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 3 ORDER BY stock DESC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 20;

SELECT * FROM products WHERE stock > 25 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%luxury%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'completed' ORDER BY created_at ASC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock = (SELECT MAX(stock) FROM products WHERE category_id = 2) ORDER BY name LIMIT 30;

SELECT * FROM users WHERE status = 'vip' AND created_at >= '2024-01-01' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 19 ORDER BY created_at ASC LIMIT 10;

SELECT * FROM users WHERE created_at >= '2024-06-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%basic%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 12 ORDER BY created_at DESC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE category_id = 4 AND stock > 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%elite%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'completed') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 5) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 20 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price ASC LIMIT 20;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 5 MONTH) ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at ASC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 2 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at ASC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%plus%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'shipped' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 75 AND 175 ORDER BY stock DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND country = 'Australia' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 3) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 21 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 15 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 40 ORDER BY price ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total ASC LIMIT 10;

SELECT * FROM products WHERE category_id = 5 AND stock > 0 ORDER BY price DESC LIMIT 20;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%ultra%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock > 0 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'pending') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 1) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 22 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 3 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-08-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock = 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%compact%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 13 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY id DESC LIMIT 50;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50 OFFSET 150;

SELECT * FROM orders WHERE status = 'pending' AND total > 200 ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%standard%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 80 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 4 ORDER BY price ASC LIMIT 20;

SELECT * FROM orders WHERE user_id = 23 ORDER BY created_at ASC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 5 ORDER BY created_at ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%deluxe%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'completed' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 125 AND 325 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND created_at >= '2024-01-01' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 2) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 24 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 12 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 1 ORDER BY stock DESC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 30 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%elite%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'completed') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 3) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 25 ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 2 ORDER BY price DESC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-09-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 4 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%plus%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'shipped' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 50 AND 250 ORDER BY stock DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND country = 'Germany' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 5) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 26 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name NOT LIKE '%sale%' ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 18 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 60 ORDER BY price ASC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total ASC LIMIT 10;

SELECT * FROM products WHERE category_id = 3 AND stock > 0 ORDER BY price DESC LIMIT 20;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%basic%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, stock FROM products WHERE stock > 0 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'pending') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 4) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 27 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM users WHERE created_at >= '2024-10-01' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock = 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE name LIKE '%compact%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' AND user_id = 14 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY id DESC LIMIT 50;

SELECT name FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50 OFFSET 200;

SELECT * FROM orders WHERE status = 'pending' AND total > 300 ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%standard%' ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 85 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 5 ORDER BY price ASC LIMIT 20;

SELECT * FROM orders WHERE user_id = 28 ORDER BY created_at ASC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 10;

SELECT id, name, price FROM products WHERE category_id = 2 ORDER BY created_at ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 30;

SELECT name, price FROM products WHERE name LIKE '%ultra%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 AND status = 'completed' ORDER BY created_at DESC LIMIT 50;

SELECT name, stock FROM products WHERE stock BETWEEN 175 AND 375 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' AND created_at >= '2024-01-01' ORDER BY created_at ASC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MAX(price) FROM products WHERE category_id = 1) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 29 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY price DESC LIMIT 30;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 16 DAY) ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 3 ORDER BY stock DESC LIMIT 30;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 35 ORDER BY stock ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 10;

SELECT name, price FROM products WHERE name LIKE '%deluxe%' ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY name ASC LIMIT 50;

SELECT * FROM users WHERE status = 'active' AND id IN (SELECT user_id FROM orders WHERE status = 'completed') ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price = (SELECT MIN(price) FROM products WHERE category_id = 2) ORDER BY name LIMIT 20;

SELECT * FROM orders WHERE user_id = 30 ORDER BY created_at ASC LIMIT 10;
