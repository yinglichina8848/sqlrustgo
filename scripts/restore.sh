#!/bin/bash
# =============================================================================
# SQLRustGo Restore Script
# Usage: ./scripts/restore.sh -b <backup-id> [OPTIONS]
#
# Options:
#   -d, --database <name>    Database name (required)
#   -b, --backup-id <id>    Backup ID to restore (required)
#   -i, --input <dir>       Backup directory (default: ./backups)
#       --drop-first        Drop existing tables before restore
#
# Examples:
#   ./scripts/restore.sh -d mydb -b backup-20260422-001
#   ./scripts/restore.sh -d mydb -b backup-20260422-001 --drop-first
# =============================================================================

set -e

# Default values
DATABASE=""
BACKUP_ID=""
BACKUP_DIR="./backups"
DROP_FIRST=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--database)
            DATABASE="$2"
            shift 2
            ;;
        -b|--backup-id)
            BACKUP_ID="$2"
            shift 2
            ;;
        -i|--input)
            BACKUP_DIR="$2"
            shift 2
            ;;
        --drop-first)
            DROP_FIRST="--drop-first"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 -d <database> -b <backup-id> [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -d, --database <name>    Database name (required)"
            echo "  -b, --backup-id <id>    Backup ID to restore (required)"
            echo "  -i, --input <dir>       Backup directory (default: ./backups)"
            echo "      --drop-first        Drop existing tables before restore"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required parameters
if [ -z "$DATABASE" ]; then
    echo "Error: --database is required"
    echo "Usage: $0 -d <database> -b <backup-id>"
    exit 1
fi

if [ -z "$BACKUP_ID" ]; then
    echo "Error: --backup-id is required"
    echo "Usage: $0 -d <database> -b <backup-id>"
    exit 1
fi

# Build command
CMD="./target/debug/sqlrustgo-tools restore --database $DATABASE --backup-id $BACKUP_ID --backup-dir $BACKUP_DIR"

if [ -n "$DROP_FIRST" ]; then
    CMD="$CMD --drop-first"
fi

echo "=========================================="
echo "SQLRustGo Restore Script"
echo "=========================================="
echo "Database:     $DATABASE"
echo "Backup ID:    $BACKUP_ID"
echo "Backup Dir:   $BACKUP_DIR"
echo "Timestamp:    $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="

# Run restore
echo ""
echo "Starting restore..."
eval $CMD

echo ""
echo "Restore completed successfully!"
