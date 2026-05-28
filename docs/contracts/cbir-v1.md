# CBIR v1 Contract

Schema ID: `cbir/v1`

CBIR (Circom-Binius Intermediate Representation) is the canonical normalized IR between frontend parsing and backend lowering.

Required top-level fields:

- `metadata.schema_version` must be `"cbir/v1"`
- `metadata.toolchain_version`
- `metadata.content_hash`
- `backend`
- `field_modulus_hex`
- `wire_count`
- `public_output_count`
- `public_input_count`
- `private_input_count`
- `signals`
- `constraints`

Invariants:

- Signal IDs are unique and stable.
- Constraint IDs are unique and deterministic.
- All references are resolved or compilation fails with diagnostics.

Validation source:

- `docs/contracts/cbir-v1.schema.json`
