use seseragi_dts::*;
use seseragi_interop::*;

const SOURCE: &str = "export type User = { readonly name: string; readonly active: boolean };";
const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration='host/api.d.ts'\nspecifier='./api.js'\noutput='api'\nevaluation='pure'\nprojection_policy='host/api.policy.json'\n";
fn input() -> ConversionInput<'static> {
    ConversionInput {
        settings: SETTINGS,
        entry: "api",
        declaration: SOURCE,
        host: HostMetadata {
            manifest: "",
            resolved_package: None,
        },
        previous_metadata: None,
    }
}
fn policy(graph: &SemanticGraph) -> ProjectionPolicySet {
    ProjectionPolicySet {
        schema: 1,
        kind: PROJECTION_POLICY_KIND.into(),
        producer: ProducerIdentity {
            name: "explicit-fixture-policy".into(),
            version: "1".into(),
        },
        graph_digest: graph.digest().unwrap(),
        artifact_identity: graph.identity.clone().unwrap(),
        policies: vec![ProjectionPolicy {
            selector: ProjectionSelector::Symbol {
                symbol: "User".into(),
            },
            function_name: "decodeUser".into(),
            mapping: AdapterMapping::JsonSnapshot,
        }],
    }
}
#[test]
fn separate_policy_emits_checked_api_and_shared_evidence_without_changing_meaning() {
    let mut result = convert_entry(&input()).unwrap();
    let graph = result.semantic.clone().unwrap();
    let entry = parse_bindings(SETTINGS)
        .unwrap()
        .entries
        .remove("api")
        .unwrap();
    apply_projection_policy(
        &mut result,
        &entry,
        &serde_json::to_string(&policy(&graph)).unwrap(),
    )
    .unwrap();
    assert_eq!(result.semantic.as_ref(), Some(&graph));
    let report = result.compatibility.as_ref().unwrap();
    let user = &report.symbols["User"];
    assert_eq!(user.semantic_classification, SemanticClassification::Direct);
    assert_eq!(user.classification, SemanticClassification::SafeBoundary);
    assert!(user
        .reasons
        .iter()
        .any(|reason| reason.rule == "opaque-foreign-type"));
    assert!(user
        .reasons
        .iter()
        .any(|reason| reason.rule == "applied-semantic-json-policy"
            && reason.policy.as_ref().unwrap().name == "explicit-fixture-policy"));
    let generated = result.generated.unwrap();
    assert!(generated.source.contains("opaque type User"));
    let adapters = generated.adapters.unwrap();
    assert_eq!(adapters.input_boundary, "std/json::Json");
    assert!(adapters.semantic_source.contains("pub fn decodeUser"));
    assert!(!adapters.semantic_source.contains("foreign \"typescript\""));
    let json: serde_json::Value = serde_json::from_str(&generated.report).unwrap();
    assert_eq!(json["compatibility"], serde_json::to_value(report).unwrap());
}
#[test]
fn stale_or_invalid_policy_has_its_own_source_diagnostic_and_no_partial_output() {
    let entry = parse_bindings(SETTINGS)
        .unwrap()
        .entries
        .remove("api")
        .unwrap();
    for source in ["bad JSON".to_owned(), {
        let mut set = policy(convert_entry(&input()).unwrap().semantic.as_ref().unwrap());
        set.graph_digest = "0".repeat(64);
        serde_json::to_string(&set).unwrap()
    }] {
        let mut result = convert_entry(&input()).unwrap();
        apply_projection_policy(&mut result, &entry, &source).unwrap();
        assert!(result.generated.is_none());
        let error = result
            .diagnostics
            .iter()
            .find(|d| d.code == "SES-F0105")
            .unwrap();
        assert_eq!(error.file, "host/api.policy.json");
        assert_eq!((error.start, error.end), (0, source.len()));
        assert_eq!(
            result.compatibility.unwrap().symbols["User"].classification,
            SemanticClassification::PolicyRequired
        );
    }
}
#[cfg(feature = "filesystem")]
#[test]
fn filesystem_publication_freshness_and_policy_removal_preserve_owned_boundary() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    std::fs::create_dir(package.join("host")).unwrap();
    std::fs::write(package.join("bindings.toml"), SETTINGS).unwrap();
    std::fs::write(package.join("host/api.d.ts"), SOURCE).unwrap();
    std::fs::write(package.join("host/package.json"), "{}").unwrap();
    let set = policy(convert_entry(&input()).unwrap().semantic.as_ref().unwrap());
    std::fs::write(
        package.join("host/api.policy.json"),
        serde_json::to_string(&set).unwrap(),
    )
    .unwrap();
    let request = ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    let result = convert_package(&request).unwrap();
    assert!(!result.has_errors());
    assert!(package.join("generated/api/semantic.ssrg").is_file());
    assert!(validate_generated_bindings(
        package,
        &request.generated_root,
        &request.bindings,
        &request.host_manifest
    )
    .is_ok());
    let mut changed = set;
    changed.policies[0].function_name = "decodeOther".into();
    std::fs::write(
        package.join("host/api.policy.json"),
        serde_json::to_string(&changed).unwrap(),
    )
    .unwrap();
    assert!(validate_generated_bindings(
        package,
        &request.generated_root,
        &request.bindings,
        &request.host_manifest
    )
    .unwrap_err()
    .iter()
    .any(|e| e.message.contains("stale")));
    convert_package(&request).unwrap();
    assert!(
        std::fs::read_to_string(package.join("generated/api/semantic.ssrg"))
            .unwrap()
            .contains("decodeOther")
    );
    let settings = SETTINGS.replace("projection_policy='host/api.policy.json'\n", "");
    std::fs::write(package.join("bindings.toml"), settings).unwrap();
    convert_package(&request).unwrap();
    assert!(!package.join("generated/api/semantic.ssrg").exists());
    assert!(!package.join("generated/api.semantic.json").exists());
}

