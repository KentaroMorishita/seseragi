use seseragi_interop::*;
use std::collections::BTreeMap;

fn generic_graph() -> SemanticGraph {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(1),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "A".to_owned(),
                constraint: Some(TypeId(4)),
                default: None,
            }],
            body: TypeId(2),
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Callable {
            parameters: vec![CallParameter {
                name: "value".to_owned(),
                origins: vec![],
                type_ref: TypeId(3),
                optional: false,
                rest: false,
            }],
            result: TypeId(3),
            receiver: None,
        },
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Parameter {
            binder: TypeId(1),
            index: 0,
        },
    );
    graph.types.insert(
        TypeId(4),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "name".to_owned(),
                ShapeField {
                    type_ref: TypeId(5),
                    optional: false,
                    readonly: true,
                },
            )]),
        },
    );
    graph.types.insert(
        TypeId(5),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    graph.sources.insert(
        "schema/input".to_owned(),
        SourceIdentity {
            digest: "a".repeat(64),
            byte_length: 80,
        },
    );
    graph.symbols.insert(
        "schema::identity".to_owned(),
        SemanticSymbol {
            declaration_name: "identity".to_owned(),
            kind: SymbolKind::Type,
            public_name: "identity".to_owned(),
            type_ref: TypeId(1),
            origins: vec![Origin {
                source: "schema/input".to_owned(),
                start: 0,
                end: 80,
                rule: "public-callable".to_owned(),
            }],
            runtime: Some(RuntimeBinding {
                exact_identity: "package:example@1.0.0".to_owned(),
                export_path: vec!["identity".to_owned()],
                targets: vec!["browser".to_owned(), "node".to_owned(), "bun".to_owned()],
                convention: CallingConvention::Function,
                evaluation: Evaluation::Effectful,
                failure: FailureBoundary::Unresolved,
            }),
        },
    );
    graph
}

#[test]
fn generic_constraint_and_runtime_uncertainty_survive_portable_roundtrip() {
    let graph = generic_graph();
    let json = graph.to_json().unwrap();
    let restored = SemanticGraph::from_json(&json).unwrap();
    assert_eq!(restored, graph);
    assert_eq!(restored.to_json().unwrap(), json);
    assert_eq!(restored.digest().unwrap(), graph.digest().unwrap());
    assert!(json.contains("unresolved"));
    // Maps use canonical key order, independent of frontend insertion order.
    let mut reordered = graph.clone();
    reordered.types = graph.types.into_iter().rev().collect();
    assert_eq!(reordered.to_json().unwrap(), json);
}

#[test]
fn recursive_shape_references_are_valid_without_a_source_parser() {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(1),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "next".to_owned(),
                ShapeField {
                    type_ref: TypeId(2),
                    optional: true,
                    readonly: true,
                },
            )]),
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Reference {
            symbol: "schema::List".to_owned(),
        },
    );
    graph.symbols.insert(
        "schema::List".to_owned(),
        SemanticSymbol {
            declaration_name: "identity".to_owned(),
            kind: SymbolKind::Type,
            public_name: "List".to_owned(),
            type_ref: TypeId(1),
            origins: vec![],
            runtime: None,
        },
    );
    assert!(graph.validate().is_ok());
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn declaration_references_do_not_capture_another_declarations_type_parameters() {
    let mut graph = generic_graph();
    graph.symbols.insert(
        "schema::escaped".to_owned(),
        SemanticSymbol {
            declaration_name: "identity".to_owned(),
            kind: SymbolKind::Type,
            public_name: "escaped".to_owned(),
            type_ref: TypeId(3),
            origins: vec![],
            runtime: None,
        },
    );
    let error = graph.validate().unwrap_err();
    assert!(error.to_string().contains("escapes its lexical binder"));
    assert!(graph.to_json().is_err());
}

