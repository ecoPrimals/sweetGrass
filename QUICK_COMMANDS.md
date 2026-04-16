# 🌾 SweetGrass Quick Commands

**One-line commands for all common operations.**

---

## 🚀 IMMEDIATE DEPLOYMENT

```bash
# Build and run (zero configuration!)
cargo build --release && ./target/release/sweetgrass server
```

That's it! Service runs with zero config.

### UniBin Subcommands

```bash
# Server (default)
./target/release/sweetgrass server

# Status (HTTP health check)
./target/release/sweetgrass status

# Capabilities (offline capability dump)
./target/release/sweetgrass capabilities

# Socket (print resolved UDS path)
./target/release/sweetgrass socket
```

---

## 🔨 BUILD COMMANDS

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check --workspace

# Clean build artifacts
cargo clean
```

---

## ✅ TEST COMMANDS

```bash
# All tests (1,502 local + 58 Docker CI)
cargo test --all-features

# Unit tests only
cargo test --workspace --lib

# Integration tests only
cargo test --workspace --test '*'

# Specific crate
cargo test --package sweet-grass-core

# With output
cargo test --workspace -- --nocapture

# Single test
cargo test test_name

# Chaos tests (17 scenarios)
cargo test --test chaos

# PostgreSQL tests (requires Docker)
cargo test --package sweet-grass-store-postgres --test integration -- --ignored

# Watch mode (requires cargo-watch)
cargo watch -x test
```

---

## 📊 COVERAGE COMMANDS

```bash
# Generate HTML coverage report
cargo llvm-cov --workspace --html

# Open coverage report
open target/llvm-cov/html/index.html  # macOS
xdg-open target/llvm-cov/html/index.html  # Linux

# Coverage summary to console
cargo llvm-cov --workspace

# Coverage with specific test
cargo llvm-cov --workspace --test integration
```

---

## 🎯 BENCHMARK COMMANDS

```bash
# Run criterion benchmarks (7 groups: braid, store, hash, query, attribution, compression, traversal)
cargo bench --package sweet-grass-service

# Save baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# Open HTML report
open target/criterion/report/index.html
```

---

## 🧹 CODE QUALITY

```bash
# Clippy (pedantic + nursery, zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Format code
cargo fmt --all

# Doc check
cargo doc --workspace --no-deps

# All quality checks
cargo clippy --workspace --all-targets -- -D warnings && \
cargo fmt --all -- --check && \
cargo doc --workspace --no-deps
```

---

## 📖 DOCUMENTATION

```bash
# Generate and open docs
cargo doc --workspace --no-deps --open

# Generate docs without opening
cargo doc --workspace --no-deps

# Check doc warnings
cargo doc --workspace --no-deps 2>&1 | grep warning

# Serve docs (requires python)
cd target/doc && python3 -m http.server 8000
```

---

## 🏥 SERVICE HEALTH CHECKS

```bash
# Basic health
curl http://localhost:8080/health

# Detailed health with metrics
curl http://localhost:8080/health/detailed

# Kubernetes liveness
curl http://localhost:8080/live

# Kubernetes readiness
curl http://localhost:8080/ready

# JSON-RPC health probes (wateringHole protocol v3.0)
curl -X POST http://localhost:8080/jsonrpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":1}'

curl -X POST http://localhost:8080/jsonrpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"health.readiness","params":{},"id":1}'

# Pretty JSON
curl http://localhost:8080/health/detailed | jq
```

---

## 🌐 API TESTING

```bash
# Create a braid
curl -X POST http://localhost:8080/api/v1/braids \
  -H "Content-Type: application/json" \
  -d '{
    "data": "SGVsbG8gV29ybGQ=",
    "mime_type": "text/plain",
    "attributed_to": "did:key:z6MkTest"
  }'

# Query braids
curl http://localhost:8080/api/v1/braids?limit=10

# Get by hash
curl http://localhost:8080/api/v1/braids/sha256:abc123

# Get provenance graph
curl http://localhost:8080/api/v1/provenance/sha256:abc123

# Query by agent
curl http://localhost:8080/api/v1/braids?agent=did:key:z6MkTest

# Pretty JSON
curl http://localhost:8080/api/v1/braids | jq
```

---

## 🐳 DOCKER COMMANDS

```bash
# PostgreSQL for integration tests
docker compose up -d postgres

# PostgreSQL + pgAdmin
docker compose --profile admin up -d

# View logs
docker compose logs -f

# Stop
docker compose down

# Clean slate (remove volumes)
docker compose down -v
```

---

## 🔧 ENVIRONMENT VARIABLES

```bash
# Memory storage (default)
./target/release/sweetgrass server

# PostgreSQL
DATABASE_URL=postgresql://user:pass@localhost:5432/sweetgrass \
./target/release/sweetgrass server

# redb (recommended embedded)
STORAGE_BACKEND=redb \
STORAGE_PATH=/var/lib/sweetgrass/data.redb \
./target/release/sweetgrass server

# With discovery
DISCOVERY_ADDRESS=discovery.example.com:9090 \
DATABASE_URL=postgresql://user:pass@localhost:5432/sweetgrass \
./target/release/sweetgrass server

# Custom ports
REST_PORT=8080 \
TARPC_PORT=9090 \
./target/release/sweetgrass server

# Debug logging
RUST_LOG=debug \
RUST_BACKTRACE=1 \
./target/release/sweetgrass server
```

---

## 🔍 DEBUGGING COMMANDS

```bash
# Run with debug logs
RUST_LOG=debug cargo run

# Run with trace logs (very verbose)
RUST_LOG=trace cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Run specific test with output
cargo test test_name -- --nocapture

# Check for outdated dependencies
cargo outdated

# Dependency tree
cargo tree

