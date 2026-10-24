use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::CoreExpr;
use seseragi_semantics::{TypedDecl, TypedExpr};

#[test]
fn literal_succeed_keeps_one_checked_unit_value_through_typed_and_core() {
    for (name, source) in [
        ("plain", "pub effect fn literal = succeed ()\n"),
        (
            "nominal",
            "struct Unit {value: Int}\npub effect fn literal = succeed ()\n",
        ),
        (
            "scoped",
            "pub effect fn literal<Unit> value: Unit = succeed ()\n",
        ),
        ("annotated", "pub effect fn literal -> Unit = succeed ()\n"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new("unit-value-雪🙂.ssrg", "fixture/unit-value", source)
                    .with_profile(profile),
            )
            .unwrap();
            assert!(compiled.diagnostics.diagnostics.is_empty());
            let body = compiled
                .typed_hir
                .declarations
                .iter()
                .find_map(|decl| match decl {
                    TypedDecl::EffectFn { symbol, body, .. } if symbol.ends_with("::literal") => {
                        Some(body)
                    }
                    _ => None,
                })
                .unwrap();
            let TypedExpr::EffectCall {
                effect, arguments, ..
            } = body
            else {
                panic!("the prelude operation remains a checked cold effect");
            };
            let [TypedExpr::Unit { type_ref, .. }] = arguments.as_slice() else {
                panic!("{name}: succeed takes a Unit value, not zero source arguments");
            };
            assert_eq!(&effect.success, type_ref);
            assert!(matches!(&compiled.core_ir.functions[0].body,
                CoreExpr::EffectOperation { operation, arguments, .. }
                if operation == "effect.succeed" && matches!(arguments.as_slice(), [CoreExpr::Unit { .. }])));
            // TS already materializes this source value as undefined; keeping
            // it in shared IR must not change the established runtime call.
            assert!(compiled
                .generated
                .typescript
                .contains("_ssrg_effect_succeed(undefined)"));
            if let Some(directory) = std::env::var_os("SESERAGI_UNIT_VALUE_CAPTURE_DIR") {
                let path = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&path).unwrap();
                let profile = format!("{profile:?}");
                for (suffix, text) in [
                    (
                        "typed.json",
                        serde_json::to_string_pretty(&compiled.typed_hir).unwrap(),
                    ),
                    (
                        "core.json",
                        serde_json::to_string_pretty(&compiled.core_ir).unwrap(),
                    ),
                    ("ts", compiled.generated.typescript),
                ] {
                    std::fs::write(path.join(format!("{name}-{profile}.{suffix}")), text).unwrap();
                }
            }
        }
    }
}

#[test]
fn readline_retains_its_separate_zero_runtime_argument_contract() {
    let compiled = compile_module(CompileInput::new(
        "read-unit.ssrg",
        "fixture/read-unit",
        "pub effect fn input = readLine ()\n",
    ))
    .unwrap();
    assert!(matches!(&compiled.core_ir.functions[0].body,
        CoreExpr::EffectOperation { arguments, .. } if arguments.is_empty()));
}
