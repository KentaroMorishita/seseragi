use seseragi_dts::{
    convert_entry, interpret_declaration, project_graph, ConversionInput, HostMetadata,
};
use seseragi_interop::*;
use std::collections::BTreeMap;

const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration = 'api.d.ts'\noutput = 'api'\nspecifier = './api.js'\nevaluation = 'pure'\n";

#[test]
fn projections_extraction_and_callback_binders_keep_relations_without_widening() {
    let source = "export type Keys<T> = keyof T; export type Member<T, K extends keyof T> = T[K]; export type Element<T> = T extends Array<infer U> ? U : never; export type Callback<T> = <T extends string>(value: T) => T; export type Snapshot = { readonly id: string; readonly name?: string }; export type Identity = { value: string };";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    for (symbol, relation) in [
        ("Keys", ProjectionRelation::Keys),
        ("Member", ProjectionRelation::Member),
        ("Element", ProjectionRelation::Extract),
    ] {
        let SemanticType::Generic { body, .. } = graph.types[&graph.symbols[symbol].type_ref]
        else {
            panic!()
        };
        assert!(
            matches!(graph.types[&body], SemanticType::Projection { relation: r, .. } if r == relation)
        );
    }
    let SemanticType::Generic { body: inner, .. } =
        graph.types[&graph.symbols["Callback"].type_ref]
    else {
        panic!()
    };
    let SemanticType::Generic { body, .. } = graph.types[&inner] else {
        panic!()
    };
    let SemanticType::Callable {
        parameters, result, ..
    } = &graph.types[&body]
    else {
        panic!()
    };
    for id in [parameters[0].type_ref, *result] {
        assert!(
            matches!(graph.types[&id], SemanticType::Parameter { binder, index: 0 } if binder == inner)
        );
    }
    assert!(matches!(
        graph.types[&graph.symbols["Snapshot"].type_ref],
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            ..
        }
    ));
    assert!(matches!(
        graph.types[&graph.symbols["Identity"].type_ref],
        SemanticType::Shape {
            identity: ShapeIdentity::MutableIdentity,
            ..
        }
    ));
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn closed_conditional_domains_normalize_and_open_or_unsafe_domains_explain_policy() {
    let source = "export type Yes = 1 extends number ? string : boolean; export type No = string extends number ? string : boolean; export type Open<T> = T extends string ? string : number; export type Unsafe = any extends number ? string : boolean; export type Both = string & number; export type ReadonlyTuple = readonly [string, number];";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    for (symbol, expected) in [
        ("Yes", PrimitiveDomain::String),
        ("No", PrimitiveDomain::Boolean),
    ] {
        assert_eq!(
            graph.types[&graph.symbols[symbol].type_ref],
            SemanticType::Primitive { domain: expected }
        );
    }
    assert!(matches!(
        graph.types[&graph.symbols["Both"].type_ref],
        SemanticType::Primitive {
            domain: PrimitiveDomain::Never
        }
    ));
    assert!(matches!(
        graph.types[&graph.symbols["Unsafe"].type_ref],
        SemanticType::Unresolved { .. }
    ));
    assert!(
        matches!(&graph.types[&graph.symbols["ReadonlyTuple"].type_ref], SemanticType::Unresolved { reason } if reason.contains("readonly qualifier"))
    );
    assert!(graph.normalizations.iter().any(|step| step.classification
        == SemanticClassification::Normalized
        && step.inputs.len() == 4));
    assert!(graph.normalizations.iter().any(|step| step.classification
        == SemanticClassification::PolicyRequired
        && step.inputs.len() == 4));
}

