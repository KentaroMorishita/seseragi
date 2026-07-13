use crate::{ManifestError, ModuleGraphError, PackageIdentity, PackageName};
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LocalPackageGraphError {
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    NotDirectory(PathBuf),
    Manifest {
        path: PathBuf,
        error: ManifestError,
    },
    UnsupportedLanguageVersion {
        package: PackageName,
        requirement: String,
        implemented: String,
    },
    RegistryDependencyUnsupported {
        package: Box<PackageIdentity>,
        key: String,
    },
    DependencyNameMismatch {
        package: Box<PackageIdentity>,
        key: String,
        expected: PackageName,
        actual: PackageName,
        dependency_root: PathBuf,
    },
    DependencyConfusion {
        first: Box<PackageIdentity>,
        second: Box<PackageIdentity>,
    },
    Graph(Box<ModuleGraphError<PackageIdentity>>),
}

impl LocalPackageGraphError {
    pub(super) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub const fn code(&self) -> &'static str {
        match self {
            Self::Manifest { .. } | Self::UnsupportedLanguageVersion { .. } => "SES-K0101",
            Self::RegistryDependencyUnsupported { .. } => "SES-K0102",
            Self::DependencyNameMismatch { .. } | Self::DependencyConfusion { .. } => "SES-K0104",
            _ => "SES-K0001",
        }
    }
}

impl fmt::Display for LocalPackageGraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { action, path, source } => {
                write!(formatter, "failed to {action} `{}`: {source}", path.display())
            }
            Self::NotDirectory(path) => {
                write!(formatter, "package root `{}` is not a directory", path.display())
            }
            Self::Manifest { path, error } => {
                write!(formatter, "invalid manifest `{}`: {error}", path.display())
            }
            Self::UnsupportedLanguageVersion {
                package,
                requirement,
                implemented,
            } => write!(
                formatter,
                "package `{}` requires Seseragi `{requirement}`, but this compiler implements `{implemented}`",
                package.as_str()
            ),
            Self::RegistryDependencyUnsupported { package, key } => write!(
                formatter,
                "local package `{}` dependency `{key}` requires registry resolution",
                package.name().as_str()
            ),
            Self::DependencyNameMismatch {
                package,
                key,
                expected,
                actual,
                dependency_root,
            } => write!(
                formatter,
                "package `{}` dependency `{key}` expects `{}`, but `{}` declares `{}`",
                package.name().as_str(),
                expected.as_str(),
                dependency_root.display(),
                actual.as_str()
            ),
            Self::DependencyConfusion { first, second } => write!(
                formatter,
                "package `{}` version `{}` resolves from both `{}` and `{}`",
                first.name().as_str(),
                first.version(),
                first.source(),
                second.source()
            ),
            Self::Graph(error) => write!(formatter, "invalid package graph: {error:?}"),
        }
    }
}

impl std::error::Error for LocalPackageGraphError {}
