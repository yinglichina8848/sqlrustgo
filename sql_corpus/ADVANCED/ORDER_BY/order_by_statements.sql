-- ORDER BY Test Cases
-- Compatibility: MySQL 5.7+

SELECT * FROM users ORDER BY name;

SELECT * FROM products ORDER BY price;

SELECT name, age FROM users ORDER BY age DESC;

SELECT id, name, price FROM products ORDER BY price ASC;

SELECT * FROM orders ORDER BY created_at DESC;

SELECT name, price FROM products ORDER BY name ASC;

SELECT id, name, total FROM orders ORDER BY total DESC;

SELECT name, created_at FROM users ORDER BY created_at ASC;

SELECT * FROM products ORDER BY price DESC, name ASC;

SELECT * FROM orders ORDER BY user_id, created_at DESC;

SELECT name, price, stock FROM products ORDER BY stock DESC, price ASC;

SELECT id, name, total FROM orders ORDER BY total ASC, created_at DESC;

SELECT * FROM users WHERE age > 25 ORDER BY age DESC, name ASC;

SELECT name, price FROM products WHERE price > 50 ORDER BY price DESC;

SELECT id, name, total FROM orders WHERE status = 'completed' ORDER BY total DESC;

SELECT name, created_at FROM users ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY price DESC LIMIT 5;

SELECT id, name, total FROM orders ORDER BY created_at DESC LIMIT 20;

SELECT name, age FROM users ORDER BY age ASC, name DESC;

SELECT * FROM products ORDER BY category_id, price ASC;

SELECT * FROM orders ORDER BY status, total DESC;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price ASC;

SELECT id, name, total FROM orders WHERE user_id = 1 ORDER BY created_at DESC;

SELECT name FROM users ORDER BY LENGTH(name) DESC;

SELECT name, price FROM products ORDER BY CHAR_LENGTH(name) ASC;

SELECT * FROM users ORDER BY COALESCE(last_name, first_name) ASC;

SELECT name, total FROM orders ORDER BY total DESC, created_at ASC;

SELECT id, name, price FROM products ORDER BY price DESC, created_at ASC;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC, name ASC;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC;

SELECT id, name, total FROM orders WHERE total > 100 ORDER BY total DESC;

SELECT name FROM users ORDER BY SUBSTRING(name, 1, 1) ASC;

SELECT * FROM products ORDER BY LEFT(name, 3), price DESC;

SELECT id, name, total FROM orders ORDER BY YEAR(created_at), MONTH(created_at), total DESC;

SELECT name, price FROM products ORDER BY UPPER(name) ASC;

SELECT * FROM users ORDER BY LOWER(email) ASC;

SELECT name, price FROM products ORDER BY price * quantity DESC;

SELECT id, name, price FROM products ORDER BY ABS(price - 100) ASC;

SELECT name, created_at FROM users ORDER BY DATE(created_at) DESC;

SELECT * FROM orders ORDER BY TIME(created_at) DESC;

SELECT name, total FROM orders ORDER BY HOUR(created_at) DESC;

SELECT id, name, total FROM orders ORDER BY DAYNAME(created_at) ASC;

SELECT name, price FROM products ORDER BY MONTHNAME(created_at) DESC;

SELECT * FROM users ORDER BY QUARTER(created_at) ASC, name DESC;

SELECT name, price FROM products ORDER BY WEEK(created_at) DESC;

SELECT id, name, total FROM orders ORDER BY DAYOFYEAR(created_at) ASC;

SELECT name, total FROM orders ORDER BY WEEKDAY(created_at) DESC;

SELECT * FROM products ORDER BY FLOOR(price / 10) DESC, price ASC;

SELECT name, age FROM users ORDER BY age % 10 ASC;

SELECT id, name, price FROM products ORDER BY price DIV 10 ASC, name DESC;

SELECT * FROM orders ORDER BY NULLIF(YEAR(created_at), 2023) DESC;

SELECT name, total FROM orders ORDER BY IFNULL(status, 'pending') ASC;

SELECT id, name, price FROM products WHERE category_id IS NOT NULL ORDER BY category_id, price DESC;

SELECT name, price FROM products ORDER BY CASE WHEN price > 100 THEN 0 ELSE 1 END, price ASC;

SELECT * FROM users ORDER BY FIELD(status, 'pending', 'active', 'completed');

SELECT name, total FROM orders ORDER BY FIND_IN_SET(status, 'pending,active,completed') ASC;

SELECT id, name, price FROM products ORDER BY INSTR(name, 'pro') DESC;

SELECT name, created_at FROM users ORDER BY STRCMP(LEFT(name, 1), 'A') DESC;

SELECT * FROM products ORDER BY MD5(name) ASC;

SELECT name, total FROM orders WHERE user_id IN (1, 2, 3) ORDER BY FIND_IN_SET(user_id, '3,1,2') ASC;

SELECT id, name, total FROM orders ORDER BY YEARWEEK(created_at) DESC;

SELECT name, price FROM products ORDER BY DECODE(category_id, 1, 0, 2, 1, 3, 2) ASC;

SELECT * FROM users ORDER BY TIMESTAMPDIFF(YEAR, birth_date, CURDATE()) DESC;

SELECT name, total FROM orders ORDER BY SUM(total) OVER (PARTITION BY user_id) DESC;

SELECT * FROM products ORDER BY ROW_NUMBER() OVER (ORDER BY price DESC) ASC;

SELECT name, price FROM products ORDER BY RANK() OVER (ORDER BY price DESC) ASC;

SELECT id, name, total FROM orders ORDER BY DENSE_RANK() OVER (ORDER BY total DESC) ASC;

SELECT name, price FROM products ORDER BY PERCENT_RANK() OVER (ORDER BY price) ASC;

SELECT * FROM users WHERE created_at > '2024-01-01' ORDER BY created_at DESC, name ASC;

SELECT name, total FROM orders WHERE status IN ('pending', 'completed') ORDER BY FIELD(status, 'pending', 'completed'), total DESC;

SELECT id, name, price FROM products WHERE price BETWEEN 50 AND 200 ORDER BY price ASC;

SELECT name, created_at FROM users ORDER BY created_at DESC LIMIT 100 OFFSET 0;

SELECT * FROM products ORDER BY price DESC LIMIT 10 OFFSET 10;

SELECT name, total FROM orders ORDER BY total DESC LIMIT 25 OFFSET 25;

SELECT id, name, total FROM orders ORDER BY created_at DESC LIMIT 50 OFFSET 0;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 5;

SELECT * FROM users ORDER BY last_login DESC NULLS LAST;

SELECT name, total FROM orders ORDER BY total ASC NULLS FIRST;

SELECT id, name, price FROM products ORDER BY stock DESC, price ASC NULLS LAST;

SELECT name, age FROM users ORDER BY age DESC NULLS LAST, name ASC;

SELECT * FROM orders ORDER BY status = 'completed' DESC, created_at DESC;

