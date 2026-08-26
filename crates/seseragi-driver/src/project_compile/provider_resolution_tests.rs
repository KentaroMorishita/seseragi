use super::{
    analyze_project_with_providers, compile_project_with_providers, ProjectCompileError,
    ProjectModuleInput,
};
use crate::{
    CandidateVisibility, CompilerFeatureRequirement, ContractVersion, ProjectProviderConfiguration,
    ProviderCandidate, ProviderCompatibilityContext, ProviderConformanceRequirement,
    ProviderContract, ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext,
    RequiredService, RequirementTrace, RuntimePackageCompatibility, ServiceRequirement,
    TargetExtensionRequirement,
};
use serde_json::json;
use seseragi_project::ModuleGraph;
use std::collections::{BTreeMap, BTreeSet};

const MODULE: &str = "fixture/provider::main";
const SOURCE: &str =
    "pub type Clock = | Clock\n\npub effect fn main -> Unit with clock: Clock =\n  succeed ()\n";
const SERVICE: &str = "fixture/provider::main::Clock";
const PROVIDER: &str = "fixture/runtime-bun#clock";

fn graph_and_inputs(source: &str) -> (ModuleGraph<String>, Vec<ProjectModuleInput>) {
    let mut graph = ModuleGraph::new();
    graph.add_module(MODULE.to_owned(), []).unwrap();
    (
        graph,
        vec![
            ProjectModuleInput::new("src/main.ssrg", MODULE, source, "dist/main.js")
                .with_package_scope("fixture/provider@1.0.0"),
        ],
    )
}

fn contract(version: u64) -> ProviderContract {
    ProviderContract::from_json(
        &json!({
            "schema": 1,
            "kind": "provider-contract",
            "identity": SERVICE,
            "version": { "major": version, "minor": 0 },
            "requirement": { "field": "clock", "type": SERVICE },
            "operations": [{
                "identity": format!("{SERVICE}#now"),
                "kind": "one-shot",
                "input": { "kind": "unit" },
                "success": { "kind": "primitive", "name": "int" },
                "failure": { "kind": "never" },
                "portability": { "kind": "portable" },
                "summary": "Read a deterministic clock."
            }]
        })
        .to_string(),
    )
    .unwrap()
}

fn candidate(identity: &str, contract_version: u64, abi: u64, feature: &str) -> ProviderCandidate {
    let contract = contract(contract_version);
    let manifest = ProviderManifest::from_json(
        &json!({
            "schema": 1,
            "kind": "runtime-provider",
            "identity": identity,
            "service": SERVICE,
            "contractVersion": { "major": contract_version, "minor": 0 },
            "backend": { "family": "typescript", "abiMajor": abi },
            "targets": ["bun-process"],
            "entry": { "module": "fixture/runtime-bun/clock", "export": "provider" },
            "requires": { "runtimeFeatures": [feature], "hostPackages": [] }
        })
        .to_string(),
    )
    .unwrap();
    ProviderCandidate {
        manifest,
        contract,
        visibility: CandidateVisibility::RootDirectDependency,
        package: ProviderPackageMetadata {
            version: "1.0.0".to_owned(),
            source_identity: "registry:fixture/runtime-bun@1.0.0".to_owned(),
            content_digest: "sha256:package".to_owned(),
        },
        artifact_digest: "sha256:artifact".to_owned(),
        host_packages: Vec::new(),
    }
}

fn trace() -> RequirementTrace {
    let start = SOURCE.find("clock: Clock").unwrap() as u32;
    RequirementTrace {
        package: "fixture/provider@1.0.0".to_owned(),
        module: MODULE.to_owned(),
        source: "src/main.ssrg".to_owned(),
        start,
        end: start + "clock: Clock".len() as u32,
    }
}

fn configuration() -> ProjectProviderConfiguration {
    ProjectProviderConfiguration {
        entry_module: MODULE.to_owned(),
        contracts: vec![contract(1)],
        candidates: vec![candidate(PROVIDER, 1, 1, "foreign.task-load")],
        context: ProviderResolutionContext {
            target: "bun-process".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::new(),
        },
        transitive_requirements: Vec::new(),
        compatibility: ProviderCompatibilityContext::default(),
    }
}

fn matching_diagnostic(
    source: &str,
    configuration: ProjectProviderConfiguration,
) -> crate::ProjectProviderDiagnostic {
    let (graph, inputs) = graph_and_inputs(source);
    let compile =
        compile_project_with_providers(graph.clone(), inputs.clone(), configuration.clone())
            .unwrap_err();
    let analyze = analyze_project_with_providers(graph, inputs, configuration).unwrap_err();
    let ProjectCompileError::Provider {
        diagnostic: compiled,
    } = compile
    else {
        panic!("compile must stop at provider planning")
    };
    let ProjectCompileError::Provider {
        diagnostic: analyzed,
    } = analyze
    else {
        panic!("analyze must stop at provider planning")
    };
    assert_eq!(compiled, analyzed);
    assert_eq!(compiled.trace.as_ref().unwrap().source, "src/main.ssrg");
    assert!(compiled.trace.as_ref().unwrap().end > compiled.trace.as_ref().unwrap().start);
    compiled
}

