use crate::ModulePath;
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PackageIdentity(String);

impl PackageIdentity {
    /// Stores an identity already resolved from manifest, lockfile, and source.
    /// Its internal grammar remains owned by the package resolver.
    pub fn from_canonical(value: impl Into<String>) -> Result<Self, PackageIdentityError> {
        let value = value.into();
        if value.is_empty() {
            return Err(PackageIdentityError::Empty);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModuleRoot {
    Source,
    Test,
    Benchmark,
    Generated,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleIdentity {
    package: PackageIdentity,
    root: ModuleRoot,
    path: ModulePath,
}

impl ModuleIdentity {
    pub const fn new(package: PackageIdentity, root: ModuleRoot, path: ModulePath) -> Self {
        Self {
            package,
            root,
            path,
        }
    }

    pub const fn package(&self) -> &PackageIdentity {
        &self.package
    }

    pub const fn root(&self) -> ModuleRoot {
        self.root
    }

    pub const fn path(&self) -> &ModulePath {
        &self.path
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageIdentityError {
    Empty,
}

impl fmt::Display for PackageIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("package identity must not be empty"),
        }
    }
}

impl std::error::Error for PackageIdentityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_root_kind_in_structural_module_identity() {
        let package = PackageIdentity::from_canonical("locked-package-identity").unwrap();
        let path = ModulePath::parse("game/main").unwrap();
        let source = ModuleIdentity::new(package.clone(), ModuleRoot::Source, path.clone());
        let test = ModuleIdentity::new(package, ModuleRoot::Test, path);

        assert_ne!(source, test);
        assert_eq!(source.path().as_str(), "game/main");
        assert_eq!(source.root(), ModuleRoot::Source);
    }

    #[test]
    fn rejects_only_an_absent_canonical_package_identity_at_this_boundary() {
        assert_eq!(
            PackageIdentity::from_canonical("").unwrap_err(),
            PackageIdentityError::Empty
        );
    }
}