SELECT name, price FROM products ORDER BY status = 'active' DESC, price ASC;

SELECT id, name, total FROM orders ORDER BY user_id = 1 DESC, total DESC;

SELECT * FROM users WHERE age > 30 ORDER BY age DESC, created_at ASC;

SELECT name, total FROM orders ORDER BY status = 'pending', total DESC;

SELECT id, name, price FROM products ORDER BY category_id IS NULL, category_id, price DESC;

SELECT * FROM orders ORDER BY IF(status = 'completed', 0, 1), created_at DESC;

SELECT name, price FROM products ORDER BY COALESCE(discounted_price, price) DESC;

SELECT id, name, total FROM orders ORDER BY total > 0 DESC, total DESC;

SELECT name, created_at FROM users ORDER BY created_at > '2024-06-01' DESC, created_at DESC;

SELECT * FROM products ORDER BY stock = 0 DESC, stock ASC;

SELECT name, total FROM orders WHERE user_id = 1 ORDER BY total DESC, created_at ASC;

SELECT id, name, price FROM products WHERE category_id = 2 ORDER BY price DESC, name ASC;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC, name ASC;

SELECT name, price FROM products WHERE stock > 0 ORDER BY stock DESC, price ASC;

SELECT id, name, total FROM orders WHERE total > 500 ORDER BY total DESC, created_at ASC;

SELECT * FROM products ORDER BY LEFT(category, 1), price DESC;

SELECT name, created_at FROM users ORDER BY SUBSTRING_INDEX(email, '@', 1) ASC;

SELECT id, name, total FROM orders ORDER BY MONTH(created_at) DESC, DAY(created_at) DESC;

SELECT name, price FROM products ORDER BY CONCAT(category, name) ASC;

SELECT * FROM users WHERE age BETWEEN 20 AND 30 ORDER BY age ASC, name DESC;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price / stock DESC;

SELECT name, total FROM orders ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC, total DESC;

SELECT * FROM products WHERE category_id IS NOT NULL ORDER BY category_id, stock DESC, price ASC;

SELECT name, created_at FROM users WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC;

SELECT id, name, total FROM orders WHERE QUARTER(created_at) = 4 ORDER BY total DESC;

SELECT name, price FROM products WHERE price < 100 ORDER BY price ASC, name DESC;

SELECT * FROM users WHERE LENGTH(name) > 5 ORDER BY LENGTH(name) DESC, name ASC;

SELECT id, name, price FROM products ORDER BY price DESC, created_at ASC;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC;

SELECT * FROM products WHERE name LIKE '%pro%' ORDER BY price DESC;

SELECT name, created_at FROM users WHERE email LIKE '%@company.com' ORDER BY created_at DESC;

SELECT id, name, total FROM orders WHERE total > (SELECT AVG(total) FROM orders) ORDER BY total DESC;

SELECT name, price FROM products WHERE category_id IN (1, 2, 3) ORDER BY FIELD(category_id, 3, 1, 2), price ASC;

SELECT * FROM users ORDER BY DATE_FORMAT(created_at, '%Y%m%d') DESC;

SELECT name, total FROM orders ORDER BY LPAD(CAST(total AS CHAR), 10, '0') DESC;

SELECT id, name, price FROM products ORDER BY REPEAT(name, 1) ASC;

SELECT * FROM users WHERE status = 'vip' ORDER BY age DESC, created_at ASC;

SELECT name, price FROM products ORDER BY price * 1.1 DESC;

SELECT id, name, total FROM orders ORDER BY YEAR(created_at), total DESC;

SELECT name, created_at FROM users WHERE DAYOFWEEK(created_at) IN (1, 7) ORDER BY created_at DESC;

SELECT * FROM products ORDER BY REVERSE(name) ASC;

SELECT name, total FROM orders WHERE user_id = (SELECT id FROM users WHERE name = 'John') ORDER BY total DESC;

SELECT id, name, price FROM products WHERE stock = (SELECT MAX(stock) FROM products) ORDER BY price ASC;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY MONTHNAME(created_at), total DESC;

SELECT * FROM users ORDER BY id DESC LIMIT 100;

SELECT name, price FROM products ORDER BY price ASC LIMIT 50;

SELECT id, name, total FROM orders ORDER BY created_at DESC LIMIT 10 OFFSET 90;

SELECT name, created_at FROM users ORDER BY created_at ASC LIMIT 25 OFFSET 75;

SELECT * FROM products ORDER BY stock DESC, price ASC LIMIT 20;

SELECT name, age FROM users WHERE age > 25 ORDER BY age DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 100 ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 100;

SELECT name, total FROM users u JOIN orders o ON u.id = o.user_id ORDER BY o.total DESC;

SELECT p.name, c.name FROM products p JOIN categories c ON p.category_id = c.id ORDER BY c.name, p.price DESC;

SELECT name, price FROM products ORDER BY price DESC LIMIT 5 OFFSET 0
UNION
SELECT name, price FROM products ORDER BY price ASC LIMIT 5 OFFSET 0;

SELECT * FROM (SELECT * FROM products ORDER BY price DESC LIMIT 10) t ORDER BY name ASC;

SELECT name, price FROM products ORDER BY CASE category_id WHEN 1 THEN 0 WHEN 2 THEN 1 ELSE 2 END, price ASC;

SELECT * FROM users ORDER BY COALESCE(last_login, created_at) DESC;

SELECT name, total FROM orders WHERE user_id IN (1, 2, 3) ORDER BY FIELD(user_id, 2, 3, 1), total DESC;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY stock DESC, created_at ASC;

SELECT name, price FROM products ORDER BY LEFT(REVERSE(name), 1) ASC;

SELECT * FROM users WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = 1 ORDER BY created_at DESC;

SELECT id, name, total FROM orders ORDER BY CONCAT(YEAR(created_at), MONTH(created_at)) DESC;

SELECT name, price FROM products WHERE price > 0 ORDER BY price / NULLIF(stock, 0) DESC;

SELECT * FROM users WHERE status = 'active' ORDER BY IF(age > 30, 0, 1), age DESC;

SELECT name, total FROM orders ORDER BY total BETWEEN 100 AND 500 DESC, total DESC;

SELECT id, name, price FROM products ORDER BY category_id IS NULL, category_id, price DESC;

SELECT * FROM orders WHERE user_id = 1 AND status = 'completed' ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY MOD(age, 10) ASC, age DESC;

SELECT * FROM products ORDER BY price ASC, stock DESC, name ASC;

SELECT name, price FROM products WHERE name LIKE '%a%' ORDER BY price DESC, name ASC;

SELECT id, name, total FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) ORDER BY total DESC;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) ORDER BY created_at DESC;

SELECT name, total FROM orders ORDER BY DAY(created_at) = 15 DESC, DAY(created_at) ASC;

SELECT * FROM products ORDER BY REPLACE(name, ' ', '') ASC;

