//! Host execution boundary for compiled Seseragi programs.
//!
//! This crate does not parse, resolve, type-check, or lower source. It accepts
//! one [`seseragi_driver::CompiledModule`], validates the public `main`
//! contract, stages the versioned TypeScript runtime, and invokes the target
//! adapter. CLI, conformance, and future interactive surfaces can therefore
//! share compiler output without reimplementing language semantics.

mod contract;
mod package;
mod process;

pub use package::stage_typescript_package;
pub use process::{run_main, RunError, RunOutcome};
