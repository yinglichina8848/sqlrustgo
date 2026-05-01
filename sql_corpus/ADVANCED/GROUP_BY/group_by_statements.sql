-- GROUP BY and HAVING Test Cases
-- Compatibility: MySQL 5.7+

SELECT category_id, COUNT(*) FROM products GROUP BY category_id;

SELECT status, SUM(total), AVG(total) FROM orders GROUP BY status;

SELECT YEAR(created_at) AS year, MONTH(created_at) AS month, SUM(total) FROM orders GROUP BY YEAR(created_at), MONTH(created_at);

SELECT category_id, COUNT(*) AS cnt, AVG(price) AS avg_price FROM products GROUP BY category_id HAVING cnt > 5;

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id HAVING COUNT(*) > 3;

SELECT user_id, SUM(total) AS total_spent FROM orders GROUP BY user_id HAVING SUM(total) > 1000 ORDER BY total_spent DESC;

SELECT category_id, MAX(price), MIN(price), AVG(price) FROM products GROUP BY category_id;

SELECT LEFT(name, 1) AS initial, COUNT(*) FROM users GROUP BY LEFT(name, 1);

SELECT DATE(created_at) AS date, COUNT(*) AS orders FROM orders GROUP BY DATE(created_at) ORDER BY date;

SELECT category_id, SUM(quantity) AS total_sold FROM order_items GROUP BY category_id;

SELECT YEAR(created_at), COUNT(*) FROM users GROUP BY YEAR(created_at);

SELECT status, AVG(TIMESTAMPDIFF(DAY, created_at, updated_at)) AS avg_days FROM orders GROUP BY status;

SELECT category_id, COUNT(DISTINCT user_id) AS unique_customers FROM orders GROUP BY category_id;

SELECT DATE_FORMAT(created_at, '%Y-%m') AS month, SUM(total) FROM orders GROUP BY DATE_FORMAT(created_at, '%Y-%m');

SELECT user_id, COUNT(*) AS orders, SUM(total) AS total FROM orders GROUP BY user_id HAVING COUNT(*) > 5;

SELECT category, SUM(revenue) FROM (SELECT category_id, SUM(price * quantity) AS revenue FROM order_items GROUP BY category_id) t GROUP BY category;

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id HAVING AVG(salary) > 50000;

SELECT name, COUNT(*) FROM products GROUP BY name HAVING COUNT(*) > 1;

SELECT YEAR, MONTH, DAY, SUM(sales) FROM sales_data GROUP BY YEAR, MONTH, DAY WITH ROLLUP;

SELECT region, COUNT(DISTINCT customer_id), SUM(amount) FROM orders GROUP BY region;

SELECT MONTHNAME(created_at) AS month, COUNT(*) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY MONTHNAME(created_at);

SELECT user_id, GROUP_CONCAT(DISTINCT status ORDER BY created_at SEPARATOR ',') FROM orders GROUP BY user_id;

SELECT category_id, SUM(price * stock) AS inventory_value FROM products GROUP BY category_id;

SELECT WEEK(created_at) AS week_num, AVG(total) FROM orders GROUP BY WEEK(created_at);

SELECT LEFT(email, POSITION('@' IN email) - 1) AS domain, COUNT(*) FROM users GROUP BY LEFT(email, POSITION('@' IN email) - 1);

SELECT price_range, COUNT(*) FROM (SELECT CASE WHEN price < 50 THEN 'cheap' WHEN price < 100 THEN 'medium' ELSE 'expensive' END AS price_range FROM products) t GROUP BY price_range;

SELECT department_id, COUNT(*) AS headcount FROM employees WHERE title LIKE '%Manager%' GROUP BY department_id;

SELECT DATE(created_at) AS date, COUNT(DISTINCT user_id) FROM orders GROUP BY DATE(created_at) HAVING COUNT(DISTINCT user_id) > 10;

SELECT category_id, AVG(price - cost) AS avg_profit FROM products GROUP BY category_id;

