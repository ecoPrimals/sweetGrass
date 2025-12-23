# sweetGrass

Attribution Layer - Semantic Provenance & PROV-O

## Status

🌱 **Nascent** — Scaffolded from SourDough

## Quick Start

```bash
# Build
cargo build

# Test
cargo test

# Run
cargo run
```

## Architecture

```
sweetGrass/
├── Cargo.toml           # Workspace manifest
├── crates/
│   └── sweet-grass-core/  # Core library
├── specs/               # Specifications
└── showcase/            # Demonstrations
```

## Integration

sweetGrass integrates with the ecoPrimals ecosystem via SourDough traits:

- `PrimalLifecycle` — Start/stop/reload
- `PrimalHealth` — Health checks
- `PrimalIdentity` — BearDog integration (TODO)
- `PrimalDiscovery` — Songbird integration (TODO)

## License

AGPL-3.0

---

*Born from SourDough, growing into an ecoPrimal.*
