-- =====================================================
-- 实验 7: INSERT 数据插入
-- =====================================================
-- 目标: 掌握数据插入

-- 7.1 插入单行
INSERT INTO users (username, email, password, status)
VALUES ('john_doe', 'john@example.com', 'hashed_pwd', 'active');

-- 7.2 插入多行
INSERT INTO products (name, category, price, stock)
VALUES 
    ('Product A', 'electronics', 99.99, 100),
    ('Product B', 'books', 29.99, 50),
    ('Product C', 'clothing', 49.99, 200);

-- 7.3 INSERT ... SELECT
INSERT INTO archived_orders (order_id, user_id, amount, archived_at)
SELECT id, user_id, amount, NOW()
FROM orders 
WHERE created_at < '2023-01-01';

-- 7.4 ON DUPLICATE KEY UPDATE (MySQL 特有)
INSERT INTO products (id, name, stock)
VALUES (1, 'Updated Product', 999)
ON DUPLICATE KEY UPDATE stock = stock + 1;

-- =====================================================
-- 教学提示:
-- - 插入数据时注意字段类型匹配
-- - 可使用子查询插入数据
-- - ON DUPLICATE KEY UPDATE 是 MySQL 特有语法
-- =====================================================
