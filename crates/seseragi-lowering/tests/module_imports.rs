use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir_with_plan, lower_typed_module,
    TypeScriptDecisionTest, TypeScriptExpr, TypeScriptFunction, TypeScriptLoweringError,
    TypeScriptOutputPlan,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_semantics::{analyze_linked_module, analyze_module_interface};
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::collections::BTreeMap;

#[test]
fn lowers_an_imported_alias_call_to_a_planned_typescript_module_import() {
    let domain_source = "pub fn increment value: Int -> Int = value + 1\n";
    let main_source =
        "import { increment as next } from \"./domain\"\n\npub fn run value: Int -> Int = next value\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let missing = lower_core_module_to_typescript_ir_with_plan(
        core.clone(),
        &TypeScriptOutputPlan::default(),
    )
    .unwrap_err();
    assert_eq!(
        missing,
        TypeScriptLoweringError::MissingOutputSpecifier {
            module: "fixture/game::domain".to_owned(),
            source_specifier: "./domain".to_owned(),
        }
    );
    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();

    assert_eq!(typescript.source_imports.len(), 1);
    assert_eq!(typescript.source_imports[0].specifier, "./domain.js");
    assert_eq!(typescript.source_imports[0].bindings.len(), 1);
    assert_eq!(typescript.source_imports[0].bindings[0].local, "next");
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::Call { callee, .. },
            ..
        } if callee == "next"
    ));

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .starts_with("import { increment as next } from \"./domain.js\"\n\n"));
    assert!(!generated.typescript.contains(".ssrg"));
    assert!(generated
        .typescript
        .contains("export const run = (value: bigint) => next(value)"));
}

#[test]
fn lowers_a_namespace_member_call_to_a_selected_named_import() {
    let domain_source = "pub fn identity<A> value: A -> A = value\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn run value: String -> String = domain.identity value\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();

    let binding = &typescript.source_imports[0].bindings[0];
    assert_eq!(binding.imported, "identity");
    assert_eq!(binding.local, "domain_identity");
    assert_eq!(binding.source_local, "domain.identity");
    assert_eq!(binding.canonical, "fixture/game::domain::identity");
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            body: TypeScriptExpr::Call { callee, .. },
            ..
        } if callee == "domain_identity"
    ));

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .starts_with("import { identity as domain_identity } from \"./domain.js\"\n\n"));
    assert!(generated
        .typescript
        .contains("export const run = (value: string) => domain_identity(value)"));
}

#[test]
fn lowers_a_namespace_type_member_to_a_selected_type_import() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn keep value: domain.Hand -> domain.Hand = value\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();

    let binding = &typescript.source_imports[0].bindings[0];
    assert_eq!(binding.imported, "Hand");
    assert_eq!(binding.local, "domain_Hand");
    assert_eq!(binding.source_local, "domain.Hand");
    assert_eq!(binding.canonical, "fixture/game::domain::Hand");
    assert!(binding.type_only);
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction { parameters, .. }
            if parameters[0].type_name == "domain_Hand"
    ));

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated.typescript.starts_with(
        "import { type Hand as domain_Hand } from \"./domain.js\"\nimport \"./domain.js\"\n\n"
    ));
    assert!(generated
        .typescript
        .contains("export const keep = (value: domain_Hand) => value"));
}

#[test]
fn lowers_namespace_qualified_adt_values_and_patterns_to_selected_imports() {
    let domain_source = "pub type Hand =\n  | Rock\n  | Paper\n  | Scissors\n";
    let main_source = r#"import * as domain from "./domain"

fn domain_Rock unit: Unit -> Unit = ()

pub fn cycle hand: domain.Hand -> domain.Hand =
  match hand {
    domain.Rock -> domain.Paper
    domain.Paper -> domain.Scissors
    domain.Scissors -> domain.Rock
  }
"#;
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();

    let bindings = &typescript.source_imports[0].bindings;
    assert!(bindings.iter().any(|binding| {
        binding.imported == "Hand"
            && binding.local == "domain_Hand"
            && binding.source_local == "domain.Hand"
            && binding.canonical == "fixture/game::domain::Hand"
            && binding.type_only
    }));
    assert!(bindings.iter().any(|binding| {
        binding.imported == "Rock"
            && binding.local == "domain_Rock_1"
            && binding.source_local == "domain.Rock"
            && binding.canonical == "fixture/game::domain::Rock"
            && !binding.type_only
    }));
    for constructor in ["Paper", "Scissors"] {
        assert!(bindings.iter().any(|binding| {
            binding.imported == constructor
                && binding.local == format!("domain_{constructor}")
                && binding.source_local == format!("domain.{constructor}")
                && binding.canonical == format!("fixture/game::domain::{constructor}")
                && !binding.type_only
        }));
    }

    let TypeScriptFunction::ConstFunction { body, .. } = typescript
        .functions
        .iter()
        .find(|function| {
            matches!(
                function,
                TypeScriptFunction::ConstFunction { name, .. } if name == "cycle"
            )
        })
        .unwrap();
    let TypeScriptExpr::Decision { branches, .. } = body else {
        panic!("expected the qualified constructor match to lower to a decision");
    };
    for (branch, (tag, result)) in branches[..2]
        .iter()
        .zip([("Rock", "domain_Paper"), ("Paper", "domain_Scissors")])
    {
        assert!(matches!(
            branch.tests.as_slice(),
            [TypeScriptDecisionTest::TagEquals { tag: actual, .. }] if actual == tag
        ));
        assert!(matches!(
            &branch.value,
            TypeScriptExpr::Identifier { name } if name == result
        ));
    }
    assert!(branches[2].tests.is_empty());
    assert!(matches!(
        &branches[2].value,
        TypeScriptExpr::Identifier { name } if name == "domain_Rock_1"
    ));

    let generated = emit_typescript_module(typescript, main_source);
    assert!(
        generated.typescript.contains(
            "import { type Hand as domain_Hand, Rock as domain_Rock_1, Paper as domain_Paper, Scissors as domain_Scissors } from \"./domain.js\""
        ),
        "{}",
        generated.typescript
    );
    assert!(generated
        .typescript
        .contains("const domain_Rock = (unit: undefined) => undefined"));
    assert!(generated.typescript.contains(
        "$ssrg_match.tag === \"Rock\" ? domain_Paper : $ssrg_match.tag === \"Paper\" ? domain_Scissors : domain_Rock_1"
    ));
}

