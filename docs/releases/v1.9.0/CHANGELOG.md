# v1.9.0 Changelog

## v1.9.0 (开发中)

### 新增功能

#### 数据完整性 & SQL 扩展
- 索引支持优化 (B+ Tree / Hash)
- 外键约束实现
- 事务增强 (SAVEPOINT / ROLLBACK TO SAVEPOINT)

#### SQL 特性增强
- 子查询支持增强 (Scalar, IN, EXISTS, ANY, ALL)
- JOIN 优化 (INNER / LEFT / RIGHT / CROSS)
- 窗口函数基础支持 (PARTITION BY, AVG/SUM/ROW_NUMBER)

#### 性能优化
- 批量 INSERT 优化
- 查询缓存
- 连接池实现

#### 运维与安全
- 日志与监控 (SQL日志/错误日志/执行计划)
- 用户权限管理 (READ/WRITE)
- 数据备份导出 (CSV/JSON)
- 数据恢复功能
- 崩溃恢复测试增强
- 生产场景测试

### 改进
- 开发流程文档
- 教学示例脚本

## v1.8.0 (2026-03-25)
- SQL-92 合规性增强
- 性能优化
- 错误处理改进
