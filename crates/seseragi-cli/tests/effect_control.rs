use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-effect-control-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn compare(source: &str, stdout: &[u8], error: Option<(&str, &str)>) {
    let scratch = Scratch::new();
    let file = scratch.0.join("effect-control-雪🙂.ssrg");
    fs::write(&file, source).unwrap();
    let ts = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&file)
            .args(["--diagnostic-format", "json"]),
        &scratch.0,
    );
    let exit = match error {
        Some(("Defect", _)) => 70,
        Some(("TypedFailure", _)) => 1,
        None => 0,
        _ => panic!("unexpected error kind"),
    };
    assert_eq!(ts.status.code(), Some(exit), "{ts:?}");
    assert_eq!(ts.stdout, stdout);
    if let Some((kind, message)) = error {
        let diagnostic: serde_json::Value = serde_json::from_slice(&ts.stderr).unwrap();
        assert_eq!(diagnostic["kind"], kind);
        assert_eq!(diagnostic["message"], message);
    } else {
        assert!(ts.stderr.is_empty(), "{ts:?}");
    }
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&file)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let result = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(result.status.code(), ts.status.code(), "{result:?}");
    assert_eq!(result.stdout, ts.stdout);
    let stderr = String::from_utf8(result.stderr).unwrap();
    let mut lines = stderr.lines();
    if let Some((kind, message)) = error {
        let diagnostic = lines.next().unwrap();
        assert!(
            diagnostic.starts_with(&format!("{}:", file.display())),
            "{diagnostic}"
        );
        let expected = if kind == "Defect" {
            format!("Seseragi Defect: {message}")
        } else {
            message.to_owned()
        };
        assert!(
            diagnostic.ends_with(&format!(": {expected}")),
            "{diagnostic}"
        );
    }
    assert!(
        lines.next().unwrap().starts_with("native managed-live=0 "),
        "{stderr}"
    );
    assert_eq!(lines.next(), None, "{stderr}");
}

#[test]
fn direct_nothing_failure_payloads_and_nested_attempt_match_ts_and_native_at_runtime() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-effect-control-effects.ssrg"
    );
    for needle in ["require total absent", "require \"nested\" absent"] {
        assert_eq!(original.matches(needle).count(), 1);
    }
    let source = original
        .replace("require total absent", "effects.fromMaybe total absent")
        .replace(
            "require \"nested\" absent",
            "effects.fromMaybe \"nested\" absent",
        );
    compare(
        &source,
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-effect-control-effects.stdout"
        ),
        None,
    );
}

fn source(expression: &str) -> String {
    format!(
        r#"
import * as effects from "std/effect"
import * as iterator from "std/iterator"
effect fn ignore _error:ConsoleError -> Unit with Console = effects.succeed ()
effect fn say message:String -> Unit with Console = println message |> effects.recover ignore
fn advance n:Int -> Maybe<(Int,Int)> = Just (n,n + 1)
newtype Broken = Unit
instance Iterable<Broken,Int> {{
  fn iterate _source:Broken -> Iterator<Int> = if 42 / 0 == 0 then iterator.unfold advance 0 else iterator.unfold advance 1
}}
fn step _n:Int -> Effect<{{}},Never,effects.LoopControl> = effects.succeed effects.Continue
fn brokenConstruct _n:Int -> Effect<{{}},Never,effects.LoopControl> = if 42 / 0 == 0 then effects.succeed effects.Break else effects.succeed effects.Continue
effect fn brokenRelease _value:Unit -> Unit = effects.die "deliberate"
effect fn brokenEffect value:Int -> effects.LoopControl with Console = do {{
  say ("action: " + show value)
  effects.die "deliberate"
}}
pub effect fn main = do {{
  effects.acquireRelease (say "acquired") (\_:Unit -> say "released")
  let absent:Maybe<Int> = Nothing
  _result <- {expression}
  say "continued"
}}
"#
    )
}

#[test]
fn attempt_propagates_explicit_defects_and_for_each_until_stops_later_actions() {
    for (expression, stdout) in [
        (
            "effects.attempt (effects.die \"deliberate\")",
            b"acquired\nreleased\n".as_slice(),
        ),
        (
            "effects.attempt (effects.forEachUntil brokenEffect [1,2,3])",
            b"acquired\naction: 1\nreleased\n".as_slice(),
        ),
    ] {
        compare(&source(expression), stdout, Some(("Defect", "deliberate")));
    }
}

#[test]
fn from_maybe_arguments_and_iterable_stages_keep_defects_and_finalizers() {
    for expression in [
        "effects.attempt (effects.fromMaybe (42 / 0) (Just 1))",
        "effects.attempt (effects.fromMaybe \"missing\" (Just (42 / 0)))",
        "effects.attempt (effects.forEachUntil brokenConstruct [1,2,3])",
        "effects.attempt (effects.forEachUntil step (Broken ()))",
        "effects.attempt (effects.forEachUntil step (iterator.unfold (\\n:Int -> Just (42 / 0,n + 1)) 0))",
    ] {
        compare(&source(expression), b"acquired\nreleased\n", Some(("Defect", "Seseragi Int division by zero")));
    }
}

#[test]
fn from_maybe_nothing_is_an_unhandled_typed_failure() {
    compare(
        &source("effects.fromMaybe \"missing\" absent"),
        b"acquired\nreleased\n",
        Some(("TypedFailure", "missing")),
    );
}

#[test]
fn from_maybe_nothing_as_the_entry_result_keeps_the_typed_failure_kind() {
    let expression = "effects.fromMaybe \"missing\" absent";
    let original = source(expression);
    let tail = format!("_result <- {expression}\n  say \"continued\"");
    assert_eq!(original.matches(&tail).count(), 1);
    assert_eq!(
        original.matches("let absent:Maybe<Int> = Nothing").count(),
        1
    );
    compare(
        &original.replace(&tail, expression).replace(
            "let absent:Maybe<Int> = Nothing",
            "let absent:Maybe<Unit> = Nothing",
        ),
        b"acquired\nreleased\n",
        Some(("TypedFailure", "missing")),
    );
}

#[test]
fn attempt_does_not_convert_a_scoped_finalizer_defect_into_a_value() {
    compare(&source("effects.attempt (effects.scoped (do { effects.acquireRelease (effects.succeed ()) brokenRelease; effects.fromMaybe \"typed\" absent }))"), b"acquired\nreleased\n", Some(("Defect", "deliberate")));
}