SELECT name, created_at FROM users ORDER BY TIMESTAMPDIFF(DAY, created_at, NOW()) ASC;

SELECT id, name, price FROM products WHERE category_id NOT IN (1, 2) ORDER BY category_id, price DESC;

SELECT name, total FROM orders WHERE user_id > 100 ORDER BY total DESC LIMIT 50;

SELECT * FROM users ORDER BY CONCAT(first_name, last_name) ASC;

SELECT name, price FROM products WHERE stock > 0 ORDER BY stock ASC, price DESC LIMIT 25;

SELECT id, name, total FROM orders WHERE MONTH(created_at) = MONTH(CURDATE()) ORDER BY total DESC;

SELECT * FROM products ORDER BY LTRIM(name) ASC;

SELECT name, price FROM products ORDER BY TRIM(name) ASC;

SELECT * FROM users ORDER BY UPPER(LEFT(name, 1)) DESC;

SELECT id, name, price FROM products WHERE price > (SELECT MIN(price) FROM products) ORDER BY price ASC;

SELECT name, total FROM orders ORDER BY NULLIF(total, 0) DESC;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY email DESC;

SELECT name, price FROM products WHERE category_id = (SELECT id FROM categories WHERE name = 'Electronics') ORDER BY price DESC;

SELECT id, name, total FROM orders WHERE HOUR(created_at) BETWEEN 9 AND 17 ORDER BY created_at DESC;

SELECT * FROM products WHERE name REGEXP '^[A-Z]' ORDER BY name ASC;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, total DESC;

SELECT * FROM users ORDER BY password IS NULL DESC, name ASC;

SELECT id, name, price FROM products ORDER BY price > 0 DESC, price ASC;

SELECT name, created_at FROM orders ORDER BY DATE(created_at) DESC, TIME(created_at) DESC;

SELECT * FROM products WHERE stock > 0 AND price > 0 ORDER BY price * stock DESC;

SELECT name, total FROM orders WHERE user_id = 1 OR user_id = 2 ORDER BY FIELD(user_id, 1, 2), total DESC;

SELECT * FROM users WHERE status IN ('active', 'vip') ORDER BY status = 'vip' DESC, name ASC;

SELECT id, name, price FROM products WHERE name LIKE '%' ORDER BY name ASC;

SELECT * FROM users ORDER BY YEAR(created_at) DESC, MONTH(created_at) ASC, name DESC;

SELECT name, total FROM orders WHERE total > 0 ORDER BY total DESC LIMIT 10 OFFSET 0;

SELECT * FROM products WHERE price > 0 ORDER BY price ASC, created_at DESC;

SELECT name, age FROM users ORDER BY age BETWEEN 20 AND 30 DESC, age ASC;

SELECT id, name, total FROM orders WHERE status = 'pending' ORDER BY total DESC, created_at ASC;

SELECT * FROM users ORDER BY LEFT(email, POSITION('@' IN email) - 1) ASC;

SELECT name, price FROM products ORDER BY category_id, price DESC, stock ASC;

SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at DESC LIMIT 5;

SELECT name, price FROM products WHERE stock BETWEEN 10 AND 100 ORDER BY stock DESC, price ASC;

SELECT * FROM users ORDER BY IF(status = 'active', 0, 1), created_at DESC;

SELECT id, name, total FROM orders ORDER BY CONCAT(user_id, '-', id) ASC;

SELECT name, total FROM orders WHERE YEAR(created_at) >= 2024 ORDER BY created_at DESC;

SELECT * FROM products WHERE category_id IS NOT NULL AND stock > 0 ORDER BY category_id, price DESC;

SELECT name, created_at FROM users ORDER BY DATE(created_at) DESC, TIME(created_at) DESC;

SELECT id, name, price FROM products ORDER BY price * 0.9 DESC;

SELECT * FROM users WHERE name LIKE '%Smith%' ORDER BY name ASC, created_at DESC;

SELECT name, total FROM orders WHERE total > (SELECT AVG(total) FROM orders WHERE status = 'completed') ORDER BY total DESC;

SELECT * FROM products ORDER BY LENGTH(name) ASC, name ASC;

SELECT id, name, price FROM products ORDER BY category_id DESC, price ASC LIMIT 20;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC;

SELECT * FROM users WHERE email LIKE '%@%.com' ORDER BY email ASC;

SELECT name, price FROM products WHERE price > 50 ORDER BY price DESC LIMIT 10 OFFSET 10;

SELECT * FROM orders ORDER BY HOUR(created_at) DESC, total ASC;

SELECT name, age FROM users ORDER BY age DESC, name ASC LIMIT 50;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY name ASC, price DESC;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY created_at DESC LIMIT 100;

SELECT name, total FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip') ORDER BY total DESC;

SELECT * FROM products ORDER BY price > 100 DESC, price ASC;

SELECT id, name, price FROM products ORDER BY IFNULL(discount, 0) DESC;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 25;

SELECT * FROM users ORDER BY created_at DESC, last_login DESC;

SELECT name, price FROM products WHERE name LIKE '%pro%' OR name LIKE '%premium%' ORDER BY price DESC;

SELECT id, name, total FROM orders ORDER BY YEARWEEK(created_at) DESC, total DESC;

SELECT * FROM users WHERE age > 18 ORDER BY age ASC, created_at DESC;

SELECT name, price FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 5;

SELECT * FROM orders WHERE user_id = 1 AND status = 'completed' ORDER BY created_at DESC;

SELECT name, total FROM orders ORDER BY DAYOFMONTH(created_at) ASC;

SELECT id, name, price FROM products ORDER BY price > 0 DESC, name ASC;

SELECT * FROM users ORDER BY LEFT(name, 3) DESC, name ASC;

SELECT name, created_at FROM orders WHERE status = 'completed' ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY category_id, stock DESC;

SELECT name, age FROM users WHERE age BETWEEN 25 AND 35 ORDER BY age DESC, name ASC;

SELECT id, name, total FROM orders WHERE total > 0 ORDER BY total DESC LIMIT 50;

SELECT name, price FROM products ORDER BY REVERSE(SUBSTRING(name, 1, 3)) ASC;

SELECT * FROM users WHERE status = 'active' AND age > 30 ORDER BY created_at DESC;

SELECT id, name, price FROM products WHERE price > 100 ORDER BY price ASC, stock DESC;

SELECT name, total FROM orders ORDER BY status = 'pending' DESC, status = 'completed' DESC, total DESC;

SELECT * FROM orders WHERE user_id = 1 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE category_id IN (SELECT id FROM categories WHERE active = TRUE) ORDER BY name ASC;

SELECT id, name, total FROM orders ORDER BY LPAD(CAST(id AS CHAR), 5, '0') DESC;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY MONTH(created_at) ASC, name DESC;

SELECT name, price FROM products ORDER BY COALESCE(category, 'Other') ASC, price DESC;