#[test]
fn malformed_references_and_generic_owners_are_rejected_before_projection() {
    let mut graph = generic_graph();
    graph.types.insert(
        TypeId(3),
        SemanticType::Parameter {
            binder: TypeId(5),
            index: 2,
        },
    );
    graph
        .types
        .insert(TypeId(6), SemanticType::Nullable { value: TypeId(999) });
    let error = graph.validate().unwrap_err().to_string();
    assert!(error.contains("matching generic binder"));
    assert!(error.contains("dangling type reference"));
}

#[test]
fn incompatible_schema_unknown_fields_and_invalid_provenance_fail_closed() {
    let graph = generic_graph();
    let mut json: serde_json::Value = serde_json::from_str(&graph.to_json().unwrap()).unwrap();
    json["schema"] = 2.into();
    assert!(SemanticGraph::from_json(&json.to_string()).is_err());
    let mut literal = serde_json::to_value(SemanticGraph::default()).unwrap();
    literal["types"]["0"] = serde_json::json!({"kind":"literal", "value":{"kind":"string", "value":"x", "unknownMandatory":true}});
    assert!(SemanticGraph::from_json(&literal.to_string()).is_err());
    json["schema"] = 1.into();
    json["mandatoryButUnknown"] = true.into();
    assert!(SemanticGraph::from_json(&json.to_string()).is_err());
    let mut graph = graph;
    graph.symbols.get_mut("schema::identity").unwrap().origins[0].end = 81;
    assert!(graph
        .validate()
        .unwrap_err()
        .to_string()
        .contains("provenance"));
}

#[test]
fn mutable_identity_cannot_be_labelled_as_a_readonly_snapshot() {
    let mut graph = generic_graph();
    let SemanticType::Shape { fields, .. } = graph.types.get_mut(&TypeId(4)).unwrap() else {
        panic!()
    };
    fields.get_mut("name").unwrap().readonly = false;
    assert!(graph
        .validate()
        .unwrap_err()
        .to_string()
        .contains("mutable field"));
    let SemanticType::Shape { identity, .. } = graph.types.get_mut(&TypeId(4)).unwrap() else {
        panic!()
    };
    *identity = ShapeIdentity::MutableIdentity;
    assert!(graph.validate().is_ok());
}

#[test]
fn projection_relations_keep_keys_and_reasons_explicit() {
    let mut graph = generic_graph();
    graph.types.insert(
        TypeId(6),
        SemanticType::Projection {
            relation: ProjectionRelation::Keys,
            subject: TypeId(4),
            key: None,
        },
    );
    graph.types.insert(
        TypeId(7),
        SemanticType::Unresolved {
            reason: "open domain requires a binding policy".to_owned(),
        },
    );
    assert!(graph.validate().is_ok());
    graph.types.insert(
        TypeId(6),
        SemanticType::Projection {
            relation: ProjectionRelation::Member,
            subject: TypeId(4),
            key: None,
        },
    );
    assert!(graph
        .validate()
        .unwrap_err()
        .to_string()
        .contains("projection key"));
}

#[test]
fn transformed_nodes_retain_multiple_origins_and_explicit_policy_decisions() {
    let mut graph = generic_graph();
    let origin = graph.symbols["schema::identity"].origins[0].clone();
    graph.type_origins.insert(TypeId(4), vec![origin.clone()]);
    graph.normalizations.push(Normalization {
        output: TypeId(5),
        inputs: vec![TypeId(4)],
        rule: "member-domain".to_owned(),
        classification: SemanticClassification::PolicyRequired,
        reason: "boundary decoder must be selected explicitly".to_owned(),
        policy: None,
    });
    assert_eq!(graph.origins_for_type(TypeId(5)), vec![origin]);
    let restored = SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap();
    assert_eq!(restored.normalizations, graph.normalizations);
    assert_eq!(
        restored.origins_for_type(TypeId(5)),
        graph.origins_for_type(TypeId(5))
    );
}

