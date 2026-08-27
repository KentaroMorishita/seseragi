use serde::Serialize;
use seseragi_driver::{validate_provider_target, ServiceRequirement};

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
        writeln!(
            formatter,
            "target mismatch before execution [SES-K0203 provider.target-mismatch]"
        )?;
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
    let required = unique_services(contract.environment.iter().map(|binding| binding.service));
    let requirements = required
        .iter()
        .enumerate()
        .map(|(index, service)| ServiceRequirement {
            field: format!("service{index}"),
            service: service_identity(*service).to_owned(),
        })
        .collect::<Vec<_>>();
    validate_provider_target(&requirements, target.name()).map_err(|mismatch| TargetMismatch {
        target,
        required,
        missing: mismatch
            .missing
            .iter()
            .filter_map(|service| host_service(service))
            .collect(),
        available: mismatch
            .available
            .iter()
            .filter_map(|service| host_service(service))
            .collect(),
        compatible_targets: mismatch
            .compatible_targets
            .iter()
            .filter_map(|target| execution_target(target))
            .collect(),
    })
}

fn service_identity(service: HostService) -> &'static str {
    match service {
        HostService::Console => "std/prelude::Console",
        HostService::Logger => "std/log::Logger",
        HostService::Stdin => "std/prelude::Stdin",
        HostService::Process => "std/process::Process",
        HostService::Dom => "std/web/dom::Dom",
        HostService::Clock => "std/clock::Clock",
        HostService::FileSystem => "std/fs::FileSystem",
        HostService::Navigation => "std/web/navigation::Navigation",
        HostService::Storage => "std/web/storage::Storage",
        HostService::HttpClient => "std/http::HttpClient",
        HostService::HttpServer => "std/http/server::HttpServer",
        HostService::WebSocketClient => "std/websocket::WebSocketClient",
        HostService::WebSocketServer => "std/websocket/server::WebSocketServer",
        HostService::Postgres => "seseragi/postgres::Postgres",
        HostService::Sqlite => "seseragi/sqlite::Sqlite",
    }
}

fn host_service(identity: &str) -> Option<HostService> {
    match identity {
        "std/prelude::Console" => Some(HostService::Console),
        "std/log::Logger" => Some(HostService::Logger),
        "std/prelude::Stdin" => Some(HostService::Stdin),
        "std/process::Process" => Some(HostService::Process),
        "std/web/dom::Dom" => Some(HostService::Dom),
        "std/clock::Clock" => Some(HostService::Clock),
        "std/fs::FileSystem" => Some(HostService::FileSystem),
        "std/web/navigation::Navigation" => Some(HostService::Navigation),
        "std/web/storage::Storage" => Some(HostService::Storage),
        "std/http::HttpClient" => Some(HostService::HttpClient),
        "std/http/server::HttpServer" => Some(HostService::HttpServer),
        "std/websocket::WebSocketClient" => Some(HostService::WebSocketClient),
        "std/websocket/server::WebSocketServer" => Some(HostService::WebSocketServer),
        "seseragi/postgres::Postgres" => Some(HostService::Postgres),
        "seseragi/sqlite::Sqlite" => Some(HostService::Sqlite),
        _ => None,
    }
}

fn execution_target(target: &str) -> Option<ExecutionTarget> {
    match target {
        "process" => Some(ExecutionTarget::Process),
        "browser" => Some(ExecutionTarget::Browser),
        _ => None,
    }
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
            "target mismatch before execution [SES-K0203 provider.target-mismatch]\n  required capabilities: console, dom\n  selected target: process\n  selected target capabilities: console, logger, stdin, process\n  missing capabilities: dom\n  available target contracts: browser"
        );
    }
}