SELECT * FROM orders WHERE total > 0 ORDER BY total DESC, created_at ASC LIMIT 100;

SELECT name, age FROM users ORDER BY CASE WHEN age IS NULL THEN 1 ELSE 0 END, age ASC;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY stock ASC, price DESC LIMIT 25;

SELECT * FROM users WHERE status = 'vip' ORDER BY total_orders DESC;

SELECT name, total FROM orders WHERE user_id = (SELECT user_id FROM orders ORDER BY total DESC LIMIT 1) ORDER BY total DESC;

SELECT * FROM products ORDER BY IF(stock > 0, 0, 1), stock DESC, price ASC;

SELECT name, created_at FROM users WHERE email IS NOT NULL ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products ORDER BY price BETWEEN 50 AND 100 DESC, price ASC;

SELECT * FROM orders ORDER BY SUBSTRING_INDEX(name, ' ', 1) ASC;

SELECT name, total FROM orders WHERE status IN ('pending', 'processing', 'shipped', 'completed') ORDER BY FIELD(status, 'pending', 'processing', 'shipped', 'completed'), total DESC;

SELECT * FROM users WHERE age > 20 ORDER BY age DESC, created_at ASC LIMIT 25;

SELECT name, price FROM products ORDER BY price ASC, name COLLATE utf8_bin ASC;

SELECT id, name, total FROM orders ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC, total DESC;

SELECT * FROM products WHERE name LIKE '%a%' ORDER BY name ASC, price DESC;

SELECT name, age FROM users WHERE status = 'active' ORDER BY age ASC, name DESC;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY category_id, price DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 1 AND YEAR(created_at) = 2024 ORDER BY created_at DESC;

SELECT name, total FROM orders WHERE total > 1000 ORDER BY total DESC LIMIT 20;

SELECT * FROM users ORDER BY SUBSTRING(name, LENGTH(name)-3, 4) ASC;

SELECT id, name, price FROM products ORDER BY name LIKE '%sale%' DESC, price ASC;

SELECT * FROM orders WHERE user_id IN (1, 2, 3) ORDER BY FIELD(user_id, 1, 2, 3), total DESC;

SELECT name, price FROM products WHERE price > 0 ORDER BY price / NULLIF(stock, 0) ASC;

SELECT * FROM users WHERE created_at >= '2024-01-01' ORDER BY created_at DESC, name ASC;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC, created_at ASC LIMIT 25;

SELECT id, name, price FROM products WHERE category_id = 1 OR category_id = 2 ORDER BY category_id, price DESC;

SELECT * FROM users ORDER BY IF(name IS NULL, 1, 0), name ASC;

SELECT name, price FROM products ORDER BY REPLACE(REPLACE(name, '-', ''), ' ', '') ASC;

SELECT * FROM orders WHERE user_id = 1 ORDER BY IFNULL(shipped_at, created_at) DESC;

SELECT name, age FROM users WHERE age > 0 ORDER BY age DESC LIMIT 50;

SELECT id, name, price FROM products ORDER BY price DESC LIMIT 10 OFFSET 20;

SELECT * FROM users WHERE email LIKE '%@company.com' ORDER BY email ASC, created_at DESC;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC, created_at ASC LIMIT 30;

SELECT * FROM products WHERE stock > 0 AND price > 0 ORDER BY category_id, price ASC;

SELECT name, age FROM users ORDER BY age BETWEEN 30 AND 40 DESC, age ASC;

SELECT id, name, total FROM orders WHERE user_id > 0 ORDER BY user_id ASC, total DESC;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC;

SELECT name, price FROM products ORDER BY name ASC, price DESC LIMIT 100;

SELECT * FROM orders ORDER BY CONCAT(YEAR(created_at), LPAD(MONTH(created_at), 2, '0')) DESC;

SELECT name, total FROM orders WHERE user_id = 1 ORDER BY YEAR(created_at) DESC, total DESC;

SELECT id, name, price FROM products WHERE price > 50 ORDER BY CHAR_LENGTH(name) DESC;

SELECT * FROM users ORDER BY COALESCE(last_login, created_at) DESC, name ASC;

SELECT name, price FROM products WHERE category_id NOT IN (1, 2, 3) ORDER BY category_id, name ASC;

SELECT id, name, total FROM orders ORDER BY status DESC, total DESC;

SELECT * FROM products WHERE name LIKE '%' ORDER BY name ASC LIMIT 50;

SELECT name, age FROM users WHERE age IS NOT NULL ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id IN (SELECT user_id FROM orders GROUP BY user_id HAVING COUNT(*) > 5) ORDER BY created_at DESC;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY stock ASC, created_at DESC;

SELECT name, total FROM orders WHERE total > 0 ORDER BY total ASC LIMIT 10;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT name, price FROM products ORDER BY CASE WHEN stock > 0 THEN 0 ELSE 1 END, name ASC;

SELECT id, name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 25;

SELECT * FROM products ORDER BY LENGTH(name) ASC, name DESC;

SELECT name, total FROM orders WHERE user_id = 2 ORDER BY total DESC LIMIT 10;

SELECT * FROM users ORDER BY YEAR(created_at) ASC, name DESC;

SELECT id, name, price FROM products ORDER BY price DESC, stock ASC LIMIT 30;

SELECT * FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) ORDER BY created_at DESC;

SELECT name, age FROM users ORDER BY age ASC, created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 20;

SELECT * FROM users WHERE created_at > '2024-01-01' ORDER BY created_at ASC, name DESC;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 15;

SELECT * FROM products ORDER BY name COLLATE NOCASE ASC;

SELECT id, name, price FROM products WHERE stock BETWEEN 10 AND 50 ORDER BY stock DESC, price ASC;

SELECT * FROM users WHERE email LIKE '%@business.com' ORDER BY email ASC;

SELECT name, total FROM orders WHERE user_id = 3 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 1 ORDER BY price DESC LIMIT 10;

SELECT * FROM users ORDER BY IF(age IS NULL, 0, 1), age DESC LIMIT 50;

SELECT id, name, total FROM orders WHERE YEAR(created_at) = 2024 ORDER BY MONTH(created_at) ASC, total DESC;

SELECT name, price FROM products WHERE name LIKE '%pro%' ORDER BY price ASC;

SELECT * FROM orders ORDER BY status = 'completed' DESC, created_at DESC LIMIT 50;

SELECT name, age FROM users ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC, name ASC;

SELECT id, name, price FROM products ORDER BY category_id DESC, price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY created_at DESC LIMIT 25;

SELECT name, total FROM orders WHERE total > (SELECT MAX(total) * 0.5 FROM orders) ORDER BY total DESC;

SELECT * FROM products ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM users WHERE DAY(created_at) = 1 ORDER BY created_at DESC;

SELECT id, name, price FROM products WHERE price BETWEEN 25 AND 75 ORDER BY price DESC;

