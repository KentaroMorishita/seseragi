use seseragi_dts::{
    attach_runtime_linkage, convert_entry, parse_bindings, ConversionInput, HostMetadata,
};
use seseragi_interop::{RuntimeArtifactIdentity, RuntimeLinkageEvidence, RuntimeTarget};
use std::collections::BTreeMap;

fn converted(declaration: &str) -> (seseragi_dts::ConversionResult, seseragi_dts::EntryConfig) {
    let settings =
        "schema=1\n[entries.api]\ndeclaration=\"api.d.ts\"\nspecifier=\"api\"\noutput=\"api\"\n";
    let result = convert_entry(&ConversionInput {
        settings,
        entry: "api",
        declaration,
        host: HostMetadata {
            manifest: "{\"name\":\"api\",\"version\":\"1.0.0\"}",
            resolved_package: None,
        },
        previous_metadata: None,
    })
    .unwrap();
    (
        result,
        parse_bindings(settings)
            .unwrap()
            .entries
            .remove("api")
            .unwrap(),
    )
}
fn artifact() -> RuntimeArtifactIdentity {
    RuntimeArtifactIdentity {
        exact_identity: "npm:api@1.0.0/index.js".into(),
        content_digest: format!("sha256:{}", "a".repeat(64)),
    }
}
#[test]
fn emitted_callable_persists_exact_host_evidence_and_report_identity() {
    let (mut result, entry) =
        converted("export declare function run(value: string): Promise<string>;");
    assert!(result.generated.is_some());
    attach_runtime_linkage(
        &mut result,
        &entry,
        &BTreeMap::from([(RuntimeTarget::Node, artifact())]),
    )
    .unwrap();
    let metadata: serde_json::Value =
        serde_json::from_str(&result.generated.as_ref().unwrap().metadata).unwrap();
    let evidence: RuntimeLinkageEvidence =
        serde_json::from_value(metadata["runtimeLinkage"].clone()).unwrap();
    let fresh = evidence
        .contract
        .exports
        .keys()
        .map(|key| (key.clone(), artifact()))
        .collect();
    evidence
        .contract
        .validate_for(&evidence.graph, RuntimeTarget::Node, &fresh)
        .unwrap();
    assert!(evidence
        .contract
        .validate_for(&evidence.graph, RuntimeTarget::Browser, &fresh)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "SES-F0203"));
    let report: serde_json::Value =
        serde_json::from_str(&result.generated.as_ref().unwrap().report).unwrap();
    assert_eq!(
        report["compatibility"],
        serde_json::to_value(result.compatibility.as_ref().unwrap()).unwrap()
    );
    assert!(!result.compatibility.as_ref().unwrap().symbols["run"].runtime_abi_unresolved);
    assert_eq!(
        result.compatibility.as_ref().unwrap().graph_digest,
        evidence.graph.digest().unwrap()
    );
    assert_ne!(
        evidence.contract.exports.values().next().unwrap().targets[&RuntimeTarget::Node]
            .content_digest
            .trim_start_matches("sha256:"),
        evidence.contract.declaration.source_digests["api.d.ts"]
    );
}
#[test]
fn declaration_only_conversion_preserves_artifacts_without_runtime_claim() {
    let (mut result, entry) = converted("export declare function run(): string;");
    let original = result.clone();
    attach_runtime_linkage(&mut result, &entry, &BTreeMap::new()).unwrap();
    assert_eq!(result, original);
}
#[test]
fn actual_native_host_bytes_are_pinned_and_changed_bytes_are_stale() {
    use std::{fs, path::Path};
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/playground/tests/fixtures/interop-base64-js-1.5.1");
    let temp = tempfile::tempdir().unwrap();
    let mapping: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(fixture.join("workspace-files.json")).unwrap())
            .unwrap();
    for file in mapping {
        let destination = temp.path().join(file["path"].as_str().unwrap());
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(fixture.join(file["fixture"].as_str().unwrap()), destination).unwrap();
    }
    let request = seseragi_dts::ConvertRequest {
        package_root: temp.path().to_owned(),
        generated_root: temp.path().join(".seseragi/generated"),
        bindings: "seseragi.bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    let converted = seseragi_dts::convert_package(&request).unwrap();
    assert!(!converted.has_errors(), "{:?}", converted.diagnostics);
    let raw_path = &converted.converted[0].source;
    let original_raw = fs::read_to_string(raw_path).unwrap();
    fs::write(
        raw_path,
        format!("{original_raw}\npub fn injected value: String -> String = value\n"),
    )
    .unwrap();
    assert!(
        seseragi_dts::validate_generated_bindings(
            temp.path(),
            &request.generated_root,
            &request.bindings,
            &request.host_manifest
        )
        .is_err(),
        "hand-edited generated source must not bypass persisted ABI"
    );
    fs::write(raw_path, original_raw).unwrap();
    let text = fs::read_to_string(&converted.converted[0].metadata).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let evidence: RuntimeLinkageEvidence =
        serde_json::from_value(value["runtimeLinkage"].clone()).unwrap();
    assert_eq!(evidence.contract.exports.len(), 3);
    let resolve = || {
        seseragi_project::resolve_foreign_typescript_module_for_target(
            temp.path(),
            seseragi_project::parse_manifest(
                &fs::read_to_string(temp.path().join("seseragi.toml")).unwrap(),
            )
            .unwrap()
            .foreign_typescript
            .as_ref(),
            &converted.converted[0].source,
            "base64-js",
            RuntimeTarget::Node,
        )
        .unwrap()
    };
    let original = resolve();
    assert_eq!(
        evidence.contract.exports.values().next().unwrap().targets[&RuntimeTarget::Node]
            .content_digest,
        original.content_digest()
    );
    fs::write(
        temp.path().join("host/node_modules/base64-js/index.js"),
        "exports.byteLength = () => 999;",
    )
    .unwrap();
    let changed = resolve();
    let fresh = evidence
        .contract
        .exports
        .keys()
        .map(|key| {
            (
                key.clone(),
                RuntimeArtifactIdentity {
                    exact_identity: changed.exact_identity().into(),
                    content_digest: changed.content_digest().into(),
                },
            )
        })
        .collect();
    assert!(evidence
        .contract
        .validate_for(&evidence.graph, RuntimeTarget::Node, &fresh)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "SES-F0201"));
    assert!(seseragi_project::validate_binding_runtime_linkage(
        &text,
        RuntimeTarget::Node,
        true,
        |_| Ok(RuntimeArtifactIdentity {
            exact_identity: changed.exact_identity().into(),
            content_digest: changed.content_digest().into()
        })
    )
    .is_err());
}

