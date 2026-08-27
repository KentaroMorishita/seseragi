use crate::ServiceRequirement;

const CONSOLE: &str = "std/prelude::Console";
const LOGGER: &str = "std/log::Logger";
const STDIN: &str = "std/prelude::Stdin";
const DOM: &str = "std/web/dom::Dom";

struct TargetProfile {
    name: &'static str,
    services: &'static [&'static str],
}

const TARGET_PROFILES: &[TargetProfile] = &[
    TargetProfile {
        name: "process",
        services: &[CONSOLE, LOGGER, STDIN],
    },
    TargetProfile {
        name: "browser",
        services: &[CONSOLE, LOGGER, STDIN, DOM],
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderTargetMismatch {
    pub target: String,
    pub required: Vec<String>,
    pub missing: Vec<String>,
    pub available: Vec<String>,
    pub compatible_targets: Vec<String>,
}

impl ProviderTargetMismatch {
    pub const fn code(&self) -> &'static str {
        "SES-K0203"
    }

    pub const fn label(&self) -> &'static str {
        "provider.target-mismatch"
    }
}

/// Rejects only services owned by the shared toolchain target registry.
/// Unknown service identities remain eligible for provider resolution.
pub fn validate_provider_target(
    requirements: &[ServiceRequirement],
    target: &str,
) -> Result<(), ProviderTargetMismatch> {
    let profile = profile(target);
    let required = unique_sorted(
        requirements
            .iter()
            .filter(|requirement| is_builtin_service(&requirement.service))
            .map(|requirement| requirement.service.clone()),
    );
    let available = profile
        .map(|profile| {
            profile
                .services
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let missing = required
        .iter()
        .filter(|service| !available.contains(service))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    let compatible_targets = TARGET_PROFILES
        .iter()
        .filter(|candidate| {
            required
                .iter()
                .all(|service| candidate.services.contains(&service.as_str()))
        })
        .map(|candidate| candidate.name.to_owned())
        .collect();
    Err(ProviderTargetMismatch {
        target: target.to_owned(),
        required,
        missing,
        available,
        compatible_targets,
    })
}

pub fn is_builtin_service(service: &str) -> bool {
    [CONSOLE, LOGGER, STDIN, DOM].contains(&service)
}

fn profile(target: &str) -> Option<&'static TargetProfile> {
    let canonical = match target {
        "bun-process" | "node-process" => "process",
        other => other,
    };
    TARGET_PROFILES
        .iter()
        .find(|candidate| candidate.name == canonical)
}

fn unique_sorted(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(field: &str, service: &str) -> ServiceRequirement {
        ServiceRequirement {
            field: field.to_owned(),
            service: service.to_owned(),
        }
    }

    #[test]
    fn central_registry_rejects_dom_on_process_without_claiming_provider_services() {
        let requirements = [
            requirement("console", CONSOLE),
            requirement("dom", DOM),
            requirement("clock", "std/clock::Clock"),
        ];
        let mismatch = validate_provider_target(&requirements, "bun-process").unwrap_err();
        assert_eq!(mismatch.code(), "SES-K0203");
        assert_eq!(mismatch.missing, [DOM]);
        assert_eq!(mismatch.compatible_targets, ["browser"]);

        validate_provider_target(&[requirement("clock", "std/clock::Clock")], "future-target")
            .unwrap();
    }
}
