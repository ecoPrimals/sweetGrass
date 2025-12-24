# 🍄 SweetGrass + ToadStool Integration

ToadStool provides universal compute for the ecoPrimals ecosystem.
SweetGrass tracks provenance of compute operations.

## Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    Compute Pipeline                          │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Input Braid ──► ToadStool ──► Output Braid                 │
│       │            Compute           │                       │
│       │              │               │                       │
│       └──────────────┼───────────────┘                       │
│                      │                                       │
│              SweetGrass Records:                             │
│              • Input provenance                              │
│              • Compute activity                              │
│              • Output attribution                            │
│              • Resource usage (compute units)                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Live Demo

```bash
./demo-compute-provenance.sh
```

## Requirements

- ToadStool binary from `../../bins/toadstool-cli`
- SweetGrass built and available

## What's Demonstrated

1. **Compute Job Submission**: Send data to ToadStool for processing
2. **Provenance Tracking**: SweetGrass records the transformation
3. **Attribution Chain**: Credits flow to compute provider
4. **Resource Accounting**: Compute units recorded for rewards

## Example Attribution

When ToadStool processes data:

```
Input Data (Alice):     30% attribution
Code/Model (Bob):       20% attribution  
ToadStool Compute:      50% attribution
```

The compute provider (ToadStool operator) receives fair compensation
for resources used.