#[test]
fn conditional_generic_delivery_keeps_explicit_call_site_report_requirement() {
    let (mut result, entry) = converted("export declare function run<T>(value: T): T;");
    attach_runtime_linkage(
        &mut result,
        &entry,
        &BTreeMap::from([(RuntimeTarget::Node, artifact())]),
    )
    .unwrap();
    assert!(result.generated.is_some());
    let metadata: serde_json::Value =
        serde_json::from_str(&result.generated.as_ref().unwrap().metadata).unwrap();
    let evidence: RuntimeLinkageEvidence =
        serde_json::from_value(metadata["runtimeLinkage"].clone()).unwrap();
    assert!(evidence
        .contract
        .exports
        .values()
        .all(|export| export.delivery == seseragi_interop::RuntimeDelivery::CodecSelected));
    let fresh = evidence
        .contract
        .exports
        .keys()
        .map(|key| (key.clone(), artifact()))
        .collect();
    evidence
        .contract
        .validate_for(&evidence.graph, RuntimeTarget::Node, &fresh)
        .unwrap();
    assert!(result.generated.is_some());
    let symbol = &result.compatibility.as_ref().unwrap().symbols["run"];
    assert!(symbol.runtime_abi_unresolved);
    assert!(symbol
        .reasons
        .iter()
        .any(|reason| reason.rule == "runtime-linkage-codec-selection"
            && reason.reason.contains("SES-F0202")));
    let report: serde_json::Value =
        serde_json::from_str(&result.generated.as_ref().unwrap().report).unwrap();
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["code"] == "SES-F0202"));
}

