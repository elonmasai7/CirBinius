use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct Bits2NumDetector;

impl PatternDetector for Bits2NumDetector {
    fn name(&self) -> &'static str {
        "bits2num"
    }
    fn description(&self) -> &'static str {
        "Detects Bits2Num pattern: boolean inputs + weighted sum"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["bits2num", "b2n"]) {
            return Some(Confidence::Strong);
        }
        if constraint.kind == "add"
            && constraint.a.terms.len() > 1
            && constraint.b.terms.is_empty()
            && has_hint_token(constraint, &["sum", "reconstruct"])
        {
            return Some(Confidence::Heuristic);
        }
        None
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // Reverse of bit decomposition: n boolean inputs + weighted sum to output
        let sum_constraints: Vec<&CbirConstraint> = constraints
            .iter()
            .filter(|c| c.kind == "add" && c.a.terms.len() > 1 && c.c.terms.len() == 1)
            .collect();

        if sum_constraints.is_empty() {
            return vec![];
        }

        sum_constraints
            .into_iter()
            .map(|c| {
                let mut ids = vec![c.id];
                // Include boolean input constraints
                for term in &c.a.terms {
                    for other in constraints {
                        if other.id != c.id
                            && other.kind == "mul"
                            && other.a.terms.len() == 1
                            && other.a.terms[0].wire_id == term.wire_id
                        {
                            ids.push(other.id);
                        }
                    }
                }
                ids.sort();
                ids.dedup();
                ids
            })
            .collect()
    }
}
