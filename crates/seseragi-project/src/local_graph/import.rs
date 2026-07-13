use super::{LocalPackageGraph, ResolvedPackageImport};
use crate::{classify_specifier, ImportSpecifier, ModulePath, ModulePathError, PackageIdentity};
use std::fmt;

impl LocalPackageGraph {
    /// Resolves a bare package import through the importing package's direct
    /// dependency keys and the target package's public export map.
    pub fn resolve_package_import(
        &self,
        importer: &PackageIdentity,
        specifier: &str,
    ) -> Result<ResolvedPackageImport, PackageImportError> {
        let package_specifier = match classify_specifier(specifier) {
            Ok(ImportSpecifier::Package(value)) => value,
            Ok(kind) => {
                return Err(PackageImportError::NotPackageSpecifier {
                    specifier: specifier.to_owned(),
                    kind,
                });
            }
            Err(error) => {
                return Err(PackageImportError::InvalidSpecifier {
                    specifier: specifier.to_owned(),
                    reason: error.to_string(),
                });
            }
        };
        let importer_manifest =
            self.package(importer)
                .ok_or_else(|| PackageImportError::UnknownImporter {
                    importer: Box::new(importer.clone()),
                })?;
        let dependency_key = importer_manifest
            .manifest()
            .dependencies
            .keys()
            .filter(|key| {
                package_specifier == key.as_str()
                    || package_specifier
                        .strip_prefix(key.as_str())
                        .is_some_and(|suffix| suffix.starts_with('/'))
            })
            .max_by_key(|key| key.as_str().len())
            .ok_or_else(|| PackageImportError::UndeclaredDependency {
                importer: Box::new(importer.clone()),
                specifier: specifier.to_owned(),
            })?;
        let suffix = package_specifier
            .strip_prefix(dependency_key.as_str())
            .expect("selected dependency key is a specifier prefix");
        let export_key = if suffix.is_empty() {
            ".".to_owned()
        } else {
            let subpath = suffix
                .strip_prefix('/')
                .expect("non-empty dependency suffix starts with a slash");
            ModulePath::parse(subpath).map_err(|error| PackageImportError::InvalidSubpath {
                specifier: specifier.to_owned(),
                error,
            })?;
            subpath.to_owned()
        };
        let target = self
            .graph()
            .dependency_for(importer, dependency_key.as_str())
            .ok_or_else(|| PackageImportError::MissingDependencyEdge {
                importer: Box::new(importer.clone()),
                dependency_key: dependency_key.as_str().to_owned(),
            })?;
        let target_manifest =
            self.package(target)
                .ok_or_else(|| PackageImportError::MissingTargetPackage {
                    target: Box::new(target.clone()),
                })?;
        let module = target_manifest
            .manifest()
            .exports
            .get(&export_key)
            .cloned()
            .ok_or_else(|| PackageImportError::MissingExport {
                package: Box::new(target.clone()),
                export_key: export_key.clone(),
                specifier: specifier.to_owned(),
            })?;
        Ok(ResolvedPackageImport::new(
            dependency_key.as_str().to_owned(),
            export_key,
            target.clone(),
            module,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageImportError {
    InvalidSpecifier {
        specifier: String,
        reason: String,
    },
    NotPackageSpecifier {
        specifier: String,
        kind: ImportSpecifier,
    },
    UnknownImporter {
        importer: Box<PackageIdentity>,
    },
    UndeclaredDependency {
        importer: Box<PackageIdentity>,
        specifier: String,
    },
    InvalidSubpath {
        specifier: String,
        error: ModulePathError,
    },
    MissingDependencyEdge {
        importer: Box<PackageIdentity>,
        dependency_key: String,
    },
    MissingTargetPackage {
        target: Box<PackageIdentity>,
    },
    MissingExport {
        package: Box<PackageIdentity>,
        export_key: String,
        specifier: String,
    },
}

impl PackageImportError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UndeclaredDependency { .. } => "SES-K0103",
            Self::MissingExport { .. }
            | Self::InvalidSpecifier { .. }
            | Self::NotPackageSpecifier { .. }
            | Self::InvalidSubpath { .. } => "SES-N0104",
            Self::UnknownImporter { .. }
            | Self::MissingDependencyEdge { .. }
            | Self::MissingTargetPackage { .. } => "SES-K0001",
        }
    }
}

impl fmt::Display for PackageImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpecifier { specifier, reason } => {
                write!(formatter, "invalid package import `{specifier}`: {reason}")
            }
            Self::NotPackageSpecifier { specifier, kind } => write!(
                formatter,
                "module specifier `{specifier}` is not a package import ({kind:?})"
            ),
            Self::UnknownImporter { importer } => write!(
                formatter,
                "package `{}` is not part of this graph",
                importer.name().as_str()
            ),
            Self::UndeclaredDependency {
                importer,
                specifier,
            } => write!(
                formatter,
                "package `{}` imports undeclared dependency `{specifier}`",
                importer.name().as_str()
            ),
            Self::InvalidSubpath { specifier, error } => {
                write!(formatter, "invalid package import `{specifier}`: {error}")
            }
            Self::MissingDependencyEdge {
                importer,
                dependency_key,
            } => write!(
                formatter,
                "package graph is missing dependency `{dependency_key}` for `{}`",
                importer.name().as_str()
            ),
            Self::MissingTargetPackage { target } => write!(
                formatter,
                "package graph is missing target package `{}`",
                target.name().as_str()
            ),
            Self::MissingExport {
                package,
                export_key,
                specifier,
            } => write!(
                formatter,
                "package `{}` does not export `{export_key}` for import `{specifier}`",
                package.name().as_str()
            ),
        }
    }
}

impl std::error::Error for PackageImportError {}
