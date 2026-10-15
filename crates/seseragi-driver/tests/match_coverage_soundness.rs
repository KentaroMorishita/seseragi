use seseragi_driver::{analyze_module, compile_module, CompileInput};
use seseragi_semantics::{TypedDecl, TypedExpr};

fn input(source: &str) -> CompileInput<'_> {
    CompileInput::new("coverage.ssrg", "fixture/coverage", source)
}

fn total(source: &str) {
    let compiled =
        compile_module(input(source)).unwrap_or_else(|errors| panic!("{source}\n{errors:#?}"));
    let body = compiled
        .typed_hir
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::Fn {
                body: TypedExpr::Match { exhaustive, .. },
                ..
            } => Some(*exhaustive),
            _ => None,
        });
    assert_eq!(
        body,
        Some(true),
        "coverage must actually prove totality, not silently skip it: {source}"
    );
}

fn rejects(source: &str, key: &str) {
    let compiled = compile_module(input(source)).expect_err("invalid match must fail compilation");
    assert!(
        compiled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == key),
        "{source}\n{compiled:#?}"
    );
    let analyzed = analyze_module(input(source));
    assert_eq!(
        analyzed.diagnostics, compiled,
        "analysis and compilation must share diagnostic authority"
    );
}

#[test]
fn recursive_matches_require_coverage_and_reject_unreachable_or_guard_only_arms() {
    let declaration = "type Tree = | Leaf Int | Branch Tree\n";
    for body in [
        "match tree { Leaf x -> x }",
        "match tree { Leaf x -> x; Branch _ when True -> 0 }",
        "match tree { Branch (Leaf x) -> x; Leaf x -> x }",
    ] {
        rejects(
            &format!("{declaration}pub fn value tree: Tree -> Int = {body}\n"),
            "match.non-exhaustive",
        );
    }
    rejects(
        &format!(
            "{declaration}pub fn value tree: Tree -> Int = match tree {{ _ -> 0; Leaf x -> x }}\n"
        ),
        "match.unreachable-arm",
    );
    for body in [
        "match tree { Leaf x -> x; Branch _ -> 0 }",
        "match tree { Leaf x -> x; Branch (Leaf x) -> x; Branch (Branch _) -> 0 }",
        "match tree { Branch (Leaf x) when x == 0 -> 1; _ -> 0 }",
    ] {
        total(&format!(
            "{declaration}pub fn value tree: Tree -> Int = {body}\n"
        ));
    }
}

#[test]
fn mutual_and_generic_recursive_types_preserve_totality() {
    total("type Tree<A> = | Leaf A | Branch Tree<A>\npub fn value<A> tree: Tree<A> -> Int = match tree { Leaf _ -> 1; Branch _ -> 2 }\n");
    total("type First = | End | Next Second\ntype Second = | Back First\npub fn value tree: First -> Int = match tree { End -> 0; Next (Back _) -> 1 }\n");
    rejects("type Tree<A> = | Leaf A | Branch Tree<A>\npub fn value<A> tree: Tree<A> -> Int = match tree { Leaf _ -> 1 }\n", "match.non-exhaustive");
}

#[test]
fn large_boolean_product_domains_are_proved_symbolically() {
    let types = vec!["Bool"; 13].join(", ");
    let trues = vec!["True"; 13].join(", ");
    let falses = vec!["False"; 13].join(", ");
    let parameter = format!("pub fn value flags: ({types}) -> Int = ");
    rejects(
        &format!("{parameter}match flags {{ ({trues}) -> 1; ({falses}) -> 2 }}\n"),
        "match.non-exhaustive",
    );
    total(&format!(
        "{parameter}match flags {{ ({trues}) -> 1; ({falses}) -> 2; _ -> 3 }}\n"
    ));
    let tail = vec!["_"; 12].join(", ");
    total(&format!(
        "{parameter}match flags {{ (True, {tail}) -> 1; (False, {tail}) -> 2 }}\n"
    ));
}

