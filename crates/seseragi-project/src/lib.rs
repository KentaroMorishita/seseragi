//! Package and module identity owned by the project layer.
//!
//! Compiler stages consume identities produced here; they do not infer them
//! from diagnostic source labels or process working directories.

mod identity;
mod link;
mod module_path;
mod package_name;
mod specifier;

pub use identity::{
    ModuleIdentity, ModuleRoot, PackageIdentity, PackageSourceIdentity, SourceIdentityError,
};
pub use link::{link_module, LinkError, LinkedDependency, LinkedImport, LinkedModule};
pub use module_path::{ModulePath, ModulePathError};
pub use package_name::{PackageName, PackageNameError};
pub use specifier::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, RelativeSpecifierError,
    SpecifierError,
};
