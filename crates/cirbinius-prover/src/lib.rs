use anyhow::{Result, anyhow, ensure};
use cirbinius_artifacts::{PerConstraintProof, ProofArtifact, WireValueEntry};
use cirbinius_cbir::{CbirConstraint, CbirDocument, CbirLinearCombination};
use num_bigint::BigUint;
use num_traits::{Num, Zero};
use sha2::{Digest, Sha256};

pub fn prove(
    cbir: &CbirDocument,
    witness_values: &[String],
    field_modulus_hex: &str,
) -> Result<ProofArtifact> {
    let prime = parse_field_modulus(field_modulus_hex)?;
    let n_wires = cbir.wire_count as usize;

    ensure!(
        witness_values.len() >= n_wires,
        "witness has {} values, expected at least {}",
        witness_values.len(),
        n_wires
    );

    // Normalize witness values modulo prime
    let normed: Vec<String> = witness_values
        .iter()
        .take(n_wires)
        .map(|v| {
            let parsed = parse_biguint(v) % &prime;
            biguint_to_hex(&parsed)
        })
        .collect();

    // Build sorted wire-value pairs for the Merkle tree
    let wire_value_pairs: Vec<(u32, String)> = (0..n_wires)
        .map(|i| (i as u32, normed[i].clone()))
        .collect();

    let tree = MerkleTree::new(&wire_value_pairs);

    // Evaluate each constraint and build constraint proofs
    let mut constraint_proofs = Vec::with_capacity(cbir.constraints.len());
    for constraint in &cbir.constraints {
        let (a_eval, b_eval, c_eval, resolved) = evaluate_constraint(constraint, &normed, &prime)?;

        // Collect wire IDs used in this constraint (deduplicated, sorted)
        let mut wire_ids: Vec<u32> = collect_wire_ids(constraint);
        wire_ids.sort();
        wire_ids.dedup();

        let mut wire_values = Vec::with_capacity(wire_ids.len());
        for wid in &wire_ids {
            let val = normed[*wid as usize].clone();
            let merkle_proof = tree
                .proof(*wid, &wire_value_pairs)
                .ok_or_else(|| anyhow!("failed to generate merkle proof for wire {}", wid))?;
            wire_values.push(WireValueEntry {
                wire_id: *wid,
                value_hex: val,
                merkle_siblings: merkle_proof
                    .siblings
                    .iter()
                    .map(|h| hex_encode(h))
                    .collect(),
            });
        }

        constraint_proofs.push(PerConstraintProof {
            constraint_id: constraint.id,
            wire_values,
            a_eval_hex: biguint_to_hex(&a_eval),
            b_eval_hex: biguint_to_hex(&b_eval),
            c_eval_hex: biguint_to_hex(&c_eval),
            resolved,
        });
    }

    let merkle_root = hex_encode(&tree.root());
    let public_input_hash = compute_public_inputs_hash(&normed, cbir.public_input_count);

    let mut artifact = ProofArtifact {
        schema_version: "proof-artifact/v1".to_string(),
        toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
        proof_hash: String::new(),
        source_cbir_hash: cbir.metadata.content_hash.clone(),
        merkle_root,
        num_constraints: cbir.constraints.len() as u64,
        num_wires: cbir.wire_count,
        public_input_count: cbir.public_input_count,
        public_output_count: cbir.public_output_count,
        constraint_proofs,
        public_inputs_hash: public_input_hash,
        verifier_key_fingerprint: None,
    };
    artifact.seal_hash();
    Ok(artifact)
}