#[cfg(feature = "filesystem")]
#[test]
fn handwritten_and_edited_semantic_files_survive_conversion_and_policy_removal() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path();
    std::fs::create_dir_all(package.join("host")).unwrap();
    std::fs::create_dir_all(package.join("generated/api")).unwrap();
    std::fs::write(package.join("host/api.d.ts"), SOURCE).unwrap();
    std::fs::write(package.join("host/package.json"), "{}").unwrap();
    let without_policy = SETTINGS.replace("projection_policy='host/api.policy.json'\n", "");
    std::fs::write(package.join("bindings.toml"), &without_policy).unwrap();
    let semantic = package.join("generated/api/semantic.ssrg");
    std::fs::write(&semantic, "handwritten module\n").unwrap();
    let request = ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    assert!(!convert_package(&request).unwrap().has_errors());
    assert_eq!(
        std::fs::read_to_string(&semantic).unwrap(),
        "handwritten module\n"
    );
    std::fs::write(package.join("bindings.toml"), SETTINGS).unwrap();
    let set = policy(convert_entry(&input()).unwrap().semantic.as_ref().unwrap());
    std::fs::write(
        package.join("host/api.policy.json"),
        serde_json::to_string(&set).unwrap(),
    )
    .unwrap();
    let collision = convert_package(&request).unwrap();
    assert!(collision.has_errors());
    assert!(collision
        .diagnostics
        .iter()
        .any(|error| error.code == "SES-F0106"));
    assert_eq!(
        std::fs::read_to_string(&semantic).unwrap(),
        "handwritten module\n"
    );
    // Removing our test's handwritten file permits the first owned publication.
    std::fs::remove_file(&semantic).unwrap();
    assert!(!convert_package(&request).unwrap().has_errors());
    std::fs::write(&semantic, "edited owned module\n").unwrap();
    assert!(convert_package(&request).unwrap().has_errors());
    assert_eq!(
        std::fs::read_to_string(&semantic).unwrap(),
        "edited owned module\n"
    );
    std::fs::write(package.join("bindings.toml"), without_policy).unwrap();
    let removed = convert_package(&request).unwrap();
    assert!(removed.has_errors());
    assert_eq!(
        std::fs::read_to_string(&semantic).unwrap(),
        "edited owned module\n"
    );
    assert!(package.join("generated/api.semantic.json").is_file());
}

