# Level 1: Hello Provenance

**Time**: 5 minutes  
**Difficulty**: Beginner  
**Prerequisites**: None

---

## What You'll Learn

- How to create a Braid (provenance record)
- What provenance means
- W3C PROV-O basics
- REST API usage

---

## Run the Demo

```bash
./demo-first-braid.sh
```

This demo uses a **REAL** SweetGrass service - no mocks!

---

## What Happens

1. **Starts SweetGrass service** (in-memory backend)
2. **Creates a Braid** via REST API
3. **Retrieves it** to verify
4. **Queries by tag** to demonstrate search
5. **Shows PROV-O structure**

---

## Key Concepts

### What is a Braid?

A Braid is a provenance record that tracks:
- **What**: The data (content hash, MIME type, size)
- **Who**: Agent(s) who created/modified it
- **When**: Timestamps
- **How**: Activities that generated it
- **Why**: Purpose and context

### Why Provenance Matters

- **Trust**: Know where data came from
- **Credit**: Fair attribution for contributors
- **Audit**: Complete history for compliance (HIPAA, GDPR)
- **Science**: Reproducible research
- **Economics**: Fair compensation (sunCloud)

---

## Output Files

After running the demo, check `outputs/demo-*/`:
- `braid-request.json` - What we sent to create the Braid
- `braid-response.json` - The Braid that was created
- `braid-retrieved.json` - Retrieved Braid (full structure)
- `query-result.json` - Query results
- `sweetgrass.log` - Service logs

---

## Next Steps

```bash
cd ../02-attribution-basics
./demo-fair-credit.sh
```

Learn how SweetGrass calculates fair attribution for multiple contributors!

