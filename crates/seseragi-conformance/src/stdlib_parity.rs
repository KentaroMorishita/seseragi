use serde::Serialize;
use seseragi_project::{
    standard_module_registry_surface, StandardModuleRegistrySurface, StandardModuleStatus,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleParitySurface {
    pub schema: u32,
    pub kind: &'static str,
    pub language_version: &'static str,
    pub routes: Vec<StandardModuleProductRoute>,
    pub target_diagnostic: StandardModuleTargetDiagnostic,
    pub modules: Vec<StandardModuleProductSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleProductRoute {
    pub id: &'static str,
    pub evidence: &'static str,
    pub products: &'static [&'static str],
    pub modules: &'static [&'static str],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleProductSurface {
    pub specifier: &'static str,
    pub targets: &'static [&'static str],
    #[serde(default, skip_serializing_if = "<[&str]>::is_empty")]
    pub capability_services: &'static [&'static str],
    pub interface_fingerprint: String,
    pub implementation: &'static str,
    pub route: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardModuleTargetDiagnostic {
    pub code: &'static str,
    pub label: &'static str,
    pub evidence: &'static str,
    pub products: &'static [&'static str],
}

struct RouteDefinition {
    id: &'static str,
    evidence: &'static str,
    products: &'static [&'static str],
    modules: &'static [&'static str],
}

const ROUTES: &[RouteDefinition] = &[
    RouteDefinition {
        id: "portable-project",
        evidence: "examples/spec/fixtures/projects/std-parity-portable",
        products: &[
            "cli-build",
            "cli-run",
            "lsp-project",
            "wasm-project",
            "playground-execution",
        ],
        modules: &[
            "std/number",
            "std/int",
            "std/float",
            "std/array",
            "std/list",
            "std/bytes",
            "std/json",
            "std/text",
        ],
    },
    RouteDefinition {
        id: "canonical-web-project",
        evidence: "examples/samples/project-flow-app",
        products: &[
            "cli-build",
            "lsp-project",
            "wasm-project",
            "playground-browser",
        ],
        modules: &["std/web/html", "std/web/dom", "std/signal"],
    },
    RouteDefinition {
        id: "browser-provider-project",
        evidence: "apps/playground/tests/playground.integration.test.ts",
        products: &["wasm-project", "playground-execution"],
        modules: &[
            "std/clock",
            "std/time",
            "std/http",
            "std/web/navigation",
            "std/web/storage",
        ],
    },
    RouteDefinition {
        id: "process-provider-project",
        evidence: "examples/spec/fixtures/projects/provider-http-server-e2e",
        products: &["cli-run"],
        modules: &["std/http/server"],
    },
    RouteDefinition {
        id: "websocket-provider-project",
        evidence: "examples/spec/fixtures/projects/provider-websocket-e2e",
        products: &["cli-run", "playground-execution"],
        modules: &["std/websocket", "std/websocket/server"],
    },
    RouteDefinition {
        id: "sse-stream-project",
        evidence: "examples/spec/fixtures/projects/sse-server-client-e2e",
        products: &["cli-run"],
        modules: &["std/sse"],
    },
    RouteDefinition {
        id: "file-multipart-browser-project",
        evidence: "examples/spec/fixtures/projects/file-multipart-browser-e2e",
        products: &[
            "cli-build",
            "lsp-project",
            "wasm-project",
            "playground-browser",
        ],
        modules: &["std/web/file", "std/http/multipart"],
    },
    RouteDefinition {
        id: "effect-temporal-project",
        evidence: "examples/spec/fixtures/projects/effect-temporal-control",
        products: &["cli-run"],
        modules: &["std/effect", "std/ref"],
    },
    RouteDefinition {
        id: "effect-concurrency-project",
        evidence: "examples/spec/fixtures/projects/effect-concurrency-primitives",
        products: &["cli-run"],
        modules: &["std/deferred", "std/queue", "std/semaphore"],
    },
    RouteDefinition {
        id: "stream-core-project",
        evidence: "examples/spec/fixtures/projects/stream-cold-resource",
        products: &["cli-run", "wasm-project"],
        modules: &["std/stream"],
    },
    RouteDefinition {
        id: "console-logger-stdin-project",
        evidence: "examples/spec/fixtures/projects/stdin-lines",
        products: &["cli-run"],
        modules: &["std/console", "std/log", "std/stdin"],
    },
    RouteDefinition {
        id: "filesystem-provider-project",
        evidence: "examples/spec/fixtures/projects/filesystem-temporary-cleanup",
        products: &["cli-run"],
        modules: &["std/path", "std/fs"],
    },
    RouteDefinition {
        id: "process-core-project",
        evidence: "examples/spec/fixtures/projects/process-shutdown-forward",
        products: &["cli-run"],
        modules: &["std/non-empty-list", "std/process"],
    },
    RouteDefinition {
        id: "child-process-provider-project",
        evidence: "examples/spec/fixtures/projects/child-process-captured",
        products: &["cli-run"],
        modules: &["std/child-process"],
    },
];

