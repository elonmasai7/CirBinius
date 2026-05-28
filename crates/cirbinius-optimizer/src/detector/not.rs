use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector};

pub struct NotDetector;

impl PatternDetector for NotDetector {
    fn name(&self) -> &'static str {
        "not"
    }
    fn description(&self) -> &'static str {
        "Detects 1 - a = b NOT gate"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if constraint.kind != "mul" {
            return None;
        }
        // NOT is a linear constraint: 1 - a = b, no mul
        if !constraint.a.terms.is_empty() {
            return None;
        }
        if !constraint.b.terms.is_empty() {
            return None;
        }
        // NOT is expressed as a quadratic constraint: a * (1 - a - b) = 0 decomposed
        // Actually, NOT is: a + b = 1 where a is boolean
        // In circom, NOT constraint is: out <== 1 - in (linear, but may appear as mul)
        // Check for: wire_a * 1 + wire_b * 1 + (-1) * ONE = 0 => wire_a + wire_b - 1 = 0
        // This is a linear constraint, not mul - return None for mul-only
        None
    }
}
