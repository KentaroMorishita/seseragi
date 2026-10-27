use seseragi_driver::{
    compile_core_module, compile_module, BuildProfile, CompileInput, CompiledModule,
};
use seseragi_semantics::{TypedDecl, TypedEffect, TypedType};

#[path = "support/frontend_evidence.rs"]
mod evidence;

const PRELUDE: &str = r#"
import * as effects from "std/effect"
effect fn ignore _error: ConsoleError -> Unit with Console = effects.succeed ()
effect fn say message: String -> Unit with Console = println message |> effects.recover ignore
"#;

fn effect<'a>(compiled: &'a CompiledModule, name: &str) -> &'a TypedEffect {
    compiled
        .typed_hir
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::EffectFn { symbol, effect, .. }
                if symbol.ends_with(&format!("::{name}")) =>
            {
                Some(effect)
            }
            _ => None,
        })
        .unwrap()
}

fn compile_both(source: &str, mut check: impl FnMut(&CompiledModule)) {
    let input = CompileInput::new("deferred-雫.ssrg", "fixture/deferred-environment", source);
    let core = compile_core_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(input.with_profile(profile))
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
        assert_eq!(compiled.core_ir, core.core_ir);
        check(&compiled);
    }
}

#[test]
fn compact_branch_tails_keep_the_environment_of_preceding_executed_statements() {
    for tail in [
        "if True then effects.succeed 42 else effects.succeed 0",
        "match True { True -> effects.succeed 42; False -> effects.succeed 0 }",
    ] {
        let source = format!(
            "{PRELUDE}\neffect fn compact = do {{ say \"before\"; {tail} }}\n\
             effect fn named -> Int with Console = do {{ say \"before\"; {tail} }}\n"
        );
        compile_both(&source, |compiled| {
            let compact = effect(compiled, "compact");
            assert_eq!(compact, effect(compiled, "named"));
            assert_eq!(compact.success, named("Int"));
            let TypedType::Record {
                closed: true,
                fields,
            } = &compact.environment
            else {
                panic!("closed environment: {compact:?}")
            };
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "console");
        });
    }
}

#[test]
fn compact_branch_tails_keep_the_failure_of_preceding_executed_statements() {
    let source = format!(
        "{PRELUDE}\neffect fn compact = do {{ effects.fail \"prefix failure\"; \
         if True then effects.succeed 42 else effects.succeed 0 }}\n\
         effect fn named -> Int fails String = do {{ effects.fail \"prefix failure\"; \
         if True then effects.succeed 42 else effects.succeed 0 }}\n"
    );
    compile_both(&source, |compiled| {
        let compact = effect(compiled, "compact");
        assert_eq!(compact, effect(compiled, "named"));
        assert_eq!(compact.failure, named("String"));
        assert_eq!(compact.success, named("Int"));
    });
}

#[test]
fn a_nested_cold_effect_is_the_success_value_of_one_execution() {
    let source = format!(
        "{PRELUDE}\neffect fn compact = do {{ say \"outer\"; effects.succeed (say \"inner\") }}\n\
         effect fn named -> Effect<{{ console: Console }}, Never, Unit> with Console = \
         do {{ say \"outer\"; effects.succeed (say \"inner\") }}\n"
    );
    compile_both(&source, |compiled| {
        let compact = effect(compiled, "compact");
        assert_eq!(compact, effect(compiled, "named"));
        let TypedType::Named { name, arguments } = &compact.success else {
            panic!("returned cold Effect: {compact:?}")
        };
        assert_eq!(name, "Effect");
        assert_eq!(arguments.len(), 3);
        assert_eq!(arguments[2], named("Unit"));
    });
}

#[test]
fn creating_an_unused_cold_plan_does_not_add_its_contract_to_the_do_block() {
    let source = format!(
        "{PRELUDE}\neffect fn compact = do {{ let unused = say \"unused\"; \
         if True then effects.succeed 42 else effects.succeed 0 }}\n\
         effect fn named -> Int = effects.succeed 42\n"
    );
    compile_both(&source, |compiled| {
        assert_eq!(effect(compiled, "compact"), effect(compiled, "named"));
    });
}

#[test]
fn missing_environment_and_incompatible_success_or_failure_are_positioned_refusals() {
    for (declaration, key) in [
        (
            "effect fn wrong -> Bool = do { let cold = effects.defer $ \
             \\_: Unit -> do { say \"cold\"; if True then effects.succeed True \
             else effects.succeed False }; cold }",
            "effect.explicit-environment-mismatch",
        ),
        (
            "effect fn wrong -> Int with Console = do { say \"before\"; \
             if True then effects.succeed True else effects.succeed False }",
            "effect.explicit-success-mismatch",
        ),
        (
            "effect fn wrong -> Int fails Int = do { effects.fail \"prefix failure\"; \
             if True then effects.succeed 42 else effects.succeed 0 }",
            "effect.explicit-failure-mismatch",
        ),
    ] {
        let source = format!("{PRELUDE}\n{declaration}\n");
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let error = compile_module(
                CompileInput::new("wrong-deferred.ssrg", "fixture/deferred-wrong", &source)
                    .with_profile(profile),
            )
            .expect_err("an invalid Effect contract must be rejected before code generation");
            assert!(
                error.diagnostics.iter().any(|diagnostic| {
                    diagnostic.message_key == key
                        && diagnostic.primary.start < diagnostic.primary.end
                        && diagnostic.primary.end as usize <= source.len()
                }),
                "{source}\n{error:#?}"
            );
        }
    }
}

#[test]
fn original_explicit_instance_and_nested_cold_execution_fixtures_keep_core_in_both_profiles() {
    for (label, source, stdout) in [
        (
            "deferred-branch-environment",
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/deferred-branch-environment.ssrg"
            ),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/deferred-branch-environment.stdout"
            ),
        ),
        (
            "deferred-nested-cold",
            include_str!("../../../examples/spec/fixtures/rust-backend/deferred-nested-cold.ssrg"),
            include_str!(
                "../../../examples/spec/fixtures/rust-backend/deferred-nested-cold.stdout"
            ),
        ),
    ] {
        let input = CompileInput::new("deferred-雫.ssrg", "fixture/deferred-environment", source);
        let core = compile_core_module(input).unwrap_or_else(|error| panic!("{error:#?}"));
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(input.with_profile(profile))
                .unwrap_or_else(|error| panic!("{label}: {error:#?}"));
            assert_eq!(compiled.core_ir, core.core_ir);
            evidence::record(label, profile, "main", stdout, [("main", &compiled)]);
        }
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