SELECT YEAR(created_at) AS year, QUARTER(created_at) AS quarter, SUM(total) FROM orders GROUP BY YEAR(created_at), QUARTER(created_at);

SELECT country, COUNT(DISTINCT city) FROM locations GROUP BY country HAVING COUNT(DISTINCT city) > 5;

SELECT LEFT(name, 3), COUNT(*) FROM products GROUP BY LEFT(name, 3) ORDER BY COUNT(*) DESC LIMIT 5;

SELECT user_id, COUNT(*) AS orders, SUM(total) AS total, CASE WHEN SUM(total) > 5000 THEN 'VIP' WHEN SUM(total) > 1000 THEN 'Regular' ELSE 'New' END AS tier FROM orders GROUP BY user_id;

SELECT category_id, SUM(quantity) FROM order_items GROUP BY category_id HAVING SUM(quantity) > 100;

SELECT HOUR(created_at) AS hour, COUNT(*) FROM orders GROUP BY HOUR(created_at);

SELECT department_id, SUM(salary) FROM employees GROUP BY department_id WITH ROLLUP;

SELECT status, COUNT(*) FROM orders WHERE created_at > DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY status;

SELECT DATE_SUB(DATE(created_at), INTERVAL WEEKDAY(created_at) DAY) AS week_start, SUM(total) FROM orders GROUP BY week_start;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id ORDER BY COUNT(*) DESC LIMIT 3;

SELECT user_id, COUNT(*) AS orders FROM orders WHERE YEAR(created_at) = 2024 GROUP BY user_id HAVING COUNT(*) > 12;

SELECT country, AVG(age) FROM users GROUP BY country HAVING AVG(age) > 30;

SELECT MONTH(created_at) AS m, COUNT(DISTINCT user_id), SUM(total) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY MONTH(created_at);

SELECT LEFT(phone, 3) AS area_code, COUNT(*) FROM users GROUP BY LEFT(phone, 3);

SELECT product_id, SUM(quantity * price) AS revenue FROM order_items GROUP BY product_id ORDER BY revenue DESC LIMIT 5;

SELECT category_id, COUNT(*) AS products, AVG(price) AS avg_price FROM products WHERE price > 0 GROUP BY category_id;

SELECT YEARWEEK(created_at) AS yearweek, SUM(total) FROM orders GROUP BY YEARWEEK(created_at);

SELECT user_id, MAX(created_at) AS last_order FROM orders GROUP BY user_id;

SELECT status, MIN(total), MAX(total), AVG(total) FROM orders GROUP BY status;

SELECT DATE(created_at) AS date, COUNT(*) FROM orders GROUP BY date HAVING COUNT(*) > 5;

SELECT LEFT(name, 1) AS letter, COUNT(DISTINCT status) FROM users GROUP BY LEFT(name, 1);

SELECT country, COUNT(*) FROM users GROUP BY country HAVING COUNT(*) BETWEEN 10 AND 100;

SELECT department_id, GROUP_CONCAT(DISTINCT title SEPARATOR '; ') FROM employees GROUP BY department_id;

SELECT category_id, SUM(stock) AS total_stock FROM products GROUP BY category_id HAVING SUM(stock) > 1000;

SELECT HOUR(created_at) AS hour_bucket, COUNT(*) FROM orders GROUP BY FLOOR(HOUR(created_at) / 4);

SELECT user_id, COUNT(*) FROM orders WHERE status IN ('completed', 'shipped') GROUP BY user_id HAVING COUNT(*) >= 3;

SELECT category, SUM(revenue) FROM (SELECT category_id, SUM(price * quantity) AS revenue FROM order_items GROUP BY category_id) t GROUP BY category;

SELECT status, YEAR(created_at) AS year, COUNT(*) FROM orders GROUP BY status, YEAR(created_at);

SELECT user_id, SUM(total) FROM orders WHERE created_at > '2024-01-01' GROUP BY user_id HAVING SUM(total) > 500;

SELECT department_id, COUNT(*) AS employees, MAX(salary) AS max_sal FROM employees GROUP BY department_id;

