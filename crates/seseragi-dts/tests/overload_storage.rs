use seseragi_dts::{interpret_declaration, project_graph};
use seseragi_interop::*;
use std::fmt::Write;

const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration = 'api.d.ts'\noutput = 'api'\nspecifier = './api.js'\nevaluation = 'pure'\n";

/// Count stored graph data, including unreachable historical nodes. Counting
/// only the exported overload would miss a retained prefix for every signature.
fn stored_counts(graph: &SemanticGraph) -> [usize; 4] {
    [
        graph.types.len(),
        graph
            .types
            .values()
            .map(|node| match node {
                SemanticType::Overload { signatures } => signatures.len(),
                _ => 0,
            })
            .sum(),
        graph.type_origins.values().map(Vec::len).sum::<usize>()
            + graph
                .symbols
                .values()
                .map(|symbol| symbol.origins.len())
                .sum::<usize>()
            + graph
                .types
                .values()
                .map(|node| match node {
                    SemanticType::Callable { parameters, .. } => parameters
                        .iter()
                        .map(|parameter| parameter.origins.len())
                        .sum(),
                    _ => 0,
                })
                .sum::<usize>(),
        graph.normalizations.len(),
    ]
}

fn signatures(graph: &SemanticGraph, id: TypeId, count: usize, rule: &str) -> Vec<TypeId> {
    let SemanticType::Overload { signatures } = &graph.types[&id] else {
        panic!("expected an overload set")
    };
    assert_eq!(signatures.len(), count);
    let origins = &graph.type_origins[&id];
    assert_eq!(origins.len(), count - 1);
    assert!(origins.iter().all(|origin| origin.rule == rule));
    assert_eq!(
        graph
            .normalizations
            .iter()
            .filter(|step| step.output == id)
            .count(),
        1,
        "the stable set needs one explicit overload-selection decision"
    );
    signatures.clone()
}

fn assert_linear_counts(counts: &[[usize; 4]]) {
    // The fixtures have 4, 8, then 16 overloads. Fixed metadata can stay
    // constant, while each additional signature/origin contributes once.
    for metric in 0..counts[0].len() {
        assert_eq!(
            counts[2][metric] - counts[1][metric],
            2 * (counts[1][metric] - counts[0][metric]),
            "stored metric {metric} must grow linearly: {counts:?}"
        );
    }
}

#[test]
fn free_overload_storage_is_linear_and_preserves_alias_order_origins_and_policy() {
    let mut counts = Vec::new();
    for count in [4, 8, 16] {
        let mut source = String::new();
        for index in 0..count {
            let ty = if index % 2 == 0 { "string" } else { "boolean" };
            writeln!(
                source,
                "declare function internal(value{index}: {ty}): {ty};"
            )
            .unwrap();
        }
        source.push_str("export { internal as renamed };\n");
        let graph = interpret_declaration(&source, "api.d.ts", SETTINGS).unwrap();
        let symbol = &graph.symbols["renamed"];
        assert_eq!(symbol.public_name, "renamed");
        assert_eq!(symbol.declaration_name, "internal");
        assert_eq!(symbol.origins.len(), 2 * count);
        assert_eq!(
            symbol
                .origins
                .iter()
                .filter(|origin| origin.rule == "declaration")
                .map(|origin| source[origin.start..origin.end].trim())
                .collect::<Vec<_>>(),
            source
                .lines()
                .take(count)
                .map(|line| line.strip_prefix("declare ").unwrap())
                .collect::<Vec<_>>()
        );
        for (index, id) in signatures(&graph, symbol.type_ref, count, "overload-set")
            .into_iter()
            .enumerate()
        {
            let SemanticType::Callable {
                parameters,
                receiver,
                ..
            } = &graph.types[&id]
            else {
                panic!("the overload must retain its callable signature")
            };
            assert_eq!(parameters[0].name, format!("value{index}"));
            assert!(receiver.is_none());
            let origin = &graph.type_origins[&id][0];
            assert_eq!(origin.rule, "callable-signature");
            assert_eq!(
                source[origin.start..origin.end].trim(),
                source
                    .lines()
                    .nth(index)
                    .unwrap()
                    .strip_prefix("declare ")
                    .unwrap()
            );
        }
        let stored = stored_counts(&graph);
        assert_eq!(stored[1], count, "no historical signature prefixes remain");
        counts.push(stored);
        let rejected = project_graph(&graph, SETTINGS, "api").unwrap();
        assert!(rejected.source.is_none());
        let diagnostic = rejected
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "SES-F0102")
            .unwrap();
        assert_eq!(diagnostic.symbol.as_deref(), Some("renamed"));
        assert_eq!(&source[diagnostic.start..diagnostic.end], "renamed");

        let settings = format!(
            "{SETTINGS}\n[entries.api.symbols.renamed.overloads]\nfirst = {{ signature = 0 }}\nlast = {{ signature = {} }}\n",
            count - 1
        );
        let selected = interpret_declaration(&source, "api.d.ts", &settings).unwrap();
        let projected = project_graph(&selected, &settings, "api").unwrap();
        assert!(projected.diagnostics.is_empty());
        let output = projected.source.unwrap();
        assert!(output.contains("pure fn first value0: String -> String = \"renamed\""));
        assert!(output.contains(&format!(
            "pure fn last value{}: Bool -> Bool = \"renamed\"",
            count - 1
        )));
        assert_eq!(
            SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
            graph
        );
    }
    assert_linear_counts(&counts);
}

#[test]
fn nominal_instance_and_generic_static_method_overload_storage_is_linear() {
    let mut counts = Vec::new();
    for count in [4, 8, 16] {
        let mut source = String::from("export class Host<T> {\n");
        for index in 0..count {
            writeln!(source, "read(value{index}: T): T;").unwrap();
            writeln!(source, "static create<U>(value{index}: U): U;").unwrap();
        }
        source.push_str("}\n");
        let graph = interpret_declaration(&source, "api.d.ts", SETTINGS).unwrap();
        let SemanticType::Generic { body, .. } = graph.types[&graph.symbols["Host"].type_ref]
        else {
            panic!("the nominal class must retain its generic binder")
        };
        let SemanticType::Nominal {
            instance,
            static_members,
            ..
        } = &graph.types[&body]
        else {
            panic!("the class must retain its nominal identity")
        };
        let SemanticType::Shape { fields, .. } = &graph.types[instance] else {
            panic!("the instance must retain its own members")
        };
        for (name, id) in [
            ("read", fields["read"].type_ref),
            ("create", static_members["create"].type_ref),
        ] {
            for (index, signature) in signatures(&graph, id, count, "method-overload-set")
                .into_iter()
                .enumerate()
            {
                let body = if name == "create" {
                    let SemanticType::Generic { parameters, body } = &graph.types[&signature]
                    else {
                        panic!("static method overloads must retain each generic binder")
                    };
                    assert_eq!(parameters[0].name, "U");
                    *body
                } else {
                    signature
                };
                let SemanticType::Callable {
                    parameters,
                    receiver,
                    ..
                } = &graph.types[&body]
                else {
                    panic!("each nominal overload must remain callable")
                };
                assert_eq!(parameters[0].name, format!("value{index}"));
                assert!(receiver.is_some());
                let origin = &graph.type_origins[&body][0];
                assert!(source[origin.start..origin.end].contains(&format!("value{index}:")));
            }
        }
        let stored = stored_counts(&graph);
        assert_eq!(stored[1], 2 * count);
        counts.push(stored);
        assert_eq!(
            SemanticGraph::from_json(&graph.to_json().unwrap()).unwrap(),
            graph
        );
    }
    assert_linear_counts(&counts);
}
