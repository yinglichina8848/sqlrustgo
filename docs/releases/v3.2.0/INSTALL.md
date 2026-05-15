# v3.2.0 Installation Guide

> **Version**: 3.2.0
> **Date**: 2026-05-15
> **Status**: Beta Phase

---

## Prerequisites

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|--------------|
| CPU | 2 cores | 4+ cores |
| Memory | 4 GB | 8+ GB |
| Disk | 10 GB | 50+ GB |
| OS | Linux/macOS | Linux/macOS |

### Dependencies

- **Rust** 1.70+ (with cargo)
- **OpenSSL** (for TLS)
- **CMake** (for native builds)

---

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/openclaw/sqlrustgo.git
cd sqlrustgo

# Checkout version
git checkout v3.2.0

# Build with all features
cargo build --all-features --release

# Install
cargo install --all-features --path .
```

### Pre-built Binary

See [GitHub Releases](https://github.com/openclaw/sqlrustgo/releases) for pre-built binaries.

---

## Configuration

### Basic Configuration

Create `sqlrustgo.toml`:

```toml
[server]
host = "0.0.0.0"
port = 3306

[database]
path = "./data/sqlrustgo.db"

[performance]
max_connections = 200
buffer_pool_size = 1024

[gmp]
enable = true
hsm_provider = "software"  # or "pkcs11"
```

### GMP Configuration

```toml
[gmp]
enable = true
signature_algorithm = "ecdsa"
timestamp_server = "http://timestamp.digicert.com"
audit_chain_enabled = true
```

### TLS Configuration

```toml
[tls]
enabled = true
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
```

---

## Running

### Development Mode

```bash
cargo run --bin sqlrustgo
```

### Production Mode

```bash
# Using configuration file
sqlrustgo --config sqlrustgo.toml

# Or with environment variables
SQLRUSTGO_HOST=0.0.0.0 SQLRUSTGO_PORT=3306 sqlrustgo
```

---

## Docker

### Build Image

```bash
docker build -t sqlrustgo:v3.2.0 .
```

### Run Container

```bash
docker run -d \
  --name sqlrustgo \
  -p 3306:3306 \
  -v /data:/data \
  sqlrustgo:v3.2.0
```

---

## Verification

### Health Check

```bash
mysql -h localhost -P 3306 -u root -p -e "SELECT 1"
```

### GMP Verification

```sql
-- Check audit chain
SELECT * FROM audit_chain LIMIT 10;

-- Check electronic signatures
SELECT * FROM electronic_signatures LIMIT 10;
```

---

## Troubleshooting

### Build Issues

```bash
# Clean and rebuild
cargo clean
cargo build --all-features
```

### Connection Issues

```bash
# Check port availability
lsof -i :3306

# Check logs
tail -f /var/log/sqlrustgo.log
```

---

## Next Steps

- Read [QUICK_START.md](./QUICK_START.md)
- Configure [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)
- Review [RELEASE_NOTES.md](./RELEASE_NOTES.md)

---

**Installation Date**: 2026-05-15
**Maintenance**: hermes-z6g4