SELECT LEFT(email, POSITION('.' IN email) - 1) AS name_part, COUNT(*) FROM users GROUP BY name_part;

SELECT DAYOFWEEK(created_at) AS dow, AVG(total) FROM orders GROUP BY dow;

SELECT category_id, SUM(ABS(price - AVG(price))) FROM products GROUP BY category_id;

SELECT YEAR(created_at), MONTH(created_at), COUNT(*) FROM users GROUP BY YEAR(created_at), MONTH(created_at);

SELECT region, SUM(total) AS total FROM orders GROUP BY region ORDER BY total DESC LIMIT 5;

SELECT MONTH(created_at) AS m, COUNT(DISTINCT user_id) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY m;

SELECT name, COUNT(*) FROM products GROUP BY name HAVING COUNT(*) = (SELECT COUNT(*) FROM products GROUP BY name ORDER BY COUNT(*) DESC LIMIT 1);

SELECT category_id, COUNT(DISTINCT user_id) FROM orders GROUP BY category_id;

SELECT DATE(created_at) AS date, SUM(total) FROM orders WHERE status = 'completed' GROUP BY date;

SELECT LEFT(phone, 3) AS area, COUNT(DISTINCT user_id) FROM users GROUP BY area;

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id HAVING COUNT(*) > AVG(COUNT(*)) OVER ();

SELECT category_id, SUM(quantity) FROM order_items GROUP BY category_id WITH ROLLUP;

SELECT user_id, COUNT(*) AS cnt FROM orders GROUP BY user_id ORDER BY cnt DESC LIMIT 10;

SELECT status, SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY) GROUP BY status;

SELECT YEAR(created_at), COUNT(*) FROM users WHERE YEAR(created_at) >= 2020 GROUP BY YEAR(created_at);

SELECT country, AVG(price) FROM products GROUP BY country HAVING AVG(price) > 50;

SELECT MONTHNAME(created_at) AS month, SUM(total) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY MONTHNAME(created_at);

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id HAVING COUNT(*) >= ALL (SELECT COUNT(*) FROM employees GROUP BY department_id);

SELECT LEFT(name, 10), COUNT(*) FROM products GROUP BY LEFT(name, 10);

SELECT user_id, SUM(total), COUNT(*) FROM orders GROUP BY user_id HAVING SUM(total) > 0 AND COUNT(*) > 0;

SELECT category_id, AVG(price) FROM products GROUP BY category_id HAVING AVG(price) > (SELECT AVG(price) FROM products);

SELECT status, YEAR(created_at), SUM(total) FROM orders GROUP BY status, YEAR(created_at) WITH ROLLUP;

SELECT country, COUNT(DISTINCT city) FROM users GROUP BY country ORDER BY COUNT(DISTINCT city) DESC;

SELECT DATE(created_at) AS date, COUNT(DISTINCT user_id), SUM(total) FROM orders GROUP BY date ORDER BY date DESC LIMIT 30;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) >= 1;

SELECT user_id, COUNT(DISTINCT status) FROM orders GROUP BY user_id HAVING COUNT(DISTINCT status) > 1;

SELECT LEFT(name, 1), SUM(stock) FROM products GROUP BY LEFT(name, 1);

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id HAVING AVG(salary) > 30000 ORDER BY AVG(salary);

SELECT YEARWEEK(created_at) AS yw, COUNT(*) FROM orders GROUP BY yw ORDER BY yw DESC LIMIT 10;

SELECT status, AVG(TIMESTAMPDIFF(HOUR, created_at, updated_at)) FROM orders GROUP BY status;

SELECT user_id, GROUP_CONCAT(DISTINCT product_id ORDER BY quantity DESC SEPARATOR ',') FROM order_items GROUP BY user_id;

SELECT category_id, SUM(price * quantity) AS revenue FROM order_items GROUP BY category_id ORDER BY revenue DESC LIMIT 5;

