-- AGGREGATE Function Test Cases
-- Compatibility: MySQL 5.7+

SELECT COUNT(*) FROM users;

SELECT COUNT(*) FROM products WHERE price > 100;

SELECT COUNT(DISTINCT category_id) FROM products;

SELECT COUNT(DISTINCT user_id) FROM orders;

SELECT SUM(total) FROM orders;

SELECT SUM(price * quantity) FROM order_items;

SELECT AVG(price) FROM products;

SELECT AVG(age) FROM users WHERE age > 18;

SELECT MAX(price) FROM products;

SELECT MIN(price) FROM products;

SELECT MIN(created_at) FROM orders WHERE user_id = 1;

SELECT MAX(total) FROM orders WHERE status = 'completed';

SELECT COUNT(*), SUM(total), AVG(total), MAX(total), MIN(total) FROM orders;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id;

SELECT user_id, SUM(total) FROM orders GROUP BY user_id;

SELECT status, AVG(total) FROM orders GROUP BY status;

SELECT category_id, MAX(price), MIN(price), AVG(price) FROM products GROUP BY category_id;

SELECT user_id, COUNT(*) AS order_count FROM orders GROUP BY user_id HAVING COUNT(*) > 5;

SELECT category_id, SUM(stock) FROM products GROUP BY category_id HAVING SUM(stock) > 1000;

SELECT user_id, AVG(total) FROM orders WHERE total > 0 GROUP BY user_id;

SELECT department_id, COUNT(DISTINCT user_id) FROM employees GROUP BY department_id;

SELECT DATE(created_at), COUNT(*) FROM orders GROUP BY DATE(created_at);

SELECT COUNT(*) FROM products WHERE category_id IN (SELECT id FROM categories WHERE name = 'Electronics');

SELECT SUM(total) FROM orders WHERE YEAR(created_at) = 2024;

SELECT AVG(price) FROM products WHERE price BETWEEN 50 AND 200;

SELECT COUNT(DISTINCT email) FROM users;

SELECT MAX(age) - MIN(age) AS age_range FROM users;

SELECT SUM(price) / COUNT(*) AS avg_price FROM products;

SELECT category_id, SUM(quantity) FROM order_items GROUP BY category_id ORDER BY SUM(quantity) DESC LIMIT 5;

SELECT COUNT(*) FROM users WHERE created_at > DATE_SUB(NOW(), INTERVAL 30 DAY);

SELECT AVG(salary) FROM employees WHERE department_id = 1;

SELECT SUM(total) FROM orders WHERE status = 'shipped';

SELECT MAX(created_at) FROM orders WHERE user_id = 1;

SELECT MIN(price) FROM products WHERE category_id = 3;

SELECT COUNT(DISTINCT status) FROM orders;

SELECT SUM(quantity) FROM order_items WHERE product_id = 1;

SELECT AVG(stock) FROM products GROUP BY category_id;

SELECT user_id, SUM(total) AS total_spent, COUNT(*) AS order_count FROM orders GROUP BY user_id ORDER BY total_spent DESC LIMIT 10;

SELECT COUNT(*) FROM products WHERE name LIKE '%widget%';

SELECT SUM(total) FROM orders WHERE DATE(created_at) = CURDATE();

SELECT MAX(price) - AVG(price) FROM products;

SELECT COUNT(*) FROM users WHERE email LIKE '%@company.com';

SELECT category_id, COUNT(*) FROM products WHERE stock > 0 GROUP BY category_id;

SELECT SUM(total) FROM orders WHERE status IN ('pending', 'processing');

SELECT AVG(price) FROM products WHERE name LIKE '%pro%';

SELECT MAX(quantity) FROM order_items;

SELECT MIN(created_at) FROM orders WHERE status = 'completed';

SELECT COUNT(DISTINCT LEFT(email, POSITION('@' IN email) - 1)) FROM users;

SELECT SUM(price * stock) FROM products;

SELECT AVG(CHAR_LENGTH(name)) FROM users;

SELECT COUNT(*) FROM products WHERE price = (SELECT MAX(price) FROM products);

SELECT SUM(total) FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip');

SELECT category_id, AVG(price - cost) AS avg_profit FROM products GROUP BY category_id;

SELECT COUNT(*) FROM users WHERE YEAR(created_at) = 2024;

SELECT MAX(total) - MIN(total) FROM orders GROUP BY user_id;

SELECT SUM(quantity * price) FROM order_items GROUP BY order_id;

SELECT AVG(COUNT(*)) FROM orders GROUP BY user_id;

SELECT user_id, MAX(created_at) FROM orders GROUP BY user_id;

SELECT status, COUNT(DISTINCT user_id) FROM orders GROUP BY status;

