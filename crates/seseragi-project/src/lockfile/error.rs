use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LockError {
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    InvalidToml(String),
    UnsupportedSchema(u64),
    InvalidField {
        field: String,
        reason: String,
    },
    DuplicatePackage(String),
    DuplicateIdentity(String),
    DuplicateDependency {
        package: String,
        import: String,
    },
    DanglingRoot(String),
    DanglingDependency {
        package: String,
        dependency: String,
    },
    Missing,
    Stale(String),
    PackageGraph(String),
}

impl LockError {
    pub(crate) fn io(action: &'static str, path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.into(),
            source,
        }
    }

    pub const fn code(&self) -> &'static str {
        match self {
            Self::Missing | Self::Stale(_) => "SES-K0102",
            Self::DuplicateIdentity(_) => "SES-K0104",
            _ => "SES-K0001",
        }
    }
}

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                action,
                path,
                source,
            } => {
                write!(
                    formatter,
                    "failed to {action} `{}`: {source}",
                    path.display()
                )
            }
            Self::InvalidToml(reason) => write!(formatter, "invalid seseragi.lock TOML: {reason}"),
            Self::UnsupportedSchema(schema) => {
                write!(
                    formatter,
                    "unsupported seseragi.lock schema major `{schema}`"
                )
            }
            Self::InvalidField { field, reason } => {
                write!(formatter, "invalid seseragi.lock `{field}`: {reason}")
            }
            Self::DuplicatePackage(id) => write!(formatter, "duplicate locked package `{id}`"),
            Self::DuplicateIdentity(identity) => {
                write!(formatter, "dependency identity confusion for `{identity}`")
            }
            Self::DuplicateDependency { package, import } => write!(
                formatter,
                "locked package `{package}` contains duplicate dependency import `{import}`"
            ),
            Self::DanglingRoot(root) => {
                write!(
                    formatter,
                    "lock root `{root}` does not name a locked package"
                )
            }
            Self::DanglingDependency {
                package,
                dependency,
            } => write!(
                formatter,
                "locked package `{package}` depends on missing package `{dependency}`"
            ),
            Self::Missing => formatter
                .write_str("seseragi.lock is missing; run `seseragi lock update` explicitly"),
            Self::Stale(reason) => write!(
                formatter,
                "seseragi.lock is stale: {reason}; run `seseragi lock update` explicitly"
            ),
            Self::PackageGraph(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for LockError {}
