use crate::{ModulePathError, PackageNameError};
use std::fmt;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub enum ManifestError {
    Toml {
        message: String,
        range: Option<Range<usize>>,
    },
    PackageName(PackageNameError),
    InvalidVersion(String),
    InvalidLanguageRequirement(String),
    InvalidDependencyKey(String, PackageNameError),
    SelfDependencyKey(String),
    InvalidDependencyPackage {
        key: String,
        package: String,
        error: PackageNameError,
    },
    InvalidDependencyVersion {
        key: String,
        value: String,
    },
    InvalidDependencyPath {
        key: String,
        value: String,
    },
    MissingDependencySource(String),
    ConflictingDependencySources(String),
    InvalidLayoutPath {
        field: &'static str,
        value: String,
    },
    OverlappingLayoutRoots {
        left: &'static str,
        right: &'static str,
    },
    InvalidExportKey(String, ModulePathError),
    InvalidExportTarget(String, ModulePathError),
    InvalidRunEntry(String, ModulePathError),
    InvalidTarget(String),
    InvalidSeed {
        field: &'static str,
        value: String,
    },
    ShutdownGraceWithForwardSignal,
}

impl ManifestError {
    pub(super) fn toml(error: toml::de::Error, source: &str) -> Self {
        Self::Toml {
            range: error.span().map(|range| expand_table_header(range, source)),
            message: error.message().to_owned(),
        }
    }

    pub const fn code(&self) -> &'static str {
        "SES-K0101"
    }

    pub fn range(&self) -> Option<Range<usize>> {
        match self {
            Self::Toml { range, .. } => range.clone(),
            _ => None,
        }
    }
}

fn expand_table_header(range: Range<usize>, source: &str) -> Range<usize> {
    let start = range
        .start
        .checked_sub(1)
        .filter(|start| source.as_bytes().get(*start) == Some(&b'['))
        .unwrap_or(range.start);
    let end = if source.as_bytes().get(range.end) == Some(&b']') {
        range.end + 1
    } else {
        range.end
    };
    start..end
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml { message, .. } => write!(formatter, "invalid manifest: {message}"),
            Self::PackageName(error) => error.fmt(formatter),
            Self::InvalidVersion(value) => write!(formatter, "invalid package version `{value}`"),
            Self::InvalidLanguageRequirement(value) => {
                write!(formatter, "invalid language version requirement `{value}`")
            }
            Self::InvalidDependencyKey(value, error) => {
                write!(formatter, "invalid dependency key `{value}`: {error}")
            }
            Self::SelfDependencyKey(value) => write!(
                formatter,
                "dependency key `{value}` conflicts with the current package name"
            ),
            Self::InvalidDependencyPackage {
                key,
                package,
                error,
            } => write!(
                formatter,
                "dependency `{key}` has invalid package name `{package}`: {error}"
            ),
            Self::InvalidDependencyVersion { key, value } => write!(
                formatter,
                "dependency `{key}` has invalid version requirement `{value}`"
            ),
            Self::InvalidDependencyPath { key, value } => write!(
                formatter,
                "dependency `{key}` path is not package-relative: `{value}`"
            ),
            Self::MissingDependencySource(key) => write!(
                formatter,
                "dependency `{key}` must specify exactly one of `version` or `path`"
            ),
            Self::ConflictingDependencySources(key) => write!(
                formatter,
                "dependency `{key}` cannot specify both `version` and `path`"
            ),
            Self::InvalidLayoutPath { field, value } => {
                write!(
                    formatter,
                    "{field} is not a package-relative directory: `{value}`"
                )
            }
            Self::OverlappingLayoutRoots { left, right } => {
                write!(formatter, "{left} overlaps {right}")
            }
            Self::InvalidExportKey(value, error) => {
                write!(formatter, "invalid export key `{value}`: {error}")
            }
            Self::InvalidExportTarget(value, error) => {
                write!(formatter, "invalid export target `{value}`: {error}")
            }
            Self::InvalidRunEntry(value, error) => {
                write!(formatter, "invalid run entry `{value}`: {error}")
            }
            Self::InvalidTarget(value) => write!(formatter, "invalid target id `{value}`"),
            Self::InvalidSeed { field, value } => {
                write!(
                    formatter,
                    "{field} must be `entropy` or an Int, got `{value}`"
                )
            }
            Self::ShutdownGraceWithForwardSignal => formatter
                .write_str("run.shutdown_grace_ms is only valid when signal_mode is `cancel`"),
        }
    }
}

impl std::error::Error for ManifestError {}
