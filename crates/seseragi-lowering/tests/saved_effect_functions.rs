use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module, CoreExpr,
    CoreStatement, CoreType,
};
use seseragi_project::{link_module, standard_module_target};
use seseragi_semantics::analyze_linked_module;
use seseragi_syntax::{parse_diagnostics, parse_unlinked_module_interface};
use std::collections::BTreeMap;

fn core(source: &str) -> seseragi_lowering::CoreModule {
    let imports = ["std/console", "std/stdin", "std/effect"]
        .map(|name| (name.to_owned(), standard_module_target(name).unwrap()));
    let unlinked = parse_unlinked_module_interface("saved.ssrg", "fixture/saved", source);
    let linked = link_module(unlinked, &BTreeMap::from(imports)).unwrap();
    let analyzed = analyze_linked_module(parse_diagnostics("saved.ssrg", source), linked, source)
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
    lower_typed_module(analyzed.typed_hir).expect("valid test lowering input")
}

#[test]
fn saved_effect_functions_remain_function_values() {
    for (reference, name, runtime, call) in [
        (
            "console.flush",
            "std/console::flush",
            "_ssrg_console_flush",
            "saved(undefined)",
        ),
        (
            "stdin.readLine",
            "std/stdin::readLine",
            "_ssrg_stdin_readLine",
            "saved(undefined)",
        ),
        (
            "effects.succeed",
            "std/effect::succeed",
            "_ssrg_effect_succeed",
            "saved<undefined>(undefined)",
        ),
    ] {
        for block in ["", "do "] {
            let source = format!(
                "import * as console from \"std/console\"\n\
                 import * as stdin from \"std/stdin\"\n\
                 import * as effects from \"std/effect\"\n\
                 pub effect fn main = {block}{{ let saved = ({reference}); saved () }}\n"
            );
            let core = core(&source);
            let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
                panic!("{source}\n{:?}", core.functions[0].body);
            };
            assert!(
                matches!(statements.first(), Some(CoreStatement::PureLet {
                value: CoreExpr::Variable { name: actual, evidence, type_ref: CoreType::Function { .. }, .. },
                ..
            }) if actual == name && evidence.is_empty()),
                "{source}\n{statements:#?}"
            );
            let generated = emit_typescript_module(
                lower_core_module_to_typescript_ir(core).expect("valid test lowering input"),
                &source,
            )
            .expect("valid test lowering input");
            assert!(
                generated.typescript.contains(&format!("= {runtime};")),
                "{source}\n{}",
                generated.typescript
            );
            assert!(
                generated.typescript.contains(call),
                "{source}\n{}",
                generated.typescript
            );
        }
    }
}

#[test]
fn passing_effect_functions_as_arguments_does_not_invoke_them() {
    let source = "import * as console from \"std/console\"\n\
                  import * as stdin from \"std/stdin\"\n\
                  import * as effects from \"std/effect\"\n\
                  fn keep<A> value: A -> A = value\n\
                  pub effect fn main -> Unit with Stdin, Console fails String = do {\n\
                    let flush = keep console.flush\n\
                    let read = keep stdin.readLine\n\
                    flush () |> effects.mapError show\n\
                    read () |> effects.mapError show\n\
                    effects.succeed ()\n\
                  }\n";
    let core = core(source);
    let CoreExpr::Sequence { statements, .. } = &core.functions[1].body else {
        panic!("expected saved function bindings");
    };
    for (statement, name) in statements[..2]
        .iter()
        .zip(["std/console::flush", "std/stdin::readLine"])
    {
        assert!(
            matches!(statement, CoreStatement::PureLet {
            value: CoreExpr::Call { arguments, .. }, ..
        } if matches!(arguments.as_slice(), [CoreExpr::Call {
            callee: actual, arguments, evidence, type_ref: CoreType::Function { .. }, ..
        }] if actual == name && arguments.is_empty() && evidence.is_empty())),
            "{statement:#?}"
        );
    }
    let generated = emit_typescript_module(
        lower_core_module_to_typescript_ir(core).expect("valid test lowering input"),
        source,
    )
    .expect("valid test lowering input");
    assert!(!generated.typescript.contains("_ssrg_console_flush()"));
    assert!(!generated.typescript.contains("_ssrg_stdin_readLine()"));
}

#[test]
fn applied_unit_functions_construct_effects() {
    for (reference, runtime) in [
        ("console.flush", "_ssrg_console_flush"),
        ("stdin.readLine", "_ssrg_stdin_readLine"),
    ] {
        let source = format!(
            "import * as console from \"std/console\"\n\
             import * as stdin from \"std/stdin\"\n\
             pub effect fn main = {{ let action = {reference} (); action }}\n"
        );
        let core = core(&source);
        let CoreExpr::Sequence { statements, .. } = &core.functions[0].body else {
            panic!("expected the effect binding");
        };
        assert!(
            matches!(statements.first(), Some(CoreStatement::PureLet {
            value: CoreExpr::EffectInvoke { arguments, .. }, ..
        }) if matches!(arguments.as_slice(), [CoreExpr::Unit { .. }])),
            "{statements:#?}"
        );
        let generated = emit_typescript_module(
            lower_core_module_to_typescript_ir(core).expect("valid test lowering input"),
            &source,
        )
        .expect("valid test lowering input");
        assert!(
            generated
                .typescript
                .contains(&format!("{runtime}(undefined)")),
            "{}",
            generated.typescript
        );
    }
}