#[test]
fn analyze_and_compile_share_every_provider_resolution_diagnostic_code() {
    let mut cases = Vec::new();

    let mut missing = configuration();
    missing.candidates.clear();
    cases.push(("SES-K0201", "provider.missing", missing));

    let mut ambiguous = configuration();
    ambiguous.candidates.push(candidate(
        "fixture/alternate#clock",
        1,
        1,
        "foreign.task-load",
    ));
    cases.push(("SES-K0202", "provider.ambiguous", ambiguous));

    let mut contract_mismatch = configuration();
    contract_mismatch.candidates = vec![candidate(PROVIDER, 2, 1, "foreign.task-load")];
    cases.push(("SES-K0204", "provider.contract-mismatch", contract_mismatch));

    let mut abi_mismatch = configuration();
    abi_mismatch.candidates = vec![candidate(PROVIDER, 1, 2, "foreign.task-load")];
    cases.push(("SES-K0205", "provider.abi-mismatch", abi_mismatch));

    let mut feature_mismatch = configuration();
    feature_mismatch.candidates = vec![candidate(PROVIDER, 1, 1, "foreign.unavailable")];
    cases.push((
        "SES-K0206",
        "provider.runtime-feature-mismatch",
        feature_mismatch,
    ));

    let mut conflict = configuration();
    conflict.transitive_requirements.push(RequiredService {
        requirement: ServiceRequirement {
            field: "clock".to_owned(),
            service: SERVICE.to_owned(),
        },
        contract_version: ContractVersion { major: 2, minor: 0 },
        traces: vec![trace()],
    });
    cases.push(("SES-K0207", "provider.requirement-conflict", conflict));

    let mut unavailable = configuration();
    unavailable.context.explicit.insert(
        SERVICE.to_owned(),
        "fixture/hidden-provider#clock".to_owned(),
    );
    cases.push(("SES-K0208", "provider.selection-unavailable", unavailable));

    let mut extension = configuration();
    extension
        .compatibility
        .target_extensions
        .push(TargetExtensionRequirement {
            extension: "browser".to_owned(),
            trace: trace(),
        });
    cases.push(("SES-K0209", "provider.extension-mismatch", extension));

    let mut runtime = configuration();
    runtime
        .compatibility
        .runtime_packages
        .push(RuntimePackageCompatibility {
            provider: PROVIDER.to_owned(),
            required_identity: "@seseragi/runtime@1.0.0".to_owned(),
            required_digest: "sha256:locked".to_owned(),
            actual_identity: "@seseragi/runtime@1.0.1".to_owned(),
            actual_digest: "sha256:actual".to_owned(),
            trace: trace(),
        });
    cases.push(("SES-K0210", "provider.runtime-mismatch", runtime));

    let mut compiler = configuration();
    compiler
        .compatibility
        .compiler_features
        .push(CompilerFeatureRequirement {
            provider: PROVIDER.to_owned(),
            required: BTreeSet::from(["provider-resource-v1".to_owned()]),
            supported: BTreeSet::new(),
            trace: trace(),
        });
    cases.push(("SES-K0211", "provider.compiler-mismatch", compiler));

    let mut conformance = configuration();
    conformance
        .compatibility
        .conformance
        .push(ProviderConformanceRequirement {
            provider: PROVIDER.to_owned(),
            required_profile: "clock-v1".to_owned(),
            required_digest: "sha256:required".to_owned(),
            actual_profile: None,
            actual_digest: None,
            trace: trace(),
        });
    cases.push(("SES-K0212", "provider.conformance-mismatch", conformance));

    for (code, label, configuration) in cases {
        let diagnostic = matching_diagnostic(SOURCE, configuration);
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.label, label);
    }
}

#[test]
fn shared_target_prefilter_rejects_once_before_provider_resolution() {
    let source = "pub effect fn main -> Unit with Dom =\n  succeed ()\n";
    let mut configuration = configuration();
    configuration.contracts.clear();
    configuration.candidates.clear();
    let diagnostic = matching_diagnostic(source, configuration);
    assert_eq!(diagnostic.code, "SES-K0203");
    assert_eq!(diagnostic.label, "provider.target-mismatch");
    assert_eq!(diagnostic.details.compatible_targets, ["browser"]);
}

#[test]
fn browser_only_standard_import_reports_the_import_range_on_process() {
    let source = r#"import * as file from "std/web/file"

pub effect fn main -> Unit = succeed ()
"#;
    let diagnostic = matching_diagnostic(source, configuration());
    assert_eq!(diagnostic.code, "SES-K0203");
    assert_eq!(diagnostic.label, "provider.target-mismatch");
    assert_eq!(diagnostic.details.required, ["std/web/file"]);
    assert_eq!(diagnostic.details.actual, ["process"]);
    assert_eq!(diagnostic.details.compatible_targets, ["browser"]);
    let trace = diagnostic
        .trace
        .expect("import diagnostic must retain a range");
    assert_eq!(trace.source, "src/main.ssrg");
    assert_eq!(trace.start, 0);
    assert!(trace.end > trace.start);
}

#[test]
fn successful_plan_is_recorded_before_lowered_modules_are_consumed() {
    let (graph, inputs) = graph_and_inputs(SOURCE);
    let compiled =
        compile_project_with_providers(graph.clone(), inputs.clone(), configuration()).unwrap();
    let analyzed = analyze_project_with_providers(graph, inputs, configuration()).unwrap();
    assert_eq!(
        compiled.provider_resolution.as_ref().unwrap().selected[0].provider,
        PROVIDER
    );
    assert_eq!(compiled.provider_resolution, analyzed.provider_resolution);
}