SELECT DATE_FORMAT(created_at, '%Y-%m-%d %H:00:00') AS hour, COUNT(*) FROM orders GROUP BY hour;

SELECT status, COUNT(DISTINCT user_id) FROM orders GROUP BY status;

SELECT LEFT(email, POSITION('@' IN email)) AS domain, COUNT(*) FROM users GROUP BY domain;

SELECT user_id, SUM(total) FROM orders WHERE status = 'completed' GROUP BY user_id HAVING SUM(total) > 1000;

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id HAVING COUNT(*) < 10;

SELECT category_id, AVG(price) AS avg, MIN(price) AS min, MAX(price) AS max FROM products GROUP BY category_id;

SELECT YEAR(created_at), COUNT(*) FROM orders GROUP BY YEAR(created_at) ORDER BY YEAR(created_at);

SELECT status, SUM(total) FROM orders GROUP BY status HAVING SUM(total) > 10000;

SELECT user_id, COUNT(*) FROM orders WHERE created_at > DATE_SUB(NOW(), INTERVAL 30 DAY) GROUP BY user_id;

SELECT country, COUNT(DISTINCT email) FROM users GROUP BY country;

SELECT category_id, SUM(stock) FROM products WHERE stock > 0 GROUP BY category_id;

SELECT DATE(created_at), COUNT(*) FROM orders WHERE status = 'pending' GROUP BY DATE(created_at);

SELECT LEFT(phone, 2) AS country_code, COUNT(*) FROM users GROUP BY LEFT(phone, 2);

SELECT category_id, COUNT(*) AS products FROM products GROUP BY category_id ORDER BY products DESC LIMIT 5;

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) BETWEEN 100 AND 1000;

SELECT HOUR(created_at) AS hour, COUNT(*) FROM orders GROUP BY hour HAVING COUNT(*) > 10;

SELECT department_id, SUM(salary) FROM employees GROUP BY department_id WITH ROLLUP HAVING SUM(salary) IS NOT NULL;

SELECT status, COUNT(*) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY status;

SELECT category_id, COUNT(DISTINCT name) FROM products GROUP BY category_id;

SELECT LEFT(name, 1) AS initial, SUM(quantity) FROM order_items GROUP BY initial ORDER BY SUM(quantity) DESC;

SELECT user_id, MAX(total) FROM orders GROUP BY user_id HAVING MAX(total) > 500;

SELECT region, AVG(age) FROM users GROUP BY region HAVING AVG(age) BETWEEN 25 AND 40;

SELECT DATE(created_at) AS date, COUNT(*) FROM orders GROUP BY date ORDER BY date DESC LIMIT 7;

SELECT category_id, SUM(price) FROM products GROUP BY category_id HAVING SUM(price) > 1000;

SELECT user_id, COUNT(*) FROM orders WHERE status = 'cancelled' GROUP BY user_id HAVING COUNT(*) > 1;

SELECT YEAR(created_at), MONTH(created_at), COUNT(DISTINCT user_id) FROM orders GROUP BY YEAR(created_at), MONTH(created_at);

SELECT department_id, COUNT(DISTINCT title) FROM employees GROUP BY department_id;

SELECT LEFT(email, POSITION('@' IN email) - 1) AS username, COUNT(*) FROM users GROUP BY username HAVING COUNT(*) > 1;

SELECT status, AVG(price) FROM orders GROUP BY status;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) != (SELECT COUNT(*) FROM products GROUP BY category_id ORDER BY COUNT(*) LIMIT 1);

SELECT user_id, SUM(total) FROM orders WHERE created_at >= '2024-01-01' GROUP BY user_id;

SELECT YEAR(created_at), MONTHNAME(created_at), SUM(total) FROM orders GROUP BY YEAR(created_at), MONTH(created_at);

SELECT LEFT(name, 2) AS prefix, COUNT(*) FROM products GROUP BY LEFT(name, 2);

SELECT status, SUM(total), COUNT(*) FROM orders GROUP BY status HAVING SUM(total) > 5000;

SELECT user_id, COUNT(DISTINCT DATE(created_at)) FROM orders GROUP BY user_id;

