# Workspace Members

This directory contains the modular crate structure for SQLRustGo v1.2.0+.

## Current Status

The crates/ directory is being migrated from the monolithic src/ structure.

### Migration Progress

- [x] Phase 1: Create workspace structure
- [ ] Phase 2: Migrate types module
- [ ] Phase 3: Migrate parser module
- [ ] Phase 4: Migrate other modules

### Crate Dependencies

```
sqlrustgo-server (depends on all)
├── sqlrustgo-planner
│   ├── sqlrustgo-parser
│   └── sqlrustgo-common
├── sqlrustgo-optimizer
│   └── sqlrustgo-types
├── sqlrustgo-executor
│   └── sqlrustgo-storage
├── sqlrustgo-storage
├── sqlrustgo-catalog
└── sqlrustgo-transaction
```

### Migration Commands

```bash
# Build workspace
cargo build --workspace

# Build single crate
cargo build -p sqlrustgo-types
```
