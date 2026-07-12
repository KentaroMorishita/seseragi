use std::fmt;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModulePath(String);

impl ModulePath {
    pub fn parse(value: &str) -> Result<Self, ModulePathError> {
        if value.is_empty() {
            return Err(ModulePathError::Empty);
        }
        if value.starts_with('/') {
            return Err(ModulePathError::Absolute);
        }
        if value.contains('\\') {
            return Err(ModulePathError::Backslash);
        }
        if value.ends_with(".ssrg") {
            return Err(ModulePathError::ExtensionSuffix);
        }

        let normalized = value.nfc().collect::<String>();
        for (index, segment) in normalized.split('/').enumerate() {
            match segment {
                "" => return Err(ModulePathError::EmptySegment { index }),
                "." => return Err(ModulePathError::DotSegment { index }),
                ".." => return Err(ModulePathError::ParentSegment { index }),
                _ => {}
            }
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn segments(&self) -> Vec<&str> {
        self.0.split('/').collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModulePathError {
    Empty,
    Absolute,
    Backslash,
    ExtensionSuffix,
    EmptySegment { index: usize },
    DotSegment { index: usize },
    ParentSegment { index: usize },
}

impl fmt::Display for ModulePathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("module path must not be empty"),
            Self::Absolute => formatter.write_str("module path must be relative to its root"),
            Self::Backslash => formatter.write_str("module path must use `/` separators"),
            Self::ExtensionSuffix => formatter.write_str("module path must omit `.ssrg`"),
            Self::EmptySegment { index } => {
                write!(formatter, "module path segment {index} is empty")
            }
            Self::DotSegment { index } => write!(formatter, "module path segment {index} is `.`"),
            Self::ParentSegment { index } => {
                write!(formatter, "module path segment {index} is `..`")
            }
        }
    }
}

impl std::error::Error for ModulePathError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_module_paths_to_unicode_nfc() {
        let decomposed = ModulePath::parse("model/cafe\u{301}").unwrap();
        let composed = ModulePath::parse("model/café").unwrap();

        assert_eq!(decomposed, composed);
        assert_eq!(decomposed.as_str(), "model/café");
    }

    #[test]
    fn rejects_noncanonical_manifest_module_path_shapes() {
        assert_eq!(ModulePath::parse("").unwrap_err(), ModulePathError::Empty);
        assert_eq!(
            ModulePath::parse("/main").unwrap_err(),
            ModulePathError::Absolute
        );
        assert_eq!(
            ModulePath::parse("game\\main").unwrap_err(),
            ModulePathError::Backslash
        );
        assert_eq!(
            ModulePath::parse("game/main.ssrg").unwrap_err(),
            ModulePathError::ExtensionSuffix
        );
        assert_eq!(
            ModulePath::parse("game//main").unwrap_err(),
            ModulePathError::EmptySegment { index: 1 }
        );
        assert_eq!(
            ModulePath::parse("game/../main").unwrap_err(),
            ModulePathError::ParentSegment { index: 1 }
        );
    }
}
