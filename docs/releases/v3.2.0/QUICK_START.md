# v3.2.0 Quick Start

> **Version**: 3.2.0
> **Date**: 2026-05-15
> **Status**: Beta Phase

---

## 5-Minute Quick Start

### 1. Build

```bash
git clone https://github.com/openclaw/sqlrustgo.git
cd sqlrustgo
git checkout v3.2.0
cargo build --all-features --release
```

### 2. Run

```bash
cargo run --bin sqlrustgo --release
```

### 3. Connect

```bash
mysql -h localhost -P 3306 -u root -p
```

---

## First Steps

### Create a Table

```sql
CREATE TABLE products (
    id INT PRIMARY KEY,
    name VARCHAR(255),
    price DECIMAL(10, 2)
);
```

### Insert Data

```sql
INSERT INTO products (id, name, price) VALUES
    (1, 'Product A', 99.99),
    (2, 'Product B', 149.99);
```

### Query

```sql
SELECT * FROM products WHERE price > 100;
```

---

## GMP Quick Demo

### Enable GMP

```toml
[gmp]
enable = true
audit_chain_enabled = true
```

### Create Signed Record

```sql
-- Records are automatically signed and audited
INSERT INTO products (id, name, price)
VALUES (3, 'Signed Product', 199.99);

-- Check audit chain
SELECT * FROM audit_chain;
```

---

## Configuration

### Minimal Config

```toml
[server]
port = 3306

[database]
path = "./data/sqlrustgo.db"

[performance]
max_connections = 50
```

---

## Next Steps

1. Read [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)
2. Configure for [Production](./DEPLOYMENT_GUIDE.md#production)
3. Review [TEST_PLAN.md](./TEST_PLAN.md)

---

**Quick Start Date**: 2026-05-15
**Maintenance**: hermes-z6g4
