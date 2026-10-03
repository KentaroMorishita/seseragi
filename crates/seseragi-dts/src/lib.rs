mod config;
mod convert;
#[cfg(feature = "filesystem")]
mod filesystem;
mod host;
mod model;
mod parser;

pub use config::{BindingsConfig, EntryConfig};
pub use convert::{
    ConversionDiagnostic, ConversionResult, ConvertError, Converter, DeclarationInput,
    DiagnosticSeverity, GeneratedBinding, HostModuleIdentity,
};
#[cfg(feature = "filesystem")]
pub use filesystem::{
    convert_package, validate_generated_bindings, ConversionOutcome, ConvertRequest,
    ValidationError,
};
pub use host::resolve_host_module_identity;
