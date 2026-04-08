# 🌾 SweetGrass — Development Guide

**Last Updated**: April 7, 2026  
**Version**: v0.7.27

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.87+ (Edition 2024)
- Docker & Docker Compose (for PostgreSQL tests)
- cargo-llvm-cov (for coverage)

### Installation

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-llvm-cov cargo-audit cargo-watch

# Clone and build
git clone <repo-url>
cd sweetGrass
cargo build --all-features
```

### Run Tests

```bash
# Unit tests only (no Docker needed)
cargo test --all-features

# All tests including PostgreSQL (requires Docker)
docker compose up -d
cargo test --all-features
docker compose down
```

### Coverage Report

```bash
# Start PostgreSQL
docker compose up -d

# Generate coverage
cargo llvm-cov --all-features --workspace --html

# View report
open target/llvm-cov/html/index.html

# Stop PostgreSQL
docker compose down
```

---

## 🐳 Docker Development Environment

### Start Services

```bash
# PostgreSQL only
docker compose up -d postgres

# PostgreSQL + pgAdmin (for database management)
docker compose --profile admin up -d
```

### Access Services

- **PostgreSQL**: `postgresql://postgres:postgres@localhost:5432/sweetgrass_test`
- **pgAdmin**: http://localhost:5050 (admin@sweetgrass.local / admin)

### Stop Services

```bash
docker compose down

# Remove volumes (clean slate)
docker compose down -v
```

---

## 🧪 Testing

### Test Hierarchy

```
crates/
├── Unit Tests          (600+ tests)  - src/ modules across all 10 crates
├── Integration Tests   (74 tests)    - sweet-grass-service/tests/*.rs
├── Chaos Tests        (17 tests)    - sweet-grass-service/tests/chaos.rs
├── Fault Injection    (9 tests)     - sweet-grass-service/tests/fault_injection.rs
├── E2E HTTP           (19 tests)    - sweet-grass-service/tests/e2e_http.rs
├── Property Tests     (12 tests)    - Uses proptest
└── PostgreSQL         (23 tests)    - sweet-grass-store-postgres/tests/
```

### Run Specific Tests

```bash
# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration

# Chaos tests
cargo test --test chaos

# PostgreSQL tests (requires Docker)
docker compose up -d
cargo test --package sweet-grass-store-postgres
docker compose down

# Single test
cargo test test_name

# With output
cargo test -- --nocapture
```

### Test Categories

**Fast Tests** (no Docker):
- Core data structures
- Factory & attribution
- Compression logic
- Query engine
- Memory store

**Slow Tests** (requires Docker):
- PostgreSQL store (23 tests)
- Migration tests
- Schema validation

**Ignored Tests** (requires live services):
- Multi-primal integration
- Service discovery
- tarpc RPC tests

---

## 🔍 Code Quality

### Linting

```bash
# Format check
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all

# Clippy (pedantic + nursery)
cargo clippy --all-targets --all-features -- -D warnings

# Documentation check
cargo doc --no-deps --all-features
```

### Pre-commit Checks

```bash
# Run all checks
./scripts/check.sh

# Or manually:
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

### Security Audit

```bash
cargo audit
```

---

## 📊 Coverage Goals

### Current Status
- **Overall**: 1,213 tests passing (90.90% region coverage)
- **Target**: 90%+ coverage on core crates (achieved)

### Per-Crate Coverage

| Crate | Current | Target | Status |
|-------|---------|--------|--------|
| sweet-grass-core | 97% | 90% | ✅ Excellent |
| sweet-grass-factory | 96% | 90% | ✅ Excellent |
| sweet-grass-compression | 96% | 90% | ✅ Excellent |
| sweet-grass-query | 94% | 90% | ✅ Excellent |
| sweet-grass-service | 92% | 90% | ✅ Above target |
| sweet-grass-store | 96% | 90% | ✅ Excellent |
| sweet-grass-store-redb | 90%+ | 90% | ✅ Above target |
| sweet-grass-store-postgres | 5% | 80% | ❌ Needs CI/Docker |
| sweet-grass-integration | 80% | 80% | ✅ At target |
| sweet-grass-store-sled | 80% | 90% | ⚠️ Legacy (feature-gated) |

### Improving Coverage

**PostgreSQL**: Run with Docker
```bash
docker compose up -d
cargo test --package sweet-grass-store-postgres
```

**Integration**: Requires live primals (future work)

---

## 🏗️ Architecture Principles

### 1. Infant Discovery
**Self-knowledge only. Discover others at runtime.**

```rust
// ❌ Bad: Hardcoded primal address
const BEARDOG_ADDR: &str = "localhost:8888";

// ✅ Good: Capability-based discovery
let signer = discovery
    .find_one(&Capability::Signing)
    .await?;
```

### 2. Zero Unsafe Code
**All crates: `#![forbid(unsafe_code)]`**

```rust
// ❌ Never use unsafe
unsafe { ... }

// ✅ Always use safe alternatives
Arc, RefCell, channels, async, etc.
```

### 3. Zero Production Unwraps
**Never panic in production.**

```rust
// ❌ Bad: Can panic
let value = option.unwrap();

// ✅ Good: Handle errors
let value = option.ok_or_else(|| Error::Missing)?;
```

### 4. Mock Isolation
**Mocks only in tests.**

```rust
// ✅ Properly gated
#[cfg(any(test, feature = "test"))]
pub struct MockClient { ... }

#[cfg(test)]
mod tests {
    use super::MockClient;
}
```