SELECT name, total FROM orders WHERE user_id = 4 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders ORDER BY IFNULL(shipped_at, DATE_ADD(created_at, INTERVAL 7 DAY)) DESC;

SELECT name, age FROM users WHERE age > 20 ORDER BY age ASC, name DESC LIMIT 50;

SELECT id, name, price FROM products ORDER BY LEFT(name, 1) DESC, price ASC;

SELECT * FROM products WHERE category_id = (SELECT category_id FROM products GROUP BY category_id ORDER BY COUNT(*) DESC LIMIT 1) ORDER BY price DESC;

SELECT name, created_at FROM users ORDER BY DATE(created_at) DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 5 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products ORDER BY REPLACE(name, 'Product ', '') ASC;

SELECT * FROM users WHERE status = 'active' ORDER BY COALESCE(last_login, created_at) DESC;

SELECT id, name, total FROM orders ORDER BY status, total DESC LIMIT 100;

SELECT * FROM products WHERE stock > 10 ORDER BY stock DESC, created_at ASC LIMIT 25;

SELECT name, total FROM orders WHERE YEAR(created_at) = 2023 ORDER BY total DESC;

SELECT * FROM users ORDER BY LENGTH(email) DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY name ASC, created_at DESC;

SELECT * FROM orders WHERE user_id = 6 ORDER BY total DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age ASC LIMIT 50;

SELECT * FROM products WHERE category_id IS NOT NULL ORDER BY category_id, name ASC;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) ORDER BY created_at DESC;

SELECT name, total FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 25;

SELECT id, name, price FROM products ORDER BY price ASC LIMIT 50 OFFSET 50;

SELECT * FROM users ORDER BY name ASC LIMIT 100;

SELECT * FROM orders WHERE user_id = 7 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price * stock DESC;

SELECT * FROM products ORDER BY IF(price > 0 AND stock > 0, 0, 1), created_at DESC;

SELECT name, age FROM users WHERE age BETWEEN 18 AND 65 ORDER BY age DESC, name ASC;

SELECT id, name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY email DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 8 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE name LIKE '%a%' ORDER BY name ASC, price DESC;

SELECT * FROM orders ORDER BY DATE_FORMAT(created_at, '%Y%m%d%H%i%s') DESC;

SELECT id, name, price FROM products WHERE stock > 0 AND price > 0 ORDER BY price ASC, stock DESC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, total DESC LIMIT 25;

SELECT * FROM products WHERE category_id = 3 ORDER BY price DESC LIMIT 15;

SELECT * FROM users WHERE name REGEXP '^[A-E]' ORDER BY name ASC;

SELECT id, name, total FROM orders WHERE user_id = 9 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders WHERE YEAR(created_at) = 2024 ORDER BY MONTH(created_at) ASC, total DESC;

SELECT name, price FROM products ORDER BY name COLLATE utf8mb4_unicode_ci ASC;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 10;

SELECT name, age FROM users WHERE age > 25 ORDER BY age ASC, name DESC;

SELECT id, name, price FROM products ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 10 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE stock = (SELECT MAX(stock) FROM products WHERE stock < 1000) ORDER BY name ASC;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 25;

SELECT * FROM products WHERE name LIKE '%e%' ORDER BY LENGTH(name) DESC, name ASC;

SELECT id, name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT name, total FROM orders WHERE user_id = 11 ORDER BY total DESC LIMIT 10;

SELECT * FROM users ORDER BY YEAR(created_at) DESC, MONTH(created_at) ASC;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT name, age FROM users WHERE age IS NOT NULL ORDER BY age DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 12 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price > 100 ORDER BY price ASC LIMIT 20;

SELECT * FROM users WHERE email LIKE '%@company%' ORDER BY created_at DESC;

SELECT * FROM orders ORDER BY total DESC LIMIT 10 OFFSET 50;

SELECT name, price FROM products WHERE category_id IN (1, 2) ORDER BY category_id, price DESC;

SELECT * FROM users WHERE status = 'vip' ORDER BY total_spent DESC;

SELECT id, name, total FROM orders WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = 1 ORDER BY total DESC;

SELECT * FROM products WHERE stock BETWEEN 5 AND 50 ORDER BY stock DESC, name ASC;

SELECT * FROM users ORDER BY LENGTH(name) ASC, name DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 13 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders WHERE user_id = 14 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products ORDER BY price ASC, name DESC LIMIT 25;

SELECT * FROM users WHERE created_at > '2023-01-01' ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE name LIKE '%product%' ORDER BY name ASC;

SELECT * FROM orders ORDER BY status DESC, created_at DESC LIMIT 30;

SELECT name, age FROM users ORDER BY age ASC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY stock ASC, created_at DESC LIMIT 25;

SELECT * FROM users WHERE status = 'new' ORDER BY created_at DESC;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 20;

SELECT id, name, price FROM products WHERE price BETWEEN 20 AND 80 ORDER BY price DESC;

SELECT * FROM orders WHERE user_id = 15 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE age > 30 ORDER BY age ASC, created_at DESC;

SELECT name, price FROM products WHERE category_id = 4 ORDER BY price DESC LIMIT 15;

SELECT * FROM products ORDER BY name COLLATE latin1_general_cs ASC;

SELECT * FROM orders ORDER BY DATE(created_at) DESC, total DESC LIMIT 50;

SELECT name, age FROM users WHERE age BETWEEN 20 AND 40 ORDER BY age DESC LIMIT 25;

SELECT id, name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 15;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY email ASC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT name, total FROM orders WHERE user_id = 16 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders WHERE user_id = 17 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products ORDER BY price DESC LIMIT 30 OFFSET 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50;

SELECT id, name, price FROM products WHERE name LIKE '%sale%' ORDER BY name ASC;

SELECT * FROM orders WHERE status IN ('completed', 'shipped') ORDER BY created_at DESC LIMIT 30;

SELECT name, age FROM users ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM products WHERE category_id = 5 ORDER BY stock DESC, name ASC;

SELECT * FROM users WHERE YEAR(created_at) = 2023 ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 18 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders ORDER BY IF(status = 'pending', 0, IF(status = 'processing', 1, IF(status = 'shipped', 2, 3))), total DESC;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY stock ASC, price DESC LIMIT 25;

SELECT * FROM users WHERE email LIKE '%@business%' ORDER BY created_at DESC;

SELECT * FROM products WHERE name REGEXP 'pro|premium|elite' ORDER BY price DESC;

SELECT * FROM orders WHERE user_id = 19 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age ASC LIMIT 50;

SELECT * FROM products ORDER BY category_id, stock DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 30;

SELECT * FROM users ORDER BY last_name ASC, first_name ASC;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY name ASC, created_at DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 20 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price < 50 ORDER BY price ASC;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT id, name, price FROM products WHERE category_id = 6 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE user_id = 21 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE age > 0 ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE stock > 100 ORDER BY stock ASC, name ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT name, price FROM products ORDER BY LEFT(name, 1) DESC, price ASC;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY created_at DESC LIMIT 50;

