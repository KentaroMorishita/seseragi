use seseragi_dts::{
    convert_entry, interpret_declaration, project_graph, ConversionInput, HostMetadata,
};
use seseragi_interop::*;
use std::collections::BTreeMap;

const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration = 'api.d.ts'\noutput = 'api'\nspecifier = './api.js'\nevaluation = 'pure'\n";

#[test]
fn generic_constraints_defaults_and_lexical_scopes_survive_json_and_projection() {
    let source = "export declare function identity<T extends string = string>(value: T): T; export declare function other<T extends number>(value: T): T;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let SemanticType::Generic { parameters, body } =
        &graph.types[&graph.symbols["identity"].type_ref]
    else {
        panic!()
    };
    assert!(matches!(
        graph.types[&parameters[0].constraint.unwrap()],
        SemanticType::Primitive {
            domain: PrimitiveDomain::String
        }
    ));
    assert!(matches!(
        graph.types[&parameters[0].default.unwrap()],
        SemanticType::Primitive {
            domain: PrimitiveDomain::String
        }
    ));
    let SemanticType::Callable {
        parameters: args,
        result,
        ..
    } = &graph.types[body]
    else {
        panic!()
    };
    for id in [args[0].type_ref, *result] {
        assert!(
            matches!(graph.types[&id], SemanticType::Parameter { binder, index: 0 } if binder == graph.symbols["identity"].type_ref)
        );
        let origins = graph.origins_for_type(id);
        assert_eq!(&source[origins[0].start..origins[0].end], "T");
    }
    assert_ne!(
        graph.symbols["identity"].type_ref,
        graph.symbols["other"].type_ref
    );
    let json = graph.to_json().unwrap();
    let restored = SemanticGraph::from_json_for(&json, graph.identity.as_ref().unwrap()).unwrap();
    let projection = project_graph(&restored, SETTINGS, "api").unwrap();
    assert!(projection.source.is_none());
    assert!(projection
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.message.contains("erase their meaning")));
    let unconstrained = interpret_declaration(
        "export declare function identity<T>(value: T): T;",
        "api.d.ts",
        SETTINGS,
    )
    .unwrap();
    assert_eq!(
        project_graph(&unconstrained, SETTINGS, "api")
            .unwrap()
            .source
            .unwrap(),
        "pub foreign \"typescript\" from \"./api.js\" {\n  pure fn identity<A> value: A -> A\n}\n"
    );
    let changed = SETTINGS.replace("evaluation = 'pure'", "evaluation = 'task'");
    assert!(project_graph(&restored, &changed, "api")
        .unwrap_err()
        .to_string()
        .contains("different binding settings"));
}

#[test]
fn overloads_namespaces_and_export_aliases_have_semantic_ownership() {
    let source = "declare function internal(x: string): string; export { internal as renamed }; export namespace Metrics { export function count(x: number): number; } export namespace Metrics { export function count(x: string): string; export function reset(): void; }";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    assert_eq!(graph.symbols["renamed"].declaration_name, "internal");
    let SemanticType::Namespace { members } = &graph.types[&graph.symbols["Metrics"].type_ref]
    else {
        panic!()
    };
    assert_eq!(
        members,
        &BTreeMap::from([
            ("count".to_owned(), "Metrics.count".to_owned()),
            ("reset".to_owned(), "Metrics.reset".to_owned())
        ])
    );
    let SemanticType::Overload { signatures } =
        &graph.types[&graph.symbols["Metrics.count"].type_ref]
    else {
        panic!()
    };
    assert_eq!(signatures.len(), 2);
    for id in signatures {
        assert!(matches!(graph.types[id], SemanticType::Callable { .. }));
        assert_eq!(graph.origins_for_type(*id)[0].rule, "callable-signature");
    }
    let projected = project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(projected.source.is_none());
    assert!(projected
        .diagnostics
        .iter()
        .any(|d| d.code == "SES-F0102" && d.symbol.as_deref() == Some("Metrics.count")));
}

#[test]
fn merged_declarations_keep_every_source_and_explain_the_remaining_structural_gap() {
    let source = "export interface User { readonly id: bigint; }\nexport interface User { readonly name: string; }";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let symbol = &graph.symbols["User"];
    assert_eq!(
        symbol
            .origins
            .iter()
            .filter(|origin| origin.rule == "declaration")
            .count(),
        2
    );
    let origins = graph.origins_for_type(symbol.type_ref);
    assert!(origins
        .iter()
        .any(|origin| source[origin.start..origin.end].contains("readonly id")));
    assert!(origins
        .iter()
        .any(|origin| source[origin.start..origin.end].contains("readonly name")));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.output == symbol.type_ref
            && !step.inputs.is_empty()
            && step.classification == SemanticClassification::Uninterpreted));
    assert_eq!(
        project_graph(&graph, SETTINGS, "api")
            .unwrap()
            .source
            .unwrap(),
        "pub foreign \"typescript\" from \"./api.js\" {\n  opaque type User\n}\n"
    );
}

