-- =====================================================
-- 实验 8: UPDATE 数据更新
-- =====================================================
-- 目标: 掌握数据更新

-- 8.1 更新单列
UPDATE users 
SET status = 'inactive' 
WHERE id = 1;

-- 8.2 更新多列
UPDATE products 
SET 
    price = price * 0.9,
    updated_at = NOW()
WHERE category = 'electronics' AND stock < 10;

-- 8.3 基于子查询更新
UPDATE orders 
SET status = 'cancelled'
WHERE user_id = (
    SELECT id FROM users 
    WHERE membership = 'expired'
);

-- 8.4 替换更新 (MySQL 特有)
REPLACE INTO user_sessions (user_id, token, expires_at)
VALUES (1, 'new_token', '2024-12-31');

-- =====================================================
-- 教学提示:
-- - UPDATE 必须加 WHERE 条件，否则更新所有记录
-- - 建议先 SELECT 确认再 UPDATE
-- - REPLACE 会先删除再插入，注意主键
-- =====================================================