### 5. File Size Discipline
**Maximum 1000 lines per file.**

Large files should be refactored smartly:
- Extract cohesive modules
- Split by concern (not arbitrarily)
- Maintain logical grouping

### 6. Pure Rust Sovereignty
**No gRPC, no protobuf, no C dependencies.**

```rust
// ✅ Pure Rust stack
tarpc     // RPC (not gRPC)
serde     // Serialization (not protobuf)
bincode   // Binary format
tokio     // Async runtime
```

---

## 🔧 Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Develop with Live Reload

```bash
# Watch and rebuild on changes
cargo watch -x build

# Watch and test
cargo watch -x test
```

### 3. Run Quality Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

### 4. Check Coverage

```bash
docker compose up -d
cargo llvm-cov --all-features --workspace
docker compose down
```

### 5. Commit

```bash
git add -A
git commit -m "feat: add my feature"
```

### 6. Push & Create PR

```bash
git push origin feature/my-feature
gh pr create
```

---

## 📚 Documentation

### API Documentation

```bash
# Generate docs
cargo doc --no-deps --all-features

# Open in browser
cargo doc --no-deps --all-features --open
```

### Documentation Standards

**All public items must have doc comments:**

```rust
/// Brief description.
///
/// ## Arguments
///
/// * `param` - Parameter description
///
/// ## Returns
///
/// Return value description
///
/// ## Errors
///
/// Error conditions
///
/// ## Example
///
/// ```rust
/// let result = function(param)?;
/// ```
pub fn function(param: Type) -> Result<Output> {
    // ...
}
```

---

## 🐛 Debugging

### Logging

```bash
# Enable tracing
RUST_LOG=debug cargo run --bin sweetgrass -- server

# Specific module
RUST_LOG=sweet_grass_service=debug cargo run

# Trace level
RUST_LOG=trace cargo test test_name
```

### Tokio Console

See [docs/guides/TOKIO_CONSOLE_GUIDE.md](./docs/guides/TOKIO_CONSOLE_GUIDE.md)

### Database Debugging

```bash
# Connect to PostgreSQL
docker compose up -d
psql postgresql://postgres:postgres@localhost:5432/sweetgrass_test

# View schema
\dt
\d+ braids

# Query data
SELECT * FROM braids LIMIT 10;
```

---

## 🚀 Release Process

### 1. Update Version

```bash
# Update workspace version in root Cargo.toml
# Then update CHANGELOG.md and ROADMAP.md
```

### 2. Final Checks

```bash
# All tests pass
docker compose up -d
cargo test --all-features

# Coverage meets target
cargo llvm-cov --all-features --workspace

# Clippy clean
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

### 3. Build Release

```bash
# Default (glibc, host target)
cargo build --release

# musl-static for plasmidBin / container deployment (ecoBin standard)
rustup target add x86_64-unknown-linux-musl
sudo apt-get install -y musl-tools          # Ubuntu/Debian
cargo build --profile release-static --target x86_64-unknown-linux-musl

# Verify static linkage
file target/x86_64-unknown-linux-musl/release-static/sweetgrass
ldd target/x86_64-unknown-linux-musl/release-static/sweetgrass
# Expected: "statically linked", ~4.5 MB stripped

# ARM64 musl (aarch64, requires cross-linker)
rustup target add aarch64-unknown-linux-musl
cargo build --profile release-static --target aarch64-unknown-linux-musl
```

### 4. Tag & Push

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

---

## 🤝 Contributing

### Code Review Checklist

- [ ] ✅ Zero unsafe code
- [ ] ✅ Zero production unwraps
- [ ] ✅ Mocks test-gated
- [ ] ✅ No hardcoding (ports, addresses, primal names)
- [ ] ✅ Files < 1000 lines
- [ ] ✅ Tests added (coverage maintained/improved)
- [ ] ✅ Documentation updated
- [ ] ✅ Clippy clean
- [ ] ✅ Formatted

### Git Commit Messages

```
type(scope): brief description

Longer explanation if needed.

- Bullet points for details
- Multiple changes explained
```

**Types**: feat, fix, refactor, test, docs, style, chore

---

## 📈 Performance

### Benchmarking

```bash
cargo bench
```

### Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile
cargo flamegraph --bin sweetgrass

# Open flamegraph.svg
```

### Zero-Copy Optimization

See [docs/guides/ZERO_COPY_OPPORTUNITIES.md](./docs/guides/ZERO_COPY_OPPORTUNITIES.md)

---

## 🆘 Troubleshooting

### PostgreSQL Won't Start

```bash
# Check if port 5432 is in use
lsof -i :5432

# Remove old containers
docker compose down -v
docker compose up -d
```

### Tests Failing Randomly

```bash
# Clean build
cargo clean
cargo build --all-features

# Reset database
docker compose down -v
docker compose up -d
cargo test --all-features
```

### Coverage Not Generating

```bash
# Ensure cargo-llvm-cov is installed
cargo install cargo-llvm-cov

# Clean and regenerate
cargo clean
cargo llvm-cov --all-features --workspace
```

---

## 📞 Support

- **Documentation**: See [README.md](./README.md) and [specs/](./specs/)
- **Issues**: Create GitHub issue
- **Discussions**: GitHub Discussions

---

**🌾 Fair attribution. Complete transparency. Human dignity preserved. 🌾**
