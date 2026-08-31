mod digest;
mod error;
mod model;
mod parse;
mod validate;
mod write;

#[cfg(test)]
mod tests;

pub use error::LockError;
pub use model::{
    LockDependency, LockForeignModule, LockHostPackage, LockPackage, LockProviderSelection,
    LockSourceKind, Lockfile,
};
pub use parse::parse_lockfile;
pub use validate::{
    generate_lockfile, read_and_validate_development_lockfile, read_and_validate_lockfile,
};
pub use write::write_lockfile;

pub(crate) const LOCKFILE_NAME: &str = "seseragi.lock";
pub(crate) const LOCK_SCHEMA: u64 = 1;
pub(crate) const STANDARD_LIBRARY_VERSION: &str = crate::IMPLEMENTED_LANGUAGE_VERSION;
pub const UNICODE_VERSION: &str = "16.0.0";
pub(crate) const TIMEZONE_DATABASE_VERSION: &str = "2025b";