#[test]
fn collection_elements_retain_tuple_record_and_nominal_domains() {
    for (collection, prefix) in [("Array", ""), ("List", "`")] {
        total(&format!("pub fn inspect values: {collection}<(Int, Int)> -> Int = match values {{ {prefix}[] -> 0; {prefix}[(a, b), ...tail] -> a }}\n"));
        total(&format!("type Flag = | On | Off\npub fn inspect values: {collection}<Flag> -> Int = match values {{ {prefix}[] -> 0; {prefix}[On, ...tail] -> 1; {prefix}[Off, ...tail] -> 2 }}\n"));
        rejects(&format!("type Flag = | On | Off\npub fn inspect values: {collection}<Flag> -> Int = match values {{ {prefix}[] -> 0; {prefix}[On, ...tail] -> 1 }}\n"), "match.non-exhaustive");
        total(&format!("type Flag = | On | Off\npub fn inspect values: {collection}<{{ flag: Flag, pair: (Int, Int) }}> -> Int = match values {{ {prefix}[] -> 0; {prefix}[{{ flag: On, pair: (a, b) }}, ...tail] -> a; {prefix}[{{ flag: Off }}, ...tail] -> 2 }}\n"));
        total(&format!("pub fn inspect values: {collection}<{collection}<Int>> -> Int = match values {{ {prefix}[] -> 0; {prefix}[{prefix}[], ...tail] -> 1; {prefix}[{prefix}[head, ...rest], ...tail] -> head }}\n"));
    }
}

#[test]
fn record_and_struct_fields_retain_nested_products_and_adt_identity() {
    total(
        "pub fn first value: { pair: (Int, Int) } -> Int = match value { { pair: (a, b) } -> a }\n",
    );
    total("type Flag = | On | Off\nstruct Box { flag: Flag }\npub fn flag value: Box -> Int = match value { Box { flag: On } -> 1; Box { flag: Off } -> 2 }\n");
    total("type Flag = | On | Off\npub fn flag value: { box: { flag: Flag } } -> Int = match value { { box: { flag: On } } -> 1; { box: { flag: Off } } -> 2 }\n");
    rejects("type Flag = | On | Off\npub fn flag value: { box: { flag: Flag } } -> Int = match value { { box: { flag: On } } -> 1 }\n", "match.non-exhaustive");
}

#[test]
fn runtime_equivalent_integer_patterns_are_unreachable_with_original_ranges() {
    for (first, second) in [
        ("1000", "1_000"),
        ("0", "-0"),
        ("-1000", "-1_000"),
        ("42", "0x2A"),
        ("0b101010", "0o52"),
        ("-0x2A", "-42"),
        ("0x0", "-0b0"),
    ] {
        let source = format!("pub fn value input: Int -> Int = match input {{ {first} -> 1; {second} -> 2; _ -> 3 }}\n");
        rejects(&source, "match.unreachable-arm");
        let diagnostics = compile_module(input(&source)).unwrap_err();
        let diagnostic = diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message_key == "match.unreachable-arm")
            .unwrap();
        let start = source.find(&format!("{second} -> 2")).unwrap();
        assert_eq!(diagnostic.primary.start, start);
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            format!("{second} -> 2")
        );
    }
    total("pub fn value input: Int -> Int = match input { 1000 -> 1; 1001 -> 2; _ -> 3 }\n");
}

#[test]
fn deeply_nested_source_patterns_fail_closed_at_the_explicit_analysis_limit() {
    let nested = format!("{}Leaf value", "Branch ".repeat(260));
    let source = format!("type Tree = | Leaf Int | Branch Tree\npub fn inspect tree: Tree -> Int = match tree {{ {nested} -> value; _ -> 0 }}\n");
    let diagnostics = compile_module(input(&source))
        .expect_err("analysis budget must never silently accept an unproved arm set");
    assert!(
        diagnostics.diagnostics.iter().any(|diagnostic| diagnostic
            .message()
            .contains("coverage exceeded its source-complexity limit")),
        "{diagnostics:#?}"
    );
    // Recursive type depth alone is not a source-complexity failure.
    total("type Tree = | Leaf Int | Branch Tree\npub fn inspect tree: Tree -> Int = match tree { _ -> 0 }\n");
}