#[test]
fn recursive_aliases_and_uninterpreted_types_are_not_erased_or_widened() {
    let source = "export type Recursive<T> = readonly Recursive<T>[]; export type Conditional<T> = T extends string ? string : number; export declare function unknownExternal(x: Missing): any;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    assert!(graph
        .types
        .values()
        .any(|ty| matches!(ty, SemanticType::Reference { symbol } if symbol == "Recursive")));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.classification == SemanticClassification::Uninterpreted));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.classification == SemanticClassification::RuntimeAbiUnresolved));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.classification == SemanticClassification::PolicyRequired));
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
    let converted = convert_entry(&ConversionInput {
        settings: SETTINGS,
        entry: "api",
        declaration: source,
        host: HostMetadata {
            manifest: "{}",
            resolved_package: None,
        },
        previous_metadata: None,
    })
    .unwrap();
    assert!(converted.generated.is_none());
    assert_eq!(converted.semantic, Some(graph));
    assert!(converted
        .diagnostics
        .iter()
        .any(|d| d.message.contains("`any` requires")));
}

#[test]
fn common_graph_can_be_constructed_and_projected_without_a_typescript_source() {
    let graph = SemanticGraph {
        types: BTreeMap::from([
            (
                TypeId(0),
                SemanticType::Primitive {
                    domain: PrimitiveDomain::String,
                },
            ),
            (
                TypeId(1),
                SemanticType::Callable {
                    parameters: vec![],
                    result: TypeId(0),
                    receiver: None,
                },
            ),
        ]),
        symbols: BTreeMap::from([(
            "read".to_owned(),
            SemanticSymbol {
                public_name: "read".to_owned(),
                declaration_name: "read".to_owned(),
                kind: SymbolKind::Callable,
                type_ref: TypeId(1),
                origins: vec![],
                runtime: None,
            },
        )]),
        ..SemanticGraph::default()
    };
    let projection = project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(projection.diagnostics.is_empty());
    assert_eq!(
        projection.source.unwrap(),
        "pub foreign \"typescript\" from \"./api.js\" {\n  pure fn read -> String\n}\n"
    );
}

#[test]
fn recursive_inline_graph_requires_an_explicit_boundary_instead_of_recursing_forever() {
    let graph = SemanticGraph {
        types: BTreeMap::from([
            (
                TypeId(0),
                SemanticType::Collection {
                    element: TypeId(0),
                    mutable: false,
                },
            ),
            (
                TypeId(1),
                SemanticType::Callable {
                    parameters: vec![],
                    result: TypeId(0),
                    receiver: None,
                },
            ),
        ]),
        symbols: BTreeMap::from([(
            "read".to_owned(),
            SemanticSymbol {
                public_name: "read".to_owned(),
                declaration_name: "read".to_owned(),
                kind: SymbolKind::Callable,
                type_ref: TypeId(1),
                origins: vec![],
                runtime: None,
            },
        )]),
        ..SemanticGraph::default()
    };
    let projection = project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(projection.source.is_none());
    assert!(projection
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("explicit opaque boundary")));
}

#[test]
fn declaration_interpretation_is_deterministic_and_retains_byte_provenance() {
    let source = "// 日本語 comment\nexport declare function read(value: string | null | undefined): PromiseLike<readonly number[]>;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    assert_eq!(
        graph.to_json().unwrap(),
        interpret_declaration(source, "api.d.ts", SETTINGS)
            .unwrap()
            .to_json()
            .unwrap()
    );
    for origins in graph.type_origins.values() {
        for origin in origins {
            assert!(source.is_char_boundary(origin.start));
            assert!(source.is_char_boundary(origin.end));
            assert!(!source[origin.start..origin.end].is_empty());
        }
    }
    let converted = convert_entry(&ConversionInput {
        settings: SETTINGS,
        entry: "api",
        declaration: source,
        host: HostMetadata {
            manifest: "{}",
            resolved_package: None,
        },
        previous_metadata: None,
    })
    .unwrap();
    assert_eq!(converted.semantic.as_ref().unwrap(), &graph);
    assert!(converted
        .generated
        .unwrap()
        .source
        .contains("task fn read value: Js.UndefinedOr<Js.NullOr<String>> -> Array<Float>"));
}
