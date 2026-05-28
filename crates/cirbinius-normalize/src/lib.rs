use cirbinius_r1cs::{ParsedR1cs, R1csConstraint, R1csLinearCombination, R1csTerm};
use cirbinius_symbols::SymbolTable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedTerm {
    pub wire_id: u32,
    pub coeff_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedLinearCombination {
    pub terms: Vec<NormalizedTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedConstraint {
    pub id: u64,
    pub a: NormalizedLinearCombination,
    pub b: NormalizedLinearCombination,
    pub c: NormalizedLinearCombination,
    pub signal_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedSignal {
    pub wire_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedCircuit {
    pub field_modulus_hex: String,
    pub wire_count: u32,
    pub public_output_count: u32,
    pub public_input_count: u32,
    pub private_input_count: u32,
    pub constraints: Vec<NormalizedConstraint>,
    pub signals: Vec<NormalizedSignal>,
}

pub fn normalize(parsed: &ParsedR1cs, symbols: &SymbolTable) -> NormalizedCircuit {
    let constraints = parsed
        .constraints
        .iter()
        .enumerate()
        .map(|(idx, constraint)| normalize_constraint(idx as u64 + 1, constraint, symbols))
        .collect::<Vec<_>>();

    let signals = symbols
        .by_wire
        .iter()
        .map(|(wire_id, name)| NormalizedSignal {
            wire_id: *wire_id,
            name: name.clone(),
        })
        .collect::<Vec<_>>();

    NormalizedCircuit {
        field_modulus_hex: parsed.header.field_modulus_hex.clone(),
        wire_count: parsed.header.wire_count,
        public_output_count: parsed.header.public_output_count,
        public_input_count: parsed.header.public_input_count,
        private_input_count: parsed.header.private_input_count,
        constraints,
        signals,
    }
}

fn normalize_constraint(
    id: u64,
    constraint: &R1csConstraint,
    symbols: &SymbolTable,
) -> NormalizedConstraint {
    let a = normalize_linear_combination(&constraint.a);
    let b = normalize_linear_combination(&constraint.b);
    let c = normalize_linear_combination(&constraint.c);

    let mut signal_hints = collect_signal_hints(&a.terms, symbols);
    signal_hints.extend(collect_signal_hints(&b.terms, symbols));
    signal_hints.extend(collect_signal_hints(&c.terms, symbols));
    signal_hints.sort();
    signal_hints.dedup();

    NormalizedConstraint {
        id,
        a,
        b,
        c,
        signal_hints,
    }
}

fn normalize_linear_combination(linear: &R1csLinearCombination) -> NormalizedLinearCombination {
    let mut terms = linear
        .terms
        .iter()
        .filter(|term| !is_zero_coeff(term))
        .map(|term| NormalizedTerm {
            wire_id: term.wire_id,
            coeff_hex: canonical_hex(&term.coeff_hex),
        })
        .collect::<Vec<_>>();
    terms.sort_by_key(|term| term.wire_id);

    NormalizedLinearCombination { terms }
}

fn collect_signal_hints(terms: &[NormalizedTerm], symbols: &SymbolTable) -> Vec<String> {
    terms
        .iter()
        .filter_map(|term| symbols.get(term.wire_id).map(str::to_string))
        .collect()
}

fn is_zero_coeff(term: &R1csTerm) -> bool {
    term.coeff_hex
        .trim_start_matches("0x")
        .chars()
        .all(|ch| ch == '0')
}

fn canonical_hex(value: &str) -> String {
    let without_prefix = value.trim_start_matches("0x");
    let trimmed = without_prefix.trim_start_matches('0');
    if trimmed.is_empty() {
        "0x0".to_string()
    } else if trimmed.len() % 2 == 1 {
        format!("0x0{trimmed}")
    } else {
        format!("0x{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::canonical_hex;

    #[test]
    fn canonicalizes_hex_values() {
        assert_eq!(canonical_hex("0x0001"), "0x01");
        assert_eq!(canonical_hex("0x0"), "0x0");
        assert_eq!(canonical_hex("0xabc"), "0x0abc");
    }
}