SELECT SUM(price) FROM products WHERE category_id = (SELECT id FROM categories WHERE name = 'Sale');

SELECT COUNT(*) FROM users WHERE email NOT LIKE '%@spam.com';

SELECT category_id, SUM(stock * price) FROM products GROUP BY category_id HAVING SUM(stock * price) > 10000;

SELECT MIN(price), MAX(price) FROM products GROUP BY category_id;

SELECT COUNT(DISTINCT DATE(created_at)) FROM orders;

SELECT AVG(total) FROM orders WHERE status = 'completed' GROUP BY user_id;

SELECT SUM(ABS(price - AVG(price))) FROM products GROUP BY category_id;

SELECT MAX(age) FROM users WHERE status = 'active';

SELECT MIN(price) FROM products WHERE stock > 0;

SELECT COUNT(*) FROM products WHERE category_id NOT IN (SELECT id FROM categories WHERE active = FALSE);

SELECT user_id, AVG(TIMESTAMPDIFF(DAY, created_at, updated_at)) FROM orders GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE created_at BETWEEN '2024-01-01' AND '2024-03-31';

SELECT category_id, COUNT(DISTINCT user_id) FROM orders GROUP BY category_id;

SELECT AVG(price) FROM products WHERE name LIKE '%air%';

SELECT MAX(stock) FROM products WHERE category_id = 5;

SELECT COUNT(*) FROM users WHERE name = (SELECT name FROM users ORDER BY created_at LIMIT 1);

SELECT SUM(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE price > 100);

SELECT status, MIN(total), MAX(total) FROM orders GROUP BY status;

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id HAVING AVG(salary) > 50000;

SELECT COUNT(*) FROM products WHERE price > (SELECT AVG(price) FROM products);

SELECT user_id, SUM(total) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY user_id;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) = (SELECT MAX(cnt) FROM (SELECT COUNT(*) AS cnt FROM products GROUP BY category_id) t);

SELECT SUM(total) FROM orders WHERE status = 'completed' AND created_at > DATE_SUB(NOW(), INTERVAL 7 DAY);

SELECT AVG(age) FROM users GROUP BY country HAVING AVG(age) > 30;

SELECT COUNT(DISTINCT LEFT(phone, 3)) FROM users;

SELECT MAX(price) FROM products WHERE name LIKE '%model%';

SELECT MIN(created_at) FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'new');

SELECT SUM(price * quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE status = 'completed');

SELECT category_id, AVG(stock) FROM products GROUP BY category_id HAVING AVG(stock) < 50;

SELECT COUNT(*) FROM users WHERE created_at > (SELECT created_at FROM users ORDER BY created_at DESC LIMIT 1);

SELECT user_id, COUNT(*) FROM orders GROUP BY user_id HAVING COUNT(*) >= ALL (SELECT COUNT(*) FROM orders GROUP BY user_id);

SELECT status, SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY status;

SELECT category_id, SUM(revenue) FROM (SELECT category_id, SUM(price * quantity) AS revenue FROM order_items GROUP BY category_id) t GROUP BY category_id;

SELECT MAX(CHAR_LENGTH(name)) FROM users;

SELECT AVG(price) FROM products WHERE price > 0 GROUP BY category_id ORDER BY AVG(price) DESC LIMIT 5;

SELECT COUNT(DISTINCT user_id) FROM orders WHERE total > (SELECT AVG(total) FROM orders);

SELECT SUM(total) FROM orders WHERE DATE_FORMAT(created_at, '%Y-%m') = '2024-01';

SELECT user_id, MAX(total) - MIN(total) FROM orders GROUP BY user_id HAVING MAX(total) - MIN(total) > 1000;

SELECT COUNT(*) FROM products WHERE category_id IN (SELECT category_id FROM order_items GROUP BY category_id HAVING SUM(quantity) > 100);

SELECT AVG(quantity) FROM order_items GROUP BY product_id;

SELECT category_id, COUNT(DISTINCT user_id) / COUNT(DISTINCT user_id) * 100 FROM orders GROUP BY category_id;

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id HAVING SUM(total) > 500;

SELECT MAX(price) FROM products WHERE name LIKE '%special%';

SELECT COUNT(*) FROM users WHERE email LIKE CONCAT('%@', SUBSTRING_INDEX(email, '@', -1));

SELECT SUM(price * stock) FROM products GROUP BY category_id HAVING SUM(price * stock) > 50000;

SELECT user_id, AVG(price) FROM order_items GROUP BY user_id;

SELECT MIN(total) FROM orders WHERE status = 'completed';

SELECT MAX(age) FROM users WHERE status = 'inactive';

