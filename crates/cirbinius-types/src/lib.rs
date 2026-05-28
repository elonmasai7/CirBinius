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
pub struct CompilerOptions {
    pub backend: Backend,
    pub mode: CompileMode,
    pub optimize: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            backend: Backend::Binius64,
            mode: CompileMode::Compatibility,
            optimize: true,
        }
    }
}
