# ⚠️  DEPRECATED - DO NOT USE

This directory (`00-standalone/`) has been **DEPRECATED** and consolidated into `00-local-primal/`.

## Why?

This directory was 80% duplicate of `00-local-primal/`, causing:
- Confusion for new users (which to start with?)
- Maintenance burden (updating both)
- Unclear learning progression

## What Happened?

Following **phase1 primal patterns** (NestGate's local-first approach):
- Merged into single authoritative `00-local-primal/` directory
- Added automated `RUN_ME_FIRST.sh` tour (like NestGate)
- Created comprehensive progressive learning path
- Added real execution verification (like Songbird)

## Migration Guide

| Old (00-standalone) | New (00-local-primal) |
|---------------------|----------------------|
| `01-braid-basics/` | `01-hello-provenance/` |
| `02-attribution-engine/` | `02-attribution-basics/` |
| `03-provenance-queries/` | `03-query-engine/` |
| `04-provo-export/` | `04-prov-o-standard/` |
| `05-privacy-controls/` | `05-privacy-controls/` |
| N/A | `06-storage-backends/` (NEW!) |
| N/A | `07-real-verification/` (NEW!) |

## What To Do Now?

**Use the new local showcase:**

```bash
cd ../00-local-primal
./RUN_ME_FIRST.sh
```

This gives you:
- ✅ Progressive levels (1-7) with time estimates
- ✅ Automated guided tour (50 minutes)
- ✅ Real execution verification (NO MOCKS!)
- ✅ Comprehensive README
- ✅ Following phase1 primal excellence

## When Will This Be Removed?

**After v0.5.0 release** (giving time for migration).

See `../../SHOWCASE_RESTRUCTURING_PLAN.md` for full details.

---

**📌 Action Required**: Update your bookmarks and scripts to use `../00-local-primal/` instead!

🌾 **Follow the evolution!** 🌾