SELECT COUNT(*) FROM products WHERE category_id = (SELECT id FROM categories WHERE name = 'Featured');

SELECT SUM(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE user_id = 1);

SELECT AVG(stock) FROM products WHERE stock > 0 GROUP BY category_id;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) BETWEEN 5 AND 20;

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) > (SELECT AVG(total) FROM orders);

SELECT MAX(created_at) - MIN(created_at) FROM orders GROUP BY user_id;

SELECT COUNT(DISTINCT email) FROM users WHERE email LIKE '%@%.com';

SELECT SUM(total) FROM orders WHERE DAYOFWEEK(created_at) = 1;

SELECT AVG(price) FROM products WHERE name REGEXP '^A';

SELECT status, COUNT(*) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY status;

SELECT department_id, MAX(salary) - MIN(salary) FROM employees GROUP BY department_id;

SELECT user_id, COUNT(DISTINCT product_id) FROM order_items GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY DATE(created_at) HAVING SUM(total) > 10000;

SELECT category_id, AVG(price) FROM products GROUP BY category_id HAVING AVG(price) > 50 AND AVG(price) < 200;

SELECT COUNT(*) FROM users WHERE created_at >= (SELECT DATE_SUB(MAX(created_at), INTERVAL 30 DAY) FROM users);

SELECT user_id, SUM(quantity) FROM order_items GROUP BY user_id ORDER BY SUM(quantity) DESC LIMIT 5;

SELECT MIN(price), MAX(price), AVG(price) FROM products WHERE price > 0;

SELECT status, AVG(TIMESTAMPDIFF(HOUR, created_at, updated_at)) FROM orders GROUP BY status;

SELECT category_id, COUNT(DISTINCT order_id) FROM order_items GROUP BY category_id;

SELECT user_id, COUNT(*) FROM orders WHERE MONTH(created_at) = MONTH(CURDATE()) GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'cancelled' GROUP BY user_id HAVING SUM(total) > 100;

SELECT MAX(price) FROM products WHERE name LIKE '%premium%';

SELECT COUNT(*) FROM users WHERE status = 'active' AND age > (SELECT AVG(age) FROM users);

SELECT SUM(price) FROM products WHERE category_id NOT IN (SELECT id FROM categories WHERE name = 'Discontinued');

SELECT AVG(quantity) FROM order_items GROUP BY order_id HAVING AVG(quantity) > 5;

SELECT user_id, MAX(created_at) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY user_id;

SELECT COUNT(DISTINCT LEFT(name, 1)) FROM users;

SELECT category_id, SUM(stock) FROM products GROUP BY category_id ORDER BY SUM(stock) DESC LIMIT 3;

SELECT SUM(total) FROM orders WHERE DATE(created_at) >= (SELECT DATE_SUB(MAX(DATE(created_at)), INTERVAL 7 DAY) FROM orders);

SELECT MAX(price) FROM products WHERE category_id = (SELECT id FROM categories ORDER BY id LIMIT 1);

SELECT user_id, AVG(total) FROM orders GROUP BY user_id HAVING AVG(total) > (SELECT AVG(total) / COUNT(DISTINCT user_id) FROM orders WHERE total > 0);

SELECT COUNT(*) FROM products WHERE price = MAX(price) OVER (PARTITION BY category_id);

SELECT status, SUM(total), COUNT(*) FROM orders GROUP BY status WITH ROLLUP;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) = ANY (SELECT COUNT(*) FROM products GROUP BY category_id);

SELECT user_id, SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 YEAR) GROUP BY user_id HAVING SUM(total) > 5000;

SELECT MIN(price), MAX(price) FROM products WHERE price > (SELECT AVG(price) FROM products);

SELECT COUNT(DISTINCT user_id) / COUNT(DISTINCT DATE(created_at)) FROM orders;

SELECT department_id, SUM(salary) FROM employees GROUP BY department_id HAVING SUM(salary) > (SELECT SUM(salary) / COUNT(*) FROM employees);

SELECT user_id, GROUP_CONCAT(DISTINCT status) FROM orders GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'shipped' GROUP BY user_id HAVING SUM(total) > 2000;

SELECT category_id, AVG(price) FROM products WHERE price > 0 GROUP BY category_id HAVING AVG(price) > (SELECT AVG(price) FROM products WHERE price > 0) * 1.5;

SELECT COUNT(*) FROM products WHERE category_id IN (SELECT category_id FROM products GROUP BY category_id HAVING COUNT(*) > 10);

SELECT MAX(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE category_id = 1);

SELECT user_id, COUNT(*) FROM orders WHERE QUARTER(created_at) = QUARTER(CURDATE()) GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id ORDER BY SUM(total) DESC LIMIT 5;

