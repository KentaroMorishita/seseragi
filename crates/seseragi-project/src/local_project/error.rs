use crate::{
    LocalPackageGraphError, ModuleGraphError, ModuleIdentity, PackageIdentity, PackageLoadError,
};
use seseragi_syntax::ByteSpan;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LocalProjectLoadError {
    Packages(LocalPackageGraphError),
    MissingRunEntry {
        package: Box<PackageIdentity>,
    },
    Filesystem {
        package: Box<PackageIdentity>,
        error: Box<PackageLoadError>,
    },
    Import {
        module: Box<ModuleIdentity>,
        specifier: String,
        origin: ByteSpan,
        code: &'static str,
        reason: String,
    },
    DuplicatePhysicalModule {
        first: Box<ModuleIdentity>,
        second: Box<ModuleIdentity>,
        canonical_path: PathBuf,
    },
    Graph(Box<ModuleGraphError<ModuleIdentity>>),
}

impl LocalProjectLoadError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Packages(error) => error.code(),
            Self::Filesystem { error, .. } => error.code(),
            Self::Import { code, .. } => code,
            _ => "SES-K0001",
        }
    }
}

impl fmt::Display for LocalProjectLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Packages(error) => error.fmt(formatter),
            Self::MissingRunEntry { package } => write!(
                formatter,
                "root package `{}` has no [run] entry",
                package.name().as_str()
            ),
            Self::Filesystem { package, error } => {
                write!(formatter, "package `{}`: {error}", package.name().as_str())
            }
            Self::Import {
                module,
                specifier,
                reason,
                ..
            } => write!(
                formatter,
                "module `{}::{}` cannot resolve import `{specifier}`: {reason}",
                module.package().name().as_str(),
                module.path().as_str()
            ),
            Self::DuplicatePhysicalModule {
                first,
                second,
                canonical_path,
            } => write!(
                formatter,
                "modules `{}::{}` and `{}::{}` resolve to the same file `{}`",
                first.package().name().as_str(),
                first.path().as_str(),
                second.package().name().as_str(),
                second.path().as_str(),
                canonical_path.display()
            ),
            Self::Graph(error) => write!(formatter, "invalid source module graph: {error:?}"),
        }
    }
}

impl std::error::Error for LocalProjectLoadError {}
