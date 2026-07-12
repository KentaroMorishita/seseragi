use crate::{ModulePath, ModulePathError};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportSpecifier {
    Relative(String),
    SelfPackage(String),
    Standard(String),
    Generated(String),
    Package(String),
}

pub fn classify_specifier(value: &str) -> Result<ImportSpecifier, SpecifierError> {
    if value.is_empty() {
        return Err(SpecifierError::Empty);
    }
    if value.starts_with('/') {
        return Err(SpecifierError::Absolute);
    }
    if value.contains('\\') {
        return Err(SpecifierError::Backslash);
    }
    if value.starts_with("./") || value.starts_with("../") {
        return Ok(ImportSpecifier::Relative(value.to_owned()));
    }
    for (prefix, constructor) in [
        (
            "self/",
            ImportSpecifier::SelfPackage as fn(String) -> ImportSpecifier,
        ),
        ("std/", ImportSpecifier::Standard),
        ("gen/", ImportSpecifier::Generated),
    ] {
        if let Some(path) = value.strip_prefix(prefix) {
            if path.is_empty() {
                return Err(SpecifierError::MissingPath { prefix });
            }
            return Ok(constructor(path.to_owned()));
        }
    }
    Ok(ImportSpecifier::Package(value.to_owned()))
}

pub fn resolve_relative_specifier(
    current: &ModulePath,
    specifier: &str,
) -> Result<ModulePath, RelativeSpecifierError> {
    if !specifier.starts_with("./") && !specifier.starts_with("../") {
        return Err(RelativeSpecifierError::NotRelative);
    }
    if specifier.contains('\\') {
        return Err(RelativeSpecifierError::InvalidPath(
            ModulePathError::Backslash,
        ));
    }

    let specifier = specifier.strip_suffix(".ssrg").unwrap_or(specifier);
    let mut output = current
        .segments()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    output.pop();
    let mut saw_target = false;
    for (index, segment) in specifier.split('/').enumerate() {
        match segment {
            "." if index == 0 => {}
            ".." if !saw_target => {
                if output.pop().is_none() {
                    return Err(RelativeSpecifierError::RootEscape);
                }
            }
            "" => {
                return Err(RelativeSpecifierError::InvalidPath(
                    ModulePathError::EmptySegment { index },
                ));
            }
            "." => {
                return Err(RelativeSpecifierError::InvalidPath(
                    ModulePathError::DotSegment { index },
                ));
            }
            ".." => {
                return Err(RelativeSpecifierError::InvalidPath(
                    ModulePathError::ParentSegment { index },
                ));
            }
            segment => {
                saw_target = true;
                output.push(segment.to_owned());
            }
        }
    }
    if !saw_target {
        return Err(RelativeSpecifierError::MissingTarget);
    }
    ModulePath::parse(&output.join("/")).map_err(RelativeSpecifierError::InvalidPath)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecifierError {
    Empty,
    Absolute,
    Backslash,
    MissingPath { prefix: &'static str },
}

impl fmt::Display for SpecifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("module specifier must not be empty"),
            Self::Absolute => formatter.write_str("module specifier must not be absolute"),
            Self::Backslash => formatter.write_str("module specifier must use `/` separators"),
            Self::MissingPath { prefix } => {
                write!(formatter, "module specifier `{prefix}` requires a path")
            }
        }
    }
}

impl std::error::Error for SpecifierError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelativeSpecifierError {
    NotRelative,
    MissingTarget,
    RootEscape,
    InvalidPath(ModulePathError),
}

impl fmt::Display for RelativeSpecifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRelative => formatter.write_str("module specifier is not relative"),
            Self::MissingTarget => formatter.write_str("relative module specifier has no target"),
            Self::RootEscape => formatter.write_str("relative module specifier escapes its root"),
            Self::InvalidPath(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RelativeSpecifierError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_specifiers_without_resolving_packages() {
        assert_eq!(
            classify_specifier("./user").unwrap(),
            ImportSpecifier::Relative("./user".to_owned())
        );
        assert_eq!(
            classify_specifier("self/internal/parser").unwrap(),
            ImportSpecifier::SelfPackage("internal/parser".to_owned())
        );
        assert_eq!(
            classify_specifier("std/text").unwrap(),
            ImportSpecifier::Standard("text".to_owned())
        );
        assert_eq!(
            classify_specifier("gen/api").unwrap(),
            ImportSpecifier::Generated("api".to_owned())
        );
        assert_eq!(
            classify_specifier("acme/http/client").unwrap(),
            ImportSpecifier::Package("acme/http/client".to_owned())
        );
        assert_eq!(
            classify_specifier("/absolute").unwrap_err(),
            SpecifierError::Absolute
        );
        assert_eq!(
            classify_specifier("game\\main").unwrap_err(),
            SpecifierError::Backslash
        );
    }

    #[test]
    fn resolves_relative_specifiers_within_the_current_root() {
        let current = ModulePath::parse("game/cli/main").unwrap();

        assert_eq!(
            resolve_relative_specifier(&current, "./input").unwrap(),
            ModulePath::parse("game/cli/input").unwrap()
        );
        assert_eq!(
            resolve_relative_specifier(&current, "../domain.ssrg").unwrap(),
            ModulePath::parse("game/domain").unwrap()
        );
    }

    #[test]
    fn rejects_root_escape_and_noncanonical_relative_paths() {
        let current = ModulePath::parse("main").unwrap();
        assert_eq!(
            resolve_relative_specifier(&current, "../outside").unwrap_err(),
            RelativeSpecifierError::RootEscape
        );

        let nested = ModulePath::parse("game/main").unwrap();
        assert_eq!(
            resolve_relative_specifier(&nested, "./domain/../model").unwrap_err(),
            RelativeSpecifierError::InvalidPath(ModulePathError::ParentSegment { index: 2 })
        );
    }
}