SELECT AVG(price) FROM products WHERE name LIKE '%basic%';

SELECT COUNT(*) FROM users WHERE YEAR(created_at) = YEAR(CURDATE()) AND MONTH(created_at) = MONTH(CURDATE());

SELECT category_id, SUM(quantity) FROM order_items GROUP BY category_id HAVING SUM(quantity) > (SELECT AVG(SUM(quantity)) FROM order_items GROUP BY category_id);

SELECT MAX(total) FROM orders WHERE status = 'refunded';

SELECT SUM(price * quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE user_id = 1 AND status = 'completed');

SELECT COUNT(DISTINCT product_id) FROM order_items GROUP BY order_id HAVING COUNT(DISTINCT product_id) > 3;

SELECT AVG(age) FROM users GROUP BY status HAVING AVG(age) > 30;

SELECT SUM(total) FROM orders WHERE user_id IN (SELECT id FROM users WHERE country = 'USA') GROUP BY user_id;

SELECT category_id, MAX(stock) FROM products GROUP BY category_id;

SELECT COUNT(*) FROM products WHERE price > (SELECT MIN(price) FROM products) * 2;

SELECT user_id, MAX(total) FROM orders GROUP BY user_id HAVING MAX(total) > (SELECT AVG(MAX(total)) FROM orders GROUP BY user_id);

SELECT SUM(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE stock < 10);

SELECT AVG(CHAR_LENGTH(name)) FROM users GROUP BY status;

SELECT status, COUNT(*) FROM orders WHERE created_at > (SELECT DATE_SUB(MAX(created_at), INTERVAL 30 DAY) FROM orders) GROUP BY status;

SELECT user_id, SUM(total) FROM orders WHERE status IN ('completed', 'shipped') GROUP BY user_id HAVING SUM(total) > 1000;

SELECT category_id, COUNT(*) FROM products WHERE stock > 0 GROUP BY category_id HAVING COUNT(*) >= 5;

SELECT MAX(price) FROM products WHERE name LIKE '%ultra%';

SELECT COUNT(*) FROM users WHERE email REGEXP '^[a-z].*@.*\\.com$';

SELECT SUM(total) FROM orders WHERE HOUR(created_at) BETWEEN 9 AND 17 GROUP BY HOUR(created_at);

SELECT AVG(price) FROM products GROUP BY category_id HAVING AVG(price) BETWEEN 20 AND 100;

SELECT user_id, COUNT(DISTINCT DATE(created_at)) FROM orders GROUP BY user_id HAVING COUNT(DISTINCT DATE(created_at)) > 10;

SELECT SUM(price * quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY));

SELECT COUNT(*) FROM products WHERE category_id = (SELECT id FROM categories WHERE name = (SELECT name FROM categories ORDER BY id LIMIT 1));

SELECT MIN(salary) FROM employees GROUP BY department_id HAVING MIN(salary) > 30000;

SELECT user_id, GROUP_CONCAT(DISTINCT category_id) FROM orders GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE DAY(created_at) = DAY(CURDATE()) GROUP BY user_id;

SELECT category_id, AVG(price - cost) FROM products GROUP BY category_id HAVING AVG(price - cost) > 10;

SELECT MAX(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE price > 50);

SELECT COUNT(*) FROM users WHERE created_at > (SELECT DATE_SUB(MAX(created_at), INTERVAL 1 YEAR) FROM users);

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY WEEK(created_at) HAVING SUM(total) > 5000;

SELECT user_id, MAX(total) - AVG(total) FROM orders GROUP BY user_id;

SELECT AVG(price) FROM products WHERE name LIKE '%plus%';

SELECT COUNT(DISTINCT user_id) FROM orders WHERE status = 'pending';

SELECT SUM(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE category_id IN (SELECT id FROM categories WHERE name LIKE '%tech%'));

SELECT MIN(price) FROM products GROUP BY category_id HAVING MIN(price) < 20;

SELECT MAX(age) FROM users WHERE status = 'active' GROUP BY country HAVING MAX(age) > 60;

SELECT COUNT(*) FROM orders WHERE MONTH(created_at) = 1 GROUP BY user_id HAVING COUNT(*) > 3;

SELECT SUM(total) FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) GROUP BY user_id;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) != (SELECT COUNT(*) FROM products GROUP BY category_id ORDER BY COUNT(*) LIMIT 1);

SELECT AVG(total) FROM orders GROUP BY user_id HAVING AVG(total) > (SELECT AVG(total) FROM orders WHERE total > 0);

SELECT MAX(price) FROM products WHERE stock > 0 AND price > 0;