#[test]
fn symbolic_tuple_coverage_agrees_with_an_independent_finite_value_oracle() {
    // Enumerate concrete values independently of the compiler's specialization
    // algorithm, including correlated patterns and wildcard overlap.
    let choices = ["_", "False", "True"];
    let mut patterns = Vec::new();
    for first in choices {
        for second in choices {
            let mut mask = 0u8;
            for a in [false, true] {
                for b in [false, true] {
                    let matches =
                        |pattern: &str, value: bool| pattern == "_" || (pattern == "True") == value;
                    if matches(first, a) && matches(second, b) {
                        mask |= 1 << (usize::from(a) * 2 + usize::from(b));
                    }
                }
            }
            patterns.push((format!("({first}, {second})"), mask));
        }
    }
    for (first, first_mask) in &patterns {
        for (second, second_mask) in &patterns {
            let source = format!("pub fn inspect flags: (Bool, Bool) -> Int = match flags {{ {first} -> 1; {second} -> 2 }}\n");
            let analysis = analyze_module(input(&source));
            let has = |key: &str| {
                analysis
                    .diagnostics
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message_key == key)
            };
            assert_eq!(
                has("match.unreachable-arm"),
                second_mask & !first_mask == 0,
                "{source}"
            );
            assert_eq!(
                has("match.non-exhaustive"),
                first_mask | second_mask != 0b1111,
                "{source}"
            );
            assert!(
                !has("match.coverage-limit"),
                "small finite oracle must not exhaust the analysis budget"
            );
        }
    }
}

#[test]
fn builtin_patterns_do_not_accept_same_spelled_nominal_or_higher_kinded_types() {
    for (collection, prefix) in [("Array", ""), ("List", "`")] {
        rejects(&format!("struct {collection}<A> {{ value: A }}\npub fn inspect values: {collection}<Int> -> Int = match values {{ {prefix}[] -> 0; {prefix}[item, ...tail] -> item }}\n"), "match.pattern-type-mismatch");
        rejects(&format!("pub fn inspect<{collection}<_>> values: {collection}<Int> -> Int = match values {{ {prefix}[] -> 0; {prefix}[item, ...tail] -> item }}\n"), "match.pattern-type-mismatch");
    }
    for (name, literal) in [("Int", "1"), ("Bool", "True"), ("String", "\"text\"")] {
        rejects(&format!("struct {name} {{ value: Unit }}\npub fn inspect value: {name} -> Unit = match value {{ {literal} -> (); _ -> () }}\n"), "match.pattern-type-mismatch");
    }
    total("alias Values<A> = Array<A>\npub fn inspect values: Values<Int> -> Int = match values { [] -> 0; [item, ...tail] -> item }\n");
}

#[test]
fn aliased_collections_preserve_distinct_imported_nominal_tuple_elements() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    for module in ["fixture/coverage::alpha", "fixture/coverage::beta"] {
        graph
            .add_module(module.to_owned(), std::iter::empty::<(String, String)>())
            .unwrap();
    }
    graph
        .add_module(
            "fixture/coverage::main".to_owned(),
            [
                ("./alpha".to_owned(), "fixture/coverage::alpha".to_owned()),
                ("./beta".to_owned(), "fixture/coverage::beta".to_owned()),
            ],
        )
        .unwrap();
    let source = r#"
