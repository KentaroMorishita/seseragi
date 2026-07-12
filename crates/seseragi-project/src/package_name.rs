use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageName(String);

impl PackageName {
    pub fn parse(value: &str) -> Result<Self, PackageNameError> {
        if value.is_empty() {
            return Err(PackageNameError::Empty);
        }
        for (index, segment) in value.split('/').enumerate() {
            if segment.is_empty() {
                return Err(PackageNameError::EmptySegment { index });
            }
            if !valid_segment(segment) {
                return Err(PackageNameError::InvalidSegment { index });
            }
            if index == 0 && matches!(segment, "std" | "self" | "gen") {
                return Err(PackageNameError::ReservedFirstSegment);
            }
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn valid_segment(segment: &str) -> bool {
    let bytes = segment.as_bytes();
    bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.last() != Some(&b'-')
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageNameError {
    Empty,
    EmptySegment { index: usize },
    InvalidSegment { index: usize },
    ReservedFirstSegment,
}

impl fmt::Display for PackageNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("package name must not be empty"),
            Self::EmptySegment { index } => {
                write!(formatter, "package name segment {index} is empty")
            }
            Self::InvalidSegment { index } => {
                write!(formatter, "package name segment {index} is invalid")
            }
            Self::ReservedFirstSegment => {
                formatter.write_str("package name uses a reserved first segment")
            }
        }
    }
}

impl std::error::Error for PackageNameError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ascii_lowercase_package_segments() {
        let name = PackageName::parse("acme/http-client2").unwrap();
        assert_eq!(name.as_str(), "acme/http-client2");
    }

    #[test]
    fn rejects_reserved_or_noncanonical_package_names() {
        for (value, expected) in [
            ("", PackageNameError::Empty),
            ("acme//http", PackageNameError::EmptySegment { index: 1 }),
            ("Acme/http", PackageNameError::InvalidSegment { index: 0 }),
            ("acme/http-", PackageNameError::InvalidSegment { index: 1 }),
            ("std/text", PackageNameError::ReservedFirstSegment),
            ("self/app", PackageNameError::ReservedFirstSegment),
            ("gen/api", PackageNameError::ReservedFirstSegment),
        ] {
            assert_eq!(PackageName::parse(value).unwrap_err(), expected, "{value}");
        }
    }
}