SELECT COUNT(*) FROM users WHERE email LIKE CONCAT('%', SUBSTRING_INDEX(email, '@', -1));

SELECT SUM(quantity * price) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE status = 'completed') GROUP BY product_id;

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id ORDER BY AVG(salary) DESC LIMIT 3;

SELECT user_id, COUNT(*) FROM orders WHERE DATE(created_at) >= '2024-01-01' GROUP BY user_id HAVING COUNT(*) >= 5;

SELECT category_id, SUM(stock) FROM products WHERE stock > 0 GROUP BY category_id HAVING SUM(stock) > 100;

SELECT MAX(total) FROM orders WHERE status = 'completed' GROUP BY user_id;

SELECT COUNT(*) FROM products WHERE price > (SELECT MAX(price) * 0.5 FROM products);

SELECT user_id, SUM(total) / COUNT(*) FROM orders GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) GROUP BY status;

SELECT category_id, COUNT(DISTINCT user_id) / COUNT(DISTINCT user_id) * 100 FROM orders GROUP BY category_id;

SELECT AVG(price) FROM products WHERE name LIKE '%mini%';

SELECT COUNT(*) FROM users WHERE status = 'active' GROUP BY LEFT(phone, 3);

SELECT SUM(total) FROM orders WHERE DAYOFWEEK(created_at) IN (1, 7) GROUP BY user_id;

SELECT MAX(price) FROM products GROUP BY category_id HAVING MAX(price) > 100;

SELECT user_id, COUNT(DISTINCT product_id) FROM order_items GROUP BY user_id HAVING COUNT(DISTINCT product_id) > 5;

SELECT AVG(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE user_id = 1);

SELECT SUM(price) FROM products WHERE category_id = (SELECT id FROM categories WHERE name LIKE '%sale%' LIMIT 1);

SELECT COUNT(*) FROM orders WHERE user_id IN (SELECT user_id FROM orders GROUP BY user_id HAVING SUM(total) > 10000);

SELECT MIN(total) FROM orders WHERE status = 'completed' GROUP BY user_id;

SELECT MAX(age) - MIN(age) FROM users GROUP BY status;

SELECT SUM(total) FROM orders WHERE YEAR(created_at) = 2024 AND MONTH(created_at) = MONTH(CURDATE()) GROUP BY user_id;

SELECT category_id, COUNT(*) FROM products WHERE price > 0 GROUP BY category_id HAVING COUNT(*) < 10;

SELECT user_id, GROUP_CONCAT(DISTINCT YEAR(created_at)) FROM orders GROUP BY user_id;

SELECT AVG(stock) FROM products GROUP BY category_id HAVING AVG(stock) > (SELECT AVG(stock) FROM products);

SELECT SUM(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY));

SELECT MAX(price) FROM products WHERE name LIKE '%pro%';

SELECT COUNT(*) FROM users WHERE created_at > (SELECT MIN(created_at) FROM users WHERE status = 'active');

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id HAVING SUM(total) > (SELECT AVG(total) FROM orders WHERE status = 'pending');

SELECT category_id, AVG(price) FROM products GROUP BY category_id ORDER BY AVG(price) DESC LIMIT 5;

SELECT user_id, COUNT(*) FROM orders WHERE QUARTER(created_at) = 1 GROUP BY user_id HAVING COUNT(*) > 2;

SELECT MIN(price) FROM products WHERE name LIKE '%luxury%';

SELECT COUNT(DISTINCT user_id) FROM orders WHERE status = 'cancelled';

SELECT SUM(price * stock) FROM products GROUP BY category_id HAVING SUM(price * stock) > 50000;

SELECT MAX(quantity) FROM order_items WHERE product_id = (SELECT id FROM products ORDER BY price DESC LIMIT 1);

SELECT user_id, SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY user_id HAVING SUM(total) > 500;

SELECT AVG(price) FROM products WHERE name REGEXP '^[^a-z]*[a-z]';

SELECT COUNT(*) FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) - 1 GROUP BY user_id;

SELECT category_id, MAX(stock) - MIN(stock) FROM products GROUP BY category_id;

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY DATE(created_at) ORDER BY SUM(total) DESC LIMIT 10;

SELECT user_id, MAX(created_at) FROM orders GROUP BY user_id HAVING MAX(created_at) > DATE_SUB(NOW(), INTERVAL 30 DAY);

SELECT COUNT(*) FROM products WHERE category_id IN (SELECT category_id FROM products GROUP BY category_id HAVING SUM(stock) > 1000);

SELECT AVG(age) FROM users GROUP BY LEFT(phone, 3);

SELECT status, SUM(total) FROM orders GROUP BY status HAVING SUM(total) > 10000;

