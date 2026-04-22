#!/bin/bash
# =============================================================================
# SQLRustGo Backup Script
# Usage: ./scripts/backup.sh [OPTIONS]
#
# Options:
#   -d, --database <name>    Database name (default: default)
#   -o, --output <dir>       Backup output directory (default: ./backups)
#   -t, --type <type>       Backup type: full, incremental, differential
#   -c, --compress           Enable compression
#   -s, --schema-only        Schema only (no data)
#
# Examples:
#   ./scripts/backup.sh                                    # Full backup
#   ./scripts/backup.sh -d mydb -o /backups               # Backup specific DB
#   ./scripts/backup.sh -t incremental                    # Incremental backup
#   ./scripts/backup.sh -c                                # Compressed backup
# =============================================================================

set -e

# Default values
DATABASE="default"
OUTPUT_DIR="./backups"
BACKUP_TYPE="full"
COMPRESS=""
SCHEMA_ONLY=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--database)
            DATABASE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -t|--type)
            BACKUP_TYPE="$2"
            shift 2
            ;;
        -c|--compress)
            COMPRESS="--compress"
            shift
            ;;
        -s|--schema-only)
            SCHEMA_ONLY="--schema-only"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -d, --database <name>    Database name (default: default)"
            echo "  -o, --output <dir>       Backup output directory (default: ./backups)"
            echo "  -t, --type <type>        Backup type: full, incremental, differential"
            echo "  -c, --compress           Enable compression"
            echo "  -s, --schema-only        Schema only (no data)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build command
CMD="./target/debug/sqlrustgo-tools backup --database $DATABASE --output-dir $OUTPUT_DIR --backup-type $BACKUP_TYPE"

if [ -n "$COMPRESS" ]; then
    CMD="$CMD --compress"
fi

if [ -n "$SCHEMA_ONLY" ]; then
    CMD="$CMD --schema-only"
fi

echo "=========================================="
echo "SQLRustGo Backup Script"
echo "=========================================="
echo "Database:     $DATABASE"
echo "Output Dir:   $OUTPUT_DIR"
echo "Backup Type:  $BACKUP_TYPE"
echo "Timestamp:    $(date '+%Y-%m-%d %H:%M:%S')"
echo "=========================================="

# Create output directory if not exists
mkdir -p "$OUTPUT_DIR"

# Run backup
echo ""
echo "Starting backup..."
eval $CMD

echo ""
echo "Backup completed successfully!"
echo "Output: $OUTPUT_DIR"
