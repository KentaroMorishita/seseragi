use seseragi_interop::*;
use seseragi_schema::interpret_schema;

const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration = 'customer.schema.json'\noutput = 'customer'\nspecifier = './customer.js'\nevaluation = 'pure'\n";
const CUSTOMER: &str = include_str!("fixtures/customer.schema.json");
fn read(source: &str) -> SemanticGraph {
    interpret_schema(source, "customer.schema.json", "Customer", SETTINGS).unwrap()
}
fn root(graph: &SemanticGraph) -> &SemanticType {
    &graph.types[&graph.symbols["Customer"].type_ref]
}

#[test]
fn actual_json_schema_constructs_source_neutral_graph_and_reuses_same_projector() {
    let graph = read(CUSTOMER);
    let SemanticType::Shape {
        identity: ShapeIdentity::ReadonlySnapshot,
        fields,
    } = root(&graph)
    else {
        panic!("closed object should preserve its data fields")
    };
    assert!(!fields["id"].optional);
    assert!(fields["nickname"].optional);
    assert!(!fields["status"].optional);
    assert!(fields.values().all(|field| field.readonly));
    assert!(matches!(
        graph.types[&fields["tags"].type_ref],
        SemanticType::Collection { mutable: false, .. }
    ));
    assert!(
        matches!(&graph.types[&fields["status"].type_ref], SemanticType::Reference { symbol } if symbol == "SchemaDef0")
    );
    assert_eq!(
        graph.identity.as_ref().unwrap().frontend.name,
        "seseragi-json-schema"
    );
    assert!(graph.symbols.values().all(|s| s.runtime.is_none()));
    assert_eq!(
        graph.private_symbols,
        std::collections::BTreeSet::from(["SchemaDef0".to_owned()])
    );
    assert_eq!(
        SemanticCompatibilityReport::from_graph(&graph)
            .unwrap()
            .symbols
            .len(),
        1
    );
    let output = seseragi_dts::project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let generated = output.source.unwrap();
    assert!(generated.contains("Customer"));
    // The shared projector currently preserves data identities in its existing
    // foreign backend. The source frontend itself never imports that parser.
    assert!(!generated.contains("foreign fn"));
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
    assert_eq!(read(CUSTOMER).digest().unwrap(), graph.digest().unwrap());
}

