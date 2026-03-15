# 🔍 Tokio Console Integration Guide

**Status**: Ready for integration (commented out by default)  
**Purpose**: Runtime task inspection and concurrency debugging

---

## 📊 What is Tokio Console?

Tokio Console is a diagnostic and debugging tool for Tokio-based async applications. It provides:

- **Real-time task inspection** — See all running tasks
- **Task resource usage** — CPU time, wake counts, poll durations
- **Task tree visualization** — Parent-child relationships
- **Deadlock detection** — Identify stuck tasks
- **Performance profiling** — Find bottlenecks

---

## 🚀 Quick Start

### 1. Install Tokio Console

```bash
cargo install --locked tokio-console
```

### 2. Enable Tokio Console in SweetGrass

Edit `Cargo.toml` (workspace root):

```toml
[workspace.dependencies]
# Uncomment these lines:
tokio = { version = "1.40", features = ["full", "tracing"] }
console-subscriber = "0.4"
```

### 3. Add Console Subscriber to Service

Edit `crates/sweet-grass-service/src/bin/service.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tokio-console subscriber (development only)
    #[cfg(debug_assertions)]
    {
        console_subscriber::init();
    }
    
    // ... rest of your code
}
```

### 4. Run Service with Console

```bash
# Terminal 1: Run SweetGrass service
TOKIO_CONSOLE=1 cargo run --bin sweetgrass

# Terminal 2: Run tokio-console
tokio-console
```

---

## 🎯 Use Cases

### 1. Debugging Concurrent Operations

**Scenario**: Want to see parallel compression tasks

```bash
# Run service
cargo run --bin sweetgrass

# In tokio-console, filter by task name:
# - "compress_batch"
# - Shows all 8 parallel tasks
# - See CPU time, poll counts
```

### 2. Finding Stuck Tasks

**Scenario**: Service seems hung

```bash
tokio-console
# Look for tasks with:
# - High "Idle" time
# - Zero poll counts
# - Long wait times
```

### 3. Performance Profiling

**Scenario**: Identify slow operations

```bash
tokio-console
# Sort by:
# - CPU time (most expensive)
# - Poll count (most active)
# - Total time (longest running)
```

---

## 📖 Example Output

```
Tasks:
┌────────┬──────────────────┬─────────┬─────────┬───────────┬──────────┐
│ ID     │ Name             │ State   │ CPU     │ Polls     │ Total    │
├────────┼──────────────────┼─────────┼─────────┼───────────┼──────────┤
│ 1      │ main             │ Running │ 10.2ms  │ 1,245     │ 5.3s     │
│ 2      │ compress_batch#0 │ Running │ 8.5ms   │ 523       │ 102ms    │
│ 3      │ compress_batch#1 │ Running │ 8.3ms   │ 519       │ 98ms     │
│ 4      │ compress_batch#2 │ Running │ 8.7ms   │ 531       │ 105ms    │
│ 5      │ get_batch#0      │ Running │ 2.1ms   │ 128       │ 25ms     │
│ 6      │ put_batch#0      │ Idle    │ 1.5ms   │ 89        │ 18ms     │
└────────┴──────────────────┴─────────┴─────────┴───────────┴──────────┘
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Enable tokio-console
export TOKIO_CONSOLE=1

# Configure console server port (default: 6669)
export TOKIO_CONSOLE_BIND=127.0.0.1:6669

# Set retention period for task history
export TOKIO_CONSOLE_RETENTION=60s
```

### Cargo Features

Add to specific crates that need console debugging:

```toml
[dependencies]
tokio = { workspace = true, features = ["tracing"] }
console-subscriber = { version = "0.4", optional = true }

[features]
tokio-console = ["console-subscriber"]
```

Then build with:

```bash
cargo build --features tokio-console
```

---

## 🎓 Best Practices

### 1. Development Only

**DO NOT** enable tokio-console in production:

```rust
#[cfg(debug_assertions)]
{
    console_subscriber::init();
}
```

**Reason**: Small performance overhead (~5-10%)

### 2. Task Naming

Name your tasks for easier debugging:

```rust
tokio::task::Builder::new()
    .name("compress_session_123")
    .spawn(async move {
        // ...
    })
```

