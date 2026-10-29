//! Experimental Rust target. Checked CoreIr is the semantic authority;
//! rustc validates generated implementation code, not Seseragi source types.
mod clock_provider;
mod emit;
mod program_wasm;
mod program_wasm_effect;
mod project;
mod type_names;
pub use clock_provider::{
    NativeClockProvider, NativeProviderReceipt, NativeProviderSelectionOrigin, RustTargetOptions,
    NATIVE_CLOCK_SLEEP_PROVIDER,
};
pub use program_wasm::{
    emit_program_wasm, ProgramCodec, ProgramWasmArtifact, ProgramWasmExport, ProgramWasmMetadata,
    PROGRAM_WASM_ABI, PROGRAM_WASM_EFFECT_ABI, PROGRAM_WASM_HOST_EFFECT_ABI,
};
use serde::{Deserialize, Serialize};
use seseragi_lowering::{CoreModule, SourceSpan};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustBackendError {
    pub code: String,
    pub message: String,
    pub origin: Option<SourceSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
}
impl From<String> for RustBackendError {
    fn from(message: String) -> Self {
        Self {
            code: "SES-RUST-0001".into(),
            message,
            origin: None,
            module: None,
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
    /// Canonical source module identities in runtime initialization order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<String>,
    /// Explicit finite native target bindings; absent for existing artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<NativeProviderReceipt>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustArtifact {
    pub source: String,
    pub metadata: RustMetadata,
}

/// Emits a self-contained executable source file. The explicit entry must be
/// an unconstrained Unit function. Pure entries print an inspection result;
/// Effect entries execute cold plans with native resource scopes and cooperative Fibers.
pub fn emit_rust_module(
    module: &CoreModule,
    entry: &str,
) -> Result<RustArtifact, RustBackendError> {
    emit_rust_project(&[module], &module.module, entry)
}

/// Links checked source modules without changing canonical symbols or passing
/// through TypeScript. `modules` must contain the closed dependency graph.
pub fn emit_rust_project(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: &str,
) -> Result<RustArtifact, RustBackendError> {
    project::emit(modules, entry_module, entry)
}

/// Emits with explicit native target bindings. No binding is inferred from a
/// TypeScript provider selection; the default API continues to reject Clock.
pub fn emit_rust_module_with_options(
    module: &CoreModule,
    entry: &str,
    options: &RustTargetOptions,
) -> Result<RustArtifact, RustBackendError> {
    emit_rust_project_with_options(&[module], &module.module, entry, options)
}

/// Links a checked source graph with explicit native target bindings. Receipts
/// describe only the supported operations, not a general provider ABI.
pub fn emit_rust_project_with_options(
    modules: &[&CoreModule],
    entry_module: &str,
    entry: &str,
    options: &RustTargetOptions,
) -> Result<RustArtifact, RustBackendError> {
    project::emit_with_options(modules, entry_module, entry, options)
}

#[cfg(test)]
#[allow(dead_code)]
mod runtime {
    include!("runtime.rs");
    include!("runtime_char.rs");
    include!("runtime_float_display.rs");
    include!("runtime_float.rs");
    include!("runtime_math.rs");
    include!("runtime_list.rs");
    include!("runtime_tuple.rs");
    include!("runtime_array.rs");
    include!("runtime_array_operations.rs");
    include!("runtime_text.rs");
    include!("runtime_bytes.rs");
    include!("runtime_big_int.rs");
    include!("runtime_hashing.rs");
    include!("runtime_map.rs");
    include!("runtime_int_parse.rs");
    include!("runtime_float_parse.rs");
    include!("runtime_float_conversion.rs");
    include!("runtime_trace.rs");
    include!("runtime_effect.rs");
    include!("runtime_effect_defect.rs");
    include!("runtime_scheduler.rs");
    include!("runtime_path.rs");
    include!("runtime_filesystem.rs");
    include!("runtime_host.rs");
    include!("runtime_clock.rs");
}
