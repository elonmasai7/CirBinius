# Lowering Rules

This document tracks lowering rules from Circom/R1CS patterns to CBIR and then to Binius64 components.

Rule template:

1. Source R1CS pattern
2. CBIR representation
3. Binius64 lowering
4. Correctness argument
5. Test coverage link

## Rule: Boolean Constraint

- Source R1CS pattern: `x * (x - 1) = 0`
- CBIR representation: `kind = mul`, `a = [x]`, `b = [x, -1]`, `c = []`
- Binius64 lowering: `boolean` gate
- Correctness argument: over any field, equation enforces `x ∈ {0,1}`
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_boolean_and_bit_candidates`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: Bit Decomposition Candidate

- Source R1CS pattern: signal naming/path indicates bit lanes (`bit`, `bits`, `num2bits`)
- CBIR representation: constraints with `signal_hints` containing bit tokens
- Binius64 lowering: candidate retained for `boolean`/bitwise lanes
- Correctness argument: no semantic rewrite yet; only candidate tagging
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_boolean_and_bit_candidates`)

## Rule: Range Check Candidate

- Source R1CS pattern: range/comparison gadget markers (`range`, `lt`, `leq`, `cmp`)
- CBIR representation: constraints with corresponding `signal_hints`
- Binius64 lowering: `range_check` gate classification
- Correctness argument: current phase is classification only; no arithmetic semantics changed
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_range_rule`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: XOR Chain Candidate

- Source R1CS pattern: XOR gadget/chains identified via signal naming
- CBIR representation: constraints with `signal_hints` containing `xor`
- Binius64 lowering: `xor` gate classification
- Correctness argument: classification pass preserves constraints unchanged
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_xor_rule`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: AND Chain Candidate

- Source R1CS pattern: `a * b = c` (structural) and/or AND signal markers
- CBIR representation: `kind = mul` with single-unit terms for `a`, `b`, and `c`
- Binius64 lowering: `and` gate classification
- Correctness argument: structural match is exact multiplication relation used by AND gadgets in boolean contexts
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_and_rule_from_structure`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: Mux/Selector Candidate

- Source R1CS pattern: selector/mux naming (`mux`, `selector`, `select`, `ternary`)
- CBIR representation: constraints with matching `signal_hints`
- Binius64 lowering: `mux_selector` gate classification
- Correctness argument: tagging only; constraints remain unchanged
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_mux_rule`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: Merkle Path Motif

- Source R1CS pattern: Merkle path markers (`merkle`, `path`, `sibling`, `root`)
- CBIR representation: constraints with matching `signal_hints`
- Binius64 lowering: `merkle_path` gate classification
- Correctness argument: motif tagging only in this phase
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_merkle_rule`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Rule: Hash Motif

- Source R1CS pattern: hash markers (`hash`, `poseidon`, `sha`, `keccak`, `mimc`, `pedersen`)
- CBIR representation: constraints with matching `signal_hints`
- Binius64 lowering: `hash` gate classification
- Correctness argument: motif tagging only in this phase
- Test coverage link: `crates/cirbinius-optimizer/src/lib.rs` (`analyze_detects_hash_rule`), `crates/cirbinius-binius64/src/lib.rs` (`lowering_classifies_extended_gate_families`)

## Current guarantee

- Phase 4 detection and lowering are semantics-preserving classifications.
- Unsupported or unrecognized motifs fall back to generic multiplication handling.

## Machine-readable index

- `cirbinius analyze` now emits a machine-readable lowering rules index artifact at `lowering_rules_index.json` next to the analysis report.
- Schema version: `lowering-rules-index/v1`.
- The index includes per-rule matched constraint IDs so auditors can diff motif recognition behavior across commits.
