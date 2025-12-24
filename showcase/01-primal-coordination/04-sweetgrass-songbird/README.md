# 🐦 SweetGrass + Songbird Integration

Songbird provides the discovery mesh for the ecoPrimals ecosystem.
SweetGrass uses Songbird to discover other primals at runtime.

## Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    Discovery Mesh                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  SweetGrass ──► Songbird Tower ──► Discover Primals         │
│       │              │                    │                  │
│       │              │         ┌──────────┴──────────┐       │
│       │              │         │                     │       │
│       │              │     BearDog              NestGate     │
│       │              │    (Signing)            (Storage)     │
│       │              │         │                     │       │
│       └──────────────┴─────────┴─────────────────────┘       │
│                                                              │
│  Capability-Based Discovery:                                 │
│  • "I need signing" → BearDog                               │
│  • "I need storage" → NestGate                              │
│  • "I need compute" → ToadStool                             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## Live Demo

```bash
./demo-discovery-live.sh
```

## Requirements

- Songbird binaries from `../../bins/songbird-*`
- SweetGrass built and available

## What's Demonstrated

1. **Tower Info**: System capabilities detection
2. **Discovery**: Finding primals by capability
3. **Federation**: Multi-tower mesh networking
4. **Integration**: SweetGrass using discovered services