SELECT id, name, total FROM orders WHERE user_id = 22 ORDER BY total DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_0900_ai_ci ASC;

SELECT * FROM orders WHERE user_id = 23 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 75 ORDER BY price ASC, name DESC;

SELECT * FROM users WHERE status = 'active' ORDER BY total_orders DESC LIMIT 50;

SELECT * FROM products WHERE stock BETWEEN 1 AND 20 ORDER BY stock ASC, created_at DESC;

SELECT id, name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 24 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users WHERE age < 30 ORDER BY age ASC, name DESC;

SELECT id, name, price FROM products WHERE price > 0 AND stock > 0 ORDER BY price / stock DESC LIMIT 25;

SELECT * FROM users WHERE created_at >= DATE_SUB(NOW(), INTERVAL 60 DAY) ORDER BY created_at DESC;

SELECT * FROM products WHERE name LIKE '%item%' ORDER BY name ASC;

SELECT * FROM orders ORDER BY YEAR(created_at) ASC, total DESC LIMIT 50;

SELECT name, age FROM users WHERE age BETWEEN 30 AND 50 ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE category_id IS NOT NULL ORDER BY category_id ASC, name DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 25 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price < 25 ORDER BY price DESC;

SELECT * FROM users ORDER BY name COLLATE utf8_general_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT id, name, price FROM products WHERE category_id = 7 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE user_id = 26 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY stock DESC, price ASC LIMIT 25;

SELECT * FROM users WHERE status = 'vip' ORDER BY name ASC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, total DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 27 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products ORDER BY CHAR_LENGTH(TRIM(name)) DESC LIMIT 25;

SELECT * FROM users WHERE email LIKE '%@%' ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE price > 50 ORDER BY name ASC, price DESC LIMIT 30;

SELECT * FROM orders ORDER BY total DESC LIMIT 10 OFFSET 0;

SELECT * FROM users WHERE status = 'active' ORDER BY age ASC, created_at DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%new%' ORDER BY created_at DESC;

SELECT * FROM orders WHERE user_id = 28 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age ASC LIMIT 25 OFFSET 25;

SELECT * FROM products ORDER BY category_id DESC, price ASC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC;

SELECT * FROM orders WHERE status = 'completed' ORDER BY DATE(created_at) DESC, total DESC LIMIT 50;

SELECT id, name, price FROM products WHERE stock > 10 ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 29 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY IF(status = 'active', 0, IF(status = 'vip', 1, 2)), name ASC;

SELECT * FROM products WHERE stock > 0 ORDER BY REVERSE(name) ASC;

SELECT * FROM orders ORDER BY CONCAT(user_id, created_at) DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 50;

SELECT * FROM users WHERE status IN ('active', 'new') ORDER BY created_at DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY name ASC;

SELECT * FROM orders WHERE user_id = 30 ORDER BY created_at DESC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT id, name, price FROM products ORDER BY category_id, stock DESC, name ASC LIMIT 50;

SELECT * FROM users WHERE created_at > '2024-01-01' ORDER BY created_at DESC, name ASC;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT name, age FROM users ORDER BY age DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 31 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price BETWEEN 30 AND 70 ORDER BY price DESC LIMIT 25;

SELECT * FROM users ORDER BY LEFT(name, 1) ASC, age DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 32 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 8 ORDER BY stock DESC, price ASC LIMIT 20;

SELECT * FROM orders ORDER BY DAY(created_at) = 15 DESC, DAY(created_at) ASC LIMIT 50;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 33 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_bin ASC;

SELECT * FROM orders WHERE status IN ('pending', 'completed', 'cancelled') ORDER BY FIELD(status, 'pending', 'completed', 'cancelled'), created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 25;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 9 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE user_id = 34 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users WHERE age > 0 ORDER BY age ASC, name DESC LIMIT 25;

SELECT * FROM products WHERE stock > 0 ORDER BY stock ASC, name ASC LIMIT 25;

SELECT * FROM users ORDER BY REPLACE(name, ' ', '') ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC, total DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price > 25 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 35 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY email DESC LIMIT 50;

SELECT name, total FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE category_id = 10 ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM orders ORDER BY status = 'completed' DESC, created_at DESC LIMIT 50;

SELECT name, price FROM products ORDER BY name LIKE '%sale%' DESC, price ASC LIMIT 30;

SELECT * FROM users WHERE age BETWEEN 25 AND 45 ORDER BY age ASC, created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 36 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 5 ORDER BY price DESC LIMIT 25;

SELECT * FROM users ORDER BY LENGTH(CONCAT(first_name, last_name)) DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%' ORDER BY name ASC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 37 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT id, name, price FROM products WHERE category_id = 11 ORDER BY price DESC LIMIT 15;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 38 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age DESC LIMIT 25;

SELECT * FROM products ORDER BY category_id, price ASC, stock DESC LIMIT 50;

SELECT * FROM users WHERE email LIKE '%@company%' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY stock DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 39 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY COALESCE(last_login, created_at) DESC, name ASC LIMIT 50;

SELECT name, price FROM products WHERE category_id = 12 ORDER BY price ASC LIMIT 20;

SELECT * FROM orders ORDER BY CONCAT(YEAR(created_at), MONTH(created_at), DAY(created_at)) DESC LIMIT 50;

SELECT * FROM products WHERE name REGEXP 'pro|premium|plus' ORDER BY price DESC LIMIT 25;

SELECT * FROM users WHERE age > 0 ORDER BY age ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 40 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY MONTH(created_at) ASC, name DESC;

SELECT * FROM products ORDER BY stock DESC, created_at ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 41 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price < 75 ORDER BY price ASC LIMIT 25;

SELECT * FROM users WHERE status = 'vip' ORDER BY total_spent DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT id, name, price FROM products WHERE category_id = 13 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders ORDER BY YEARWEEK(created_at) DESC, total DESC LIMIT 50;

SELECT name, age FROM users WHERE age BETWEEN 18 AND 60 ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE stock > 0 ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 42 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 30;

SELECT id, name, price FROM products WHERE stock > 0 AND price > 0 ORDER BY price / stock ASC LIMIT 25;

SELECT * FROM users WHERE status = 'active' ORDER BY name ASC, created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 43 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%item%' ORDER BY name ASC, price DESC;

SELECT * FROM users ORDER BY LEFT(name, 3) DESC, name ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT id, name, price FROM products WHERE category_id = 14 ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM orders ORDER BY DATE(created_at) DESC, total DESC LIMIT 50;

SELECT * FROM users WHERE created_at > '2023-06-01' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 44 ORDER BY created_at DESC LIMIT 10;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE stock > 10 ORDER BY created_at DESC, name ASC LIMIT 25;

SELECT * FROM users ORDER BY age ASC, name DESC LIMIT 25;

