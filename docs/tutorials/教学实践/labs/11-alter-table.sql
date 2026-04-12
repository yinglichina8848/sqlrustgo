-- =====================================================
-- 实验 11: ALTER TABLE 表修改
-- =====================================================
-- 目标: 掌握表结构修改

-- 11.1 添加列
ALTER TABLE users 
ADD COLUMN phone VARCHAR(20) AFTER email;

-- 11.2 添加带默认值的列
ALTER TABLE users 
ADD COLUMN last_login TIMESTAMP NULL;

-- 11.3 修改列定义
ALTER TABLE users 
MODIFY COLUMN username VARCHAR(100) NOT NULL;

-- 11.4 重命名列
ALTER TABLE users 
CHANGE COLUMN phone contact_phone VARCHAR(30);

-- 11.5 删除列
ALTER TABLE users 
DROP COLUMN experimental_field;

-- 11.6 添加索引
ALTER TABLE orders 
ADD INDEX idx_user_amount (user_id, amount);

-- 11.7 添加外键约束
ALTER TABLE orders 
ADD CONSTRAINT fk_user 
FOREIGN KEY (user_id) REFERENCES users(id);

-- 11.8 重命名表
ALTER TABLE users RENAME TO system_users;

-- 11.9 修改存储引擎 (MySQL)
-- ALTER TABLE orders ENGINE = InnoDB;

-- =====================================================
-- 教学提示:
-- - ALTER TABLE 会锁表，慎用
-- - 大表修改建议使用在线 DDL
-- - 修改列可能丢失数据，请提前备份
-- =====================================================
