use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir_with_plan, lower_typed_module,
    CoreExpr, CoreStatement, TypeScriptExpr, TypeScriptFunction, TypeScriptOutputPlan,
    TypeScriptStatement,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_semantics::{analyze_linked_module, analyze_module_interface};
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::collections::BTreeMap;

#[test]
fn lowers_an_imported_effect_invoke_to_a_cold_source_call() {
    let domain_source = "pub effect fn prompt label: String -> Maybe<String>\nwith Stdin\nfails StdinError =\n  readLine ()\n";
    let main_source = "import { prompt } from \"./domain\"\n\npub effect fn ask =\n  do {\n    value <- prompt \"hand\"\n    succeed value\n  }\n";
    let core = linked_core(main_source, domain_source);
    assert!(matches!(
        &core.functions[0].body,
        CoreExpr::Sequence { statements, .. }
            if matches!(
                statements.as_slice(),
                [CoreStatement::Bind {
                    value: CoreExpr::EffectInvoke { callee, .. },
                    ..
                }] if callee == "fixture/game::domain::prompt"
            )
    ));

    let typescript = lower_core_module_to_typescript_ir_with_plan(
        core,
        &TypeScriptOutputPlan::new(BTreeMap::from([(
            "fixture/game::domain".to_owned(),
            "./domain.js".to_owned(),
        )])),
    )
    .unwrap();
    assert!(typescript.source_imports[0]
        .bindings
        .iter()
        .any(|binding| binding.imported == "prompt" && binding.local == "prompt"));
    assert!(!typescript
        .imports
        .iter()
        .any(|import| import.local.contains("prompt")));
    assert!(matches!(
        &typescript.functions[0],
        TypeScriptFunction::ConstFunction {
            is_async: false,
            body: TypeScriptExpr::Sequence { statements, .. },
            ..
        } if matches!(
            statements.as_slice(),
            [TypeScriptStatement::Const {
                initializer: TypeScriptExpr::Call { callee, .. },
                ..
            }] if callee == "prompt"
        )
    ));

    let generated = emit_typescript_module(typescript, main_source);
    assert!(generated
        .typescript
        .contains("import { prompt } from \"./domain.js\""));
    assert!(generated
        .typescript
        .contains("_ssrg_effect_flatMap(prompt(\"hand\")"));
    assert!(!generated.typescript.contains("await prompt"));
    assert!(!generated
        .typescript
        .contains("_ssrg_fixture_game_domain_prompt"));
}

fn linked_core(main_source: &str, domain_source: &str) -> seseragi_lowering::CoreModule {
    let unlinked =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let interface = analyze_module_interface(
        parse_diagnostics("domain.ssrg", domain_source),
        unlinked.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let target = ModuleLinkTarget::same_package(unlinked.header, interface).unwrap();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    let linked = link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap();
    let analyzed = analyze_linked_module(
        parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    lower_typed_module(analyzed.typed_hir)
}
