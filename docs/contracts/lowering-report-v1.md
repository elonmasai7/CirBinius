# Lowering Report Contract (v1)

## Schema Version

`lowering-report/v1`

## Purpose

The lowering report is a deterministic artifact produced by the
`cirbinius lower` command. It documents how each CBIR constraint was
classified into a Binius64 gate kind during the lowering pass chain.

## Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | string | yes | Must be `"lowering-report/v1"` |
| `toolchain_version` | string | yes | CirBinius version |
| `report_hash` | string | yes | `sha256:...` hash of hash-stable view |
| `source_cbir_hash` | string | yes | `sha256:...` hash of source CBIR document |
| `total_constraints` | integer | yes | Number of constraints lowered |
| `gate_counts` | object | yes | Map of gate kind to count |
| `gates` | array | yes | Per-constraint lowering entries |
| `limb_width` | string | yes | Target limb width (u8/u16/u32/u64/auto) |
| `warnings` | array | yes | Non-fatal lowering warnings |

## Gate Kinds

- `hash` – Poseidon, SHA-256, Keccak, MiMC, Pedersen hash constraints
- `merkle_path` – Merkle tree path verification constraints
- `mux_selector` – Multiplexer / selector / ternary constraints
- `xor` – XOR operation constraints
- `and` – AND / boolean multiplication constraints
- `range_check` – Range check / comparison constraints
- `boolean` – Boolean / bit constraints
- `mul` – General field multiplication constraints
- `generic_compat` – Compatibility fallback for unrecognized constraints

## Hash-Seal

The report hash is computed by serializing a hash-stable view of the
report (excluding `report_hash` itself) to canonical JSON and computing
`sha256(serialized)` with `sha256:` prefix.

## Integrity

Consumers MUST verify:
1. `schema_version == "lowering-report/v1"`
2. `report_hash` recomputes to the same value via `validate_hash()`
