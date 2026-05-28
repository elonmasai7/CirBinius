use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    Binius64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CompileMode {
    Compatibility,
    OptimizedBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptimizerConfig {
    pub mode: CompileMode,
    pub min_confidence: String,
    pub disabled_passes: Vec<String>,
    pub allow_heuristic: bool,
    pub allow_experimental: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            mode: CompileMode::Compatibility,
            min_confidence: "Strong".to_string(),
            disabled_passes: Vec::new(),
            allow_heuristic: false,
            allow_experimental: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompilerOptions {
    pub backend: Backend,
    pub mode: CompileMode,
    pub optimize: bool,
    pub optimizer: OptimizerConfig,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            backend: Backend::Binius64,
            mode: CompileMode::Compatibility,
            optimize: true,
            optimizer: OptimizerConfig::default(),
        }
    }
}