import * as alpha from "./alpha"
import * as beta from "./beta"
alias Values<A> = Array<A>
pub fn inspect values: Values<(alpha.Flag, beta.Flag)> -> Int = match values {
  [] -> 0
  [(alpha.On, beta.On), ...tail] -> 1
  [(alpha.On, beta.Off), ...tail] -> 2
  [(alpha.Off, beta.On), ...tail] -> 3
  [(alpha.Off, beta.Off), ...tail] -> 4
}
"#;
    let input = |main| {
        [
            ProjectModuleInput::new(
                "alpha.ssrg",
                "fixture/coverage::alpha",
                "pub type Flag = | On | Off\n",
                "dist/alpha.js",
            ),
            ProjectModuleInput::new(
                "beta.ssrg",
                "fixture/coverage::beta",
                "pub type Flag = | On | Off\n",
                "dist/beta.js",
            ),
            ProjectModuleInput::new("main.ssrg", "fixture/coverage::main", main, "dist/main.js"),
        ]
    };
    compile_project(graph.clone(), input(source))
        .expect("collection and tuple retain canonical imported owners through alias expansion");
    let wrong_owner = source.replace("(alpha.Off, beta.Off)", "(alpha.Off, alpha.Off)");
    assert!(
        compile_project(graph, input(&wrong_owner)).is_err(),
        "same-spelled imported constructors must not match a different owner"
    );
}

#[test]
fn nested_collection_element_depth_includes_its_prefix_position() {
    let mut elements = vec!["_".to_owned(); 199];
    elements.push(format!("{}Leaf value", "Branch ".repeat(60)));
    let source = format!("type Tree = | Leaf Int | Branch Tree\npub fn inspect values: Array<Tree> -> Int = match values {{ [{}] -> 1; _ -> 0 }}\n", elements.join(", "));
    rejects(&source, "match.coverage-limit");
}

#[test]
fn boolean_predicates_require_primitive_identity_at_every_boundary() {
    for (declaration, value) in [
        ("struct Bool { value: Int }", "Bool { value: 0 }"),
        ("struct Bool<A> { value: A }", "Bool { value: 0 }"),
    ] {
        for (body, result, key) in [
            (
                format!("match 0 {{ _ when {value} -> 1; _ -> 2 }}"),
                "Int",
                "match.guard-not-bool",
            ),
            (
                format!("if {value} then 1 else 2"),
                "Int",
                "if.condition-not-bool",
            ),
            (
                format!("[n | n <- [1, 2], {value}]"),
                "Array<Int>",
                "match.guard-not-bool",
            ),
            (
                format!("`[n | n <- `[1, 2], {value}]"),
                "List<Int>",
                "match.guard-not-bool",
            ),
        ] {
            rejects(
                &format!("{declaration}\nfn inspect -> {result} = {body}\n"),
                key,
            );
        }
    }
    for (body, result) in [
        ("match 0 { _ when True -> 1; _ -> 2 }", "Int"),
        ("if True then 1 else 2", "Int"),
        ("[n | n <- [1, 2], n == 2]", "Array<Int>"),
        ("`[n | n <- `[1, 2], n == 2]", "List<Int>"),
    ] {
        compile_module(input(&format!("fn inspect -> {result} = {body}\n"))).unwrap();
    }
    compile_module(input(
        "alias Truth = Bool\nfn choose truth: Truth -> Int = if truth then 1 else 2\n",
    ))
    .unwrap();
    compile_module(input("alias Truth = Bool\nfn choose truth: Truth -> Int = match 0 { _ when truth -> 1; _ -> 2 }\n")).unwrap();
    for (body, result, key) in [
        ("if truth then 1 else 2", "Int", "if.condition-not-bool"),
        (
            "match 0 { _ when truth -> 1; _ -> 2 }",
            "Int",
            "match.guard-not-bool",
        ),
        (
            "[n | n <- [1, 2], truth]",
            "Array<Int>",
            "match.guard-not-bool",
        ),
        (
            "`[n | n <- `[1, 2], truth]",
            "List<Int>",
            "match.guard-not-bool",
        ),
    ] {
        rejects(
            &format!("fn choose<Bool> truth: Bool -> {result} = {body}"),
            key,
        );
    }
}

