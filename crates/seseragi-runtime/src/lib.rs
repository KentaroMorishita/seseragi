//! Host execution boundary for compiled Seseragi programs.
//!
//! This crate does not parse, resolve, type-check, or lower source. It accepts
//! one [`seseragi_driver::CompiledModule`], validates the public `main`
//! contract, stages the versioned TypeScript runtime, and invokes the target
//! adapter. CLI, conformance, and future interactive surfaces can therefore
//! share compiler output without reimplementing language semantics.

mod contract;
#[cfg(not(target_arch = "wasm32"))]
mod package;
#[cfg(not(target_arch = "wasm32"))]
mod process;

pub use contract::{main_contract, EnvironmentBinding, FailureRenderer, HostService, MainContract};
#[cfg(not(target_arch = "wasm32"))]
pub use package::stage_typescript_package;
#[cfg(not(target_arch = "wasm32"))]
pub use process::{run_local_package, run_local_project, run_main, RunError, RunOutcome};
