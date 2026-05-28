use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token, is_one};

pub struct MuxSelectorDetector;

impl PatternDetector for MuxSelectorDetector {
    fn name(&self) -> &'static str {
        "mux-selector"
    }
    fn description(&self) -> &'static str {
        "Detects MUX / selector patterns"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        // Mux2 pattern: out = sel*a + (1-sel)*b
        // => out = b + sel*(a - b)
        // => sel*(a - b) + b - out = 0
        // Look for mul with: sel*(a-b) term
        if has_hint_token(constraint, &["mux", "selector", "select"]) {
            return Some(Confidence::Strong);
        }
        // Mux2 sel boolean check: sel * (sel - 1) == 0
        if constraint.kind == "mul"
            && constraint.a.terms.len() == 1
            && constraint.b.terms.len() == 2
        {
            let wire = constraint.a.terms[0].wire_id;
            if wire > 0 && is_one(&constraint.a.terms[0].coeff_hex) {
                let has_sel = constraint
                    .b
                    .terms
                    .iter()
                    .any(|t| t.wire_id == wire && is_one(&t.coeff_hex));
                let has_neg = constraint.b.terms.iter().any(|t| t.wire_id == 0);
                if has_sel && has_neg {
                    return Some(Confidence::Heuristic);
                }
            }
        }
        None
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // Mux4 = tree of 3 Mux2s: collect related boolean + mux constraints
        let mux_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["mux", "select"]))
            .map(|c| c.id)
            .collect();
        if !mux_ids.is_empty() {
            vec![mux_ids]
        } else {
            vec![]
        }
    }
}
