use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct MiMCDetector;

impl PatternDetector for MiMCDetector {
    fn name(&self) -> &'static str {
        "mimc"
    }
    fn description(&self) -> &'static str {
        "Detects MiMC Feistel round constraints"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["mimc"]) {
            if has_hint_token(constraint, &["round", "feistel", "cipher"]) {
                Some(Confidence::Strong)
            } else {
                Some(Confidence::Heuristic)
            }
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        let mimc_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["mimc"]))
            .map(|c| c.id)
            .collect();
        if mimc_ids.len() >= 2 {
            vec![mimc_ids]
        } else {
            vec![]
        }
    }
}
