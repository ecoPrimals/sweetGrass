# 🌾 Demo: SweetGrass + RhizoCrypt

**Goal**: Compress edit sessions into Braids  
**Time**: 15 minutes  
**Complexity**: Intermediate  
**Prerequisites**: RhizoCrypt running (or mock mode)

---

## 🎯 What This Demo Shows

1. Subscribe to RhizoCrypt events
2. Receive session commits
3. Compress to Braids (0/1/Many model)
4. Track session provenance

---

## 🚀 Run the Demo

```bash
./demo-session-compression.sh
```

---

## 📖 Concepts

### Session Events

RhizoCrypt emits events as sessions progress:

| Event | Description |
|-------|-------------|
| `Started` | Session began |
| `VertexAdded` | Edit added to session |
| `BranchCreated` | Fork in the session DAG |
| `Committed` | Session finalized |
| `RolledBack` | Session discarded |

### 0/1/Many Compression

| Outcome | Braids | Description |
|---------|--------|-------------|
| Rollback/Empty | 0 | Session discarded |
| Single Commit | 1 | One coherent Braid |
| Branched DAG | N | Multiple Braids for branches |

### Session Vertices

```rust
pub struct SessionVertex {
    pub id: String,
    pub content_hash: String,
    pub mime_type: String,
    pub agent: Did,
    pub parent: Option<String>,
    pub activity_type: ActivityType,
    pub state: VertexState,
}
```

---

## 📊 Expected Output

```
🌾 SweetGrass + RhizoCrypt Demo
===============================

Step 1: Discovering RhizoCrypt...
  Looking for capability: SessionStreaming
  ✅ Found RhizoCrypt at localhost:8092

Step 2: Subscribing to events...
  ✅ Subscribed to session stream

Step 3: Receiving session...
  Event: SessionStarted (session-456)
  Event: VertexAdded (v1 - import)
  Event: VertexAdded (v2 - transform)
  Event: VertexAdded (v3 - compute)
  Event: SessionCommitted

Session Summary:
  ID: session-456
  Vertices: 3
  Branches: 1 (linear)
  Agents: [Alice, Bob, Charlie]

Step 4: Compressing session...
  Model: Single (1 Braid)
  ✅ Created Braid: urn:braid:session-456-001

Braid Details:
  Derived from: 3 vertices
  Attribution: Alice 40%, Bob 35%, Charlie 25%
  Activity: Computation

✅ Session compressed!
```

---

## 🔧 Code Walkthrough

### Subscribing to Events

```rust
use sweet_grass_integration::listener::{EventHandler, RhizoCryptClient};

let handler = EventHandler::new(discovery, compression, store).await?;
let mut stream = handler.subscribe().await?;

while let Some(event) = stream.next().await {
    match event.event_type {
        SessionEventType::Committed => {
            // Compress the session
        }
        _ => {}
    }
}
```

### Compressing a Session

```rust
use sweet_grass_compression::{CompressionEngine, Session};

let engine = CompressionEngine::new(factory);
let result = engine.compress(&session)?;

match result.count() {
    0 => println!("Session discarded"),
    1 => println!("Single Braid created"),
    n => println!("{} Braids created for branches", n),
}
```

---

## 💡 Key Insights

### Real-Time Compression
Sessions are compressed as they commit, not in batch.

### Attribution Preserved
Contributors to session vertices get attribution in the resulting Braid(s).

### Branches → Multiple Braids
If a session has branches (forks), each branch becomes a separate Braid.

---

## 🎯 Success Criteria

- [ ] Subscribed to RhizoCrypt events
- [ ] Received a session
- [ ] Compressed to Braid(s)
- [ ] Understood 0/1/Many model

---

## 📚 Next Steps

Continue to: `../03-sweetgrass-loamspine/`

Learn how to anchor commits with LoamSpine!

