use crate::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, ModulePath, ModuleRoot,
};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceImportResolution {
    Standard,
    Local(ModulePath),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourceImportError {
    Invalid(String),
    Unsupported(ImportSpecifier),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleImportResolution {
    Standard,
    Local { root: ModuleRoot, path: ModulePath },
}

/// Resolve imports within one package, preserving the source/generated root.
/// Package dependencies and public exports remain the package loader's concern.
pub fn resolve_module_import(
    root: ModuleRoot,
    current: &ModulePath,
    specifier: &str,
) -> Result<ModuleImportResolution, SourceImportError> {
    let classified = classify_specifier(specifier)
        .map_err(|error| SourceImportError::Invalid(error.to_string()))?;
    match classified {
        ImportSpecifier::Generated(path) => {
            if root == ModuleRoot::Generated {
                return Err(SourceImportError::Invalid(
                    "generated modules must use relative imports within the generated root"
                        .to_owned(),
                ));
            }
            Ok(ModuleImportResolution::Local {
                root: ModuleRoot::Generated,
                path: ModulePath::parse(&path)
                    .map_err(|error| SourceImportError::Invalid(error.to_string()))?,
            })
        }
        ImportSpecifier::SelfPackage(_) if root == ModuleRoot::Generated => {
            Err(SourceImportError::Invalid(
                "generated modules cannot import handwritten source through `self/`".to_owned(),
            ))
        }
        other => match resolve_source_import(current, specifier)? {
            SourceImportResolution::Standard => Ok(ModuleImportResolution::Standard),
            SourceImportResolution::Local(path) => Ok(ModuleImportResolution::Local {
                root: if matches!(other, ImportSpecifier::Relative(_)) {
                    root
                } else {
                    ModuleRoot::Source
                },
                path,
            }),
        },
    }
}

pub fn resolve_source_import(
    current: &ModulePath,
    specifier: &str,
) -> Result<SourceImportResolution, SourceImportError> {
    match classify_specifier(specifier)
        .map_err(|error| SourceImportError::Invalid(error.to_string()))?
    {
        ImportSpecifier::Standard(_) => Ok(SourceImportResolution::Standard),
        ImportSpecifier::Relative(value) => resolve_relative_specifier(current, &value)
            .map(SourceImportResolution::Local)
            .map_err(|error| SourceImportError::Invalid(error.to_string())),
        ImportSpecifier::SelfPackage(value) => ModulePath::parse(&value)
            .map(SourceImportResolution::Local)
            .map_err(|error| SourceImportError::Invalid(error.to_string())),
        unsupported => Err(SourceImportError::Unsupported(unsupported)),
    }
}

impl fmt::Display for SourceImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => formatter.write_str(reason),
            Self::Unsupported(specifier) => {
                write!(formatter, "unsupported source import {specifier:?}")
            }
        }
    }
}

impl std::error::Error for SourceImportError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_shared_single_package_import_contract() {
        let current = ModulePath::parse("feature/main").unwrap();

        assert_eq!(
            resolve_source_import(&current, "./model").unwrap(),
            SourceImportResolution::Local(ModulePath::parse("feature/model").unwrap())
        );
        assert_eq!(
            resolve_source_import(&current, "self/shared").unwrap(),
            SourceImportResolution::Local(ModulePath::parse("shared").unwrap())
        );
        assert_eq!(
            resolve_source_import(&current, "std/list").unwrap(),
            SourceImportResolution::Standard
        );
        assert!(matches!(
            resolve_source_import(&current, "acme/model"),
            Err(SourceImportError::Unsupported(ImportSpecifier::Package(_)))
        ));
    }
}
