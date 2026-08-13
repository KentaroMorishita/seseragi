use crate::{classify_specifier, resolve_relative_specifier, ImportSpecifier, ModulePath};
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
