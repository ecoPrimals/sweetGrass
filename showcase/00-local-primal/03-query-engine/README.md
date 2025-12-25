# Level 3: Query Engine

**Time**: 10 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Level 1-2

---

## What You'll Learn

- How to filter Braids by various criteria
- Query combining (AND conditions)
- Pagination for large result sets
- Result ordering
- Query performance

---

## Run the Demo

```bash
./demo-filters.sh
```

---

## Query Capabilities

### Filter By
- **Tag**: `?tag=research`
- **Agent**: `?agent=did:key:z6Mk...`
- **Type**: `?type=Dataset`
- **Time range**: `?from=2025-01-01&to=2025-12-31`
- **MIME type**: `?mime_type=application/json`

### Combine Filters
```bash
# Multiple tags (AND)
?tag=research&tag=ml

# Tag + Agent
?tag=research&agent=did:key:z6Mk...

# Complex queries
?tag=ml&type=Dataset&mime_type=application/json
```

### Pagination
```bash
# First page (3 results)
?limit=3&offset=0

# Second page
?limit=3&offset=3

# Custom page size
?limit=50&offset=100
```

### Ordering
```bash
# Newest first (default)
?order=created_desc

# Oldest first
?order=created_asc

# Largest first
?order=size_desc

# Smallest first
?order=size_asc
```

---

## Performance

SweetGrass uses efficient indexing:
- **Tag index**: O(1) lookup
- **Agent index**: O(1) lookup
- **Combined filters**: Intersection of index results
- **Typical query**: < 10ms

---

## Next Steps

```bash
cd ../04-prov-o-standard
./demo-prov-o-export.sh
```

Learn how to export provenance in W3C PROV-O format!