pub fn verify(
    artifact: &ProofArtifact,
    cbir: &CbirDocument,
    public_inputs: &[String],
    field_modulus_hex: &str,
) -> Result<bool> {
    if !artifact.validate_hash() {
        return Ok(false);
    }

    if artifact.num_constraints != cbir.constraints.len() as u64 {
        return Ok(false);
    }

    if artifact.source_cbir_hash != cbir.metadata.content_hash {
        return Ok(false);
    }

    let prime = parse_field_modulus(field_modulus_hex)?;

    // Verify public inputs
    if !verify_public_inputs(artifact, cbir, public_inputs, &prime) {
        return Ok(false);
    }

    // Verify each constraint proof
    for constraint_proof in &artifact.constraint_proofs {
        let constraint = cbir
            .constraints
            .iter()
            .find(|c| c.id == constraint_proof.constraint_id)
            .ok_or_else(|| {
                anyhow!(
                    "constraint {} not found in CBIR",
                    constraint_proof.constraint_id
                )
            })?;

        // Build a local witness from the constraint proof's wire values
        let local_witness: Vec<String> = (0..cbir.wire_count as usize)
            .map(|i| {
                constraint_proof
                    .wire_values
                    .iter()
                    .find(|wv| wv.wire_id == i as u32)
                    .map(|wv| wv.value_hex.clone())
                    .unwrap_or_else(|| "0x0".to_string())
            })
            .collect();

        // Verify Merkle path for each wire
        for wv in &constraint_proof.wire_values {
            let merkle_bytes: Vec<Vec<u8>> =
                wv.merkle_siblings.iter().map(|h| hex_decode(h)).collect();

            let leaf = leaf_hash(wv.wire_id, &wv.value_hex);
            let index = wv.wire_id as usize;

            if !verify_merkle_proof(&artifact.merkle_root, &leaf, &merkle_bytes, index) {
                return Ok(false);
            }
        }

        // Re-evaluate the constraint
        let (a_eval, b_eval, c_eval, _) = evaluate_constraint(constraint, &local_witness, &prime)?;

        let expected_a = parse_biguint(&constraint_proof.a_eval_hex) % &prime;
        let expected_b = parse_biguint(&constraint_proof.b_eval_hex) % &prime;
        let expected_c = parse_biguint(&constraint_proof.c_eval_hex) % &prime;

        if a_eval != expected_a || b_eval != expected_b || c_eval != expected_c {
            return Ok(false);
        }

        // Check that A * B == C (mod prime)
        let product = (&a_eval * &b_eval) % &prime;
        if product != c_eval {
            return Ok(false);
        }

        if !constraint_proof.resolved {
            return Ok(false);
        }
    }

    Ok(true)
}

fn verify_public_inputs(
    artifact: &ProofArtifact,
    cbir: &CbirDocument,
    public_inputs: &[String],
    prime: &BigUint,
) -> bool {
    let expected_count = cbir.public_input_count as usize;
    if public_inputs.len() != expected_count {
        return false;
    }

    // Compute hash over public input values and compare against artifact
    let mut hasher = Sha256::new();
    for val in public_inputs {
        let normed = parse_biguint(val) % prime;
        hasher.update(biguint_to_hex(&normed).as_bytes());
    }
    let computed_hash = format!("sha256:{:x}", hasher.finalize());

    if let Some(ref expected_hash) = artifact.public_inputs_hash
        && computed_hash != *expected_hash
    {
        return false;
    }

    true
}

fn compute_public_inputs_hash(
    normed_witness: &[String],
    public_input_count: u32,
) -> Option<String> {
    if public_input_count == 0 {
        return None;
    }
    let mut hasher = Sha256::new();
    for i in 0..public_input_count as usize {
        if let Some(val) = normed_witness.get(i) {
            hasher.update(val.as_bytes());
        }
    }
    Some(format!("sha256:{:x}", hasher.finalize()))
}

fn evaluate_constraint(
    constraint: &CbirConstraint,
    witness: &[String],
    prime: &BigUint,
) -> Result<(BigUint, BigUint, BigUint, bool)> {
    let a = eval_linear_combination(&constraint.a, witness, prime)?;
    let b = eval_linear_combination(&constraint.b, witness, prime)?;
    let c = eval_linear_combination(&constraint.c, witness, prime)?;

    let product = (&a * &b) % prime;
    let resolved = product == c;

    Ok((a, b, c, resolved))
}

fn eval_linear_combination(
    linear: &CbirLinearCombination,
    witness: &[String],
    prime: &BigUint,
) -> Result<BigUint> {
    let mut acc = BigUint::zero();
    for term in &linear.terms {
        let idx = term.wire_id as usize;
        if idx >= witness.len() {
            anyhow::bail!(
                "wire {} referenced in constraint but witness has {} values",
                term.wire_id,
                witness.len()
            );
        }
        let coeff = parse_biguint(&term.coeff_hex) % prime;
        let value = parse_biguint(&witness[idx]) % prime;
        let term_value = (coeff * value) % prime;
        acc = (acc + term_value) % prime;
    }
    Ok(acc)
}

