use seseragi_interop::*;
use std::collections::BTreeMap;
fn graph() -> SemanticGraph {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(1),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Unit,
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Callable {
            parameters: vec![],
            result: TypeId(1),
            receiver: None,
        },
    );
    graph.symbols.insert(
        "api::run".into(),
        SemanticSymbol {
            public_name: "run".into(),
            declaration_name: "run".into(),
            kind: SymbolKind::Callable,
            type_ref: TypeId(2),
            origins: vec![],
            runtime: None,
        },
    );
    graph.identity = Some(ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "schema-adapter".into(),
            version: "1".into(),
        },
        settings_digest: "a".repeat(64),
        source_digests: BTreeMap::new(),
        runtime_contract_digest: None,
    });
    graph
}
fn artifact() -> RuntimeArtifactIdentity {
    RuntimeArtifactIdentity {
        exact_identity: "npm:api@1.2.3/web.js".into(),
        content_digest: format!("sha256:{}", "b".repeat(64)),
    }
}
fn contract(graph: &SemanticGraph) -> RuntimeLinkageContract {
    let mut contract = RuntimeLinkageContract::new(graph).unwrap();
    contract.exports.insert(
        "api::run".into(),
        RuntimeExportAbi {
            semantic_owner: "api::run".into(),
            semantic_type: TypeId(2),
            specifier: "api".into(),
            declaration_module_identity: "api@1.2.3".into(),
            export_path: vec!["run".into()],
            convention: CallingConvention::Function,
            receiver: ReceiverAbi::None,
            evaluation: Evaluation::Effectful,
            delivery: RuntimeDelivery::Promise,
            failure: FailureBoundary::ThrowOrRejection,
            cancellation: CancellationAbi::Unsupported,
            callbacks: vec![],
            targets: BTreeMap::from([(RuntimeTarget::Browser, artifact())]),
        },
    );
    contract
}
#[test]
fn binds_declarations_and_runtime_bytes_without_digest_cycle() {
    let mut graph = graph();
    let contract = contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    graph.identity.as_mut().unwrap().runtime_contract_digest = Some(contract.digest().unwrap());
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    let mut stale = artifacts;
    stale.get_mut("api::run").unwrap().content_digest = format!("sha256:{}", "c".repeat(64));
    assert_eq!(
        contract
            .validate_for(&graph, RuntimeTarget::Browser, &stale)
            .unwrap_err()[0]
            .code,
        "SES-F0201"
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &BTreeMap::new())
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("declaration")));
}
#[test]
fn distinguishes_abi_target_and_semantic_boundaries() {
    let graph = graph();
    let mut contract = contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    assert_eq!(
        contract
            .validate_for(&graph, RuntimeTarget::Node, &artifacts)
            .unwrap_err()[0]
            .code,
        "SES-F0203"
    );
    contract.exports.get_mut("api::run").unwrap().failure = FailureBoundary::Unresolved;
    assert_eq!(
        contract
            .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
            .unwrap_err()[0]
            .code,
        "SES-F0202"
    );
    assert!(graph.validate().is_ok());
}
#[test]
fn requires_method_receiver_and_synchronous_constructor() {
    let graph = graph();
    let mut contract = contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.convention = CallingConvention::Method;
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    contract.exports.get_mut("api::run").unwrap().receiver = ReceiverAbi::FirstArgument;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.convention = CallingConvention::Constructor;
    abi.receiver = ReceiverAbi::None;
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.delivery = RuntimeDelivery::Synchronous;
    abi.failure = FailureBoundary::Throw;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
}
#[test]
fn conditional_exports_preserve_manifest_order_and_nested_target_conditions() {
    let source = r#"{"name":"api","version":"1.2.3","exports":{".":{"browser":{"import":"./web.js"},"node":"./node.js","bun":"./bun.js","default":"./other.js"}}}"#;
    assert_eq!(
        select_package_export(source, "api", None, RuntimeTarget::Browser)
            .unwrap()
            .path,
        "./web.js"
    );
    assert_eq!(
        select_package_export(source, "api", None, RuntimeTarget::Node)
            .unwrap()
            .path,
        "./node.js"
    );
    // Bun accepts node conditions, so an earlier node branch wins per package order.
    assert_eq!(
        select_package_export(source, "api", None, RuntimeTarget::Bun)
            .unwrap()
            .path,
        "./node.js"
    );
    let source = r#"{"name":"api","version":"1","exports":{"default":"./universal.js","browser":"./web.js"}}"#;
    assert_eq!(
        select_package_export(source, "api", None, RuntimeTarget::Browser)
            .unwrap()
            .path,
        "./universal.js"
    );
}
#[test]
fn unsupported_paths_and_host_conditions_fail_without_polyfills() {
    for export in [
        r#"{"node":"./server.js"}"#,
        r#"null"#,
        r#""./../escape.js""#,
        r#""./node_modules/escape.js""#,
        r#""./%2e%2e/escape.js""#,
        r#""other-package""#,
        r#"{"browser":"./web.js","browser":"./other.js"}"#,
    ] {
        let source = format!(r#"{{"name":"api","version":"1","exports":{export}}}"#);
        let error =
            select_package_export(&source, "api", None, RuntimeTarget::Browser).unwrap_err();
        assert_eq!(error.code, "SES-F0203");
        assert_eq!(error.target, Some(RuntimeTarget::Browser));
    }
}
#[test]
fn wire_contract_rejects_unknown_abi_fields() {
    let mut value = serde_json::to_value(contract(&graph())).unwrap();
    value["exports"]["api::run"]["cancellation"]["hidden"] = true.into();
    assert!(serde_json::from_value::<RuntimeLinkageContract>(value).is_err());
}

#[test]
fn callback_and_cancellation_indices_must_match_real_callable_signatures() {
    let graph = graph();
    let mut contract = contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    contract
        .exports
        .get_mut("api::run")
        .unwrap()
        .callbacks
        .push(CallbackAbi {
            parameter_index: 0,
            delivery: RuntimeDelivery::Synchronous,
            lifetime: CallbackLifetime::Retained,
            failure: FailureBoundary::CannotFailApproved,
        });
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("parameter ABI")));
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.callbacks.clear();
    abi.cancellation = CancellationAbi::AbortSignalParameter(0);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("parameter ABI")));
}
