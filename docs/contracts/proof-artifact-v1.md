# Proof Artifact v1

Schema ID: `proof-artifact/v1`

This contract defines the proof artifact emitted by `cirbinius prove` when real proof generation is enabled.

The proof artifact contains a Merkle-commitment-based constraint satisfaction proof over the CBIR constraints.

## Top-level fields

- `schema_version`: must be `"proof-artifact/v1"`
- `toolchain_version`: CirBinius crate version that produced the proof
- `proof_hash`: SHA-256 hash over hash-stable proof payload (`sha256:<hex>`)
- `source_cbir_hash`: hash of the source CBIR document that was proved
- `merkle_root`: Merkle tree root hash over the witness (`0x`-prefixed hex)
- `num_constraints`: number of constraints proved
- `num_wires`: number of wires in the witness
- `public_input_count`: number of public input wires
- `public_output_count`: number of public output wires
- `constraint_proofs`: array of per-constraint proofs
- `public_inputs_hash`: optional hash of public input values
- `verifier_key_fingerprint`: optional verifier key fingerprint (reserved for future use)

## Constraint proof structure

Each `constraint_proof` entry contains:

- `constraint_id`: matching the ID in the CBIR document
- `wire_values`: array of wire value entries used in this constraint
  - `wire_id`: wire index
  - `value_hex`: witness value as hex
  - `merkle_siblings`: Merkle sibling hashes for verifying this wire against the root
- `a_eval_hex`: evaluated value of linear combination A
- `b_eval_hex`: evaluated value of linear combination B
- `c_eval_hex`: evaluated value of linear combination C
- `resolved`: whether `A * B == C` (mod field prime)

## Proof system

1. Prover builds a binary Merkle tree over the witness (leaf = SHA-256(wire_id \|\| value_hex))
2. For each constraint, evaluates A(w), B(w), C(w) and collects Merkle paths
3. Proof artifact is hash-sealed for integrity
4. Verifier checks Merkle paths, re-evaluates constraints, and verifies public inputs

This is a non-zero-knowledge constraint satisfaction proof.

## Validation source

- `docs/contracts/proof-artifact-v1.schema.json`
