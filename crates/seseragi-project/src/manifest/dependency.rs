use super::model::{
    DependencyKey, DependencyPath, DependencyVersionRequirement, ManifestDependency,
};
use super::ManifestError;
use crate::PackageName;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum RawDependency {
    Version(String),
    Detailed(RawDetailedDependency),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawDetailedDependency {
    package: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

pub(super) fn parse_dependencies(
    current_package: &PackageName,
    raw: BTreeMap<String, RawDependency>,
) -> Result<BTreeMap<DependencyKey, ManifestDependency>, ManifestError> {
    raw.into_iter()
        .map(|(raw_key, raw_dependency)| {
            let key = parse_key(&raw_key)?;
            if key.as_str() == current_package.as_str() {
                return Err(ManifestError::SelfDependencyKey(raw_key));
            }
            let dependency = match raw_dependency {
                RawDependency::Version(version) => ManifestDependency::Registry {
                    package: PackageName::parse(key.as_str()).map_err(|error| {
                        ManifestError::InvalidDependencyKey(key.as_str().to_owned(), error)
                    })?,
                    version: parse_version_requirement(&raw_key, version)?,
                },
                RawDependency::Detailed(dependency) => {
                    parse_detailed_dependency(&raw_key, dependency)?
                }
            };
            Ok((key, dependency))
        })
        .collect()
}

fn parse_key(value: &str) -> Result<DependencyKey, ManifestError> {
    PackageName::parse(value)
        .map_err(|error| ManifestError::InvalidDependencyKey(value.to_owned(), error))?;
    Ok(DependencyKey::new(value.to_owned()))
}

fn parse_detailed_dependency(
    key: &str,
    raw: RawDetailedDependency,
) -> Result<ManifestDependency, ManifestError> {
    let package = PackageName::parse(&raw.package).map_err(|error| {
        ManifestError::InvalidDependencyPackage {
            key: key.to_owned(),
            package: raw.package.clone(),
            error,
        }
    })?;
    match (raw.version, raw.path) {
        (Some(version), None) => Ok(ManifestDependency::Registry {
            package,
            version: parse_version_requirement(key, version)?,
        }),
        (None, Some(path)) => Ok(ManifestDependency::Path {
            package,
            path: parse_dependency_path(key, path)?,
        }),
        (None, None) => Err(ManifestError::MissingDependencySource(key.to_owned())),
        (Some(_), Some(_)) => Err(ManifestError::ConflictingDependencySources(key.to_owned())),
    }
}

fn parse_version_requirement(
    key: &str,
    value: String,
) -> Result<DependencyVersionRequirement, ManifestError> {
    if super::requirement::validate(&value) {
        Ok(DependencyVersionRequirement::new(value))
    } else {
        Err(ManifestError::InvalidDependencyVersion {
            key: key.to_owned(),
            value,
        })
    }
}

fn parse_dependency_path(key: &str, value: String) -> Result<DependencyPath, ManifestError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == ".")
    {
        Err(ManifestError::InvalidDependencyPath {
            key: key.to_owned(),
            value,
        })
    } else {
        Ok(DependencyPath::new(value))
    }
}
