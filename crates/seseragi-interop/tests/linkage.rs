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
            deprecations: Vec::new(),
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
fn make_contract(graph: &SemanticGraph) -> RuntimeLinkageContract {
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
            delivery: RuntimeDelivery::Synchronous,
            failure: FailureBoundary::Throw,
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
    let contract = make_contract(&graph);
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
    let mut contract = make_contract(&graph);
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
    let graph = nominal_graph(true);
    let mut contract = make_contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.semantic_owner = "api::Widget".into();
    abi.semantic_type = TypeId(14);
    abi.convention = CallingConvention::Method;
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    contract.exports.get_mut("api::run").unwrap().receiver = ReceiverAbi::FirstArgument;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.semantic_type = TypeId(11);
    abi.convention = CallingConvention::Constructor;
    abi.receiver = ReceiverAbi::None;
    abi.delivery = RuntimeDelivery::Promise;
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    contract.exports.get_mut("api::run").unwrap().delivery = RuntimeDelivery::Synchronous;
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
    let mut value = serde_json::to_value(make_contract(&graph())).unwrap();
    value["exports"]["api::run"]["cancellation"]["hidden"] = true.into();
    assert!(serde_json::from_value::<RuntimeLinkageContract>(value).is_err());
}

#[test]
fn callback_and_cancellation_indices_must_match_real_callable_signatures() {
    let graph = graph();
    let mut contract = make_contract(&graph);
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

fn nominal_graph(constructible: bool) -> SemanticGraph {
    let mut graph = graph();
    graph.types.insert(
        TypeId(10),
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            fields: BTreeMap::from([(
                "run".into(),
                ShapeField {
                    type_ref: TypeId(14),
                    optional: false,
                    readonly: false,
                },
            )]),
        },
    );
    graph.types.insert(
        TypeId(11),
        SemanticType::Callable {
            parameters: vec![],
            result: TypeId(12),
            receiver: None,
        },
    );
    graph.types.insert(
        TypeId(12),
        SemanticType::Nominal {
            identity: "api.Widget".into(),
            instance: TypeId(10),
            constructors: vec![TypeId(11)],
            constructible,
            static_members: BTreeMap::new(),
        },
    );
    graph.types.insert(
        TypeId(14),
        SemanticType::Callable {
            parameters: vec![],
            result: TypeId(1),
            receiver: Some(TypeId(12)),
        },
    );
    graph.symbols.insert(
        "api::Widget".into(),
        SemanticSymbol {
            deprecations: Vec::new(),
            public_name: "Widget".into(),
            declaration_name: "Widget".into(),
            kind: SymbolKind::Type,
            type_ref: TypeId(12),
            origins: vec![],
            runtime: None,
        },
    );
    graph.validate().unwrap();
    graph
}
#[test]
fn ownership_rejects_unrelated_callable_and_inaccessible_constructor() {
    let graph = nominal_graph(true);
    let mut contract = make_contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    contract.exports.get_mut("api::run").unwrap().semantic_type = TypeId(11);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("owned")));
    let hidden = nominal_graph(false);
    let mut contract = make_contract(&hidden);
    let abi = contract.exports.get_mut("api::run").unwrap();
    abi.semantic_owner = "api::Widget".into();
    abi.semantic_type = TypeId(11);
    abi.convention = CallingConvention::Constructor;
    assert!(contract
        .validate_for(&hidden, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("owned")));
}
#[test]
fn callback_parameter_and_delivery_must_match_actual_declared_callable() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    if let Some(SemanticType::Callable { parameters, .. }) = graph.types.get_mut(&TypeId(2)) {
        parameters.push(CallParameter {
            name: "cb".into(),
            origins: vec![],
            type_ref: TypeId(3),
            optional: false,
            rest: false,
        });
    }
    let mut contract = make_contract(&graph);
    contract
        .exports
        .get_mut("api::run")
        .unwrap()
        .callbacks
        .push(CallbackAbi {
            parameter_index: 0,
            delivery: RuntimeDelivery::Synchronous,
            lifetime: CallbackLifetime::DuringCall,
            failure: FailureBoundary::Throw,
        });
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("resolved callable parameter")));
    graph.types.insert(
        TypeId(4),
        SemanticType::Callable {
            parameters: vec![],
            result: TypeId(1),
            receiver: None,
        },
    );
    if let Some(SemanticType::Callable { parameters, .. }) = graph.types.get_mut(&TypeId(2)) {
        parameters[0].type_ref = TypeId(4);
    }
    contract = make_contract(&graph);
    contract
        .exports
        .get_mut("api::run")
        .unwrap()
        .callbacks
        .push(CallbackAbi {
            parameter_index: 0,
            delivery: RuntimeDelivery::Promise,
            lifetime: CallbackLifetime::DuringCall,
            failure: FailureBoundary::ThrowOrRejection,
        });
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    contract.exports.get_mut("api::run").unwrap().callbacks[0].delivery =
        RuntimeDelivery::Synchronous;
    contract.exports.get_mut("api::run").unwrap().callbacks[0].failure = FailureBoundary::Throw;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
}
#[test]
fn delivery_is_bidirectional_for_resolved_sync_and_async_results() {
    let mut graph = graph();
    let mut contract = make_contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    contract.exports.get_mut("api::run").unwrap().delivery = RuntimeDelivery::Promise;
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    graph
        .types
        .insert(TypeId(3), SemanticType::Async { value: TypeId(1) });
    if let Some(SemanticType::Callable { result, .. }) = graph.types.get_mut(&TypeId(2)) {
        *result = TypeId(3);
    }
    contract = make_contract(&graph);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    contract.exports.get_mut("api::run").unwrap().delivery = RuntimeDelivery::Promise;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
}

