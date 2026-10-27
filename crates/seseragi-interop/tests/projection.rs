use seseragi_interop::*;
use std::collections::{BTreeMap, BTreeSet};
fn graph() -> SemanticGraph {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    graph.types.insert(
        TypeId(1),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([(
                "name".into(),
                ShapeField {
                    type_ref: TypeId(0),
                    optional: false,
                    readonly: true,
                },
            )]),
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            fields: BTreeMap::new(),
        },
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Opaque {
            identity: "host-class".into(),
            arguments: vec![],
        },
    );
    graph
        .types
        .insert(TypeId(4), SemanticType::Nullable { value: TypeId(1) });
    graph.types.insert(
        TypeId(5),
        SemanticType::Generic {
            parameters: vec![GenericParameter {
                name: "A".into(),
                constraint: None,
                default: None,
            }],
            body: TypeId(6),
        },
    );
    graph.types.insert(
        TypeId(6),
        SemanticType::Opaque {
            identity: "Schema".into(),
            arguments: vec![TypeId(7)],
        },
    );
    graph.types.insert(
        TypeId(7),
        SemanticType::Parameter {
            binder: TypeId(5),
            index: 0,
        },
    );
    graph.types.insert(
        TypeId(8),
        SemanticType::Literal {
            value: LiteralValue::String("a".into()),
        },
    );
    graph.types.insert(
        TypeId(9),
        SemanticType::Literal {
            value: LiteralValue::String("b".into()),
        },
    );
    for (id, tag) in [(10, 8), (11, 9)] {
        graph.types.insert(
            TypeId(id),
            SemanticType::Shape {
                identity: ShapeIdentity::ReadonlySnapshot,
                fields: BTreeMap::from([(
                    "kind".into(),
                    ShapeField {
                        type_ref: TypeId(tag),
                        optional: false,
                        readonly: true,
                    },
                )]),
            },
        );
    }
    graph.types.insert(
        TypeId(12),
        SemanticType::Union {
            members: vec![TypeId(10), TypeId(11)],
        },
    );
    for (name, id) in [("Snapshot", 1), ("Host", 3), ("Schema", 5), ("Tagged", 12)] {
        graph.symbols.insert(
            name.into(),
            SemanticSymbol {
                deprecations: Vec::new(),
                public_name: name.into(),
                declaration_name: name.into(),
                kind: SymbolKind::Type,
                type_ref: TypeId(id),
                origins: vec![],
                runtime: None,
            },
        );
    }
    graph.identity = Some(ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "fixture".into(),
            version: "1".into(),
        },
        settings_digest: "0".repeat(64),
        source_digests: BTreeMap::new(),
        runtime_contract_digest: None,
    });
    graph
}
fn set(graph: &SemanticGraph) -> ProjectionPolicySet {
    ProjectionPolicySet {
        schema: 1,
        kind: PROJECTION_POLICY_KIND.into(),
        producer: ProducerIdentity {
            name: "projection-policy".into(),
            version: "1".into(),
        },
        graph_digest: graph.digest().unwrap(),
        artifact_identity: graph.identity.clone().unwrap(),
        policies: vec![ProjectionPolicy {
            selector: ProjectionSelector::Symbol {
                symbol: "Snapshot".into(),
            },
            function_name: "decodeSnapshot".into(),
            mapping: AdapterMapping::JsonSnapshot,
        }],
    }
}
#[test]
fn immutable_decisions_preserve_identity_generic_relations_and_explicit_tag_candidates() {
    let graph = graph();
    let before = graph.clone();
    let plan = projection_plan(&graph).unwrap();
    assert_eq!(graph, before);
    for id in [2, 3] {
        assert!(matches!(
            plan.iter()
                .find(|p| p.type_ref == TypeId(id))
                .unwrap()
                .representation,
            ProjectionRepresentation::Opaque { .. }
        ));
    }
    assert!(
        matches!(&plan.iter().find(|p|p.type_ref==TypeId(5)).unwrap().representation,ProjectionRepresentation::GenericRelation {parameters,body} if parameters[0].name=="A" && *body==TypeId(6))
    );
    assert_eq!(
        tagged_discriminators(&graph, &[TypeId(10), TypeId(11)]),
        vec!["kind"]
    );
    assert!(matches!(
        plan.iter()
            .find(|p| p.type_ref == TypeId(12))
            .unwrap()
            .representation,
        ProjectionRepresentation::TaggedDomainCandidate { .. }
    ));
}
#[test]
fn policy_is_exact_fresh_scoped_and_never_overrides_identity_proof() {
    let graph = graph();
    let policy = set(&graph);
    let result = validate_projection_policies(&graph, &policy).unwrap();
    assert_eq!(result[0].input_boundary, "std/json::Json");
    let mut changed = graph.clone();
    changed.identity.as_mut().unwrap().settings_digest = "1".repeat(64);
    assert!(validate_projection_policies(&changed, &policy).is_err());
    let mut changed = graph.clone();
    changed.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Boolean,
        },
    );
    assert!(validate_projection_policies(&changed, &policy).is_err());
    for id in [2, 3] {
        let mut policy = policy.clone();
        policy.policies[0].selector = ProjectionSelector::Type {
            type_ref: TypeId(id),
        };
        assert!(validate_projection_policies(&graph, &policy).is_err());
    }
    let mut unsafe_name = policy.clone();
    unsafe_name.policies[0].function_name = "decode; foreign".into();
    assert!(validate_projection_policies(&graph, &unsafe_name).is_err());
    let mut duplicate = policy.clone();
    duplicate.policies.push(policy.policies[0].clone());
    assert!(validate_projection_policies(&graph, &duplicate).is_err());
    let mut private = graph.clone();
    private.private_symbols = BTreeSet::from(["Snapshot".into()]);
    assert!(validate_projection_policies(&private, &set(&private)).is_err());
}
#[test]
fn ambiguous_tags_and_unknown_policy_fields_fail_closed() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(9),
        SemanticType::Literal {
            value: LiteralValue::String("a".into()),
        },
    );
    assert!(tagged_discriminators(&graph, &[TypeId(10), TypeId(11)]).is_empty());
    let mut policy = set(&graph);
    policy.policies[0].selector = ProjectionSelector::Type {
        type_ref: TypeId(12),
    };
    policy.policies[0].mapping = AdapterMapping::JsonTaggedUnion {
        discriminator: "kind".into(),
    };
    assert!(validate_projection_policies(&graph, &policy).is_err());
    let mut json = serde_json::to_value(&policy).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("unsafeCast".into(), true.into());
    assert!(serde_json::from_value::<ProjectionPolicySet>(json).is_err());
}