#[test]
fn policy_reuses_reported_final_identity_and_stales_when_actual_runtime_bytes_change() {
    use seseragi_interop::{
        AdapterMapping, ProducerIdentity, ProjectionPolicy, ProjectionPolicySet,
        ProjectionSelector, SemanticClassification, PROJECTION_POLICY_KIND,
    };
    use std::{fs, path::Path};
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/playground/tests/fixtures/interop-base64-js-1.5.1");
    let temp = tempfile::tempdir().unwrap();
    let mapping: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(fixture.join("workspace-files.json")).unwrap())
            .unwrap();
    for file in mapping {
        let destination = temp.path().join(file["path"].as_str().unwrap());
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(fixture.join(file["fixture"].as_str().unwrap()), destination).unwrap();
    }
    // This controlled local declaration adds a snapshot type to the callable
    // fixture; no claim is made that this type is published by base64-js.
    let declaration_path = temp.path().join("host/node_modules/base64-js/index.d.ts");
    fs::write(
        &declaration_path,
        format!(
            "{}\nexport type User = {{ readonly name: string; readonly active: boolean }};\n",
            fs::read_to_string(&declaration_path).unwrap()
        ),
    )
    .unwrap();
    let settings_path = temp.path().join("seseragi.bindings.toml");
    let settings = format!(
        "{}\nprojection_policy = \"host/checked.policy.json\"\n",
        fs::read_to_string(&settings_path).unwrap()
    );
    fs::write(&settings_path, settings).unwrap();
    fs::write(temp.path().join("host/checked.policy.json"), "{}").unwrap();
    let request = seseragi_dts::ConvertRequest {
        package_root: temp.path().to_owned(),
        generated_root: temp.path().join(".seseragi/generated"),
        bindings: "seseragi.bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    let first = seseragi_dts::convert_package(&request).unwrap();
    assert!(first.has_errors());
    let report = &first.compatibility["base64"];
    assert!(report
        .identity
        .as_ref()
        .unwrap()
        .runtime_contract_digest
        .is_some());
    let policy = ProjectionPolicySet {
        schema: 1,
        kind: PROJECTION_POLICY_KIND.into(),
        producer: ProducerIdentity {
            name: "explicit-runtime-fixture".into(),
            version: "1".into(),
        },
        graph_digest: report.graph_digest.clone(),
        artifact_identity: report.identity.clone().unwrap(),
        policies: vec![ProjectionPolicy {
            selector: ProjectionSelector::Symbol {
                symbol: "User".into(),
            },
            function_name: "decodeUser".into(),
            mapping: AdapterMapping::JsonSnapshot,
        }],
    };
    fs::write(
        temp.path().join("host/checked.policy.json"),
        serde_json::to_string(&policy).unwrap(),
    )
    .unwrap();
    let converted = seseragi_dts::convert_package(&request).unwrap();
    assert!(!converted.has_errors(), "{:?}", converted.diagnostics);
    assert_eq!(
        converted.compatibility["base64"].symbols["User"].classification,
        SemanticClassification::SafeBoundary
    );
    assert!(!converted.compatibility["base64"].symbols["byteLength"].runtime_abi_unresolved);
    let adapters: seseragi_projection::SemanticAdapters = serde_json::from_str(
        &fs::read_to_string(request.generated_root.join("base64.semantic.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        adapters.graph_digest,
        converted.compatibility["base64"].graph_digest
    );
    assert!(adapters.semantic_source.contains("pub fn decodeUser"));
    let checked_source =
        fs::read_to_string(request.generated_root.join("base64/semantic.ssrg")).unwrap();
    assert_eq!(checked_source, adapters.semantic_source);
    seseragi_dts::validate_generated_bindings(
        temp.path(),
        &request.generated_root,
        &request.bindings,
        &request.host_manifest,
    )
    .unwrap();
    let repeated = seseragi_dts::convert_package(&request).unwrap();
    assert!(!repeated.has_errors(), "{:?}", repeated.diagnostics);
    assert_eq!(
        repeated.compatibility["base64"].graph_digest,
        policy.graph_digest
    );
    let runtime = temp.path().join("host/node_modules/base64-js/index.js");
    fs::write(
        &runtime,
        format!("{}\n", fs::read_to_string(&runtime).unwrap()),
    )
    .unwrap();
    let stale = seseragi_dts::convert_package(&request).unwrap();
    assert!(stale.converted.is_empty());
    assert_ne!(
        stale.compatibility["base64"].graph_digest,
        policy.graph_digest
    );
    assert_ne!(
        stale.compatibility["base64"]
            .identity
            .as_ref()
            .unwrap()
            .runtime_contract_digest,
        policy.artifact_identity.runtime_contract_digest
    );
    assert_eq!(
        fs::read_to_string(request.generated_root.join("base64/semantic.ssrg")).unwrap(),
        checked_source
    );
    assert!(stale
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-F0105"));
    assert!(seseragi_dts::validate_generated_bindings(
        temp.path(),
        &request.generated_root,
        &request.bindings,
        &request.host_manifest
    )
    .is_err());
}
