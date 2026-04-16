# Contributing to SweetGrass

SweetGrass is part of the [ecoPrimals](https://github.com/ecoPrimals)
sovereign computing ecosystem. Contributions that respect the ecosystem's
design principles are welcome.

## Prerequisites

- Rust 1.87+ (stable, Edition 2024)
- Docker (optional, for PostgreSQL integration tests)
- `cargo-llvm-cov` (optional, for coverage)

## Before You Start

1. Read `specs/PRIMAL_SOVEREIGNTY.md` — SweetGrass owns its own types and
   communicates with other primals exclusively via JSON-RPC 2.0 at runtime.
   No compile-time coupling to other primals.

2. Read `specs/ARCHITECTURE.md` for the 11-crate workspace structure.

3. Familiarize yourself with the [wateringHole standards](https://github.com/ecoPrimals/wateringHole),
   particularly `STANDARDS_AND_EXPECTATIONS.md`.

## Development Workflow

```bash
# Build
cargo build --release

# Run all tests
cargo test --all-features

# Clippy (pedantic + nursery, zero warnings required)
cargo clippy --all-features --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Doc check (zero warnings required)
cargo doc --all-features --no-deps

# Coverage
cargo llvm-cov --all-features --workspace

# Dependency audit
cargo deny check
```

## Code Standards

These are enforced at the workspace level and by CI:

| Rule | Enforcement |
|------|-------------|
| No `unsafe` code | `#![forbid(unsafe_code)]` on all crates |
| No `.unwrap()` in production | `clippy::unwrap_used = "deny"` |
| No `.expect()` in production | `clippy::expect_used = "deny"` |
| No files over 1000 lines | Split into `mod.rs` + domain modules |
| No TODO/FIXME/HACK in source | Track in ROADMAP or issues |
| No commented-out code | Git remembers |
| `Result<T, E>` everywhere | `thiserror` for typed errors |
| `#![warn(missing_docs)]` | On all 11 crates |
| SPDX header on every `.rs` file | `// SPDX-License-Identifier: AGPL-3.0-or-later` |
| Named constants | No magic numbers, ports, or primal names as string literals |

## Architecture Principles

- **Pure Rust**: Zero C/C++ dependencies in production. `cargo-deny` bans
  `openssl`, `ring`, `tonic`, `prost`, `reqwest`, and `provenance-trio-types`.

- **Primal Sovereignty**: SweetGrass knows only itself. Other primals are
  discovered at runtime via capability-based discovery. No shared type crates.

- **JSON-RPC + tarpc first**: All inter-primal communication uses JSON-RPC 2.0.
  tarpc is optional for Rust-native high-performance paths.

- **Zero-copy where possible**: `Arc<str>` for identifiers (`BraidId`, `Did`,
  `ContentHash`, `ActivityId`, `mime_type`), `Cow<'static, str>` for static
  values, `bytes::Bytes` for wire payloads.

- **DI over environment mutation**: Use `*_with_reader()` functions for
  testable environment access. No `std::env::set_var` in tests.

## Pull Request Checklist

- [ ] `cargo test --all-features` — zero failures
- [ ] `cargo clippy --all-features --all-targets -- -D warnings` — zero warnings
- [ ] `cargo doc --all-features --no-deps` — zero doc warnings
- [ ] `cargo fmt --all -- --check` — passes
- [ ] No TODO/FIXME/HACK in committed code
- [ ] No files over 1000 lines
- [ ] SPDX header on any new `.rs` files
- [ ] Tests for new functionality
- [ ] `CHANGELOG.md` updated
- [ ] `CONTEXT.md` metrics current (test count, coverage, if version bump)

## License

By contributing, you agree that your contributions will be licensed under
AGPL-3.0-or-later. See [LICENSE](./LICENSE).