### 3. Filter by Component

Use task name prefixes:

```rust
// Compression tasks
tokio::spawn(async move { /* ... */ })
    .name("compression::batch");

// Query tasks  
tokio::spawn(async move { /* ... */ })
    .name("query::ancestors");

// Filter in console: query::*
```

### 4. Monitor Specific Operations

```rust
// Before expensive operation
let span = tracing::info_span!("expensive_op");
let _enter = span.enter();

// Operation here
expensive_operation().await;

// Span automatically recorded in console
```

---

## 📊 Interpreting Metrics

### CPU Time
- **High**: Task doing computation
- **Low**: Task mostly waiting (I/O)
- **Zero**: Task never ran (stuck?)

### Poll Count
- **High**: Task waking frequently (busy)
- **Low**: Task waiting (normal for I/O)
- **Zero**: Task never polled (deadlock?)

### Idle Time
- **High**: Task waiting for I/O or signals
- **Low**: Task actively running
- **100%**: Task blocked or starved

### Busy Time
- **High**: CPU-bound work
- **Low**: I/O-bound work
- **Pattern**: Should match CPU time

---

## 🔧 Troubleshooting

### Console Won't Connect

```bash
# Check if service is running
ps aux | grep service

# Check if console port is listening
netstat -an | grep 6669

# Try explicit port
tokio-console http://127.0.0.1:6669
```

### No Tasks Visible

```bash
# Ensure tracing feature enabled
cargo build --features tokio-console

# Check TOKIO_CONSOLE env var
echo $TOKIO_CONSOLE

# Verify console-subscriber initialized
# (add debug log in your code)
```

### Performance Impact

Console has ~5-10% overhead:

```bash
# Benchmark without console
cargo bench

# Benchmark with console
TOKIO_CONSOLE=1 cargo bench

# Compare results
```

---

## 🎯 SweetGrass-Specific Monitoring

### 1. Compression Engine

Monitor parallel session compression:

```bash
# Filter: "compress"
# Expected: 8 concurrent tasks (8 cores)
# CPU time: Should be roughly equal
# Idle time: Should be low (<10%)
```

### 2. Query Engine

Monitor parallel graph traversal:

```bash
# Filter: "ancestors_parallel"
# Expected: N tasks for N ancestors
# Poll count: High (many I/O operations)
# CPU time: Low (I/O bound)
```

### 3. Storage Batch Operations

Monitor concurrent puts:

```bash
# Filter: "put_batch"
# Expected: Parallel database inserts
# Idle time: Medium (waiting for DB)
# CPU time: Low (mostly I/O)
```

### 4. Attribution Calculator

Monitor batch attribution:

```bash
# Filter: "calculate_batch"
# Expected: Parallel calculations
# CPU time: High (computation heavy)
# Idle time: Low (<5%)
```

---

## 📚 Resources

- **Tokio Console Docs**: https://docs.rs/tokio-console/
- **Console Subscriber**: https://docs.rs/console-subscriber/
- **Tracing Guide**: https://tokio.rs/tokio/topics/tracing

---

## 🚀 Quick Commands

```bash
# Install
cargo install --locked tokio-console

# Run service with console
TOKIO_CONSOLE=1 cargo run --bin sweetgrass

# Launch console UI
tokio-console

# Monitor specific component
tokio-console --filter compress

# Save console output
tokio-console --log console.log
```

---

## ✅ When to Use Tokio Console

| Scenario | Use Console? | Why |
|----------|-------------|-----|
| **Debugging stuck tasks** | ✅ Yes | See task states, find deadlocks |
| **Performance profiling** | ✅ Yes | Identify bottlenecks |
| **Verifying concurrency** | ✅ Yes | Confirm parallel execution |
| **Production monitoring** | ❌ No | Use metrics/tracing instead |
| **CI/CD pipeline** | ❌ No | Small overhead, not needed |
| **Development** | ✅ Yes | Great for understanding runtime |

---

**🌾 SweetGrass: Observable, debuggable, concurrent Rust. 🌾**

*See [CHANGELOG](../../CHANGELOG.md) for performance evolution history.*

