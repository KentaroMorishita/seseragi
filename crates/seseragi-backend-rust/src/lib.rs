//! Experimental Rust target. Checked CoreIr is the semantic authority;
//! rustc validates generated implementation code, not Seseragi source types.
mod emit;
use serde::{Deserialize, Serialize};
use seseragi_lowering::{CoreModule, SourceSpan};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustBackendError {
    pub code: String,
    pub message: String,
    pub origin: Option<SourceSpan>,
}
impl From<String> for RustBackendError {
    fn from(message: String) -> Self {
        Self {
            code: "SES-RUST-0001".into(),
            message,
            origin: None,
        }
    }
}
impl From<&str> for RustBackendError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl std::fmt::Display for RustBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for RustBackendError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustMetadata {
    pub schema: u32,
    pub backend: String,
    pub stability: String,
    pub abi: String,
    pub module: String,
    pub entry: String,
    pub edition: String,
    pub direct_calls: usize,
    pub self_tail_calls: usize,
    pub closed_dictionary_sites: usize,
    pub entry_kind: String,
    pub capabilities: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustArtifact {
    pub source: String,
    pub metadata: RustMetadata,
}

/// Emits a self-contained executable source file. The explicit entry must be
/// an unconstrained Unit function. Pure entries print an inspection result;
/// Effect entries execute the cold synchronous plan with native resource scopes.
pub fn emit_rust_module(
    module: &CoreModule,
    entry: &str,
) -> Result<RustArtifact, RustBackendError> {
    emit::emit(module, entry)
}

#[cfg(test)]
#[allow(dead_code)]
mod runtime {
    include!("runtime.rs");
    include!("runtime_char.rs");
    include!("runtime_trace.rs");
    include!("runtime_effect.rs");
}
