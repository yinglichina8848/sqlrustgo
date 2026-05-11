# v3.1.0 Installation Guide

> **Version**: 3.1.0

---

## Prerequisites

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| CPU | 2 cores | 4+ cores |
| Memory | 4 GB | 8 GB |
| Disk | 10 GB | 50 GB SSD |
| OS | macOS 12+, Linux (glibc 2.17+), Windows (WSL2) | macOS 14+, Ubuntu 22.04+ |
| Rust | 1.75+ | 1.80+ |

---

## Install Methods

### 1. Build from Source (Recommended)

```bash
# Clone
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo

# Checkout v3.1.0
git checkout develop/v3.1.0

# Build with all features
cargo build --release --all-features

# Run tests
cargo test --all-features
```

### 2. Binary Release (Future)

```bash
# Download from GitHub release (when available)
curl -LO https://github.com/minzuuniversity/sqlrustgo/releases/download/v3.1.0/sqlrustgo-x86_64-apple-darwin.tar.gz
tar -xzf sqlrustgo-x86_64-apple-darwin.tar.gz
./sqlrustgo --version
```

### 3. Docker (Future)

```bash
# Pull image
docker pull ghcr.io/minzuuniversity/sqlrustgo:v3.1.0

# Run
docker run -p 3306:3306 \
  -v /data/sqlrustgo:/var/lib/sqlrustgo \
  ghcr.io/minzuuniversity/sqlrustgo:v3.1.0
```

---

## Initial Setup

### 1. Initialize Data Directory

```bash
# Create data directory
mkdir -p /var/lib/sqlrustgo
chmod 700 /var/lib/sqlrustgo

# Initialize (first run creates system tables)
./target/release/sqlrustgo init --data-dir /var/lib/sqlrustgo
```

### 2. Configure

```bash
# Create config file
cat > /etc/sqlrustgo.toml << 'EOF'
[data]
path = "/var/lib/sqlrustgo"

[server]
bind = "0.0.0.0:3306"
max_connections = 1000

[buffer_pool]
size = "128MB"

[wal]
enabled = true
sync = "O_DSYNC"

[security]
tls_enabled = true
tls_cert = "/etc/sqlrustgo/tls/server.crt"
tls_key = "/etc/sqlrustgo/tls/server.key"

[gmp]
audit_enabled = true
audit_chain_enabled = true
encryption_enabled = true
EOF
```

### 3. Start Server

```bash
# Start in foreground (test)
./target/release/sqlrustgo --config /etc/sqlrustgo.toml

# Start as daemon
./target/release/sqlrustgo --config /etc/sqlrustgo.toml --daemon
```

### 4. Connect

```bash
# MySQL client
mysql -h 127.0.0.1 -P 3306 -u root -p

# Or with TLS
mysql -h 127.0.0.1 -P 3306 -u root -p --ssl-mode=REQUIRED
```

---

## Verify Installation

```bash
# Check version
./target/release/sqlrustgo --version
# Output: sqlrustgo 3.1.0

# Run health check
./target/release/sqlrustgo health --data-dir /var/lib/sqlrustgo

# Connect and verify
mysql -h 127.0.0.1 -u root -e "SELECT VERSION();"
# Output: 8.0.0-sqlrustgo-3.1.0
```

---

## GMP Installation (Optional)

For GMP-compliant deployment, enable additional features:

```bash
# Build with GMP features
cargo build --release --all-features --features "gmp/audit,gmp/encryption"

# Configure GMP
cat > /etc/sqlrustgo-gmp.toml << 'EOF'
[gmp]
audit_enabled = true
audit_chain_enabled = true
audit_sha256_required = true
encryption_enabled = true
encryption_algorithm = "AES-256-GCM"
gap_locking_enabled = true
serializable_enabled = true
EOF
```

---

## Uninstall

```bash
# Stop server
pkill -f sqlrustgo

# Remove data (WARNING: destructive)
rm -rf /var/lib/sqlrustgo

# Remove binary
rm /usr/local/bin/sqlrustgo

# Remove config
rm /etc/sqlrustgo.toml
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `error: cannot bind to port 3306` | Change port: `--server.bind=0.0.0.0:3307` |
| `error: data directory not initialized` | Run: `./sqlrustgo init --data-dir /path` |
| `error: permission denied` | Check directory permissions: `chmod 700 /var/lib/sqlrustgo` |
| `error: TLS certificate not found` | Generate self-signed: `scripts/generate_tls_cert.sh` |
