# Proof Bundle v1

Schema ID: `proof-bundle/v1`

This contract defines the proof-run bundle artifact emitted by `cirbinius prove`.

In the current phase, the bundle is generated in `precheck-only` mode and is intended as the stable handoff object for future prover backend integration.

## Top-level fields

- `schema_version`: must be `"proof-bundle/v1"`
- `toolchain_version`: CirBinius crate version that produced the bundle
- `bundle_hash`: SHA-256 hash over hash-stable bundle payload
- `backend`: currently `"binius64"`
- `status`: currently `"precheck-only"`
- `proof_generated`: currently `false`
- `precheck_bundle_path`: path to prove precheck bundle artifact
- `precheck_bundle_hash`: expected hash of precheck bundle artifact
- `backend_capabilities_manifest_path`: optional backend capability manifest path
- `backend_capabilities_manifest_hash`: optional backend capability manifest hash
- `proof_hash`: optional proof bytes hash (`sha256:<hex>`) for real backend mode
- `public_inputs_hash`: optional public inputs hash (`sha256:<hex>`) for real backend mode
- `verifier_key_fingerprint`: optional verifier key fingerprint for real backend mode
- `notes`: implementation and status notes

## Validation source

- `docs/contracts/proof-bundle-v1.schema.json`
