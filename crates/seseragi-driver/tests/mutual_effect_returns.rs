use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::TypeScriptFunction;

const ORIGINAL: &str = r#"
import * as effects from "std/effect"
effect fn checkpoint _trigger: Unit -> String = do {
  effects.yieldNow ()
  wait ()
}
fn wait -> Effect<{}, Never, String> = effects.defer checkpoint
pub effect fn main = println "ready"
"#;

#[test]
fn a_local_same_spelling_does_not_create_a_top_level_cycle() {
    let source = "import * as effects from \"std/effect\"\nfn next wait: (Unit -> Effect<{}, Never, String>) -> Effect<{}, Never, String> = wait ()\nfn wait -> Effect<{}, Never, String> = effects.succeed \"ready\"\npub effect fn main = println \"ready\"\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("shadow.ssrg", "fixture/shadow", source).with_profile(profile),
        )
        .unwrap();
        for function in &compiled.typescript_ir.functions {
            let TypeScriptFunction::ConstFunction { value_type, .. } = function;
            assert!(value_type.is_none(), "{}", compiled.generated.typescript);
        }
    }
}

#[test]
fn mutually_recursive_closed_effects_keep_checked_value_annotations() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("mutual.ssrg", "fixture/mutual", ORIGINAL).with_profile(profile),
        )
        .unwrap();
        for (name, annotated) in [("wait", true), ("checkpoint", false)] {
            let TypeScriptFunction::ConstFunction { value_type, .. } = compiled
                .typescript_ir
                .functions
                .iter()
                .find(|function| {
                    let TypeScriptFunction::ConstFunction { name: actual, .. } = function;
                    actual == name
                })
                .unwrap();
            assert_eq!(
                value_type.is_some(),
                annotated,
                "{name}: {}",
                compiled.generated.typescript
            );
        }
        let TypeScriptFunction::ConstFunction { value_type, .. } = compiled
            .typescript_ir
            .functions
            .iter()
            .find(|function| {
                let TypeScriptFunction::ConstFunction { name, .. } = function;
                name == "main"
            })
            .unwrap();
        assert!(
            value_type.is_none(),
            "acyclic main must keep its inferred contract"
        );
        assert!(!compiled.generated.typescript.contains(": any"));
    }
}
