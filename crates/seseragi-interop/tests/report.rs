use seseragi_interop::*;
use std::collections::BTreeMap;

fn symbol(graph: &mut SemanticGraph, name: &str, type_ref: TypeId) {
    graph.symbols.insert(
        name.to_owned(),
        SemanticSymbol {
            public_name: name.to_owned(),
            declaration_name: name.to_owned(),
            kind: SymbolKind::Type,
            type_ref,
            origins: vec![],
            runtime: None,
        },
    );
}
fn graph() -> SemanticGraph {
    let mut graph = SemanticGraph::default();
    graph.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::String,
        },
    );
    symbol(&mut graph, "Text", TypeId(0));
    graph
}
#[test]
fn counts_public_symbols_not_orphan_nodes_and_keep_reasons_identity_and_diff() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(99),
        SemanticType::Unresolved {
            reason: "old history".to_owned(),
        },
    );
    let before = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(before.types.len(), 1);
    assert_eq!(before.counts.iter().map(|c| c.count).sum::<usize>(), 1);
    assert_eq!(before.counts[0].symbols, vec!["Text"]);
    graph.types.insert(
        TypeId(1),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Dynamic,
        },
    );
    symbol(&mut graph, "Unsafe", TypeId(1));
    graph.normalizations.push(Normalization {
        output: TypeId(1),
        inputs: vec![],
        rule: "fake-direct".to_owned(),
        classification: SemanticClassification::Direct,
        reason: "producer says compatible".to_owned(),
        policy: None,
    });
    let after = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        after.symbols["Unsafe"].classification,
        SemanticClassification::PolicyRequired
    );
    assert_eq!(after.counts[0].count, 1);
    let diff = after.diff(&before);
    assert_eq!(diff.added, vec!["Unsafe"]);
    assert!(diff.changed.is_empty());
    assert!(after
        .render_text()
        .contains("dynamic domain requires explicit unsafe approval"));
    assert_eq!(
        after.to_json().unwrap(),
        SemanticCompatibilityReport::from_graph(&graph)
            .unwrap()
            .to_json()
            .unwrap()
    );
}
#[test]
fn child_unsafe_and_recursive_references_propagate_without_stack_recursion() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(1),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Dynamic,
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: BTreeMap::from([
                (
                    "unsafe".to_owned(),
                    ShapeField {
                        type_ref: TypeId(1),
                        optional: false,
                        readonly: true,
                    },
                ),
                (
                    "recursive".to_owned(),
                    ShapeField {
                        type_ref: TypeId(3),
                        optional: false,
                        readonly: true,
                    },
                ),
            ]),
        },
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Reference {
            symbol: "Recursive".to_owned(),
        },
    );
    symbol(&mut graph, "Recursive", TypeId(2));
    let report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        report.symbols["Recursive"].classification,
        SemanticClassification::PolicyRequired
    );
    assert_eq!(
        report.symbols["Recursive"].types,
        vec![TypeId(1), TypeId(2), TypeId(3)]
    );
}
#[test]
fn normalized_history_does_not_count_obsolete_policy_as_live_failure() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(99),
        SemanticType::Unresolved {
            reason: "old relation".to_owned(),
        },
    );
    graph.normalizations.push(Normalization {
        output: TypeId(99),
        inputs: vec![],
        rule: "old-open-policy".to_owned(),
        classification: SemanticClassification::PolicyRequired,
        reason: "old relation was open".to_owned(),
        policy: None,
    });
    graph.normalizations.push(Normalization {
        output: TypeId(0),
        inputs: vec![TypeId(99)],
        rule: "closed-normalization".to_owned(),
        classification: SemanticClassification::Normalized,
        reason: "closed evidence selects String".to_owned(),
        policy: Some(ProducerIdentity {
            name: "closed-core".to_owned(),
            version: "1".to_owned(),
        }),
    });
    let report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        report.symbols["Text"].classification,
        SemanticClassification::Normalized
    );
    assert_eq!(report.types.len(), 1);
    assert_eq!(
        report.symbols["Text"].reasons[0]
            .policy
            .as_ref()
            .unwrap()
            .name,
        "closed-core"
    );
}
#[test]
fn runtime_uncertainty_is_separate_from_interpreted_callable_shape() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(1),
        SemanticType::Callable {
            parameters: vec![],
            result: TypeId(0),
            receiver: None,
        },
    );
    graph.symbols.insert(
        "read".to_owned(),
        SemanticSymbol {
            public_name: "read".to_owned(),
            declaration_name: "read".to_owned(),
            kind: SymbolKind::Callable,
            type_ref: TypeId(1),
            origins: vec![],
            runtime: None,
        },
    );
    let before = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        before.symbols["read"].classification,
        SemanticClassification::Direct
    );
    assert!(before.symbols["read"].runtime_abi_unresolved);
    graph.symbols.get_mut("read").unwrap().runtime = Some(RuntimeBinding {
        exact_identity: "fixture@1/hash".to_owned(),
        export_path: vec!["read".to_owned()],
        targets: vec!["web".to_owned()],
        convention: CallingConvention::Function,
        evaluation: Evaluation::PureApproved,
        failure: FailureBoundary::CannotFailApproved,
    });
    let after = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert!(!after.symbols["read"].runtime_abi_unresolved);
    assert_eq!(after.diff(&before).changed, vec!["read"]);
}

