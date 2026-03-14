# Level 6: Storage Backends

**Time**: ~10 minutes  
**Prerequisites**: SweetGrass service binary, optional: PostgreSQL  
**Philosophy**: Right tool for the right job - multiple storage backends  

## What You'll Learn

SweetGrass supports multiple storage backends for different use cases:

1. **Memory Backend**
   - Fast, ephemeral storage
   - Perfect for testing and development
   - Data lost on restart

2. **Sled Backend**
   - Embedded key-value store
   - Pure Rust (no external dependencies!)
   - Perfect for single-node deployments
   - Persistent, ACID-compliant

3. **PostgreSQL Backend**
   - Production-grade RDBMS
   - Multi-node support
   - Advanced queries and reporting
   - Industry-standard reliability

## Quick Start

```bash
./demo-backends.sh
```

## What the Demo Does

1. Starts SweetGrass with **Memory** backend
   - Creates and queries Braids
   - Shows ephemeral nature

2. Starts SweetGrass with **Sled** backend
   - Creates and queries Braids
   - Shows persistence across restarts

3. Starts SweetGrass with **PostgreSQL** backend (if available)
   - Creates and queries Braids
   - Shows enterprise-grade storage

## Real Execution

This demo uses the **real SweetGrass service binary** with **real storage backends** (no mocks).

Multiple service instances are started on different ports, each with a different backend.

## Backend Comparison

| Backend | Speed | Persistence | Dependencies | Use Case |
|---------|-------|-------------|--------------|----------|
| Memory | ⚡⚡⚡ | ❌ Ephemeral | None | Testing, CI/CD |
| Sled | ⚡⚡ | ✅ Persistent | None | Single-node prod |
| PostgreSQL | ⚡ | ✅ Persistent | PostgreSQL | Multi-node, enterprise |

## Configuration Examples

### Memory (Default)
```bash
sweetgrass server --storage memory
```

### Sled (Pure Rust Embedded)
```bash
SLED_DB_PATH=/var/lib/sweetgrass/data \
sweetgrass server --storage sled
```

### PostgreSQL (Enterprise)
```bash
DATABASE_URL=postgres://user:pass@localhost/sweetgrass \
sweetgrass server --storage postgres
```

## Why Multiple Backends?

Different deployments have different needs:

### Development/Testing
- **Use**: Memory backend
- **Why**: Fast, no cleanup needed
- **Trade-off**: Data lost on restart

### Single-Node Production
- **Use**: Sled backend
- **Why**: Pure Rust, no dependencies, persistent
- **Trade-off**: Single-node only

### Multi-Node Production
- **Use**: PostgreSQL backend
- **Why**: Multi-node support, advanced queries
- **Trade-off**: Requires PostgreSQL infrastructure

## Primal Sovereignty Principle

Notice: **No hardcoded backend choice!**

The backend is selected at **runtime via environment variables**, not compiled in.

```rust
// ✅ GOOD: Runtime selection
let backend = StorageBackend::from_env()?;

// ❌ BAD: Compile-time hardcoding
let backend = PostgresBackend::new();
```

## Storage Interface

All backends implement the same trait:

```rust
#[async_trait]
pub trait BraidStore {
    async fn put(&self, braid: &Braid) -> Result<()>;
    async fn get(&self, id: &BraidId) -> Result<Option<Braid>>;
    async fn query(&self, filter: &Filter) -> Result<Vec<Braid>>;
    async fn delete(&self, id: &BraidId) -> Result<()>;
}
```

**Result**: Swap backends without changing application code!

## Real-World Deployments

### Scenario 1: Startup (Low Budget)
```
Sled backend on single VPS
- $5/month hosting
- No database management
- Persistent, reliable
```

### Scenario 2: Growing Company
```
PostgreSQL on managed service
- Multi-region replication
- Advanced analytics
- Team collaboration
```

### Scenario 3: Research Lab
```
Memory for experiments
PostgreSQL for published results
- Fast iteration (Memory)
- Permanent record (PostgreSQL)
```

## Next Steps

After completing this level, proceed to:
- **Level 7**: Real Verification (no-mocks validation)
- **01-primal-coordination**: See how SweetGrass integrates with other primals

## Learn More

- See `../../crates/sweet-grass-store/` for storage implementation
- Backend performance benchmarks in documentation
- Migration guides between backends

