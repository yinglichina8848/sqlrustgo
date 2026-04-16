-- === View Test Suite ===

-- === CASE: Create simple view ===
-- EXPECT: success
CREATE VIEW active_users AS SELECT id, name, email FROM users WHERE id > 5;

-- === CASE: Create view with aggregate ===
-- EXPECT: success
CREATE VIEW order_stats AS
SELECT user_id, COUNT(*) as order_count, SUM(total) as total_spent
FROM orders GROUP BY user_id;

-- === CASE: Create view with JOIN ===
-- EXPECT: success
CREATE VIEW user_orders AS
SELECT u.id, u.name, o.order_id, o.total
FROM users u JOIN orders o ON u.id = o.user_id;

-- === CASE: Create view with subquery ===
-- EXPECT: success
CREATE VIEW high_value_users AS
SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE total > 200);

-- === CASE: Drop view ===
-- EXPECT: success
DROP VIEW active_users;

-- === CASE: Select from view ===
-- EXPECT: 5 rows
SELECT * FROM user_orders WHERE user_id = 3;

-- === CASE: Select from view with aggregate ===
-- EXPECT: 3 rows
SELECT * FROM order_stats WHERE order_count > 2;

-- === CASE: Create or replace view ===
-- EXPECT: success
CREATE OR REPLACE VIEW user_orders AS
SELECT u.id, u.name, o.order_id, o.total, o.order_date
FROM users u LEFT JOIN orders o ON u.id = o.user_id;

-- === CASE: View with CASE expression ===
-- EXPECT: success
CREATE VIEW user_categories AS
SELECT id, name,
  CASE
    WHEN id <= 3 THEN 'VIP'
    WHEN id <= 7 THEN 'Regular'
    ELSE 'New'
  END as category
FROM users;

-- === CASE: View with DISTINCT ===
-- EXPECT: success
CREATE VIEW unique_emails AS
SELECT DISTINCT email FROM users WHERE email IS NOT NULL;

-- === CASE: Create view with ORDER BY ===
-- EXPECT: success
CREATE VIEW ordered_users AS
SELECT id, name, email FROM users ORDER BY id DESC;

-- === CASE: Drop view if exists ===
-- EXPECT: success
DROP VIEW IF EXISTS nonexistent_view;

-- === CASE: View with GROUP BY ===
-- EXPECT: success
CREATE VIEW monthly_orders AS
SELECT
  strftime('%Y-%m', order_date) as month,
  COUNT(*) as order_count,
  SUM(total) as total_revenue
FROM orders GROUP BY month;

-- === CASE: Select from view with WHERE ===
-- EXPECT: 2 rows
SELECT * FROM user_categories WHERE category = 'VIP';