#[test]
fn closed_keys_indexed_access_and_extraction_preserve_trace_and_optional_semantics() {
    let source = "export type Key = keyof { readonly id: string; name?: number }; export type Value = { readonly id: string }[\"id\"]; export type Maybe = { name?: number }[\"name\"]; export type Extracted = Array<string> extends Array<infer U> ? U : never;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let SemanticType::Union { members } = &graph.types[&graph.symbols["Key"].type_ref] else {
        panic!()
    };
    let names = members
        .iter()
        .map(|id| match &graph.types[id] {
            SemanticType::Literal {
                value: LiteralValue::String(s),
            } => s.as_str(),
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["id", "name"]);
    assert_eq!(
        graph.types[&graph.symbols["Value"].type_ref],
        SemanticType::Primitive {
            domain: PrimitiveDomain::String
        }
    );
    assert!(matches!(
        graph.types[&graph.symbols["Maybe"].type_ref],
        SemanticType::Optional { .. }
    ));
    assert_eq!(
        graph.types[&graph.symbols["Extracted"].type_ref],
        SemanticType::Primitive {
            domain: PrimitiveDomain::String
        }
    );
    for symbol in ["Key", "Value", "Maybe", "Extracted"] {
        assert!(graph.normalizations.iter().any(|step| step.output
            == graph.symbols[symbol].type_ref
            && step.classification == SemanticClassification::Normalized
            && step
                .inputs
                .iter()
                .any(|id| matches!(graph.types[id], SemanticType::Projection { .. }))));
        assert!(!graph
            .origins_for_type(graph.symbols[symbol].type_ref)
            .is_empty());
    }
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn contextual_receiver_never_becomes_an_unbound_plain_function_or_callback() {
    let source = "export declare function read(this: { value: string }, key: string): string; export declare function accept(callback: (this: { value: string }, key: string) => string): void;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let SemanticType::Callable {
        parameters,
        receiver,
        ..
    } = &graph.types[&graph.symbols["read"].type_ref]
    else {
        panic!()
    };
    assert_eq!(parameters.len(), 1);
    assert!(receiver.is_some());
    let projection = project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(projection.source.is_none());
    assert_eq!(
        projection
            .diagnostics
            .iter()
            .filter(|d| d.message.contains("bound-method ABI"))
            .count(),
        2
    );
}

