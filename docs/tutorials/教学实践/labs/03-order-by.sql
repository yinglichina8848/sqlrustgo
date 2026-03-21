-- =====================================================
-- 实验 3: ORDER BY 排序
-- =====================================================
-- 目标: 掌握结果排序

-- 3.1 升序排序（默认）
SELECT * FROM products ORDER BY price ASC;

-- 3.2 降序排序
SELECT * FROM products ORDER BY price DESC;

-- 3.3 多列排序
SELECT * FROM orders 
ORDER BY status ASC, created_at DESC;

-- 3.4 按表达式排序
SELECT *, price * quantity AS total 
FROM order_items 
ORDER BY total DESC;

-- 3.5 NULL 值排序位置
SELECT * FROM users ORDER BY last_login DESC NULLS LAST;

-- =====================================================
-- 教学提示:
-- - ASC 升序（默认），DESC 降序
-- - 多列排序时按顺序优先级执行
-- - NULL 值的排序位置因数据库而异
-- =====================================================