#[test]
fn private_declarations_and_unknown_delivery_do_not_acquire_runtime_abi_by_assumption() {
    let mut graph = graph();
    graph.private_symbols.insert("api::run".into());
    let contract = make_contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("private")));
    graph.private_symbols.clear();
    graph.types.insert(
        TypeId(1),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Unknown,
        },
    );
    let contract = make_contract(&graph);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|d| d.reason.contains("delivery ABI")));
}

#[test]
fn cancellation_requires_host_abort_signal_identity() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    if let SemanticType::Callable { parameters, .. } = graph.types.get_mut(&TypeId(2)).unwrap() {
        parameters.push(CallParameter {
            name: "signal".into(),
            origins: vec![],
            type_ref: TypeId(3),
            optional: false,
            rest: false,
        });
    }
    let mut contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().cancellation =
        CancellationAbi::AbortSignalParameter(0);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "SES-F0202"));
    graph.types.insert(
        TypeId(3),
        SemanticType::Opaque {
            identity: "AbortSignal".into(),
            arguments: vec![],
        },
    );
    contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().cancellation =
        CancellationAbi::AbortSignalParameter(0);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    graph.types.insert(
        TypeId(3),
        SemanticType::Opaque {
            identity: "web:AbortSignal".into(),
            arguments: vec![],
        },
    );
    contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().cancellation =
        CancellationAbi::AbortSignalParameter(0);
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
}

#[test]
fn runtime_package_must_match_declaration_name_and_version() {
    let graph = graph();
    for identity in ["npm:other@1.2.3/web.js", "npm:api@9.9.9/web.js"] {
        let mut contract = make_contract(&graph);
        let mut wrong = artifact();
        wrong.exact_identity = identity.into();
        contract
            .exports
            .get_mut("api::run")
            .unwrap()
            .targets
            .insert(RuntimeTarget::Browser, wrong.clone());
        let fresh = BTreeMap::from([("api::run".into(), wrong)]);
        assert!(contract
            .validate_for(&graph, RuntimeTarget::Browser, &fresh)
            .unwrap_err()
            .iter()
            .any(|error| error.code == "SES-F0201" && error.reason.contains("package")));
    }
}

#[test]
fn generic_transport_requires_an_upper_bound_and_ignores_default_only_parameters() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    graph.types.insert(
        TypeId(4),
        SemanticType::Parameter {
            binder: TypeId(5),
            index: 0,
        },
    );
    graph.types.insert(
        TypeId(5),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "T".into(),
                constraint: Some(TypeId(3)),
                default: Some(TypeId(3)),
            }],
            body: TypeId(2),
        },
    );
    graph.symbols.get_mut("api::run").unwrap().type_ref = TypeId(5);
    if let SemanticType::Callable { result, .. } = graph.types.get_mut(&TypeId(2)).unwrap() {
        *result = TypeId(4);
    }
    let contract = make_contract(&graph);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    graph.types.insert(
        TypeId(3),
        SemanticType::Shape {
            fields: BTreeMap::new(),
            identity: ShapeIdentity::MutableIdentity,
        },
    );
    assert!(
        make_contract(&graph)
            .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
            .is_err(),
        "object upper bounds admit Promise/thenable values"
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    if let SemanticType::Generic { parameters, .. } = graph.types.get_mut(&TypeId(5)).unwrap() {
        parameters[0].constraint = None;
    }
    let contract = make_contract(&graph);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap_err()
        .iter()
        .any(|error| error.code == "SES-F0202"));
    if let SemanticType::Generic { parameters, .. } = graph.types.get_mut(&TypeId(5)).unwrap() {
        parameters[0].default = None;
    }
    let contract = make_contract(&graph);
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
}