SELECT user_id, COUNT(DISTINCT status) FROM orders GROUP BY user_id HAVING COUNT(DISTINCT status) >= 3;

SELECT MAX(price) FROM products WHERE name LIKE '%elite%';

SELECT COUNT(*) FROM users WHERE email LIKE CONCAT('%@', (SELECT SUBSTRING_INDEX(email, '@', -1) FROM users LIMIT 1));

SELECT SUM(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE user_id IN (SELECT id FROM users WHERE status = 'vip'));

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) = ANY (SELECT COUNT(*) FROM products GROUP BY category_id);

SELECT user_id, AVG(total) FROM orders WHERE total > 0 GROUP BY user_id HAVING AVG(total) BETWEEN 50 AND 200;

SELECT SUM(total) FROM orders WHERE DAY(created_at) <= DAY(CURDATE()) GROUP BY user_id;

SELECT MAX(price) FROM products WHERE stock > 0 AND category_id = (SELECT id FROM categories WHERE name LIKE '%featured%' LIMIT 1);

SELECT COUNT(DISTINCT user_id) FROM orders WHERE status = 'completed' AND created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY);

SELECT user_id, SUM(total) FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) GROUP BY user_id HAVING SUM(total) > 1000;

SELECT AVG(CHAR_LENGTH(name)) FROM products GROUP BY category_id;

SELECT status, COUNT(*) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY) GROUP BY status;

SELECT category_id, SUM(stock) FROM products WHERE stock > 0 GROUP BY category_id ORDER BY SUM(stock) DESC LIMIT 5;

SELECT MIN(price) FROM products WHERE price > 0 GROUP BY category_id HAVING MIN(price) BETWEEN 10 AND 50;

SELECT COUNT(*) FROM users WHERE created_at > (SELECT DATE_SUB(MAX(created_at), INTERVAL 1 MONTH) FROM users);

SELECT user_id, GROUP_CONCAT(DISTINCT product_id ORDER BY quantity DESC SEPARATOR ',') FROM order_items GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'shipped' GROUP BY user_id HAVING SUM(total) > 5000;

SELECT MAX(age) FROM users GROUP BY status HAVING MAX(age) > 50;

SELECT COUNT(DISTINCT email) FROM users WHERE email LIKE '%@business.com';

SELECT category_id, AVG(price - cost) FROM products GROUP BY category_id HAVING AVG(price - cost) > 20;

SELECT user_id, COUNT(*) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 YEAR) GROUP BY user_id HAVING COUNT(*) > 10;

SELECT SUM(quantity * price) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE category_id = (SELECT id FROM categories WHERE name LIKE '%popular%' LIMIT 1));

SELECT status, AVG(price) FROM orders GROUP BY status HAVING AVG(price) > 100;

SELECT department_id, MAX(salary) FROM employees GROUP BY department_id HAVING MAX(salary) > 80000;

SELECT COUNT(*) FROM products WHERE price > (SELECT AVG(price) * 2 FROM products WHERE price > 0);

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) > (SELECT MAX(total) FROM orders) * 0.5;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) = (SELECT MIN(cnt) FROM (SELECT COUNT(*) AS cnt FROM products GROUP BY category_id) t);

SELECT AVG(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING AVG(total) > (SELECT AVG(total) FROM orders WHERE status = 'completed');

SELECT MAX(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE status = 'completed');

SELECT user_id, COUNT(DISTINCT DATE(created_at)) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE MONTHNAME(created_at) = 'January' GROUP BY user_id;

SELECT COUNT(*) FROM products WHERE category_id NOT IN (SELECT category_id FROM products WHERE stock > 0 GROUP BY category_id);

SELECT MAX(price) - MIN(price) FROM products GROUP BY category_id HAVING MAX(price) - MIN(price) > 50;

SELECT user_id, AVG(quantity) FROM order_items GROUP BY user_id HAVING AVG(quantity) > 3;

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id ORDER BY SUM(total) DESC LIMIT 10;

SELECT category_id, COUNT(DISTINCT user_id) FROM order_items GROUP BY category_id;

SELECT MIN(age) FROM users WHERE status = 'active' GROUP BY country HAVING MIN(age) < 25;

SELECT COUNT(*) FROM users WHERE YEAR(created_at) >= 2020 GROUP BY status;

SELECT user_id, SUM(total) FROM orders WHERE status = 'refunded' GROUP BY user_id HAVING SUM(total) > 100;

SELECT AVG(price) FROM products WHERE name LIKE '%standard%';

SELECT SUM(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE category_id IN (SELECT id FROM categories WHERE name LIKE '%sale%'));

SELECT MAX(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING MAX(total) > 5000;

SELECT COUNT(DISTINCT LEFT(email, POSITION('@' IN email) - 1)) FROM users;

SELECT category_id, SUM(price * stock) FROM products GROUP BY category_id HAVING SUM(price * stock) > 100000;

SELECT user_id, COUNT(*) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY user_id HAVING COUNT(*) > 3;

SELECT MIN(price) FROM products WHERE name LIKE '%deluxe%';

SELECT SUM(total) FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) - 1 GROUP BY status;

SELECT category_id, AVG(price) FROM products WHERE price > 0 GROUP BY category_id HAVING AVG(price) > (SELECT AVG(price) FROM products WHERE category_id = 1);

SELECT COUNT(*) FROM users WHERE email LIKE CONCAT('%@', (SELECT SUBSTRING_INDEX(email, '@', -1) FROM users WHERE status = 'vip' LIMIT 1));

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) BETWEEN 1000 AND 5000;