#[test]
fn finite_templates_and_mapped_shapes_keep_exact_domains_and_modifiers() {
    let source = "export type Events = `on${'read' | 'write'}${1 | 2}`; export type Flags = { readonly [K in 'id' | 'name']?: string }; export type Names = { [K in 'id' | 'name']: K }; export type Open<T> = { readonly [K in keyof T]: T[K] }; export type Homomorphic = { [K in keyof { readonly id?: string }]: string }; export type Large = `prefix${string}`; export type Quoted = 'it\\'s \\\"fine\\\"';";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let SemanticType::Union { members } = &graph.types[&graph.symbols["Events"].type_ref] else {
        panic!()
    };
    let actual = members
        .iter()
        .map(|id| match &graph.types[id] {
            SemanticType::Literal {
                value: LiteralValue::String(text),
            } => text.as_str(),
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, ["onread1", "onread2", "onwrite1", "onwrite2"]);
    let SemanticType::Shape { identity, fields } = &graph.types[&graph.symbols["Flags"].type_ref]
    else {
        panic!()
    };
    assert_eq!(*identity, ShapeIdentity::ReadonlySnapshot);
    assert!(fields
        .values()
        .all(|field| field.readonly && field.optional));
    let SemanticType::Shape { identity, fields } = &graph.types[&graph.symbols["Names"].type_ref]
    else {
        panic!()
    };
    assert_eq!(*identity, ShapeIdentity::MutableIdentity);
    for (key, field) in fields {
        assert_eq!(
            graph.types[&field.type_ref],
            SemanticType::Literal {
                value: LiteralValue::String(key.clone())
            }
        );
    }
    assert!(matches!(
        graph.types[&graph.symbols["Homomorphic"].type_ref],
        SemanticType::ShapeTransform { .. }
    ));
    assert!(matches!(
        graph.types[&graph.symbols["Large"].type_ref],
        SemanticType::Unresolved { .. }
    ));
    assert_eq!(
        graph.types[&graph.symbols["Quoted"].type_ref],
        SemanticType::Literal {
            value: LiteralValue::String("it's \"fine\"".to_owned())
        }
    );
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn private_aliases_are_interpreted_but_never_exported_as_generated_bindings() {
    let source = "type PrivateName = string; type Unused = number; export declare function read(name: PrivateName): PrivateName; export interface Keys { readonly id: string; } export type Key = keyof Keys;";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    assert_eq!(
        graph.private_symbols,
        std::collections::BTreeSet::from(["PrivateName".to_owned(), "Unused".to_owned()])
    );
    assert_eq!(
        graph.types[&graph.symbols["Key"].type_ref],
        SemanticType::Literal {
            value: LiteralValue::String("id".to_owned())
        }
    );
    let projected = project_graph(&graph, SETTINGS, "api").unwrap();
    let output = projected.source.unwrap();
    assert!(output.contains("read name: String -> String"), "{output}");
    assert!(!output.contains("PrivateName") && !output.contains("Unused"));
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
}

#[test]
fn conflicting_merged_properties_do_not_choose_a_source_order_winner() {
    let source = "export interface Value { readonly id: string; } export interface Value { readonly id: number; } export type Key = Value['id'];";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    assert!(
        matches!(&graph.types[&graph.symbols["Value"].type_ref], SemanticType::Unresolved { reason } if reason.contains("conflict"))
    );
    assert!(matches!(
        graph.types[&graph.symbols["Key"].type_ref],
        SemanticType::Projection { .. }
    ));
    assert!(graph
        .normalizations
        .iter()
        .any(|step| step.output == graph.symbols["Value"].type_ref
            && step.classification == SemanticClassification::PolicyRequired
            && step.inputs.len() == 2));
}

#[test]
fn missing_external_type_identity_cannot_generate_a_dangling_binding_type() {
    let graph = interpret_declaration(
        "export declare function read(value: Missing): string;",
        "api.d.ts",
        SETTINGS,
    )
    .unwrap();
    let projection = project_graph(&graph, SETTINGS, "api").unwrap();
    assert!(projection.source.is_none());
    assert!(
        projection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-F0110"
                && diagnostic.message.contains("Missing"))
    );
}

#[test]
fn class_constructors_methods_statics_and_contextual_this_remain_distinct() {
    let source = "export declare class Store { constructor(name: string); readonly name: string; read(key: string): string; read(key: number): number; static create(name: string): Store; chain(): this; private hidden(): void; }";
    let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
    let SemanticType::Nominal {
        instance,
        constructors,
        static_members,
        ..
    } = &graph.types[&graph.symbols["Store"].type_ref]
    else {
        panic!()
    };
    assert_eq!(constructors.len(), 1);
    assert!(matches!(
        graph.types[&constructors[0]],
        SemanticType::Callable { receiver: None, .. }
    ));
    assert!(matches!(
        graph.types[&static_members["create"].type_ref],
        SemanticType::Callable {
            receiver: Some(_),
            ..
        }
    ));
    let SemanticType::Shape { identity, fields } = &graph.types[instance] else {
        panic!()
    };
    assert_eq!(*identity, ShapeIdentity::MutableIdentity);
    assert!(fields["name"].readonly && !fields.contains_key("hidden"));
    assert!(matches!(
        graph.types[&fields["read"].type_ref],
        SemanticType::Overload { .. }
    ));
    let SemanticType::Callable {
        result, receiver, ..
    } = graph.types[&fields["chain"].type_ref]
    else {
        panic!()
    };
    assert!(receiver.is_some());
    assert!(
        matches!(&graph.types[&result], SemanticType::Unresolved { reason } if reason.contains("polymorphic"))
    );
    assert_eq!(
        SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
        graph
    );
    assert_eq!(
        project_graph(&graph, SETTINGS, "api")
            .unwrap()
            .source
            .unwrap(),
        "pub foreign \"typescript\" from \"./api.js\" {\n  opaque type Store\n}\n"
    );
}

#[test]
fn explicit_projection_subset_retains_type_dependencies_and_omitted_semantics() {
    let source = "export interface Handle {} export declare function selected(value: Handle): Handle; export declare function other<T extends string>(value: T): T; type Private = string;";
    let settings = format!("{SETTINGS}include = ['selected']\n");
    let graph = interpret_declaration(source, "api.d.ts", &settings).unwrap();
    assert!(graph.symbols.contains_key("other"));
    let projected = project_graph(&graph, &settings, "api")
        .unwrap()
        .source
        .unwrap();
    assert!(
        projected.contains("opaque type Handle")
            && projected.contains("selected value: Handle -> Handle")
    );
    assert!(!projected.contains("other") && !projected.contains("Private"));
    for include in [
        "[]",
        "['selected', 'selected']",
        "['Private']",
        "['missing']",
    ] {
        let settings = format!("{SETTINGS}include = {include}\n");
        let graph = interpret_declaration(source, "api.d.ts", &settings).unwrap();
        assert!(project_graph(&graph, &settings, "api")
            .unwrap()
            .source
            .is_none());
    }
}

#[test]
fn explicit_fixed_generic_projection_requires_compatible_primitive_constraints() {
    let source = "export declare function id<T extends string>(size?: number): T;";
    let settings = format!("{SETTINGS}\n[entries.api.symbols.id.parameters.size]\ntype = 'Int'\nrequired = true\n[entries.api.symbols.id.result]\ntype = 'String'\n");
    let graph = interpret_declaration(source, "api.d.ts", &settings).unwrap();
    let projected = project_graph(&graph, &settings, "api")
        .unwrap()
        .source
        .unwrap();
    assert!(projected.contains("id size: Int -> String"), "{projected}");
    assert!(!projected.contains("<A>"));
    let wrong = settings.replace("type = 'String'", "type = 'Int'");
    let graph = interpret_declaration(source, "api.d.ts", &wrong).unwrap();
    assert!(project_graph(&graph, &wrong, "api")
        .unwrap()
        .source
        .is_none());
    let relation = "export declare function id<T extends string>(size: T): T;";
    let graph = interpret_declaration(relation, "api.d.ts", &settings).unwrap();
    assert!(project_graph(&graph, &settings, "api")
        .unwrap()
        .source
        .is_none());
}

#[test]
fn nominal_visibility_and_static_readonly_flags_are_not_erased() {
    let graph = interpret_declaration("export abstract class Abstract { constructor(name: string); } export class Private { private constructor(); } export class Generic<T> { constructor(value: T); static readonly version: string; read(key: string): string; read(key: number): number; read(key: boolean): boolean; }", "api.d.ts", SETTINGS).unwrap();
    for name in ["Abstract", "Private"] {
        assert!(matches!(
            graph.types[&graph.symbols[name].type_ref],
            SemanticType::Nominal {
                constructible: false,
                ..
            }
        ));
    }
    let SemanticType::Generic { body, .. } = graph.types[&graph.symbols["Generic"].type_ref] else {
        panic!()
    };
    let SemanticType::Nominal {
        instance,
        static_members,
        constructors,
        ..
    } = &graph.types[&body]
    else {
        panic!()
    };
    assert!(static_members["version"].readonly);
    let SemanticType::Callable { result, .. } = graph.types[&constructors[0]] else {
        panic!()
    };
    assert!(matches!(graph.types[&result], SemanticType::Apply { .. }));
    let SemanticType::Shape { fields, .. } = &graph.types[instance] else {
        panic!()
    };
    let SemanticType::Overload { signatures } = &graph.types[&fields["read"].type_ref] else {
        panic!()
    };
    assert_eq!(signatures.len(), 3);
}

#[test]
fn default_export_aliases_keep_the_original_identity_and_public_root() {
    for source in [
        "export declare class Emitter {} export default Emitter;",
        "declare class Emitter {} export default Emitter;",
        "export default class Emitter {}",
    ] {
        let graph = interpret_declaration(source, "api.d.ts", SETTINGS).unwrap();
        assert_eq!(graph.symbols["default"].declaration_name, "Emitter");
        assert!(!graph.private_symbols.contains("default"));
        assert!(
            matches!(&graph.types[&graph.symbols["default"].type_ref], SemanticType::Nominal { identity, .. } if identity == "Emitter")
        );
        assert!(project_graph(&graph, SETTINGS, "api")
            .unwrap()
            .source
            .unwrap()
            .contains("opaque type Default"));
    }
}

#[test]
fn binary_boundaries_emit_their_required_standard_import() {
    let graph = interpret_declaration("export declare function encode(value: Uint8Array): string; export declare function decode(value: string): Uint8Array;", "api.d.ts", SETTINGS).unwrap();
    let output = project_graph(&graph, SETTINGS, "api")
        .unwrap()
        .source
        .unwrap();
    assert!(
        output.starts_with("import { Bytes } from \"std/bytes\"\n"),
        "{output}"
    );
    assert!(
        output.contains("encode value: Bytes -> String")
            && output.contains("decode value: String -> Bytes")
    );
}

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
fn merged_declarations_keep_every_source_and_combine_compatible_structural_fields() {
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
            && step.classification == SemanticClassification::Normalized));
    let SemanticType::Shape { fields, .. } = &graph.types[&symbol.type_ref] else {
        panic!()
    };
    assert_eq!(
        fields.keys().map(String::as_str).collect::<Vec<_>>(),
        ["id", "name"]
    );
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
    let source = "export type Recursive<T> = readonly Recursive<T>[]; export type Conditional<T> = T extends string ? string : number; export type Unique = unique symbol; export declare function unknownExternal(x: Missing): any;";
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
    assert!(converted.diagnostics.iter().any(|d| d.code == "SES-F0110"));
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
