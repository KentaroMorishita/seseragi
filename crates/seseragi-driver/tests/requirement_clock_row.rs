use seseragi_driver::{compile_module, CompileInput};

fn source(operation: &str) -> String {
    format!(
        r#"import * as effects from "std/effect"
import * as time from "std/time"

effect fn work -> Int with Clock fails String = effects.succeed 1

pub fn probe duration: time.Duration -> Int = {operation}
"#
    )
}

#[test]
fn temporal_controls_keep_one_clock_when_the_source_already_requires_clock() {
    for operation in [
        r#"effects.attempt (effects.timeoutFail "late" duration (work ()))"#,
        r#"effects.attempt (effects.timeout duration (work ()))"#,
    ] {
        let source = source(operation);
        let errors = compile_module(CompileInput::new(
            "main.ssrg",
            "fixture/requirement-clock-row",
            &source,
        ))
        .expect_err("the probe returns Int, so the Effect stays a type error");
        let mismatch = errors
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "SES-T0101")
            .unwrap_or_else(|| panic!("{operation}\n{errors:#?}"));
        let (_, actual) = mismatch.expected_actual_types();
        let actual = actual.unwrap_or_else(|| panic!("{operation}\n{mismatch:#?}"));
        assert!(
            !actual.contains("clock: Clock, clock: Clock"),
            "{operation}\n{actual}"
        );
        assert!(actual.contains("{ clock: Clock }"), "{operation}\n{actual}");
    }
}

#[test]
fn annotated_attempt_accepts_the_single_clock_row() {
    let source = r#"import * as effects from "std/effect"
import * as time from "std/time"
import { Clock } from "std/clock"

effect fn work -> Int with Clock fails String = effects.succeed 1

pub fn probe duration: time.Duration -> Effect<{ clock: Clock }, Never, Either<String, Int>> =
  effects.attempt (effects.timeoutFail "late" duration (work ()))
"#;
    compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/requirement-clock-row-ok",
        source,
    ))
    .unwrap_or_else(|error| panic!("{error:#?}"));
}
