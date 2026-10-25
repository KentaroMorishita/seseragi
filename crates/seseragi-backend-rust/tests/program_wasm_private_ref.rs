use seseragi_backend_rust::emit_program_wasm;
use seseragi_driver::{compile_core_module, CompileInput};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "private-ref-雪.ssrg",
        "fixture/private-ref",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn private_cells_preserve_the_existing_data_transport_boundary() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/program-wasm-private-ref.ssrg"
    ));
    let artifact = emit_program_wasm(
        &[&module],
        &module.module,
        &["create", "reader", "unit", "failing", "snapshot", "session"].map(String::from),
    )
    .unwrap();
    assert_eq!(artifact.metadata.abi, "seseragi-program-wasm-effect-v1");
    assert!(artifact.source.contains("PROGRAM_TRACK_REFS: bool = true"));
    assert!(artifact
        .declarations
        .contains("OwnedCallback<string, OwnedEffect<never, string>>"));
    assert!(!artifact.declarations.contains("Ref<"));
}

#[test]
fn cell_handles_and_callable_payloads_cannot_cross_or_hide_in_the_boundary() {
    for source in [
        "import * as refs from \"std/ref\"\nimport { Ref } from \"std/ref\"\npub effect fn value initial: Int -> Ref<Int> = refs.make initial\n",
        "import * as refs from \"std/ref\"\nimport { Ref } from \"std/ref\"\npub effect fn value cell: Ref<Int> -> Int = refs.get cell\n",
        "import * as refs from \"std/ref\"\nimport * as effects from \"std/effect\"\nfn identity value: Int -> Int = value\npub effect fn value initial: Int -> Int = do { cell <- refs.make identity; saved <- refs.get cell; effects.succeed (saved initial) }\n",
        "import * as refs from \"std/ref\"\nimport * as effects from \"std/effect\"\npub effect fn value initial: Int -> Int = do { cell <- refs.make (effects.succeed initial); saved <- refs.get cell; saved }\n",
    ] {
        let module = checked(source);
        let error = emit_program_wasm(&[&module], &module.module, &["value".into()])
            .expect_err("a public cell or unadmitted private payload produced an artifact");
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.expect("a refusal retains its source range");
        assert!(origin.end > origin.start);
    }
}

#[test]
fn private_cells_do_not_expand_the_separate_host_callback_contract() {
    let module = checked(
        "import * as refs from \"std/ref\"\npub effect fn value callback: (String -> Effect<{}, String, String>) -> String fails String = do { cell <- refs.make \"seed\"; input <- refs.get cell; callback input }\n",
    );
    let error = emit_program_wasm(&[&module], &module.module, &["value".into()]).unwrap_err();
    assert!(error.message.contains("providerless Effect ABI"), "{error}");
    assert!(error.origin.is_some());
}

#[test]
fn cell_operations_prove_source_arity_and_payload_before_emitting_runtime_code() {
    use seseragi_lowering::CoreExpr;
    let original = checked(
        "import * as refs from \"std/ref\"\nimport * as effects from \"std/effect\"\nimport { Ref } from \"std/ref\"\neffect fn store value: String -> cell: Ref<String> -> Unit = refs.set value cell\npub effect fn value input: String -> String = effects.succeed input\n",
    );
    emit_program_wasm(&[&original], &original.module, &["value".into()]).unwrap();
    for mutation in 0..3 {
        let mut module = original.clone();
        let store = module
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::store"))
            .unwrap();
        let (arguments, origin) = match &mut store.body {
            CoreExpr::Call {
                arguments, origin, ..
            }
            | CoreExpr::EffectInvoke {
                arguments, origin, ..
            }
            | CoreExpr::EffectOperation {
                arguments, origin, ..
            } => (arguments, origin),
            other => panic!("expected a checked cell operation: {other:?}"),
        };
        match mutation {
            0 => {
                arguments.pop();
            }
            1 => {
                arguments[0] = CoreExpr::Integer {
                    value: "42".into(),
                    origin: origin.clone(),
                };
            }
            _ => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                    panic!("checked cell parameter")
                };
                *type_ref = seseragi_lowering::CoreType::Named {
                    name: "other/ref::Ref".into(),
                    arguments: vec![],
                };
            }
        }
        let error = emit_program_wasm(&[&module], &module.module, &["value".into()])
            .expect_err("an unproved cell operation produced an artifact");
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.expect("cell refusal retains its source range");
        assert!(origin.end > origin.start);
    }
}
