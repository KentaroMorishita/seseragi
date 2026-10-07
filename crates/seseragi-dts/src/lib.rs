//! TypeScript declaration conversion shared by native and browser adapters.
//!
//! Browser consumers disable default features and call [`convert_entry`] with
//! declaration/settings text and explicit [`HostMetadata`]. The default
//! `filesystem` feature additionally provides package discovery, validation and
//! artifact publication for the CLI. `bun run check:dts:wasm` verifies the parser
//! and converter together in an import-free wasm32-unknown-unknown module.

mod config;
mod convert;
mod model;
mod parser;

pub use config::{BindingsConfig, EntryConfig};
#[cfg(feature = "filesystem")]
pub use convert::filesystem::{
    convert_package, validate_generated_bindings, ConversionOutcome, ConvertRequest,
    ConvertedEntry, ValidationError,
};
pub use convert::{
    convert_entry, ConversionDiagnostic, ConversionInput, ConversionResult, ConvertError,
    DiagnosticSeverity, GeneratedBinding, HostMetadata,
};
