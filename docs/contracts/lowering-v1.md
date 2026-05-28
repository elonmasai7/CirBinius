# Binius64 Lowering v1 Contract

Lowering consumes validated CBIR and emits Binius64-compatible artifacts.

Required lowering metadata:

- `schema_version`
- `mode` (`compatibility` or `optimized_binary`)
- `backend` (`binius64`)
- `gate_count`
- `signal_layout`
- `witness_layout`

Lowering rule policy:

- Unsupported patterns must emit explicit errors.
- Compatibility mode must not silently alter Circom semantics.