#[test]
fn standard_boolean_results_preserve_primitive_identity_under_local_shadowing() {
    for declaration in ["struct Bool { value: Int }", "struct Bool<A> { value: A }"] {
        for (body, result) in [
            ("if value == 0 then 1 else 2", "Int"),
            ("if !False then 1 else 2", "Int"),
            ("match value { _ when value != 0 -> 1; _ -> 2 }", "Int"),
            ("[n | n <- [value], n >= 0]", "Array<Int>"),
            ("`[n | n <- `[value], n < 0]", "List<Int>"),
            (
                "if (value <= 0 && value > -1) || False then 1 else 2",
                "Int",
            ),
            (
                "{ let predicate = value == 0; if predicate then 1 else 2 }",
                "Int",
            ),
        ] {
            compile_module(input(&format!(
                "{declaration}\nfn inspect value: Int -> {result} = {body}\n"
            )))
            .unwrap();
        }
        for body in [
            "if (Bool { value: 0 } && True) then 1 else 2",
            "if (False || Bool { value: 0 }) then 1 else 2",
        ] {
            rejects(
                &format!("{declaration}\nfn inspect -> Int = {body}\n"),
                "call.argument-type-mismatch",
            );
        }
    }
    rejects(
        "struct Bool { value: Int }\nfn inspect -> Int = if !(Bool { value: 0 }) then 1 else 2",
        "unary.operand-type-mismatch",
    );
    rejects(
        "fn inspect<Bool> truth: Bool -> Int = if !truth then 1 else 2",
        "unary.operand-type-mismatch",
    );
    compile_module(input(
        "fn inspect<Bool> ignored: Bool -> Int = if 0 == 0 then 1 else 2",
    ))
    .unwrap();
    rejects(
        "fn inspect<Bool> truth: Bool -> Int = if truth && True then 1 else 2",
        "call.argument-type-mismatch",
    );
}

#[test]
fn unary_numeric_results_preserve_primitive_identity_and_reject_nominal_operands() {
    compile_module(input("struct Int { value: Bool }\nfn inspect -> String = match (-1) { 0 -> \"zero\"; _ -> \"negative\" }")).unwrap();
    compile_module(input(
        "struct Float { value: Bool }\nfn identity<A> value: A -> A = value\nfn inspect -> String = match (identity (-1.0)) { _ -> \"negative\" }",
    ))
    .unwrap();
    compile_module(input(
        "alias Number = Int\nfn inspect value: Number -> Int = -value",
    ))
    .unwrap();
    compile_module(input(
        "alias Number = Float\nfn inspect value: Number -> Float = -value",
    ))
    .unwrap();
    for name in ["Int", "Float"] {
        rejects(&format!("struct {name} {{ value: Bool }}\nfn inspect value: {name} -> String = match (-value) {{ _ -> \"never\" }}"), "unary.operand-type-mismatch");
        rejects(
            &format!(
                "fn inspect<{name}> value: {name} -> String = match (-value) {{ _ -> \"never\" }}"
            ),
            "unary.operand-type-mismatch",
        );
    }
}

#[test]
fn primitive_hydration_retains_standard_calls_and_nominal_function_contracts() {
    compile_module(input(
        r#"
import * as arrays from "std/array"
struct Bool { value: Int }
fn nominal value: Bool -> Bool = value
fn inspect -> Int = {
  let original = Bool { value: 0 }
  let forwarded = nominal original
  let record = { held: forwarded }
  let empty = arrays.isEmpty [1]
  if empty then record.held.value else 2
}
"#,
    ))
    .unwrap();
    for expression in [
        "Bool { value: 0 }",
        "{ let predicate = Bool { value: 0 }; predicate }",
        "nominal (Bool { value: 0 })",
        "{ let predicate = nominal (Bool { value: 0 }); predicate }",
        "{ let original = Bool { value: 0 }; identity original }",
        "{ let predicate = identity (Bool { value: 0 }); predicate }",
    ] {
        rejects(&format!("struct Bool {{ value: Int }}\nfn nominal value: Bool -> Bool = value\nfn identity<A> value: A -> A = value\nfn inspect -> Int = if {expression} then 1 else 2"), "if.condition-not-bool");
    }
    for name in ["Bool", "Int", "Float", "Char", "String", "Unit"] {
        compile_module(input(&format!("struct {name} {{ value: Int }}\nfn identity value: {name} -> {name} = value\nfn forward value: {name} -> {name} = identity value"))).unwrap();
    }
}

