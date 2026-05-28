# Backend Capabilities v1

Schema ID: `backend-capabilities/v1`

This manifest is emitted by `cirbinius doctor` and consumed by `cirbinius prove` to gate backend behavior in an auditable way.

## Top-level fields

- `schema_version`: must be `"backend-capabilities/v1"`
- `toolchain_version`: CirBinius crate version that produced the manifest
- `manifest_hash`: SHA-256 hash over hash-stable manifest payload
- `backend`: currently `"binius64"`
- `capabilities`: backend feature flags
- `notes`: operator/audit notes

## Capability flags

- `precheck_only_supported`
- `proof_generation_supported`
- `verify_supported`
- `proof_hash_supported`
- `public_inputs_hash_supported`
- `verifier_key_fingerprint_supported`

## Validation source

- `docs/contracts/backend-capabilities-v1.schema.json`
