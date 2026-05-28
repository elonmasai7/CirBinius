# Prove Precheck Bundle v1

Schema ID: `prove-precheck-bundle/v1`

This contract defines the artifact emitted by `cirbinius prove` precheck after Circom witness generation and optional witness-equivalence verification.

The bundle is intended as a stable handoff format for future proving backends.

## Top-level fields

- `schema_version`: must be `"prove-precheck-bundle/v1"`
- `toolchain_version`: CirBinius crate version that produced the bundle
- `bundle_hash`: SHA-256 hash (`sha256:<hex>`) over the hash-stable payload
- `hashes`: content hashes of key proving inputs
- `report`: precheck summary and mismatch counts

## Hash fields

- `hashes.circuit_hash`: SHA-256 over input R1CS bytes
- `hashes.witness_hash`: SHA-256 over generated `.wtns` bytes
- `hashes.wasm_hash`: SHA-256 over input WASM witness generator bytes
- `hashes.input_hash`: SHA-256 over input JSON bytes
- `hashes.binius_witness_hash`: optional SHA-256 over provided Binius witness JSON bytes

## Report fields

- `report.precheck_passed`: true if precheck passed
- `report.generated_witness_path`: path to generated `.wtns`
- `report.generated_witness_len`: number of witness entries read from `.wtns`
- `report.r1cs_wire_count`: expected wire count from R1CS header
- `report.witness_equivalent`: null when no Binius witness provided; otherwise true/false
- `report.value_mismatch_count`: mismatched witness value count
- `report.constraint_failure_count`: constraint replay failure count

## Validation source

JSON schema file: `docs/contracts/prove-precheck-bundle-v1.schema.json`
