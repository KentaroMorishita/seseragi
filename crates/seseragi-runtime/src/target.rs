use serde::Serialize;

use crate::{HostService, MainContract};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionTarget {
    Process,
    Browser,
}

impl ExecutionTarget {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::Browser => "browser",
        }
    }
}

struct TargetSpec {
    target: ExecutionTarget,
    services: &'static [HostService],
}

const TARGETS: &[TargetSpec] = &[
    TargetSpec {
        target: ExecutionTarget::Process,
        services: &[HostService::Console, HostService::Stdin],
    },
    TargetSpec {
        target: ExecutionTarget::Browser,
        services: &[HostService::Console, HostService::Stdin, HostService::Dom],
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetMismatch {
    pub target: ExecutionTarget,
    pub required: Vec<HostService>,
    pub missing: Vec<HostService>,
    pub available: Vec<HostService>,
    pub compatible_targets: Vec<ExecutionTarget>,
}

impl std::fmt::Display for TargetMismatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "target mismatch before execution")?;
        writeln!(
            formatter,
            "  required capabilities: {}",
            service_names(&self.required)
        )?;
        writeln!(formatter, "  selected target: {}", self.target.name())?;
        writeln!(
            formatter,
            "  selected target capabilities: {}",
            service_names(&self.available)
        )?;
        writeln!(
            formatter,
            "  missing capabilities: {}",
            service_names(&self.missing)
        )?;
        write!(
            formatter,
            "  available target contracts: {}",
            if self.compatible_targets.is_empty() {
                "none".to_owned()
            } else {
                self.compatible_targets
                    .iter()
                    .map(|target| target.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

impl std::error::Error for TargetMismatch {}

pub fn validate_target(
    contract: &MainContract,
    target: ExecutionTarget,
) -> Result<(), TargetMismatch> {
    let spec = target_spec(target);
    let required = unique_services(contract.environment.iter().map(|binding| binding.service));
    let missing = required
        .iter()
        .copied()
        .filter(|service| !spec.services.contains(service))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    let compatible_targets = TARGETS
        .iter()
        .filter(|candidate| {
            required
                .iter()
                .all(|service| candidate.services.contains(service))
        })
        .map(|candidate| candidate.target)
        .collect();
    Err(TargetMismatch {
        target,
        required,
        missing,
        available: spec.services.to_vec(),
        compatible_targets,
    })
}

fn target_spec(target: ExecutionTarget) -> &'static TargetSpec {
    TARGETS
        .iter()
        .find(|candidate| candidate.target == target)
        .expect("every execution target has one registry entry")
}

fn unique_services(services: impl IntoIterator<Item = HostService>) -> Vec<HostService> {
    let mut unique = Vec::new();
    for service in services {
        if !unique.contains(&service) {
            unique.push(service);
        }
    }
    unique
}

fn service_names(services: &[HostService]) -> String {
    if services.is_empty() {
        "none".to_owned()
    } else {
        services
            .iter()
            .map(|service| service.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_target, ExecutionTarget};
    use crate::{EnvironmentBinding, FailureRenderer, HostService, MainContract};

    fn contract(services: &[HostService]) -> MainContract {
        MainContract {
            environment: services
                .iter()
                .enumerate()
                .map(|(index, service)| EnvironmentBinding {
                    field: format!("service{index}"),
                    service: *service,
                })
                .collect(),
            failure_renderer: FailureRenderer::Never,
        }
    }

    #[test]
    fn accepts_process_services_and_rejects_dom_with_actionable_context() {
        validate_target(
            &contract(&[HostService::Console, HostService::Stdin]),
            ExecutionTarget::Process,
        )
        .unwrap();

        let mismatch = validate_target(
            &contract(&[HostService::Console, HostService::Dom]),
            ExecutionTarget::Process,
        )
        .unwrap_err();
        assert_eq!(mismatch.missing, [HostService::Dom]);
        assert_eq!(mismatch.compatible_targets, [ExecutionTarget::Browser]);
        assert_eq!(
            mismatch.to_string(),
            "target mismatch before execution\n  required capabilities: console, dom\n  selected target: process\n  selected target capabilities: console, stdin\n  missing capabilities: dom\n  available target contracts: browser"
        );
    }
}
