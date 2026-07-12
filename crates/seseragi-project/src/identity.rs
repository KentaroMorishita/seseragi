use crate::{ModulePath, PackageName};
use semver::Version;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PackageIdentity {
    name: PackageName,
    version: Version,
    source: PackageSourceIdentity,
}

impl PackageIdentity {
    pub const fn new(name: PackageName, version: Version, source: PackageSourceIdentity) -> Self {
        Self {
            name,
            version,
            source,
        }
    }

    pub const fn name(&self) -> &PackageName {
        &self.name
    }

    pub const fn version(&self) -> &Version {
        &self.version
    }

    pub const fn source(&self) -> &PackageSourceIdentity {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PackageSourceIdentity {
    RegistryArtifact { content_digest: String },
    Path { canonical_path: PathBuf },
}

impl PackageSourceIdentity {
    pub fn registry(content_digest: impl Into<String>) -> Result<Self, SourceIdentityError> {
        let content_digest = content_digest.into();
        if content_digest.is_empty() {
            return Err(SourceIdentityError::EmptyRegistryDigest);
        }
        Ok(Self::RegistryArtifact { content_digest })
    }

    pub fn path(canonical_path: impl Into<PathBuf>) -> Result<Self, SourceIdentityError> {
        let canonical_path = canonical_path.into();
        if !canonical_path.is_absolute() {
            return Err(SourceIdentityError::PathNotAbsolute);
        }
        Ok(Self::Path { canonical_path })
    }

    pub fn canonical_path(&self) -> Option<&Path> {
        match self {
            Self::Path { canonical_path } => Some(canonical_path),
            Self::RegistryArtifact { .. } => None,
        }
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
pub enum SourceIdentityError {
    EmptyRegistryDigest,
    PathNotAbsolute,
}

impl fmt::Display for SourceIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRegistryDigest => {
                formatter.write_str("registry source identity requires a content digest")
            }
            Self::PathNotAbsolute => {
                formatter.write_str("path source identity must be canonical and absolute")
            }
        }
    }
}

impl std::error::Error for SourceIdentityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_root_kind_in_structural_module_identity() {
        let package = PackageIdentity::new(
            PackageName::parse("acme/game").unwrap(),
            Version::parse("1.2.3").unwrap(),
            PackageSourceIdentity::registry("sha256:fixture").unwrap(),
        );
        let path = ModulePath::parse("game/main").unwrap();
        let source = ModuleIdentity::new(package.clone(), ModuleRoot::Source, path.clone());
        let test = ModuleIdentity::new(package, ModuleRoot::Test, path);

        assert_ne!(source, test);
        assert_eq!(source.path().as_str(), "game/main");
        assert_eq!(source.root(), ModuleRoot::Source);
    }

    #[test]
    fn keeps_source_identity_structural_and_validated() {
        assert_eq!(
            PackageSourceIdentity::registry("").unwrap_err(),
            SourceIdentityError::EmptyRegistryDigest
        );
        assert_eq!(
            PackageSourceIdentity::path("relative/vendor").unwrap_err(),
            SourceIdentityError::PathNotAbsolute
        );
        assert!(PackageSourceIdentity::path(std::env::temp_dir())
            .unwrap()
            .canonical_path()
            .is_some());
    }
}
