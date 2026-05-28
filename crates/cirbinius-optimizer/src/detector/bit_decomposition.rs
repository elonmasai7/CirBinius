use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token, is_zero_linear};

pub struct BitDecompositionDetector;

impl PatternDetector for BitDecompositionDetector {
    fn name(&self) -> &'static str {
        "bit-decomposition"
    }
    fn description(&self) -> &'static str {
        "Detects Num2Bits pattern: boolean bits + summation"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["num2bits", "bits", "bit"])
            && (constraint.kind == "add"
                || (constraint.kind == "mul" && is_zero_linear(&constraint.c)))
        {
            Some(Confidence::Heuristic)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // Num2Bits(n): n boolean constraints + 1 summation constraint
        // Find a contiguous group where n constraints are boolean and 1 is summation
        let bool_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| {
                c.kind == "mul"
                    && c.a.terms.len() == 1
                    && c.b.terms.len() == 2
                    && is_zero_linear(&c.c)
                    && (has_hint_token(c, &["bits", "num2bits", "bit"]))
            })
            .map(|c| c.id)
            .collect();

        if bool_ids.len() < 2 {
            return vec![];
        }

        // Find the summation constraint: sum(bits[i] * 2^i) == in
        let sum_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| {
                c.kind == "add"
                    && c.a.terms.len() > 1
                    && c.b.terms.is_empty()
                    && c.c.terms.len() == 1
                    && has_hint_token(c, &["num2bits", "sum", "bits"])
            })
            .map(|c| c.id)
            .collect();

        if sum_ids.is_empty() {
            return vec![];
        }

        let mut all_ids = bool_ids.clone();
        all_ids.extend(&sum_ids);
        all_ids.sort();
        all_ids.dedup();
        vec![all_ids]
    }
}