#[test]
fn foreign_opaque_boolean_values_remain_nominal_after_forwarding() {
    let declarations = r#"
foreign "typescript" from "./host.mjs" {
  opaque type Bool
  pure fn obtain unit: Unit -> Bool
}
fn identity<A> value: A -> A = value
"#;
    compile_module(input(&format!(
        "{declarations}fn inspect -> Bool = identity (obtain ())"
    )))
    .unwrap();
    for expression in [
        "obtain ()",
        "identity (obtain ())",
        "{ let predicate = obtain (); predicate }",
    ] {
        rejects(
            &format!("{declarations}fn inspect -> Int = if {expression} then 1 else 2"),
            "if.condition-not-bool",
        );
    }
}

#[test]
fn composite_values_keep_each_field_identity_through_bindings_and_forwarding() {
    let declaration = "struct Bool { value: Int }\nfn identity<A> value: A -> A = value\nfn fieldIdentity<A> value: { predicate: A } -> { predicate: A } = value\n";
    for value in ["True", "Bool { value: 0 }"] {
        for body in [
            format!("{{ let record = {{ predicate: {value} }}; if record.predicate then 1 else 2 }}"),
            format!("{{ let record = {{ inner: {{ predicate: {value} }} }}; if record.inner.predicate then 1 else 2 }}"),
            format!("{{ let base = {{ predicate: {value} }}; let record = {{ ...base }}; if record.predicate then 1 else 2 }}"),
            format!("{{ let base = {{ predicate: False }}; let record = {{ ...base, predicate: {value} }}; if record.predicate then 1 else 2 }}"),
            format!("{{ let record = identity {{ predicate: {value} }}; if record.predicate then 1 else 2 }}"),
            format!("{{ let record = fieldIdentity {{ predicate: {value} }}; if record.predicate then 1 else 2 }}"),
            format!("{{ let {{ predicate }} = {{ predicate: {value} }}; if predicate then 1 else 2 }}"),
            format!("{{ let (record, ignored) = ({{ predicate: {value} }}, 0); if record.predicate then 1 else 2 }}"),
            format!("{{ let (predicate, ignored) = ({value}, 0); if predicate then 1 else 2 }}"),
            format!("{{ let ((predicate, ignored), outer) = (({value}, 0), 1); if predicate then 1 else 2 }}"),
            format!("{{ let (predicate, ignored) = identity ({value}, 0); if predicate then 1 else 2 }}"),
        ] {
            let source = format!("{declaration}fn inspect -> Int = {body}");
            if value == "True" { compile_module(input(&source)).unwrap(); }
            else { rejects(&source, "if.condition-not-bool"); }
        }
    }
    compile_module(input(&format!("{declaration}fn inspect -> Int = {{ let base = {{ predicate: Bool {{ value: 0 }} }}; let record = {{ ...base, predicate: True }}; if record.predicate then 1 else 2 }}"))).unwrap();
    rejects(&format!("{declaration}fn inspect -> Int = {{ let base = {{ predicate: True }}; let record = {{ ...base, predicate: Bool {{ value: 0 }} }}; if record.predicate then 1 else 2 }}"), "if.condition-not-bool");
}

#[test]
fn builtin_collections_keep_identity_through_a_same_name_nominal_scope() {
    for (name, literal) in [("Array", "[1]"), ("List", "`[1]")] {
        let declaration =
            format!("struct {name}<A> {{ value: A }}\nfn identity<A> value: A -> A = value\n");
        let prefix = if name == "List" { "`" } else { "" };
        compile_module(input(&format!(
            "{declaration}pub fn inspect -> Int = {{ let values = identity {literal}; match values {{ {prefix}[head, ...tail] -> head; {prefix}[] -> 0 }} }}"
        )))
        .unwrap();
        rejects(
            &format!(
                "{declaration}pub fn inspect value: {name}<Int> -> Int = match value {{ {prefix}[] -> 0; _ -> 1 }}"
            ),
            "match.pattern-type-mismatch",
        );
    }
}
