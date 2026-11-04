use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_semantics::{TypedBlockStatement, TypedDecl, TypedDoStatement, TypedExpr, TypedType};

#[test]
fn saved_log_functions_are_not_called_without_an_event() {
    for (imports, declaration) in [
        (
            "",
            "let saved = logs.log\npub effect fn main = effects.succeed ()",
        ),
        (
            "",
            "pub effect fn main = { let saved = logs.log; effects.succeed () }",
        ),
        (
            "",
            "pub effect fn main = do { let saved = logs.log; effects.succeed () }",
        ),
        (
            "",
            "pub effect fn main = { let saved = (logs.log); effects.succeed () }",
        ),
        (
            "import { log } from \"std/log\"\n",
            "pub effect fn main = { let saved = log; effects.succeed () }",
        ),
    ] {
        let source = format!(
            "import * as logs from \"std/log\"\nimport * as effects from \"std/effect\"\n{imports}{declaration}\n"
        );
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new("saved-log.ssrg", "fixture/saved-log", &source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{source}\n{error:#?}"));
            assert!(compiled.diagnostics.diagnostics.is_empty());
            let initializer = compiled
                .typed_hir
                .declarations
                .iter()
                .find_map(|declaration| match declaration {
                    TypedDecl::Let { value, .. } => Some(value),
                    TypedDecl::EffectFn { body, .. } => match body {
                        TypedExpr::Block { statements, .. } => match statements.first()? {
                            TypedBlockStatement::Let { value, .. } => Some(value),
                            _ => None,
                        },
                        TypedExpr::DoBlock { statements, .. } => match statements.first()? {
                            TypedDoStatement::PureLet { value, .. } => Some(value),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                })
                .unwrap();
            assert!(
                matches!(initializer, TypedExpr::Variable {
                    name, type_ref: TypedType::Function { .. }, ..
                } if name == "std/log::log"),
                "{source}\n{initializer:#?}"
            );
            let ts = compiled.generated.typescript;
            assert!(!ts.contains("_ssrg_logger_log()"), "{source}\n{ts}");
        }
    }
}

#[test]
fn saved_log_functions_require_the_same_event_type_as_direct_calls() {
    for (declarations, argument) in [
        ("", "True"),
        ("", "()"),
        ("struct LogEvent {}\n", "LogEvent {}"),
    ] {
        for body in [
            format!("let saved = logs.log\npub effect fn main = saved ({argument})"),
            format!("pub effect fn main = {{ let saved = logs.log; saved ({argument}) }}"),
            format!("pub effect fn main = do {{ let saved = logs.log; saved ({argument}) }}"),
        ] {
            let source = format!("import * as logs from \"std/log\"\n{declarations}{body}\n");
            for profile in [BuildProfile::Development, BuildProfile::Release] {
                let error = compile_module(
                    CompileInput::new("saved-log.ssrg", "fixture/saved-log", &source)
                        .with_profile(profile),
                )
                .expect_err("a saved log function still requires std/log LogEvent");
                assert!(
                    error.diagnostics.iter().any(|diagnostic| {
                        diagnostic.code == "SES-T0101"
                            && diagnostic.message_key == "call.argument-type-mismatch"
                    }),
                    "{source}\n{error:#?}"
                );
            }
        }
    }
}

#[test]
fn saved_log_references_preserve_invalid_type_argument_diagnostics() {
    for declaration in [
        "let saved = logs.log<Int>\npub effect fn main = effects.succeed ()",
        "pub effect fn main = { let saved = logs.log<Int>; effects.succeed () }",
        "pub effect fn main = do { let saved = logs.log<Int>; effects.succeed () }",
    ] {
        let source = format!(
            "import * as logs from \"std/log\"\nimport * as effects from \"std/effect\"\n{declaration}\n"
        );
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let error = compile_module(
                CompileInput::new("saved-log.ssrg", "fixture/saved-log", &source)
                    .with_profile(profile),
            )
            .expect_err("std/log log does not accept type arguments");
            assert_eq!(error.diagnostics.len(), 1, "{source}\n{error:#?}");
            assert_eq!(error.diagnostics[0].code, "SES-T0101");
            assert_eq!(
                error.diagnostics[0].message_key,
                "call.type-argument-arity-mismatch"
            );
        }
    }
}