#[test]
fn conditional_codec_delivery_requires_owned_rank_one_result_and_stays_unresolved_in_report() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Parameter {
            binder: TypeId(4),
            index: 0,
        },
    );
    graph.types.insert(
        TypeId(4),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "A".into(),
                constraint: None,
                default: None,
            }],
            body: TypeId(2),
        },
    );
    graph.symbols.get_mut("api::run").unwrap().type_ref = TypeId(4);
    if let SemanticType::Callable { result, .. } = graph.types.get_mut(&TypeId(2)).unwrap() {
        *result = TypeId(3);
    }
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    assert!(make_contract(&graph)
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    let mut contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().delivery = RuntimeDelivery::CodecSelected;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    let mut report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    report.apply_runtime_linkage(
        &graph,
        &contract,
        &BTreeMap::from([(RuntimeTarget::Browser, artifacts.clone())]),
    );
    assert!(report.symbols["api::run"].runtime_abi_unresolved);
    assert!(report.symbols["api::run"]
        .reasons
        .iter()
        .any(|reason| reason.rule == "runtime-linkage-codec-selection"));
    contract.exports.get_mut("api::run").unwrap().failure = FailureBoundary::ThrowOrRejection;
    assert!(
        contract
            .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
            .is_err(),
        "conditional primitive transport cannot promise rejection"
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    assert!(
        contract
            .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
            .is_err(),
        "a non-parameter cannot acquire codec-selected delivery"
    );
}

#[test]
fn applied_generic_declared_body_controls_promise_delivery_and_rejects_wrong_arity() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "A".into(),
                constraint: None,
                default: None,
            }],
            body: TypeId(4),
        },
    );
    graph.types.insert(
        TypeId(4),
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            fields: BTreeMap::new(),
        },
    );
    graph.types.insert(
        TypeId(5),
        SemanticType::Apply {
            constructor: TypeId(3),
            arguments: vec![TypeId(1)],
        },
    );
    if let SemanticType::Callable { result, .. } = graph.types.get_mut(&TypeId(2)).unwrap() {
        *result = TypeId(5);
    }
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    make_contract(&graph)
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    graph
        .types
        .insert(TypeId(4), SemanticType::Async { value: TypeId(1) });
    assert!(make_contract(&graph)
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    let mut contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().delivery = RuntimeDelivery::Promise;
    contract.exports.get_mut("api::run").unwrap().failure = FailureBoundary::ThrowOrRejection;
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    if let SemanticType::Apply { arguments, .. } = graph.types.get_mut(&TypeId(5)).unwrap() {
        arguments.clear();
    }
    assert!(contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .is_err());
    if let SemanticType::Callable { result, .. } = graph.types.get_mut(&TypeId(2)).unwrap() {
        *result = TypeId(3);
    }
    assert_eq!(
        runtime_export_delivery(&graph, "api::run", TypeId(2)),
        Some(RuntimeDelivery::Promise),
        "standalone generic bodies must not fall through to synchronous"
    );
}

#[test]
fn wrapper_selected_callable_abi_keeps_homogeneous_overloads_and_rejects_mixed_delivery() {
    let mut graph = graph();
    graph
        .types
        .insert(TypeId(3), graph.types[&TypeId(2)].clone());
    graph.types.insert(
        TypeId(4),
        SemanticType::Overload {
            signatures: vec![TypeId(2), TypeId(3)],
        },
    );
    graph.symbols.get_mut("api::run").unwrap().type_ref = TypeId(4);
    let artifacts = BTreeMap::from([("api::run".into(), artifact())]);
    let mut contract = make_contract(&graph);
    contract.exports.get_mut("api::run").unwrap().semantic_type = TypeId(4);
    contract
        .validate_for(&graph, RuntimeTarget::Browser, &artifacts)
        .unwrap();
    graph.types.insert(
        TypeId(5),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "A".into(),
                constraint: None,
                default: None,
            }],
            body: TypeId(4),
        },
    );
    graph.symbols.get_mut("api::run").unwrap().type_ref = TypeId(5);
    assert_eq!(
        runtime_export_delivery(&graph, "api::run", TypeId(5)),
        Some(RuntimeDelivery::Synchronous)
    );
    graph
        .types
        .insert(TypeId(6), SemanticType::Async { value: TypeId(1) });
    if let SemanticType::Callable { result, .. } = graph.types.get_mut(&TypeId(3)).unwrap() {
        *result = TypeId(6);
    }
    assert_eq!(runtime_export_delivery(&graph, "api::run", TypeId(5)), None);
}