SELECT id, name, price FROM products WHERE price BETWEEN 15 AND 85 ORDER BY price DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 45 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY category_id DESC, price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status IN ('pending', 'processing') ORDER BY FIELD(status, 'pending', 'processing'), created_at DESC LIMIT 50;

SELECT name, price FROM products ORDER BY price DESC LIMIT 25 OFFSET 25;

SELECT * FROM users WHERE YEAR(created_at) >= 2022 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 46 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY name ASC, created_at DESC LIMIT 30;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 47 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 15 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders ORDER BY IFNULL(shipped_at, DATE_ADD(created_at, INTERVAL 3 DAY)) DESC LIMIT 50;

SELECT * FROM users ORDER BY LENGTH(name) ASC, name DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY stock ASC, price DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 48 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) ASC, total DESC LIMIT 30;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price DESC, stock ASC LIMIT 25;

SELECT * FROM users WHERE email LIKE '%@business%' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE name LIKE '%pro%' ORDER BY name ASC, price DESC;

SELECT * FROM orders WHERE user_id = 49 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE category_id = 16 ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 20;

SELECT * FROM orders ORDER BY CONCAT(user_id, '-', id) ASC LIMIT 50;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 50 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE status = 'active' ORDER BY total_orders DESC LIMIT 50;

SELECT name, price FROM products ORDER BY name COLLATE utf8_bin ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM users WHERE age > 25 ORDER BY age ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 51 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price BETWEEN 40 AND 120 ORDER BY price DESC LIMIT 30;

SELECT * FROM users ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%sale%' ORDER BY name ASC;

SELECT * FROM orders ORDER BY status = 'pending' DESC, status = 'completed' DESC, created_at DESC LIMIT 50;

SELECT * FROM users WHERE YEAR(created_at) = 2023 ORDER BY MONTH(created_at) ASC, name DESC;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM products ORDER BY category_id, name ASC, price DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 52 ORDER BY total DESC LIMIT 10;

SELECT * FROM orders WHERE user_id = 53 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY COALESCE(last_login, created_at) DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 54 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price < 60 ORDER BY price DESC LIMIT 25;

SELECT * FROM products ORDER BY CHAR_LENGTH(name) DESC, name ASC LIMIT 25;

SELECT * FROM orders ORDER BY DATE(created_at) DESC LIMIT 50;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY email ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 55 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY stock DESC, created_at ASC LIMIT 25;

SELECT * FROM users ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM products WHERE category_id = 17 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders ORDER BY CONCAT(YEAR(created_at), MONTH(created_at)) DESC, total DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 56 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products ORDER BY price ASC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY name ASC, price DESC;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEARWEEK(created_at) DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 57 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 5 ORDER BY price DESC LIMIT 25;

SELECT * FROM users ORDER BY IF(status = 'active', 0, 1), name ASC LIMIT 50;

SELECT * FROM orders ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 18 ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT name, total FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 58 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_bin ASC LIMIT 50;

SELECT * FROM orders WHERE status IN ('pending', 'completed', 'cancelled') ORDER BY FIELD(status, 'pending', 'completed', 'cancelled'), created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 25;

SELECT * FROM users ORDER BY LENGTH(CONCAT(first_name, last_name)) ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 59 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT name, age FROM users WHERE age > 0 ORDER BY age ASC LIMIT 25;

SELECT * FROM orders ORDER BY YEAR(created_at) ASC, MONTH(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM products WHERE category_id = 19 ORDER BY price DESC LIMIT 15;

SELECT * FROM users WHERE email LIKE '%@%' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 60 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE price > 0 ORDER BY price ASC, name DESC LIMIT 30;

SELECT * FROM users ORDER BY REPLACE(name, ' ', '') ASC LIMIT 50;

SELECT * FROM products ORDER BY stock DESC, price ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, total DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) >= 2024 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 61 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name LIKE '%sale%' DESC, name ASC LIMIT 30;

SELECT * FROM orders ORDER BY status DESC, created_at DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY age ASC, created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 62 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 20 ORDER BY stock DESC, name ASC LIMIT 20;

SELECT * FROM orders ORDER BY DAY(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM users ORDER BY LENGTH(name) DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC, created_at DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 63 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY name ASC, created_at DESC LIMIT 50;

SELECT * FROM products ORDER BY price DESC, stock ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) = 2023 ORDER BY MONTH(created_at) ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 64 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price BETWEEN 10 AND 90 ORDER BY price DESC LIMIT 30;

SELECT * FROM products ORDER BY category_id DESC, name ASC LIMIT 30;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 30;

SELECT * FROM orders ORDER BY user_id ASC, total DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 65 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 30;

SELECT * FROM users WHERE status IN ('active', 'vip') ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT name, price FROM products ORDER BY price ASC LIMIT 25;

SELECT * FROM products WHERE name LIKE '%pro%' ORDER BY name ASC, price DESC;

SELECT * FROM orders WHERE user_id = 66 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE category_id = 21 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY name COLLATE utf8_bin ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 67 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 30;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC, name ASC LIMIT 25;

SELECT * FROM users ORDER BY LENGTH(email) DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEARWEEK(created_at) DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 68 ORDER BY created_at DESC LIMIT 10;

SELECT id, name, price FROM products ORDER BY stock DESC, created_at ASC LIMIT 25;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY CONCAT(user_id, '-', id) ASC LIMIT 50;

SELECT * FROM products WHERE category_id = 22 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 69 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8_general_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price ASC, stock DESC LIMIT 25;

SELECT * FROM users ORDER BY age ASC, name DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 70 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE name LIKE '%item%' ORDER BY name ASC, price DESC;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY status = 'completed' DESC, total DESC LIMIT 50;

SELECT * FROM products WHERE category_id = 23 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC LIMIT 30;

SELECT * FROM users ORDER BY name COLLATE utf8mb4_bin ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 71 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY created_at ASC LIMIT 50;

SELECT * FROM orders ORDER BY DAYOFMONTH(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 72 ORDER BY created_at DESC LIMIT 10;

SELECT name, price FROM products WHERE price > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM users WHERE status = 'vip' ORDER BY total_orders DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY name ASC;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM orders WHERE user_id = 73 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY category_id, price ASC, stock DESC LIMIT 50;

SELECT * FROM users ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM orders ORDER BY YEARWEEK(created_at) ASC, total DESC LIMIT 50;

SELECT id, name, price FROM products WHERE stock > 0 ORDER BY price / NULLIF(stock, 0) ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM users WHERE status = 'active' ORDER BY COALESCE(last_login, created_at) DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 74 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_0900_ai_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status IN ('pending', 'processing') ORDER BY FIELD(status, 'pending', 'processing'), created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE price > 0 ORDER BY price DESC LIMIT 25;

SELECT * FROM users ORDER BY LENGTH(CONCAT(first_name, last_name)) ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 75 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders ORDER BY DATE(created_at) DESC, total DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY name ASC, created_at DESC LIMIT 50;

SELECT * FROM products WHERE category_id = 24 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE user_id = 76 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY age ASC LIMIT 25;

SELECT * FROM products ORDER BY stock DESC, price ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) >= 2022 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 77 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name LIKE '%sale%' DESC, name ASC LIMIT 30;