#[cfg(feature = "filesystem")]
#[test]
fn removed_and_renamed_policy_outputs_remain_on_disk_but_cannot_be_imported() {
    use std::fs;
    let directory = tempfile::tempdir().unwrap();
    let package = directory.path();
    fs::create_dir_all(package.join("host")).unwrap();
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(package.join("seseragi.toml"), "[package]\nname='fixture/policy-ownership'\nversion='1.0.0'\nlanguage='^0.1.0'\n[layout]\ngenerated='generated'\n[run]\nentry='main'\n[foreign.typescript]\nresolver='node'\nmanifest='host/package.json'\nlockfile='host/bun.lock'\nbindings='bindings.toml'\n").unwrap();
    fs::write(package.join("host/package.json"), "{}").unwrap();
    fs::write(package.join("host/bun.lock"), "{}").unwrap();
    fs::write(package.join("host/api.d.ts"), SOURCE).unwrap();
    let request = ConvertRequest {
        package_root: package.into(),
        generated_root: package.join("generated"),
        bindings: "bindings.toml".into(),
        host_manifest: "host/package.json".into(),
        entry: None,
    };
    let publish = |settings: &str| {
        fs::write(package.join("bindings.toml"), settings).unwrap();
        let graph = interpret_declaration(SOURCE, "host/api.d.ts", settings).unwrap();
        fs::write(
            package.join("host/api.policy.json"),
            serde_json::to_string(&policy(&graph)).unwrap(),
        )
        .unwrap();
        let outcome = convert_package(&request).unwrap();
        assert!(!outcome.has_errors(), "{:?}", outcome.diagnostics);
    };
    publish(SETTINGS);
    let main = package.join("src/main.ssrg");
    fs::write(
        &main,
        "import {decodeUser} from \"gen/api/semantic\"\npub let main = decodeUser\n",
    )
    .unwrap();
    assert!(seseragi_project::load_local_project(package).is_ok());
    let old_semantic = package.join("generated/api/semantic.ssrg");
    let old_source = fs::read_to_string(&old_semantic).unwrap();
    fs::write(package.join("bindings.toml"), "schema=1\n[entries]\n").unwrap();
    let removed = seseragi_project::load_local_project(package).unwrap_err();
    assert_eq!(removed.code(), "SES-F0103");
    assert!(removed.to_string().contains("removed entry"));
    assert_eq!(fs::read_to_string(&old_semantic).unwrap(), old_source);
    publish(&SETTINGS.replace("output='api'", "output='next'"));
    let renamed = seseragi_project::load_local_project(package).unwrap_err();
    assert_eq!(renamed.code(), "SES-F0103");
    assert!(renamed.to_string().contains("renamed output"));
    assert_eq!(fs::read_to_string(&old_semantic).unwrap(), old_source);
    fs::write(
        &main,
        "import {decodeUser} from \"gen/next/semantic\"\npub let main = decodeUser\n",
    )
    .unwrap();
    assert!(seseragi_project::load_local_project(package).is_ok());
    let current_semantic = package.join("generated/next/semantic.ssrg");
    let current_source = fs::read_to_string(&current_semantic).unwrap();
    fs::write(
        &current_semantic,
        format!("{current_source}\n// manual edit\n"),
    )
    .unwrap();
    let edited = seseragi_project::load_local_project(package).unwrap_err();
    assert_eq!(edited.code(), "SES-F0103");
    assert!(edited.to_string().contains("has been edited"));
    fs::write(&current_semantic, current_source).unwrap();
    // User-written generated modules carry no converter ownership claim.
    fs::write(
        package.join("generated/manual.ssrg"),
        "pub let value: Int = 7\n",
    )
    .unwrap();
    fs::write(
        &main,
        "import {value} from \"gen/manual\"\npub let main = value\n",
    )
    .unwrap();
    assert!(seseragi_project::load_local_project(package).is_ok());
    // An old raw generated binding also cannot bypass the removed entry check.
    fs::write(
        &main,
        "import * as api from \"gen/api\"\npub let main: Int = 1\n",
    )
    .unwrap();
    assert_eq!(
        seseragi_project::load_local_project(package)
            .unwrap_err()
            .code(),
        "SES-F0103"
    );
}
