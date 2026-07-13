use semver::{Prerelease, Version};

pub(super) fn validate(value: &str) -> bool {
    !value.trim().is_empty()
        && value.split("||").all(|union| {
            let comparators = union.split_whitespace().collect::<Vec<_>>();
            !comparators.is_empty() && comparators.into_iter().all(parse_comparator)
        })
}

pub(super) fn matches(requirement: &str, version: &Version) -> bool {
    requirement
        .split("||")
        .map(|union| union.split_whitespace().collect::<Vec<_>>())
        .any(|comparators| {
            prerelease_allowed(&comparators, version)
                && comparators
                    .into_iter()
                    .all(|comparator| comparator_matches(comparator, version))
        })
}

fn parse_comparator(value: &str) -> bool {
    comparator_parts(value).is_some_and(|(_, version)| Version::parse(version).is_ok())
}

fn comparator_parts(value: &str) -> Option<(&str, &str)> {
    for operator in [">=", "<=", ">", "<", "^", "~"] {
        if let Some(version) = value.strip_prefix(operator) {
            return (!version.is_empty()).then_some((operator, version));
        }
    }
    (!value.is_empty()).then_some(("=", value))
}

fn comparator_matches(value: &str, candidate: &Version) -> bool {
    let Some((operator, version)) = comparator_parts(value) else {
        return false;
    };
    let Ok(version) = Version::parse(version) else {
        return false;
    };
    match operator {
        "=" => candidate == &version,
        ">" => candidate > &version,
        ">=" => candidate >= &version,
        "<" => candidate < &version,
        "<=" => candidate <= &version,
        "^" => candidate >= &version && below_upper(candidate, caret_upper(&version)),
        "~" => candidate >= &version && below_upper(candidate, tilde_upper(&version)),
        _ => false,
    }
}

fn prerelease_allowed(comparators: &[&str], candidate: &Version) -> bool {
    candidate.pre == Prerelease::EMPTY
        || comparators.iter().any(|comparator| {
            comparator_parts(comparator)
                .and_then(|(_, version)| Version::parse(version).ok())
                .is_some_and(|version| {
                    version.pre != Prerelease::EMPTY
                        && version.major == candidate.major
                        && version.minor == candidate.minor
                        && version.patch == candidate.patch
                })
        })
}

fn caret_upper(version: &Version) -> Option<Version> {
    if version.major > 0 {
        version
            .major
            .checked_add(1)
            .map(|major| Version::new(major, 0, 0))
    } else if version.minor > 0 {
        version
            .minor
            .checked_add(1)
            .map(|minor| Version::new(0, minor, 0))
    } else {
        version
            .patch
            .checked_add(1)
            .map(|patch| Version::new(0, 0, patch))
    }
}

fn tilde_upper(version: &Version) -> Option<Version> {
    version
        .minor
        .checked_add(1)
        .map(|minor| Version::new(version.major, minor, 0))
}

fn below_upper(candidate: &Version, upper: Option<Version>) -> bool {
    upper.is_none_or(|upper| candidate < &upper)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(value: &str) -> Version {
        Version::parse(value).unwrap()
    }

    #[test]
    fn matches_exact_intersection_union_caret_and_tilde_ranges() {
        assert!(matches(">=0.1.0 <0.2.0", &version("0.1.7")));
        assert!(!matches(">=0.1.0 <0.2.0", &version("0.2.0")));
        assert!(matches("^0.2.3 || ~1.4.0", &version("0.2.9")));
        assert!(matches("^0.2.3 || ~1.4.0", &version("1.4.8")));
        assert!(!matches("^0.2.3 || ~1.4.0", &version("1.5.0")));
        assert!(matches("1.2.3", &version("1.2.3")));
    }

    #[test]
    fn requires_an_explicit_same_core_prerelease_comparator() {
        assert!(!matches(">=0.1.0 <0.2.0", &version("0.2.0-alpha.1")));
        assert!(matches(">=0.2.0-alpha.1 <0.2.0", &version("0.2.0-beta.1")));
        assert!(!matches(">=0.1.0-alpha <0.2.0", &version("0.2.0-beta.1")));
    }
}