#[test]
fn private_definitions_do_not_inflate_public_counts_but_their_meaning_is_traversed() {
    let mut graph = graph();
    graph.private_symbols.insert("Text".to_owned());
    graph.types.insert(
        TypeId(1),
        SemanticType::Reference {
            symbol: "Text".to_owned(),
        },
    );
    symbol(&mut graph, "Public", TypeId(1));
    let report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(report.symbols.len(), 1);
    assert_eq!(report.counts[0].symbols, vec!["Public"]);
    assert_eq!(report.symbols["Public"].types, vec![TypeId(0), TypeId(1)]);
}
#[test]
fn projection_outcomes_keep_meaning_separate_and_cannot_spoof_unsafe_coverage() {
    let mut graph = graph();
    let evidence = ProjectionEvidence {
        symbol: "Text".to_owned(),
        classification: SemanticClassification::OpaquePreserved,
        rule: "opaque-foreign-type".to_owned(),
        reason: "rendered as nominal identity".to_owned(),
        policy: Some(ProducerIdentity {
            name: "binding-settings".to_owned(),
            version: "sha256:a".to_owned(),
        }),
    };
    let report = SemanticCompatibilityReport::from_projection(&graph, &[evidence.clone()]).unwrap();
    assert_eq!(
        report.symbols["Text"].semantic_classification,
        SemanticClassification::Direct
    );
    assert_eq!(
        report.symbols["Text"].projection_classification,
        Some(SemanticClassification::OpaquePreserved)
    );
    assert_eq!(
        report.symbols["Text"].classification,
        SemanticClassification::OpaquePreserved
    );
    graph.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Dynamic,
        },
    );
    let report = SemanticCompatibilityReport::from_projection(
        &graph,
        &[ProjectionEvidence {
            classification: SemanticClassification::Direct,
            ..evidence
        }],
    )
    .unwrap();
    assert_eq!(
        report.symbols["Text"].classification,
        SemanticClassification::PolicyRequired
    );
}

#[test]
fn nominal_or_open_relations_cannot_be_forged_as_direct_by_producer_steps() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(1),
        SemanticType::Opaque {
            identity: "foreign::Value".to_owned(),
            arguments: vec![],
        },
    );
    symbol(&mut graph, "Nominal", TypeId(1));
    graph.normalizations.push(Normalization {
        output: TypeId(1),
        inputs: vec![],
        rule: "pretend-direct".to_owned(),
        classification: SemanticClassification::Direct,
        reason: "producer labels opaque as direct".to_owned(),
        policy: None,
    });
    assert_eq!(
        SemanticCompatibilityReport::from_graph(&graph)
            .unwrap()
            .symbols["Nominal"]
            .classification,
        SemanticClassification::OpaquePreserved
    );
}

#[test]
fn known_open_policy_remains_distinct_from_uninterpreted_and_forged_direct() {
    let mut graph = graph();
    graph.types.insert(
        TypeId(0),
        SemanticType::Unresolved {
            reason: "open conditional retains operand evidence".to_owned(),
        },
    );
    graph.normalizations.push(Normalization {
        output: TypeId(0),
        inputs: vec![],
        rule: "open-domain-selection".to_owned(),
        classification: SemanticClassification::PolicyRequired,
        reason: "requires explicit conditional policy".to_owned(),
        policy: None,
    });
    assert_eq!(
        SemanticCompatibilityReport::from_graph(&graph)
            .unwrap()
            .symbols["Text"]
            .classification,
        SemanticClassification::PolicyRequired
    );
    graph.normalizations[0].classification = SemanticClassification::Direct;
    assert_eq!(
        SemanticCompatibilityReport::from_graph(&graph)
            .unwrap()
            .symbols["Text"]
            .classification,
        SemanticClassification::Uninterpreted
    );
}

#[test]
fn same_classification_and_ids_still_report_a_changed_value_domain() {
    let mut graph = graph();
    let before = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    graph.types.insert(
        TypeId(0),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Boolean,
        },
    );
    let after = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        before.symbols["Text"].classification,
        after.symbols["Text"].classification
    );
    assert_ne!(
        before.symbols["Text"].semantic_digest,
        after.symbols["Text"].semantic_digest
    );
    assert_eq!(after.diff(&before).changed, vec!["Text"]);
    graph.types.insert(
        TypeId(99),
        SemanticType::Primitive {
            domain: PrimitiveDomain::BigInteger,
        },
    );
    let orphan = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        orphan.symbols["Text"].semantic_digest,
        after.symbols["Text"].semantic_digest
    );
    assert!(orphan.diff(&after).changed.is_empty());
}

#[test]
fn reference_target_edges_change_digest_even_when_reachable_node_set_is_equal() {
    let mut graph = graph();
    graph.private_symbols.insert("Text".into());
    graph.types.insert(
        TypeId(1),
        SemanticType::Reference {
            symbol: "Text".into(),
        },
    );
    graph.types.insert(
        TypeId(2),
        SemanticType::Primitive {
            domain: PrimitiveDomain::Boolean,
        },
    );
    graph.types.insert(
        TypeId(3),
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields: [("a", 0), ("b", 2), ("alias", 1)]
                .into_iter()
                .map(|(name, id)| {
                    (
                        name.to_owned(),
                        ShapeField {
                            type_ref: TypeId(id),
                            optional: false,
                            readonly: true,
                        },
                    )
                })
                .collect(),
        },
    );
    symbol(&mut graph, "Public", TypeId(3));
    let before = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    graph.symbols.get_mut("Text").unwrap().type_ref = TypeId(2);
    let after = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    assert_eq!(
        before.symbols["Public"].types,
        after.symbols["Public"].types
    );
    assert_eq!(after.diff(&before).changed, vec!["Public"]);
}
