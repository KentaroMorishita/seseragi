//! Package and module identity owned by the project layer.
//!
//! Compiler stages consume identities produced here; they do not infer them
//! from diagnostic source labels or process working directories.

mod identity;
mod module_path;
mod specifier;

pub use identity::{ModuleIdentity, ModuleRoot, PackageIdentity, PackageIdentityError};
pub use module_path::{ModulePath, ModulePathError};
pub use specifier::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, RelativeSpecifierError,
    SpecifierError,
};