pub fn standard_module_parity_surface() -> Result<StandardModuleParitySurface, String> {
    build_standard_module_parity_surface(
        standard_module_registry_surface(),
        seseragi_lowering::runtime_provided_modules(),
    )
}

fn build_standard_module_parity_surface(
    registry: StandardModuleRegistrySurface,
    runtime_modules: &[&str],
) -> Result<StandardModuleParitySurface, String> {
    let available = registry
        .modules
        .iter()
        .filter(|module| module.status == StandardModuleStatus::Available)
        .map(|module| module.specifier)
        .collect::<BTreeSet<_>>();
    let implemented = runtime_modules.iter().copied().collect::<BTreeSet<_>>();
    if available != implemented {
        return Err(format!(
            "standard registry/runtime implementation mismatch: registry={available:?}, runtime={implemented:?}"
        ));
    }

    let mut route_by_module = BTreeMap::new();
    for route in ROUTES {
        for module in route.modules {
            if route_by_module.insert(*module, route.id).is_some() {
                return Err(format!(
                    "duplicate standard module parity route for `{module}`"
                ));
            }
        }
    }
    let classified = route_by_module.keys().copied().collect::<BTreeSet<_>>();
    if available != classified {
        return Err(format!(
            "standard parity route coverage mismatch: available={available:?}, classified={classified:?}"
        ));
    }

    let modules = registry
        .modules
        .into_iter()
        .filter(|module| module.status == StandardModuleStatus::Available)
        .map(|module| {
            let interface = module.public_interface.as_ref().ok_or_else(|| {
                format!(
                    "available standard module `{}` has no public interface",
                    module.specifier
                )
            })?;
            let interface = serde_json::to_vec(interface).map_err(|error| {
                format!(
                    "failed to encode `{}` public interface: {error}",
                    module.specifier
                )
            })?;
            Ok(StandardModuleProductSurface {
                specifier: module.specifier,
                targets: module.targets,
                capability_services: module.capability_services,
                interface_fingerprint: fnv1a64_fingerprint(&interface),
                implementation: "lowering-runtime",
                route: route_by_module[module.specifier],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(StandardModuleParitySurface {
        schema: 1,
        kind: "standard-module-product-parity",
        language_version: registry.language_version,
        routes: ROUTES
            .iter()
            .map(|route| StandardModuleProductRoute {
                id: route.id,
                evidence: route.evidence,
                products: route.products,
                modules: route.modules,
            })
            .collect(),
        target_diagnostic: StandardModuleTargetDiagnostic {
            code: "SES-K0203",
            label: "provider.target-mismatch",
            evidence: "examples/spec/fixtures/projects/std-parity-target",
            products: &["cli-run", "wasm-project", "playground-diagnostics"],
        },
        modules,
    })
}

fn fnv1a64_fingerprint(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_every_available_module_with_a_runtime_and_product_route() {
        let surface = standard_module_parity_surface().unwrap();
        assert_eq!(surface.modules.len(), 36);
        assert!(surface
            .modules
            .iter()
            .all(|module| module.interface_fingerprint.starts_with("fnv1a64:")));
    }

    #[test]
    fn rejects_a_new_available_module_without_a_route() {
        let mut registry = standard_module_registry_surface();
        let mut future = registry
            .modules
            .iter()
            .find(|module| module.specifier == "std/array")
            .unwrap()
            .clone();
        future.specifier = "std/future";
        future.identity = "std/future";
        registry.modules.push(future);

        let mut runtime = seseragi_lowering::runtime_provided_modules().to_vec();
        runtime.push("std/future");
        assert!(build_standard_module_parity_surface(registry, &runtime)
            .unwrap_err()
            .contains("route coverage mismatch"));
    }

    #[test]
    fn rejects_an_importable_module_without_a_runtime_connection() {
        let runtime = &seseragi_lowering::runtime_provided_modules()[1..];
        assert!(
            build_standard_module_parity_surface(standard_module_registry_surface(), runtime)
                .unwrap_err()
                .contains("registry/runtime implementation mismatch")
        );
    }
}