SELECT department_id, MAX(salary) - MIN(salary) AS salary_range FROM employees GROUP BY department_id;

SELECT category_id, GROUP_CONCAT(DISTINCT name ORDER BY name SEPARATOR ', ') FROM products GROUP BY category_id;

SELECT LEFT(phone, 4) AS prefix, COUNT(*) FROM users GROUP BY prefix;

SELECT status, YEAR(created_at), COUNT(*) FROM orders GROUP BY status, YEAR(created_at);

SELECT user_id, AVG(total) FROM orders WHERE total > 0 GROUP BY user_id HAVING AVG(total) > 100;

SELECT category_id, SUM(revenue) FROM (SELECT category_id, SUM(price * quantity) AS revenue FROM order_items GROUP BY category_id) t GROUP BY category_id;

SELECT YEAR(created_at) AS year, COUNT(*) FROM users GROUP BY year;

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id HAVING COUNT(*) IN (3, 5, 7);

SELECT LEFT(name, 1), COUNT(DISTINCT LEFT(name, 1)) FROM users GROUP BY LEFT(name, 1);

SELECT category_id, SUM(stock * price) AS inventory_value FROM products GROUP BY category_id ORDER BY inventory_value DESC LIMIT 5;

SELECT user_id, SUM(total) FROM orders WHERE status = 'shipped' GROUP BY user_id HAVING SUM(total) > 2000;

SELECT status, COUNT(DISTINCT user_id) FROM orders GROUP BY status HAVING COUNT(DISTINCT user_id) > 10;

SELECT DATE(created_at) AS date, COUNT(*) FROM orders WHERE status = 'completed' GROUP BY date ORDER BY date DESC LIMIT 30;

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id HAVING AVG(salary) > (SELECT AVG(salary) FROM employees);

SELECT LEFT(email, POSITION('.' IN email)) AS domain, COUNT(*) FROM users GROUP BY domain;

SELECT category_id, COUNT(*) AS cnt, SUM(stock) AS total_stock FROM products GROUP BY category_id HAVING cnt > 0 AND total_stock > 100;

SELECT user_id, MAX(created_at), MIN(created_at) FROM orders GROUP BY user_id;

SELECT YEAR(created_at), COUNT(*) FROM orders WHERE status = 'refunded' GROUP BY YEAR(created_at);

SELECT LEFT(name, 3) AS key_prefix, COUNT(*) FROM products GROUP BY key_prefix HAVING COUNT(*) > 1;

SELECT region, COUNT(DISTINCT user_id), SUM(total) FROM orders GROUP BY region ORDER BY SUM(total) DESC;

SELECT category_id, AVG(price) FROM products WHERE price > 0 GROUP BY category_id HAVING AVG(price) BETWEEN 10 AND 100;

SELECT status, YEAR(created_at), SUM(total) FROM orders GROUP BY status, YEAR(created_at) ORDER BY YEAR(created_at), status;

SELECT user_id, COUNT(*) FROM orders WHERE YEAR(created_at) = 2024 GROUP BY user_id HAVING COUNT(*) > 3;

SELECT department_id, SUM(salary) FROM employees GROUP BY department_id HAVING SUM(salary) > 500000;

SELECT LEFT(phone, 3) AS area_code, COUNT(DISTINCT user_id) FROM users GROUP BY LEFT(phone, 3);

SELECT DATE(created_at) AS date, SUM(total) FROM orders WHERE status = 'pending' GROUP BY date HAVING SUM(total) > 1000;

SELECT category_id, COUNT(DISTINCT user_id) FROM orders GROUP BY category_id ORDER BY COUNT(DISTINCT user_id) DESC LIMIT 5;

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) > 0 ORDER BY SUM(total) DESC LIMIT 10;

SELECT YEAR(created_at), MONTH(created_at), COUNT(*) FROM users GROUP BY YEAR(created_at), MONTH(created_at) ORDER BY YEAR(created_at), MONTH(created_at);

