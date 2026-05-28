# CirBinius Architecture

CirBinius is built as a modular compiler pipeline with deterministic artifact contracts.

## Layered Design

```
┌──────────────────────────────────────────────────────────────────┐
│                      CLI / API / SDK                              │
│  User-facing entry points (cirbinius, cirbinius-api, SDK crates) │
└────────────────────────────────┬─────────────────────────────────┘
                                 │
┌────────────────────────────────▼─────────────────────────────────┐
│                     cirbinius-core (dispatch)                     │
│  Validates inputs, orchestrates pipeline stages, emits artifacts  │
│  Stages: compile → analyze → optimize → lower → prove → verify    │
└──┬──────────┬──────────┬──────────┬──────────┬───────────────────┘
   │          │          │          │          │
   ▼          ▼          ▼          ▼          ▼
┌──────┐ ┌────────┐ ┌────────┐ ┌──────────┐ ┌────────┐
│Loader│ │Normal- │ │Optimi- │ │ Binius64 │ │Witness │
│R1CS +│ │izer    │ │zer +   │ │ Lowering │ │Engine  │
│SYM   │ │        │ │Analyzer│ │ Backend  │ │        │
└──────┘ └────────┘ └────────┘ └──────────┘ └────────┘
   │          │          │          │          │
   └──────────┴──────────┴──────────┴──────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────────────┐
│                   cirbinius-artifacts                             │
│  Versioned JSON contracts (CBIR, bundles, manifests)             │
│  Each artifact includes a sealed SHA-256 content hash            │
└──────────────────────────────────────────────────────────────────┘
```

## Key Principles

### 1. Deterministic Artifacts
Every artifact includes:
- `schema_version` — pinned to a versioned JSON Schema in `docs/contracts/`
- `toolchain_version` — `CARGO_PKG_VERSION` at build time
- `*_hash` — SHA-256 of the hash-stable payload, sealed at construction

### 2. Schema Governance
- `docs/contracts/*-vN.schema.json` defines the JSON Schema
- `docs/contracts/*-vN.md` documents the contract
- `*_SCHEMA_VERSION` constants in `cirbinius-artifacts` must match
- `.github/schema_guard.py` enforces alignment in CI

### 3. Fail Closed
- `prove` full mode requires `backend_capabilities.json` with `proof_generation_supported: true`
- `verify` validates bundle hash integrity before any trust
- Witness equivalence checks halt pipeline on mismatch

### 4. Two Compile Modes
- **Compatibility mode**: semantics-first lowering that preserves Circom prime-field behavior
- **Optimized binary mode**: pattern-aware lowering for binary-friendly circuits (boolean, range, XOR, AND, MUX, Merkle, hash motifs)

## Crate Dependency Graph

```
cirbinius-cli → cirbinius-core
cirbinius-api → cirbinius-core

cirbinius-core → cirbinius-frontend
              → cirbinius-normalize
              → cirbinius-cbir
              → cirbinius-optimizer
              → cirbinius-binius64
              → cirbinius-witness
              → cirbinius-artifacts

cirbinius-frontend → cirbinius-r1cs
                  → cirbinius-symbols

All crates → cirbinius-types
```
