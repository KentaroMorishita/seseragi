use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(
        "entry-failure.ssrg",
        "native/entry-failure",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn source(expression: &str) -> String {
    format!("import * as effects from \"std/effect\"\npub effect fn main = do {{\n  let absent:Maybe<Unit> = Nothing\n  _result <- {expression}\n  effects.succeed ()\n}}\n")
}

#[test]
fn entry_failure_show_keeps_failures_before_a_never_failing_continuation() {
    for expression in [
        "effects.fail \"missing\"",
        "effects.fromMaybe \"missing\" absent",
    ] {
        let core = checked(&source(expression));
        let artifact = emit_rust_module(&core, "main").unwrap();
        assert!(
            !artifact.source.contains("Rust ABI: Never failure"),
            "entry type: {:?}",
            core.functions[0].value_type
        );
    }
}

#[test]
fn entry_failure_show_requires_a_proven_unit_to_effect_signature() {
    let core = checked(&source("effects.fromMaybe \"missing\" absent"));
    emit_rust_module(&core, "main").unwrap();
    let original = core.functions[0].value_type.as_ref().unwrap();
    for replacement in [
        None,
        Some(CoreType::Hole),
        Some(CoreType::Named {
            name: "std/prelude::Unit".into(),
            arguments: vec![],
        }),
    ] {
        let mut bad = core.clone();
        bad.functions[0].value_type = replacement;
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&core.functions[0].origin));
    }
    for mutation in 0..4 {
        let mut bad = core.clone();
        let mut signature = original.clone();
        let CoreType::Function { parameter, result } = &mut signature else {
            panic!("checked function")
        };
        match mutation {
            0 => {
                **parameter = CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                }
            }
            1 => **result = CoreType::Hole,
            _ => {
                let CoreType::Named { name, arguments } = result.as_mut() else {
                    panic!("checked Effect")
                };
                if mutation == 2 {
                    *name = "other::Effect".into();
                } else {
                    arguments.pop();
                }
            }
        }
        bad.functions[0].value_type = Some(signature);
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.as_ref(), Some(&core.functions[0].origin));
    }
}