#[test]
fn exact_literal_integer_preserves_more_than_binary64_precision() {
    let graph = read(r#"{"enum":[9007199254740993,-9007199254740993]}"#);
    let json = graph.to_json().unwrap();
    assert!(json.contains("9007199254740993"));
    assert!(graph.types.values().any(|n| matches!(
        n,
        SemanticType::Literal {
            value: LiteralValue::Integer(9007199254740993)
        }
    )));
    assert_eq!(SemanticGraph::from_json(&json).unwrap(), graph);
}

#[test]
fn unresolved_validation_is_reason_bearing_without_unsafe_domain_fallback() {
    for source in [
        r#"{"type":"object","properties":{"id":{"type":"string"}}}"#,
        r#"{"type":"integer"}"#,
        r#"{"type":"number"}"#,
        r#"{"type":"string","pattern":"^id$"}"#,
        r#"{"oneOf":[{"type":"string"},{"type":"string"}]}"#,
        r#"{"$ref":"https://untrusted.example/schema"}"#,
        r#"{"const":18446744073709551615}"#,
        r#"{"const":9007199254740993.0}"#,
    ] {
        let graph = read(source);
        assert!(
            matches!(root(&graph), SemanticType::Unresolved { reason } if !reason.is_empty()),
            "{source}: {graph:?}"
        );
        assert!(graph
            .normalizations
            .iter()
            .any(|s| s.classification == SemanticClassification::PolicyRequired));
        assert!(!graph.types.values().any(|n| matches!(
            n,
            SemanticType::Primitive {
                domain: PrimitiveDomain::Unknown | PrimitiveDomain::Dynamic
            }
        )));
    }
}

#[test]
fn duplicate_keys_and_invalid_schema_shapes_fail_closed() {
    for source in [
        r#"{"type":"string","type":"boolean"}"#,
        r#"{"type":"object","properties":{"x":{"type":"string","type":"boolean"}}}"#,
        r#"{"type":[]}"#,
        r#"{"type":["string","string"]}"#,
        r#"{"type":"object","additionalProperties":false,"required":["x","x"]}"#,
        r#"{"anyOf":[]}"#,
        r#"{"required":false,"type":"object","additionalProperties":false}"#,
        r#"{"$schema":"https://json-schema.org/draft-07/schema#"}"#,
        r#"{"$defs":[]}"#,
    ] {
        assert!(
            interpret_schema(source, "input.json", "Input", SETTINGS).is_err(),
            "{source}"
        );
    }
}

#[test]
fn definitions_references_are_indexed_before_use_and_recursive_meaning_is_preserved() {
    let graph = read(
        r##"{"$defs":{"Node":{"type":"object","additionalProperties":false,"properties":{"next":{"$ref":"#/$defs/Node"}},"required":[]}},"$ref":"#/$defs/Node"}"##,
    );
    graph.validate().unwrap();
    let definition = graph.symbols["SchemaDef0"].type_ref;
    let SemanticType::Shape { fields, .. } = &graph.types[&definition] else {
        panic!()
    };
    assert!(fields["next"].optional);
    assert!(
        matches!(&graph.types[&fields["next"].type_ref], SemanticType::Reference { symbol } if symbol == "SchemaDef0")
    );
    assert!(!graph.origins_for_type(fields["next"].type_ref).is_empty());
    assert!(graph
        .origins_for_type(fields["next"].type_ref)
        .iter()
        .any(|o| o.rule.contains("properties/next")));
}

#[test]
fn global_resource_scopes_disable_local_reference_assumptions() {
    let graph = read(
        r##"{"$defs":{"Scoped":{"$id":"https://example.test/scoped","type":"string"},"Reference":{"$ref":"#/$defs/Scoped"}},"$ref":"#/$defs/Scoped"}"##,
    );
    assert!(
        matches!(root(&graph), SemanticType::Unresolved { reason } if reason.contains("resource scopes"))
    );
}

#[test]
fn composition_keeps_constraints_and_applicators_do_not_narrow_without_type() {
    let incompatible = read(r#"{"type":"boolean","const":"text"}"#);
    match root(&incompatible) {
        SemanticType::Primitive {
            domain: PrimitiveDomain::Never,
        } => {}
        SemanticType::Intersection { members } => {
            assert_eq!(members.len(), 2);
            assert!(members.iter().any(|id| matches!(
                incompatible.types[id],
                SemanticType::Primitive {
                    domain: PrimitiveDomain::Boolean
                }
            )));
            assert!(members.iter().any(|id|matches!(&incompatible.types[id],SemanticType::Literal { value: LiteralValue::String(s) } if s == "text")));
        }
        other => panic!("incompatible constraints were silently lost: {other:?}"),
    }
    let other_domains =
        read(r#"{"properties":{"name":{"type":"string"}},"additionalProperties":false}"#);
    assert!(
        matches!(root(&other_domains),SemanticType::Unresolved { reason } if reason.contains("also accept other JSON domains"))
    );
    let impossible =
        read(r#"{"type":"object","additionalProperties":false,"required":["absent"]}"#);
    assert!(matches!(
        root(&impossible),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Never
        }
    ));
}

#[test]
fn bounded_inputs_and_exact_settings_identity_are_enforced() {
    let many = format!(
        "{{\"enum\":[{}]}}",
        (0..600)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(interpret_schema(&many, "input.json", "Input", SETTINGS)
        .unwrap_err()
        .to_string()
        .contains("budget"));
    assert!(interpret_schema(&" ".repeat(1_048_577), "input.json", "Input", SETTINGS).is_err());
    let graph = read(CUSTOMER);
    assert!(seseragi_dts::project_graph(&graph,"schema = 1\n[entries.api]\ndeclaration = 'different.json'\noutput = 'customer'\nspecifier = './customer.js'\nevaluation = 'pure'\n","api").is_err());
    assert!(graph
        .origins_for_type(graph.symbols["Customer"].type_ref)
        .iter()
        .all(|o| o.start == 0 && o.end == CUSTOMER.len()));
}