SELECT MAX(age) FROM users GROUP BY LEFT(phone, 3) HAVING MAX(age) > 40;

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING SUM(total) > (SELECT AVG(SUM(total)) FROM orders GROUP BY user_id);

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) IN (SELECT COUNT(*) FROM products GROUP BY category_id ORDER BY COUNT(*) DESC LIMIT 3);

SELECT user_id, GROUP_CONCAT(DISTINCT LEFT(name, 1)) FROM products GROUP BY user_id;

SELECT MIN(price) FROM products WHERE name LIKE '%premium%';

SELECT COUNT(*) FROM orders WHERE created_at >= (SELECT DATE_SUB(MAX(created_at), INTERVAL 7 DAY) FROM orders WHERE status = 'pending');

SELECT SUM(total) FROM orders WHERE QUARTER(created_at) = QUARTER(CURDATE()) GROUP BY user_id HAVING SUM(total) > 2000;

SELECT AVG(price) FROM products WHERE name REGEXP '^[^a-zA-Z]*[a-zA-Z]';

SELECT category_id, SUM(stock) FROM products WHERE stock > 0 GROUP BY category_id HAVING SUM(stock) > 500;

SELECT MAX(quantity) FROM order_items WHERE order_id IN (SELECT id FROM orders WHERE user_id = (SELECT id FROM users WHERE status = 'vip' LIMIT 1));

SELECT user_id, COUNT(*) FROM orders WHERE status = 'pending' GROUP BY user_id HAVING COUNT(*) >= 5;

SELECT SUM(total) FROM orders WHERE status IN ('completed', 'shipped') GROUP BY user_id ORDER BY SUM(total) DESC LIMIT 10;

SELECT category_id, COUNT(DISTINCT product_id) FROM order_items GROUP BY category_id;

SELECT MIN(salary) FROM employees GROUP BY department_id HAVING MIN(salary) BETWEEN 30000 AND 50000;

SELECT COUNT(*) FROM products WHERE price > (SELECT MAX(price) * 0.8 FROM products WHERE stock > 0);

SELECT user_id, AVG(total) FROM orders GROUP BY user_id HAVING AVG(total) > (SELECT AVG(total) FROM orders WHERE status = 'completed');

SELECT SUM(total) FROM orders WHERE status = 'cancelled' GROUP BY user_id HAVING SUM(total) > (SELECT AVG(SUM(total)) FROM orders GROUP BY user_id WHERE status = 'cancelled');

SELECT MAX(price) FROM products WHERE name LIKE '%ultimate%';

SELECT COUNT(DISTINCT user_id) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY user_id HAVING COUNT(DISTINCT user_id) > 1;

SELECT category_id, SUM(price * quantity) FROM order_items GROUP BY category_id ORDER BY SUM(price * quantity) DESC LIMIT 5;

SELECT user_id, COUNT(*) FROM orders WHERE YEAR(created_at) = YEAR(CURDATE()) - 1 GROUP BY user_id HAVING COUNT(*) >= 12;

SELECT AVG(stock) FROM products WHERE category_id = (SELECT id FROM categories WHERE name LIKE '%featured%' LIMIT 1);

SELECT SUM(total) FROM orders WHERE DAY(created_at) = DAY(CURDATE()) GROUP BY user_id HAVING SUM(total) > 100;

SELECT MAX(price) FROM products GROUP BY category_id HAVING MAX(price) < 100;

SELECT COUNT(*) FROM users WHERE email LIKE CONCAT('%@', (SELECT SUBSTRING_INDEX(email, '@', -1) FROM users WHERE status = 'active' LIMIT 1));

