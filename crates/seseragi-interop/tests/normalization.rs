use seseragi_interop::*;
use std::collections::BTreeMap;

fn graph() -> SemanticGraph {
    SemanticGraph {
        types: BTreeMap::from([
            (
                TypeId(0),
                SemanticType::Primitive {
                    domain: PrimitiveDomain::String,
                },
            ),
            (
                TypeId(1),
                SemanticType::Shape {
                    identity: ShapeIdentity::ReadonlySnapshot,
                    fields: BTreeMap::from([(
                        "name".to_owned(),
                        ShapeField {
                            type_ref: TypeId(0),
                            optional: true,
                            readonly: true,
                        },
                    )]),
                },
            ),
            (
                TypeId(2),
                SemanticType::Projection {
                    relation: ProjectionRelation::Keys,
                    subject: TypeId(1),
                    key: None,
                },
            ),
            (
                TypeId(3),
                SemanticType::Literal {
                    value: LiteralValue::String("name".to_owned()),
                },
            ),
            (
                TypeId(4),
                SemanticType::Projection {
                    relation: ProjectionRelation::Member,
                    subject: TypeId(1),
                    key: Some(TypeId(3)),
                },
            ),
        ]),
        ..SemanticGraph::default()
    }
}

#[test]
fn source_independent_closed_relations_keep_original_meaning_and_optional_domain() {
    let mut graph = graph();
    graph
        .normalize_closed(NormalizationBudget::default())
        .unwrap();
    assert_eq!(
        graph.types[&TypeId(2)],
        SemanticType::Literal {
            value: LiteralValue::String("name".to_owned())
        }
    );
    assert_eq!(
        graph.types[&TypeId(4)],
        SemanticType::Optional { value: TypeId(0) }
    );
    for id in [TypeId(2), TypeId(4)] {
        let step = graph
            .normalizations
            .iter()
            .find(|step| step.output == id)
            .unwrap();
        assert_eq!(step.classification, SemanticClassification::Normalized);
        assert!(matches!(
            graph.types[&step.inputs[0]],
            SemanticType::Projection { .. }
        ));
    }
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn bounded_normalization_preserves_unresolved_relations_and_reports_exhaustion() {
    let mut graph = graph();
    graph
        .normalize_closed(NormalizationBudget {
            steps: 0,
            domain_members: 1,
        })
        .unwrap();
    assert!(matches!(
        graph.types[&TypeId(2)],
        SemanticType::Projection { .. }
    ));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.output == TypeId(2)
            && step.rule == "normalization-budget"
            && step.classification == SemanticClassification::PolicyRequired));
    // A self-recursive reference cannot masquerade as a closed shape.
    graph.types.insert(
        TypeId(1),
        SemanticType::Reference {
            symbol: "Loop".to_owned(),
        },
    );
    graph.symbols.insert(
        "Loop".to_owned(),
        SemanticSymbol {
            public_name: "Loop".to_owned(),
            declaration_name: "Loop".to_owned(),
            kind: SymbolKind::Type,
            type_ref: TypeId(1),
            origins: vec![],
            runtime: None,
        },
    );
    graph
        .normalize_closed(NormalizationBudget::default())
        .unwrap();
    assert!(matches!(
        graph.types[&TypeId(4)],
        SemanticType::Projection { .. }
    ));
}

#[test]
fn mutable_identity_and_ambiguous_intersection_are_never_snapshot_normalized() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(5),
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            fields: BTreeMap::new(),
        },
    );
    graph.types.insert(
        TypeId(6),
        SemanticType::Intersection {
            members: vec![TypeId(1), TypeId(5)],
        },
    );
    graph
        .normalize_closed(NormalizationBudget::default())
        .unwrap();
    assert!(matches!(
        graph.types[&TypeId(6)],
        SemanticType::Intersection { .. }
    ));
}
