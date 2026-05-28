use std::collections::HashSet;

use cirbinius_cbir::CbirConstraint;

use super::{
    AndDetector, BitDecompositionDetector, Bits2NumDetector, BooleanConstraintDetector,
    CircomlibGadgetDetector, Confidence, GreaterThanDetector, HashPreimageDetector,
    IsEqualDetector, IsZeroDetector, LessThanDetector, MerklePathDetector, MiMCDetector,
    MuxSelectorDetector, NotDetector, OrDetector, PatternDetector, PoseidonDetector,
    RangeCheckDetector, SelectorDetector, ShaDetector, XorDetector,
};

pub struct DetectorRegistry {
    detectors: Vec<Box<dyn PatternDetector>>,
    min_confidence: Confidence,
    disabled: HashSet<String>,
}

impl DetectorRegistry {
    pub fn new(min_confidence: Confidence) -> Self {
        Self {
            detectors: Vec::new(),
            min_confidence,
            disabled: HashSet::new(),
        }
    }

    pub fn register(&mut self, detector: Box<dyn PatternDetector>) {
        self.detectors.push(detector);
    }

    pub fn disable(&mut self, name: &str) {
        self.disabled.insert(name.to_string());
    }

    fn is_enabled(&self, name: &str) -> bool {
        !self.disabled.contains(name)
    }

    pub fn detect_all(&self, constraint: &CbirConstraint) -> Vec<(String, Confidence)> {
        let mut results: Vec<(String, Confidence)> = self
            .detectors
            .iter()
            .filter(|d| self.is_enabled(d.name()))
            .filter_map(|d| {
                d.detect(constraint)
                    .filter(|c| c.meets_threshold(self.min_confidence))
                    .map(|c| (d.name().to_string(), c))
            })
            .collect();
        results.sort_by(|a, b| a.1.cmp(&b.1));
        results
    }

    pub fn detect_groups(&self, constraints: &[CbirConstraint]) -> Vec<super::DetectedPattern> {
        let mut groups = Vec::new();
        for detector in &self.detectors {
            if !self.is_enabled(detector.name()) {
                continue;
            }
            let matches = detector.detect_group(constraints);
            for ids in matches {
                let confidence = detector.default_confidence();
                if confidence.meets_threshold(self.min_confidence) {
                    groups.push(super::DetectedPattern {
                        pattern_name: detector.name().to_string(),
                        confidence,
                        constraint_ids: ids,
                    });
                }
            }
        }
        groups
    }
}

pub fn default_registry(min_confidence: Confidence) -> DetectorRegistry {
    let mut reg = DetectorRegistry::new(min_confidence);
    reg.register(Box::new(SelectorDetector));
    reg.register(Box::new(BooleanConstraintDetector));
    reg.register(Box::new(NotDetector));
    reg.register(Box::new(AndDetector));
    reg.register(Box::new(OrDetector));
    reg.register(Box::new(XorDetector));
    reg.register(Box::new(IsZeroDetector));
    reg.register(Box::new(IsEqualDetector));
    reg.register(Box::new(BitDecompositionDetector));
    reg.register(Box::new(Bits2NumDetector));
    reg.register(Box::new(RangeCheckDetector));
    reg.register(Box::new(LessThanDetector));
    reg.register(Box::new(GreaterThanDetector));
    reg.register(Box::new(MuxSelectorDetector));
    reg.register(Box::new(MerklePathDetector));
    reg.register(Box::new(HashPreimageDetector));
    reg.register(Box::new(PoseidonDetector));
    reg.register(Box::new(ShaDetector));
    reg.register(Box::new(MiMCDetector));
    reg.register(Box::new(CircomlibGadgetDetector));
    reg
}