#[test]
fn keeps_a_namespace_only_edge_as_a_side_effect_import() {
    let domain_source = "pub let answer: Int = 42\n";
    let main_source =
        "import * as domain from \"./domain\"\n\npub fn run unit: Unit -> Unit = ()\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();
    assert!(typescript.source_imports[0].bindings.is_empty());

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .starts_with("import \"./domain.js\"\n\n"));
}

#[test]
fn emits_a_type_binding_and_a_runtime_edge_for_an_imported_adt_alias() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import { Hand as LocalHand, Rock } from \"./domain\"\n\npub fn keep hand: LocalHand -> LocalHand = hand\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();
    assert_eq!(typescript.source_imports[0].bindings.len(), 1);
    assert!(typescript.source_imports[0].bindings[0].type_only);

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated.typescript.starts_with(
        "import { type Hand as LocalHand } from \"./domain.js\"\nimport \"./domain.js\"\n\n"
    ));
    assert!(generated
        .typescript
        .contains("export const keep = (hand: LocalHand) => hand"));
}

#[test]
fn keeps_same_export_spelling_distinct_through_canonical_import_aliases() {
    let first_source = "pub fn render value: Int -> String = \"first\"\n";
    let second_source = "pub fn render value: Int -> String = \"second\"\n";
    let main_source = "import { render as renderFirst } from \"./first\"\nimport { render as renderSecond } from \"./second\"\n\npub fn both value: Int -> (String, String) = (renderFirst value, renderSecond value)\n";
    let core = linked_core(
        main_source,
        [
            ("./first", "fixture/game::first", first_source),
            ("./second", "fixture/game::second", second_source),
        ],
    );
    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([
            ("fixture/game::first", "./first.js"),
            ("fixture/game::second", "./second.js"),
        ]),
    )
    .unwrap();

    assert_eq!(
        typescript.source_imports[0].bindings[0].local,
        "renderFirst"
    );
    assert_eq!(
        typescript.source_imports[1].bindings[0].local,
        "renderSecond"
    );
    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .contains("import { render as renderFirst } from \"./first.js\""));
    assert!(generated
        .typescript
        .contains("import { render as renderSecond } from \"./second.js\""));
    assert!(generated
        .typescript
        .contains("[renderFirst(value), renderSecond(value)] as const"));
}

#[test]
fn freshens_a_runtime_helper_around_a_source_module_import() {
    let domain_source = "pub fn identity value: Int -> Int = value\n";
    let main_source = "import { identity as _ssrg_int64_add } from \"./domain\"\n\npub fn addOne value: Int -> Int = _ssrg_int64_add value + 1\n";
    let core = linked_core(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &plan([("fixture/game::domain", "./domain.js")]),
    )
    .unwrap();

    assert_eq!(
        typescript.source_imports[0].bindings[0].local,
        "_ssrg_int64_add"
    );
    assert!(typescript
        .imports
        .iter()
        .any(|import| import.feature == "core.int64.add" && import.local == "_ssrg_int64_add_1"));
    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .contains("import { identity as _ssrg_int64_add } from \"./domain.js\""));
    assert!(generated
        .typescript
        .contains("import { add as _ssrg_int64_add_1 } from \"@seseragi/runtime/int64\""));
    assert!(generated
        .typescript
        .contains("_ssrg_int64_add_1(_ssrg_int64_add(value), 1n)"));
}

fn linked_core<const N: usize>(
    main_source: &str,
    dependencies: [(&str, &str, &str); N],
) -> seseragi_lowering::CoreModule {
    let targets = dependencies
        .into_iter()
        .map(|(specifier, module, source)| {
            let unlinked = parse_unlinked_module_interface("dependency.ssrg", module, source);
            let interface = analyze_module_interface(
                parse_diagnostics("dependency.ssrg", source),
                unlinked.interface.clone(),
                source,
            )
            .unwrap()
            .typed_interface
            .into_link_interface();
            (
                specifier.to_owned(),
                ModuleLinkTarget::same_package(unlinked.header, interface).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    let linked = link_module(main, &targets).unwrap();
    let analyzed = analyze_linked_module(
        parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    lower_typed_module(analyzed.typed_hir)
}

fn plan<const N: usize>(entries: [(&str, &str); N]) -> TypeScriptOutputPlan {
    TypeScriptOutputPlan::new(
        entries
            .into_iter()
            .map(|(module, specifier)| (module.to_owned(), specifier.to_owned())),
    )
}