fn collect_wire_ids(constraint: &CbirConstraint) -> Vec<u32> {
    let mut ids = Vec::new();
    for term in &constraint.a.terms {
        ids.push(term.wire_id);
    }
    for term in &constraint.b.terms {
        ids.push(term.wire_id);
    }
    for term in &constraint.c.terms {
        ids.push(term.wire_id);
    }
    ids
}

fn parse_field_modulus(hex: &str) -> Result<BigUint> {
    let trimmed = hex.trim_start_matches("0x");
    BigUint::from_str_radix(trimmed, 16).map_err(|_| anyhow!("invalid field modulus hex: {}", hex))
}

fn parse_biguint(value: &str) -> BigUint {
    let raw = value.trim();
    if let Some(stripped) = raw.strip_prefix("0x") {
        BigUint::from_str_radix(stripped, 16).unwrap_or_else(|_| BigUint::zero())
    } else {
        BigUint::from_str_radix(raw, 10).unwrap_or_else(|_| BigUint::zero())
    }
}

fn biguint_to_hex(value: &BigUint) -> String {
    if value.is_zero() {
        "0x0".to_string()
    } else {
        format!("0x{value:x}")
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::from("0x");
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn hex_decode(hex_str: &str) -> Vec<u8> {
    let stripped = hex_str.trim_start_matches("0x");
    if !stripped.len().is_multiple_of(2) {
        return Vec::new();
    }
    let bytes: Vec<u8> = (0..stripped.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&stripped[i..i + 2], 16).ok())
        .collect();
    bytes
}

/// Binary Merkle tree with leaf = SHA-256(wire_id || value_hex)
struct MerkleTree {
    levels: Vec<Vec<Vec<u8>>>,
}

impl MerkleTree {
    fn new(wire_values: &[(u32, String)]) -> Self {
        let leaves: Vec<Vec<u8>> = wire_values
            .iter()
            .map(|(id, val)| leaf_hash(*id, val))
            .collect();

        let mut levels = vec![leaves];
        let mut current = levels[0].clone();

        while current.len() > 1 {
            if current.len() % 2 == 1 {
                current.push(current.last().unwrap().clone());
            }
            let mut next = Vec::with_capacity(current.len() / 2);
            for chunk in current.chunks(2) {
                next.push(node_hash(&chunk[0], &chunk[1]));
            }
            levels.push(next.clone());
            current = next;
        }

        Self { levels }
    }

    fn root(&self) -> Vec<u8> {
        self.levels
            .last()
            .and_then(|l| l.first())
            .cloned()
            .unwrap_or_else(|| Sha256::digest(b"empty").to_vec())
    }

    fn proof(&self, wire_id: u32, wire_values: &[(u32, String)]) -> Option<MerkleProof> {
        let index = wire_values.iter().position(|(id, _)| *id == wire_id)?;
        let mut siblings = Vec::new();
        let mut idx = index;
        for level in &self.levels[..self.levels.len().saturating_sub(1)] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            if sibling_idx < level.len() {
                siblings.push(level[sibling_idx].clone());
            }
            idx /= 2;
        }
        Some(MerkleProof { siblings })
    }
}

struct MerkleProof {
    siblings: Vec<Vec<u8>>,
}

fn leaf_hash(wire_id: u32, value_hex: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(wire_id.to_le_bytes());
    hasher.update(value_hex.as_bytes());
    hasher.finalize().to_vec()
}

fn node_hash(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().to_vec()
}