SELECT * FROM orders ORDER BY status = 'pending' DESC, status = 'completed' DESC, created_at DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT name, total FROM orders WHERE user_id = 78 ORDER BY total DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 25 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders ORDER BY CONCAT(YEAR(created_at), MONTH(created_at)) DESC LIMIT 50;

SELECT * FROM users ORDER BY LENGTH(name) ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC, created_at DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 79 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 50;

SELECT * FROM products ORDER BY price DESC, stock ASC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM users WHERE YEAR(created_at) = 2024 ORDER BY MONTH(created_at) ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 80 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY category_id DESC, name ASC LIMIT 30;

SELECT * FROM orders ORDER BY DAY(created_at) = 1 DESC, DAY(created_at) ASC LIMIT 50;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 40;

SELECT * FROM orders ORDER BY user_id ASC, total DESC LIMIT 50;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE user_id = 81 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users WHERE status = 'active' ORDER BY total_orders DESC LIMIT 50;

SELECT * FROM products ORDER BY name COLLATE utf8_bin ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT * FROM products WHERE name LIKE '%pro%' ORDER BY name ASC, price DESC;

SELECT * FROM orders WHERE user_id = 82 ORDER BY created_at DESC LIMIT 10;

SELECT name, age FROM users ORDER BY age DESC LIMIT 25;

SELECT * FROM products WHERE category_id = 26 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders ORDER BY YEARWEEK(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM orders WHERE user_id = 83 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM users ORDER BY name COLLATE utf8mb4_bin ASC LIMIT 50;

SELECT * FROM orders ORDER BY status = 'completed' DESC, total DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 84 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY age ASC, name DESC LIMIT 25;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC LIMIT 30;

SELECT * FROM products WHERE category_id = 27 ORDER BY price DESC LIMIT 15;

SELECT * FROM users WHERE email IS NOT NULL ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY DATE(created_at) DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 85 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY stock DESC, created_at ASC LIMIT 25;

SELECT * FROM users ORDER BY name ASC LIMIT 50;

SELECT * FROM products ORDER BY price ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 86 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE name LIKE '%basic%' ORDER BY name ASC;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY CONCAT(user_id, '-', id) ASC LIMIT 50;

SELECT * FROM products WHERE category_id = 28 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEARWEEK(created_at) DESC LIMIT 30;

SELECT * FROM users ORDER BY LENGTH(name) DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 87 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8_general_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price DESC, stock ASC LIMIT 25;

SELECT * FROM users ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 88 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY category_id, name ASC, price DESC LIMIT 50;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY status = 'pending' DESC, created_at DESC LIMIT 50;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC, created_at DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 89 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 50;

SELECT * FROM orders ORDER BY YEAR(created_at) DESC, MONTH(created_at) DESC LIMIT 50;

SELECT * FROM products ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 90 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE category_id = 29 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders ORDER BY IFNULL(shipped_at, DATE_ADD(created_at, INTERVAL 3 DAY)) DESC LIMIT 50;

SELECT * FROM users ORDER BY LENGTH(CONCAT(first_name, last_name)) ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'completed' ORDER BY total DESC LIMIT 30;

SELECT * FROM orders WHERE user_id = 91 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_bin ASC LIMIT 50;

SELECT * FROM users WHERE status = 'vip' ORDER BY name ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at ASC LIMIT 20;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 92 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY age ASC, name DESC LIMIT 25;

SELECT * FROM products WHERE category_id = 30 ORDER BY stock DESC LIMIT 20;

SELECT * FROM orders ORDER BY YEARWEEK(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM users WHERE status = 'active' ORDER BY created_at ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 93 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY price DESC, stock ASC LIMIT 25;

SELECT * FROM orders ORDER BY status = 'completed' DESC, total DESC LIMIT 50;

SELECT * FROM users WHERE YEAR(created_at) >= 2024 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM products WHERE name LIKE '%item%' ORDER BY name ASC, price DESC;

SELECT * FROM orders WHERE user_id = 94 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM users ORDER BY name COLLATE utf8_bin ASC LIMIT 50;

SELECT * FROM orders ORDER BY DATE(created_at) DESC, total DESC LIMIT 50;

SELECT * FROM products ORDER BY category_id DESC, name ASC LIMIT 30;

SELECT * FROM orders WHERE status = 'pending' ORDER BY total DESC LIMIT 20;

SELECT * FROM orders WHERE user_id = 95 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products WHERE stock > 0 ORDER BY name ASC LIMIT 30;

SELECT * FROM users WHERE status = 'inactive' ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY CONCAT(user_id, '-', id) ASC LIMIT 50;

SELECT * FROM products WHERE category_id = 31 ORDER BY price DESC LIMIT 15;

SELECT * FROM orders WHERE status = 'completed' ORDER BY YEAR(created_at) DESC LIMIT 30;

SELECT * FROM users ORDER BY LENGTH(name) ASC LIMIT 50;

SELECT * FROM orders WHERE user_id = 96 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8mb4_unicode_ci ASC LIMIT 50;

SELECT * FROM orders ORDER BY status = 'pending' DESC, created_at DESC LIMIT 50;

SELECT name, price FROM products WHERE stock > 0 ORDER BY price ASC, stock DESC LIMIT 25;

SELECT * FROM users ORDER BY age DESC, name ASC LIMIT 25;

SELECT * FROM orders WHERE user_id = 97 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY category_id, price ASC, stock DESC LIMIT 50;

SELECT * FROM users WHERE YEAR(created_at) >= 2023 ORDER BY created_at DESC LIMIT 50;

SELECT * FROM orders ORDER BY DAYOFMONTH(created_at) ASC, total DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 98 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY price DESC, name ASC LIMIT 25;

SELECT * FROM users ORDER BY created_at DESC LIMIT 10 OFFSET 50;

SELECT * FROM orders order by user_id ASC, total DESC LIMIT 50;

SELECT * FROM orders WHERE user_id = 99 ORDER BY created_at DESC LIMIT 10;

SELECT * FROM products ORDER BY name COLLATE utf8_general_ci ASC LIMIT 50;

SELECT * FROM orders WHERE status = 'shipped' ORDER BY shipped_at DESC LIMIT 30;

SELECT * FROM products WHERE stock > 0 ORDER BY created_at DESC LIMIT 20;

SELECT * FROM users ORDER BY age ASC, name DESC LIMIT 25;

SELECT * FROM orders WHERE user_id = 100 ORDER BY created_at DESC LIMIT 10;