# Audit dependencies for vulnerabilities
cargo audit
```

---

## 📈 MONITORING COMMANDS

```bash
# Watch logs in real-time
tail -f /var/log/sweetgrass/service.log

# Monitor with journalctl (systemd)
journalctl -u sweetgrass -f

# Check resource usage
ps aux | grep sweetgrass

# Memory usage
pmap $(pgrep sweetgrass)

# Network connections
netstat -an | grep :8080

# HTTP request rate
watch -n 1 'curl -s http://localhost:8080/health/detailed | jq .requests_served'
```

---

## 🛠️ DEVELOPMENT WORKFLOW

```bash
# 1. Make changes
vim crates/sweet-grass-core/src/lib.rs

# 2. Check compilation
cargo check

# 3. Run tests
cargo test --workspace

# 4. Check coverage
cargo llvm-cov --workspace --html

# 5. Lint
cargo clippy --workspace --all-targets -- -D warnings

# 6. Format
cargo fmt --all

# 7. Build release
cargo build --release

# 8. Run
./target/release/sweetgrass server

# Or all at once:
cargo check && \
cargo test --workspace && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo fmt --all -- --check && \
cargo build --release
```

---

## 🚢 RELEASE WORKFLOW

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Update CHANGELOG.md
vim CHANGELOG.md

# 3. Run full test suite
cargo test --workspace

# 4. Run chaos tests
cargo test --test chaos

# 5. Check coverage
cargo llvm-cov --workspace

# 6. Run benchmarks
cargo bench --workspace

# 7. Build release
cargo build --release

# 8. Tag release
git tag -a vX.Y.Z -m "Release vX.Y.Z"

# 9. Push
git push && git push --tags
```

---

## 🧪 INTEGRATION TESTING

```bash
# Start PostgreSQL with Docker
docker run -d \
  --name sweetgrass-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=sweetgrass \
  -p 5432:5432 \
  postgres:16

# Run service with PostgreSQL
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/sweetgrass \
./target/release/sweetgrass server

# Run integration tests
cargo test --workspace --test '*' -- --ignored

# Stop PostgreSQL
docker stop sweetgrass-postgres
docker rm sweetgrass-postgres
```

---

## 📦 DEPENDENCY MANAGEMENT

```bash
# Add dependency
cargo add tokio --features full

# Add dev dependency
cargo add --dev proptest

# Update dependencies
cargo update

# Check for outdated
cargo outdated

# Audit for vulnerabilities
cargo audit

# Show dependency tree
cargo tree

# Show specific crate dependencies
cargo tree --package sweet-grass-core
```

---

## 🔐 SECURITY CHECKS

```bash
# Check for unsafe code (should be zero)
rg "unsafe" --type rust crates/

# Check for unwraps in production
rg "\.unwrap\(\)" --type rust crates/ | grep -v "#\[cfg\(test\)\]"

# Audit dependencies
cargo audit

# Check for hardcoded secrets
rg "password|secret|api_key" --type rust crates/

# SAST scan (if using cargo-geiger)
cargo geiger
```

---

## 📊 METRICS COLLECTION

```bash
# Request count
curl -s http://localhost:8080/health/detailed | jq .requests_served

# Uptime
curl -s http://localhost:8080/health/detailed | jq .uptime_seconds

# Version
curl -s http://localhost:8080/health/detailed | jq .version

# Storage status
curl -s http://localhost:8080/health/detailed | jq .storage

# All metrics
curl -s http://localhost:8080/health/detailed | jq
```

---

## 🎯 QUICK VERIFICATION

```bash
# Verify everything is working
cargo build --release && \
cargo test --workspace && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo fmt --all -- --check && \
cargo llvm-cov --workspace && \
./target/release/sweetgrass &
sleep 2 && \
curl http://localhost:8080/health && \
pkill sweetgrass && \
echo "✅ All checks passed!"
```

---

## 📚 DOCUMENTATION FILES

```bash
# Project overview
cat README.md

# Development guide
cat DEVELOPMENT.md

# Roadmap
cat ROADMAP.md

# Version history
cat CHANGELOG.md

# Zero-copy opportunities
cat docs/guides/ZERO_COPY_OPPORTUNITIES.md

# Specifications
ls specs/
```

---

## 🌟 ONE-LINERS

```bash
# Build, test, and run
cargo build --release && cargo test --workspace && ./target/release/sweetgrass server

# Full quality check
cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check && cargo test --workspace

# Coverage + benchmark
cargo llvm-cov --workspace --html && cargo bench --workspace

# Deploy check
cargo build --release && cargo test --workspace && ./target/release/sweetgrass --help

# Health check loop
watch -n 5 'curl -s http://localhost:8080/health | jq'
```

---

## 🎉 SUCCESS VERIFICATION

```bash
# Verify production readiness
echo "Building..." && cargo build --release && \
echo "Testing..." && cargo test --workspace && \
echo "Linting..." && cargo clippy --workspace --all-targets -- -D warnings && \
echo "Formatting..." && cargo fmt --all -- --check && \
echo "Coverage..." && cargo llvm-cov --workspace && \
echo "Benchmarking..." && cargo bench --workspace && \
echo "Starting service..." && ./target/release/sweetgrass &
sleep 2 && \
echo "Health check..." && curl http://localhost:8080/health && \
echo "" && \
echo "✅ SweetGrass is PRODUCTION READY!" && \
pkill sweetgrass
```

---

🌾 **SweetGrass: One command away from production.** 🌾

**Quick Deploy**: `cargo build --release && ./target/release/sweetgrass`  
**Full Check**: `cargo test --workspace && cargo clippy --workspace -- -D warnings`  
**Coverage**: `cargo llvm-cov --workspace --html`  
**Benchmarks**: `cargo bench --workspace`

**Status**: ✅ **READY TO DEPLOY**