SELECT user_id, SUM(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING SUM(total) > 50000;

SELECT category_id, AVG(price) FROM products GROUP BY category_id HAVING AVG(price) > (SELECT AVG(price) FROM products) * 2;

SELECT MIN(quantity) FROM order_items GROUP BY product_id HAVING MIN(quantity) > 1;

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id HAVING SUM(total) > (SELECT MAX(SUM(total)) FROM orders GROUP BY user_id) * 0.5;

SELECT MAX(age) FROM users GROUP BY status HAVING MAX(age) BETWEEN 40 AND 60;

SELECT COUNT(*) FROM products WHERE category_id IN (SELECT category_id FROM products WHERE price > 100 GROUP BY category_id);

SELECT user_id, GROUP_CONCAT(DISTINCT DATE(created_at)) FROM orders GROUP BY user_id;

SELECT AVG(price) FROM products WHERE name LIKE '%compact%';

SELECT SUM(total) FROM orders WHERE created_at >= (SELECT MIN(created_at) FROM orders WHERE status = 'completed');

SELECT category_id, MAX(stock) FROM products GROUP BY category_id HAVING MAX(stock) > 100;

SELECT user_id, COUNT(*) FROM orders WHERE MONTH(created_at) = MONTH(CURDATE()) - 1 GROUP BY user_id;

SELECT MAX(price) FROM products WHERE name REGEXP '^[0-9]';

SELECT COUNT(DISTINCT user_id) FROM orders WHERE status = 'shipped' GROUP BY user_id HAVING COUNT(DISTINCT user_id) > 5;

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING SUM(total) > (SELECT SUM(total) / COUNT(DISTINCT user_id) FROM orders WHERE status = 'completed');

SELECT category_id, COUNT(*) FROM products WHERE price > 0 GROUP BY category_id HAVING COUNT(*) >= 3;

SELECT MIN(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING MIN(total) > 100;

SELECT user_id, SUM(quantity) FROM order_items GROUP BY user_id ORDER BY SUM(quantity) DESC LIMIT 5;

SELECT AVG(price) FROM products GROUP BY category_id ORDER BY AVG(price) ASC LIMIT 5;

SELECT COUNT(*) FROM orders WHERE YEARWEEK(created_at) = YEARWEEK(NOW()) GROUP BY user_id;

SELECT MAX(price) FROM products WHERE name LIKE '%special%';

SELECT SUM(price) FROM products WHERE category_id IN (SELECT id FROM categories WHERE name LIKE '%sale%' OR name LIKE '%clearance%');

SELECT user_id, COUNT(*) FROM orders WHERE status = 'cancelled' GROUP BY user_id HAVING COUNT(*) > 1;

SELECT AVG(age) FROM users GROUP BY LEFT(phone, 2) HAVING AVG(age) > 30;

SELECT category_id, SUM(stock * price) FROM products GROUP BY category_id HAVING SUM(stock * price) BETWEEN 10000 AND 50000;

SELECT MIN(price) FROM products WHERE stock > 0 GROUP BY category_id;

SELECT user_id, MAX(created_at) - MIN(created_at) FROM orders GROUP BY user_id HAVING MAX(created_at) - MIN(created_at) > INTERVAL 30 DAY;

SELECT COUNT(*) FROM products WHERE price > (SELECT AVG(price) FROM products WHERE category_id = 1);

SELECT SUM(total) FROM orders WHERE status = 'pending' GROUP BY user_id HAVING SUM(total) > 10000;

SELECT MAX(quantity) FROM order_items WHERE product_id IN (SELECT id FROM products WHERE category_id = (SELECT id FROM categories WHERE name LIKE '%popular%' LIMIT 1));

SELECT user_id, AVG(total) FROM orders GROUP BY user_id HAVING AVG(total) > (SELECT MAX(AVG(total)) FROM orders GROUP BY user_id) * 0.8;

SELECT category_id, COUNT(DISTINCT order_id) FROM order_items GROUP BY category_id HAVING COUNT(DISTINCT order_id) > 100;

SELECT MIN(salary) FROM employees GROUP BY department_id ORDER BY MIN(salary) DESC LIMIT 3;

SELECT COUNT(*) FROM users WHERE created_at > (SELECT DATE_SUB(MAX(created_at), INTERVAL 90 DAY) FROM users WHERE status = 'active');

SELECT user_id, GROUP_CONCAT(DISTINCT status ORDER BY created_at) FROM orders GROUP BY user_id;

SELECT SUM(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING SUM(total) > (SELECT AVG(SUM(total)) FROM orders GROUP BY user_id WHERE status = 'completed') * 1.5;

SELECT category_id, AVG(price) FROM products GROUP BY category_id HAVING AVG(price) BETWEEN (SELECT AVG(price) FROM products) * 0.5 AND (SELECT AVG(price) FROM products) * 1.5;