fn verify_merkle_proof(root_hex: &str, leaf: &[u8], siblings: &[Vec<u8>], index: usize) -> bool {
    let root_bytes = hex_decode(root_hex);
    let mut current = leaf.to_vec();
    let mut idx = index;
    for sibling in siblings {
        current = if idx.is_multiple_of(2) {
            node_hash(&current, sibling)
        } else {
            node_hash(sibling, &current)
        };
        idx /= 2;
    }
    current == root_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirbinius_cbir::{
        CbirConstraint, CbirDocument, CbirLinearCombination, CbirMetadata, CbirSignal, CbirTerm,
    };
    use cirbinius_types::Backend;

    fn make_test_cbir() -> CbirDocument {
        CbirDocument {
            metadata: CbirMetadata {
                schema_version: "cbir/v1".to_string(),
                toolchain_version: "0.1.0".to_string(),
                content_hash: "sha256:test".to_string(),
            },
            backend: Backend::Binius64,
            field_modulus_hex: "0x07".to_string(),
            wire_count: 4,
            public_output_count: 0,
            public_input_count: 1,
            private_input_count: 2,
            constraints: vec![CbirConstraint {
                id: 1,
                kind: "mul".to_string(),
                a: CbirLinearCombination {
                    terms: vec![CbirTerm {
                        wire_id: 1,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                b: CbirLinearCombination {
                    terms: vec![CbirTerm {
                        wire_id: 2,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                c: CbirLinearCombination {
                    terms: vec![CbirTerm {
                        wire_id: 3,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                signal_hints: vec![
                    "main.a".to_string(),
                    "main.b".to_string(),
                    "main.c".to_string(),
                ],
            }],
            signals: vec![
                CbirSignal {
                    wire_id: 0,
                    name: "main.pub".to_string(),
                },
                CbirSignal {
                    wire_id: 1,
                    name: "main.a".to_string(),
                },
            ],
        }
    }

    #[test]
    fn proves_and_verifies_simple_constraint() {
        let cbir = make_test_cbir();
        // Witness: wire0=1 (pub), wire1=2, wire2=3, wire3=6 (2*3=6)
        let witness = vec![
            "0x1".to_string(),
            "0x2".to_string(),
            "0x3".to_string(),
            "0x6".to_string(),
        ];

        let artifact = prove(&cbir, &witness, "0x07").expect("prove should succeed");
        assert!(artifact.validate_hash());
        assert_eq!(artifact.num_constraints, 1);
        assert!(artifact.constraint_proofs[0].resolved);
        assert!(artifact.public_inputs_hash.is_some());

        // Verify with correct public inputs
        let public_inputs = vec!["0x1".to_string()];
        let result =
            verify(&artifact, &cbir, &public_inputs, "0x07").expect("verify should succeed");
        assert!(result, "proof should verify with correct public inputs");
    }

    #[test]
    fn rejects_wrong_public_inputs() {
        let cbir = make_test_cbir();
        let witness = vec![
            "0x1".to_string(),
            "0x2".to_string(),
            "0x3".to_string(),
            "0x6".to_string(),
        ];
        let artifact = prove(&cbir, &witness, "0x07").expect("prove should succeed");

        // Verify with wrong public inputs
        let public_inputs = vec!["0x2".to_string()];
        let result =
            verify(&artifact, &cbir, &public_inputs, "0x07").expect("verify should succeed");
        assert!(!result, "proof should reject wrong public inputs");
    }

    #[test]
    fn rejects_mismatched_constraint() {
        let cbir = make_test_cbir();
        // Bad witness: 2*3 != 7
        let witness = vec![
            "0x1".to_string(),
            "0x2".to_string(),
            "0x3".to_string(),
            "0x7".to_string(),
        ];
        let artifact = prove(&cbir, &witness, "0x07").expect("prove should succeed");
        // The constraint should not be resolved (2*3=6 mod 7, not 7 mod 7 = 0)
        assert!(!artifact.constraint_proofs[0].resolved);
    }

    #[test]
    fn merkle_tree_integrity() {
        let pairs = vec![
            (0u32, "0x1".to_string()),
            (1u32, "0x2".to_string()),
            (2u32, "0x3".to_string()),
        ];
        let tree = MerkleTree::new(&pairs);
        let root = tree.root();
        assert!(!root.is_empty());

        let mp = tree.proof(1, &pairs).expect("proof for wire 1");
        let leaf = leaf_hash(1, "0x2");
        assert!(verify_merkle_proof(
            &hex_encode(&root),
            &leaf,
            &mp.siblings,
            1
        ));
    }
}
