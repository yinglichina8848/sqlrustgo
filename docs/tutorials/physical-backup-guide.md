# 物理备份工具

## 概述

SQLRustGo 提供基于存储快照的物理备份工具，支持全量备份和增量备份，可用于灾难恢复和数据迁移。

## 功能特性

- ✅ **全量备份 (Full Backup)** - 备份所有数据文件
- ✅ **增量备份 (Incremental Backup)** - 仅备份自上次备份以来更改的数据
- ✅ **WAL 归档打包** - 自动打包预写日志文件
- ✅ **GZIP 压缩** - 支持可选的压缩以节省空间
- ✅ **备份验证** - SHA256 校验和验证完整性
- ✅ **一键恢复** - 快速恢复到目标目录
- ✅ **LSN 追踪** - 支持增量备份的日志序列号追踪

## 概念说明

### 物理备份 vs 逻辑备份

| 特性 | 物理备份 | 逻辑备份 |
|------|----------|----------|
| 备份内容 | 数据库物理文件 | SQL 语句 |
| 备份速度 | 快 | 慢 |
| 恢复速度 | 快 | 慢 |
| 跨版本恢复 | 不支持 | 支持 |
| 增量备份 | 支持 | 不支持 |

### LSN (Log Sequence Number)

LSN 是日志序列号的缩写，用于追踪 WAL 文件的顺序。增量备份依赖 LSN 来确定自上次备份以来哪些文件发生了变化。

### 备份结构

```
backup_dir/
├── manifest.json       # 备份清单
├── data/               # 数据文件备份
│   ├── table1.dat
│   └── table2.dat
└── wal/                # WAL 归档
    ├── 0000000000000001.wal.gz
    └── 0000000000000002.wal.gz
```

## 使用方法

### 创建全量备份

```bash
# 基本用法
sqlrustgo-tools physical-backup backup \
    --dir /path/to/backup \
    --data-dir /var/lib/sqlrustgo/data \
    --wal-dir /var/lib/sqlrustgo/wal

# 启用压缩
sqlrustgo-tools physical-backup backup \
    --dir /path/to/backup \
    --data-dir /var/lib/sqlrustgo/data \
    --wal-dir /var/lib/sqlrustgo/wal \
    --compress
```

### 创建增量备份

增量备份需要指定父备份目录：

```bash
# 基于父备份创建增量备份
sqlrustgo-tools physical-backup backup \
    --dir /path/to/incremental_backup \
    --data-dir /var/lib/sqlrustgo/data \
    --wal-dir /var/lib/sqlrustgo/wal \
    --parent /path/to/parent_backup
```

### 列出备份

```bash
# 列出所有物理备份
sqlrustgo-tools physical-backup list --dir /path/to/backups
```

输出示例：

```
📦 Physical Backup: 20260401_120000
   Type: full
   Timestamp: 2026-04-01 12:00:00
   LSN: 0000000000001564
   Files: 42
   Total size: 1048576 bytes (1024 KB)
   Compression: enabled
   WAL archives: 2

📦 Physical Backup: 20260402_120000
   Type: incremental
   Timestamp: 2026-04-02 12:00:00
   LSN: 0000000000001892
   Parent LSN: 0000000000001564
   Files: 3
   Total size: 32768 bytes (32 KB)
   Compression: enabled
   WAL archives: 1
```

### 验证备份

```bash
# 验证备份完整性
sqlrustgo-tools physical-backup verify --dir /path/to/backup
```

输出示例：

```
✅ Manifest exists
✅ Data backup directory exists
✅ WAL archive directory exists
✅ Manifest is valid JSON
✅ Backup type: full
✅ Timestamp: 2026-04-01 12:00:00
✅ LSN: 0000000000001564
✅ File count: 42
✅ Total size: 1048576 bytes
✅ Checksum verification passed
✅ Data files verified (42 files)
✅ WAL archives verified (2 archives)
✅ Backup verification complete
```

### 恢复备份

```bash
# 恢复到目标目录
sqlrustgo-tools physical-backup restore \
    --dir /path/to/backup \
    --target /var/lib/sqlrustgo/restore

# 同时指定 WAL 恢复目标
sqlrustgo-tools physical-backup restore \
    --dir /path/to/backup \
    --target /var/lib/sqlrustgo/restore \
    --wal-target /var/lib/sqlrustgo/restore_wal
```

## 备份清单 (manifest.json)

备份清单包含备份的元数据：

```json
{
  "version": "1.0",
  "backup_type": "full",
  "timestamp": "2026-04-01T12:00:00Z",
  "lsn": "0000000000001564",
  "parent_lsn": null,
  "data_dir": "/var/lib/sqlrustgo/data",
  "wal_dir": "/var/lib/sqlrustgo/wal",
  "total_size_bytes": 1048576,
  "file_count": 42,
  "compressed": true,
  "files": [
    {
      "relative_path": "users.ibd",
      "size_bytes": 65536,
      "checksum": "sha256:abc123...",
      "is_compressed": true
    }
  ],
  "wal_archives": [
    {
      "filename": "0000000000000001.wal.gz",
      "size_bytes": 16384,
      "checksum": "sha256:def456...",
      "lsn_range": "0000000000001024-0000000000001564"
    }
  ],
  "checksum": "sha256:master789..."
}
```

## 测试

### 运行单元测试

```bash
# 运行物理备份工具的所有测试
cargo test -p sqlrustgo-tools physical_backup
```

### 测试结果

```
running 11 tests
test physical_backup::tests::test_generate_lsn ... ok
test physical_backup::tests::test_physical_backup_type_serialization ... ok
test physical_backup::tests::test_chrono_lite_timestamp ... ok
test physical_backup::tests::test_checksum_calculation ... ok
test physical_backup::tests::test_backup_file_info_serialization ... ok
test physical_backup::tests::test_physical_backup_manifest_serialization ... ok
test physical_backup::tests::test_wal_archive_info_serialization ... ok
test physical_backup::tests::test_walkdir_empty_directory ... ok
test physical_backup::tests::test_walkdir_with_files ... ok
test physical_backup::tests::test_walkdir_nested_directories ... ok
test physical_backup::tests::test_compress_and_decompress_file ... ok

test result: ok. 11 passed; 0 failed
```

### 回归测试

物理备份工具已集成到回归测试套件中：

```bash
# 运行完整回归测试
cargo test --test regression_test -- --nocapture
```

## 限制

- 物理备份只能恢复到相同版本的 SQLRustGo
- 增量备份需要父备份存在
- 恢复操作会覆盖目标目录中的现有文件
- 压缩会占用额外的 CPU 资源

## 最佳实践

1. **定期全量备份** - 建议每周进行一次全量备份
2. **频繁增量备份** - 建议每小时进行一次增量备份
3. **启用压缩** - 对于长时间存储的备份启用压缩
4. **验证备份** - 恢复前先验证备份完整性
5. **异地存储** - 将备份复制到异地存储以防灾难

## 相关 Issue

- Issue #1018: 物理备份 CLI

## 相关文档

- [mysqldump 导入工具](./mysqldump-import-guide.md) - 逻辑备份导入
- [用户手册](../releases/v2.0.0/USER_MANUAL.md) - SQLRustGo 完整用户手册
