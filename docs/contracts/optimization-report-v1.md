# Optimization Report Contract (v1)

## Schema Version

`optimization-report/v1`

## Purpose

The optimization report is a deterministic artifact produced by the
`cirbinius compile --mode optimized` command. It documents which
constraints were optimized, which fell back to compatibility mode,
how much savings were achieved, and which patterns were detected.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | yes | Must be `"optimization-report/v1"` |
| `toolchain_version` | string | yes | CirBinius version |
| `report_hash` | string | yes | `sha256:...` hash of hash-stable view |
| `source_cbir_hash` | string | yes | `sha256:...` hash of source CBIR document |
| `mode` | string | yes | `"compatibility"` or `"optimized"` |
| `min_confidence` | string | yes | Minimum confidence threshold used |
| `stats` | object | yes | Aggregate optimization statistics |
| `patterns` | array | yes | Per-pattern detection entries |
| `gate_counts` | object | yes | Map of gate kind to count |
| `warnings` | array | yes | Non-fatal optimization warnings |

## Confidence Levels

- `Exact` — Mathematically provable pattern match (e.g., structural R1CS proof)
- `Strong` — Very strong heuristic match (structural + hint corroboration)
- `Heuristic` — Reasonable match (hint-based with partial structure)
- `Experimental` — Weak match (single keyword hint, no structural verification)
- `Rejected` — Confirmed non-match after analysis

## Hash-Seal

The report hash is computed by serializing a hash-stable view of the
report (excluding `report_hash` itself) to canonical JSON and computing
`sha256(serialized)` with `sha256:` prefix.

## Integrity

Consumers MUST verify:
1. `schema_version == "optimization-report/v1"`
2. `report_hash` recomputes to the same value via `validate_hash()`