SELECT status, COUNT(*) FROM orders GROUP BY status HAVING COUNT(*) BETWEEN 10 AND 100;

SELECT department_id, MAX(salary) FROM employees GROUP BY department_id HAVING MAX(salary) > 100000;

SELECT LEFT(name, 2) AS prefix, COUNT(DISTINCT category_id) FROM products GROUP BY LEFT(name, 2);

SELECT user_id, GROUP_CONCAT(DISTINCT status ORDER BY created_at) FROM orders GROUP BY user_id;

SELECT DATE(created_at) AS date, COUNT(DISTINCT user_id) FROM orders GROUP BY date HAVING COUNT(DISTINCT user_id) > 5;

SELECT category_id, SUM(price * quantity) AS revenue FROM order_items WHERE quantity > 0 GROUP BY category_id;

SELECT status, AVG(price) FROM orders GROUP BY status HAVING AVG(price) > 50;

SELECT YEAR(created_at), COUNT(*) FROM users GROUP BY YEAR(created_at) HAVING COUNT(*) > 100;

SELECT LEFT(email, POSITION('@' IN email) - 1) AS username, COUNT(DISTINCT status) FROM orders GROUP BY username;

SELECT department_id, COUNT(*) FROM employees GROUP BY department_id ORDER BY COUNT(*) DESC LIMIT 3;

SELECT category_id, SUM(stock) FROM products GROUP BY category_id HAVING SUM(stock) > AVG(stock) * COUNT(*);

SELECT user_id, COUNT(*) FROM orders WHERE status IN ('pending', 'processing') GROUP BY user_id;

SELECT region, COUNT(DISTINCT product_id) FROM orders GROUP BY region;

SELECT status, SUM(total) FROM orders GROUP BY status HAVING SUM(total) > (SELECT AVG(SUM(total)) FROM orders GROUP BY status);

SELECT DATE(created_at) AS date, COUNT(*) FROM orders GROUP BY date ORDER BY date DESC LIMIT 7;

SELECT category_id, COUNT(*) FROM products GROUP BY category_id HAVING COUNT(*) = (SELECT MAX(cnt) FROM (SELECT COUNT(*) AS cnt FROM products GROUP BY category_id) t);

SELECT user_id, SUM(total) FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 90 DAY) GROUP BY user_id HAVING SUM(total) > 500;

SELECT LEFT(name, 1), COUNT(*), SUM(stock) FROM products GROUP BY LEFT(name, 1);

SELECT status, YEAR(created_at), COUNT(*) FROM orders GROUP BY status, YEAR(created_at) WITH ROLLUP;

SELECT department_id, AVG(salary) FROM employees GROUP BY department_id HAVING AVG(salary) > 40000 ORDER BY AVG(salary) DESC;

SELECT category_id, COUNT(DISTINCT user_id) FROM orders WHERE total > 100 GROUP BY category_id;

SELECT user_id, COUNT(*) FROM orders WHERE DATE(created_at) >= '2024-01-01' GROUP BY user_id HAVING COUNT(*) >= 5;

SELECT DATE(created_at), SUM(total) FROM orders WHERE status = 'completed' GROUP BY DATE(created_at) ORDER BY DATE(created_at) DESC LIMIT 30;

SELECT LEFT(phone, 2) AS country, COUNT(DISTINCT user_id) FROM users GROUP BY country;

SELECT category_id, SUM(quantity) FROM order_items GROUP BY category_id HAVING SUM(quantity) > 50;

SELECT status, COUNT(DISTINCT user_id) FROM orders GROUP BY status HAVING COUNT(DISTINCT user_id) > 10;

SELECT user_id, SUM(total) FROM orders GROUP BY user_id HAVING SUM(total) > (SELECT SUM(total) / COUNT(DISTINCT user_id) FROM orders);

SELECT YEAR(created_at), MONTH(created_at), COUNT(*) FROM orders GROUP BY YEAR(created_at), MONTH(created_at) ORDER BY YEAR(created_at), MONTH(created_at);