#[test]
fn cache_identity_rejects_changed_source_settings_producer_and_runtime_contract() {
    let mut graph = generic_graph();
    let identity = ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "schema-frontend".to_owned(),
            version: "1".to_owned(),
        },
        settings_digest: "b".repeat(64),
        source_digests: BTreeMap::from([("schema/input".to_owned(), "a".repeat(64))]),
        runtime_contract_digest: Some("c".repeat(64)),
    };
    graph.identity = Some(identity.clone());
    let json = graph.to_json().unwrap();
    assert_eq!(
        SemanticGraph::from_json_for(&json, &identity).unwrap(),
        graph
    );
    for changed in [
        ArtifactIdentity {
            settings_digest: "d".repeat(64),
            ..identity.clone()
        },
        ArtifactIdentity {
            frontend: ProducerIdentity {
                name: "schema-frontend".to_owned(),
                version: "2".to_owned(),
            },
            ..identity.clone()
        },
        ArtifactIdentity {
            runtime_contract_digest: Some("d".repeat(64)),
            ..identity.clone()
        },
        ArtifactIdentity {
            source_digests: BTreeMap::from([("schema/input".to_owned(), "d".repeat(64))]),
            ..identity.clone()
        },
    ] {
        assert!(SemanticGraph::from_json_for(&json, &changed).is_err());
    }
    graph.identity.as_mut().unwrap().source_digests.clear();
    assert!(graph.to_json().is_err());
}

#[test]
fn deep_and_cyclic_type_graphs_do_not_recurse_on_the_host_stack() {
    let mut graph = SemanticGraph::default();
    for id in 0..10_000 {
        graph.types.insert(
            TypeId(id),
            SemanticType::Optional {
                value: TypeId((id + 1) % 10_000),
            },
        );
    }
    graph.symbols.insert(
        "schema::cycle".to_owned(),
        SemanticSymbol {
            declaration_name: "identity".to_owned(),
            kind: SymbolKind::Type,
            public_name: "cycle".to_owned(),
            type_ref: TypeId(0),
            origins: vec![],
            runtime: None,
        },
    );
    assert!(graph.validate().is_ok());
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn namespace_ownership_cycles_and_non_callable_overloads_fail_closed() {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(0),
        SemanticType::Namespace {
            members: BTreeMap::from([("owner".to_owned(), "owner".to_owned())]),
        },
    );
    graph.symbols.insert(
        "owner".to_owned(),
        SemanticSymbol {
            public_name: "owner".to_owned(),
            declaration_name: "owner".to_owned(),
            kind: SymbolKind::Namespace,
            type_ref: TypeId(0),
            origins: vec![],
            runtime: None,
        },
    );
    assert!(graph
        .validate()
        .unwrap_err()
        .to_string()
        .contains("cyclic namespace ownership"));
    graph.symbols.clear();
    graph.types.clear();
    graph.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    graph.types.insert(
        TypeId(1),
        SemanticType::Overload {
            signatures: vec![TypeId(0)],
        },
    );
    assert!(graph
        .validate()
        .unwrap_err()
        .to_string()
        .contains("callable signatures"));
}

#[test]
fn integer_literal_wire_values_preserve_precision_on_javascript_hosts() {
    for value in [
        i64::MIN,
        -9_007_199_254_740_993,
        0,
        9_007_199_254_740_993,
        i64::MAX,
    ] {
        let mut graph = SemanticGraph::default();
        graph.types.insert(
            TypeId(0),
            SemanticType::Literal {
                value: LiteralValue::Integer(value),
            },
        );
        let json = graph.to_json().unwrap();
        let raw: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(raw["types"]["0"]["value"]["value"], value.to_string());
        assert_eq!(SemanticGraph::from_json(&json).unwrap(), graph);
    }
    let mut raw = serde_json::to_value(SemanticGraph::default()).unwrap();
    for value in [
        serde_json::json!(1),
        serde_json::json!("01"),
        serde_json::json!("-0"),
        serde_json::json!("9223372036854775808"),
    ] {
        raw["types"]["0"] =
            serde_json::json!({"kind":"literal", "value":{"kind":"integer", "value":value}});
        assert!(SemanticGraph::from_json(&raw.to_string()).is_err());
    }
}
