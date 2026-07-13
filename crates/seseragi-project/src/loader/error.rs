use crate::{ImportSpecifier, ManifestError, ModuleGraphError, ModulePath};
use seseragi_syntax::ByteSpan;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum PackageLoadError {
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    Manifest {
        path: PathBuf,
        error: ManifestError,
    },
    MissingRunEntry,
    NonCanonicalSpelling {
        expected: String,
        actual: String,
        directory: PathBuf,
    },
    CaseMismatch {
        expected: String,
        actual: String,
        directory: PathBuf,
    },
    NormalizationCollision {
        expected: String,
        candidates: Vec<String>,
        directory: PathBuf,
    },
    RootEscape {
        path: PathBuf,
        canonical_path: PathBuf,
    },
    DuplicatePhysicalModule {
        first: ModulePath,
        second: ModulePath,
        canonical_path: PathBuf,
    },
    InvalidImport {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        reason: String,
    },
    UnsupportedImport {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        kind: ImportSpecifier,
    },
    Graph(ModuleGraphError<ModulePath>),
}

impl PackageLoadError {
    pub(crate) const fn io(action: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self::Io {
            action,
            path,
            source,
        }
    }

    pub const fn code(&self) -> &'static str {
        match self {
            Self::Manifest { .. } => "SES-K0101",
            Self::UnsupportedImport {
                kind: ImportSpecifier::Package(_),
                ..
            } => "SES-K0103",
            _ => "SES-K0001",
        }
    }
}

impl fmt::Display for PackageLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { action, path, source } => {
                write!(formatter, "failed to {action} `{}`: {source}", path.display())
            }
            Self::Manifest { path, error } => {
                write!(formatter, "invalid manifest `{}`: {error}", path.display())
            }
            Self::MissingRunEntry => formatter.write_str("package manifest has no [run] entry"),
            Self::NonCanonicalSpelling { expected, actual, directory } => write!(
                formatter,
                "module path component `{actual}` in `{}` is not canonical NFC spelling `{expected}`",
                directory.display()
            ),
            Self::CaseMismatch { expected, actual, directory } => write!(
                formatter,
                "module path component `{actual}` in `{}` does not match case `{expected}`",
                directory.display()
            ),
            Self::NormalizationCollision { expected, candidates, directory } => write!(
                formatter,
                "multiple entries in `{}` normalize to `{expected}`: {}",
                directory.display(),
                candidates.join(", ")
            ),
            Self::RootEscape { path, canonical_path } => write!(
                formatter,
                "module `{}` resolves outside the source root to `{}`",
                path.display(),
                canonical_path.display()
            ),
            Self::DuplicatePhysicalModule { first, second, canonical_path } => write!(
                formatter,
                "modules `{}` and `{}` resolve to the same file `{}`",
                first.as_str(),
                second.as_str(),
                canonical_path.display()
            ),
            Self::InvalidImport {
                module,
                specifier,
                reason,
                ..
            } => write!(
                formatter,
                "module `{}` has invalid import `{specifier}`: {reason}",
                module.as_str()
            ),
            Self::UnsupportedImport { module, specifier, .. } => write!(
                formatter,
                "module `{}` imports `{specifier}`, which is not a local source module",
                module.as_str()
            ),
            Self::Graph(error) => write!(formatter, "invalid module graph: {error:?}"),
        }
    }
}

impl std::error::Error for PackageLoadError {}
