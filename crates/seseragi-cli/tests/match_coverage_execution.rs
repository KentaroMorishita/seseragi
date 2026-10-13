use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        loop {
            let nonce = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "seseragi-match-coverage-{}-{nonce}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create isolated match fixture: {error}"),
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn signed_integer_and_nested_collection_patterns_execute_in_both_profiles() {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.ssrg");
    fs::write(
        &path,
        r#"
type Flag = | On | Off
fn classify value: Int -> Int = match value { -1 -> 10; _ -> 20 }
fn inspect values: Array<Array<Flag>> -> Int = match values {
  [] -> 0
  [[], ...tail] -> 1
  [[On, ...rest], ...tail] -> 2
  [[Off, ...rest], ...tail] -> 3
}
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ show $ classify (-1)
  println $ show $ classify 0
  println $ show $ inspect []
  println $ show $ inspect [[]]
  println $ show $ inspect [[On]]
  println $ show $ inspect [[Off]]
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&path)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"10\n20\n0\n1\n2\n3\n", "{profile}");
        assert!(
            output.stderr.is_empty(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn malformed_or_nonexhaustive_patterns_fail_before_execution() {
    let fixture = Fixture::new();
    for (name, source, code) in [
        ("malformed", "fn inspect value: Int -> Int = match value { - -> 1; _ -> 2 }", "SES-P0001"),
        ("overflow", "fn inspect value: Int -> Int = match value { -9007199254740992 -> 1; _ -> 2 }", "SES-P0203"),
        ("recursive", "type Tree = | Leaf Int | Branch Tree\nfn inspect value: Tree -> Int = match value { Leaf item -> item }", "SES-T0301"),
    ] {
        let path = fixture.0.join(format!("{name}.ssrg"));
        fs::write(&path, format!("{source}\npub effect fn main -> Unit with Console fails ConsoleError = println \"must-not-run\"\n")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi")).arg("run").arg(&path).args(["--diagnostic-format", "json"]).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty(), "invalid source must not execute");
        assert!(String::from_utf8_lossy(&output.stderr).contains(code), "{name}: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn large_flat_collection_patterns_reject_without_aborting_the_cli_process() {
    let fixture = Fixture::new();
    for (collection, prefix) in [("Array", ""), ("List", "`")] {
        for (length, expected) in [(255, 0), (256, 2), (10_000, 2)] {
            let path = fixture
                .0
                .join(format!("{}-{length}.ssrg", collection.to_lowercase()));
            let elements = vec!["_"; length].join(", ");
            fs::write(&path, format!("pub fn inspect values: {collection}<Int> -> Int = match values {{ {prefix}[{elements}] -> 1; _ -> 0 }}\n")).unwrap();
            // A separate OS process makes a recursive clone/drop overflow a
            // failing regression, rather than aborting the Rust test runner.
            let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("lint")
                .arg(&path)
                .args(["--diagnostic-format", "json"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(expected),
                "{collection}/{length}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if expected == 2 {
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains("SES-T0303"),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

#[test]
fn primitive_boolean_predicates_execute_and_nominal_shadows_never_execute() {
    let fixture = Fixture::new();
    let valid = fixture.0.join("boolean-controls.ssrg");
    fs::write(
        &valid,
        r#"
fn guard value: Int -> Int = match value { _ when value == 1 -> 10; _ -> 20 }
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ show $ guard 1
  println $ show $ guard 0
  println $ show $ if True then 1 else 2
  println $ show $ if False then 1 else 2
  println $ show $ [n | n <- [1, 2], n == 2]
  println $ show $ `[n | n <- `[1, 2], n == 2]
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&valid)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"10\n20\n1\n2\n[2]\n`[2]\n", "{profile}");
        assert!(output.stderr.is_empty());
        for (name, expression, result, diagnostic) in [
            (
                "match",
                "match 0 { _ when Bool { value: 0 } -> 1; _ -> 2 }",
                "Int",
                "match.guard-not-bool",
            ),
            (
                "if",
                "if Bool { value: 0 } then 1 else 2",
                "Int",
                "if.condition-not-bool",
            ),
            (
                "array",
                "[n | n <- [1, 2], Bool { value: 0 }]",
                "Array<Int>",
                "match.guard-not-bool",
            ),
            (
                "list",
                "`[n | n <- `[1, 2], Bool { value: 0 }]",
                "List<Int>",
                "match.guard-not-bool",
            ),
        ] {
            let invalid = fixture.0.join(format!("nominal-bool-{name}.ssrg"));
            fs::write(&invalid, format!("struct Bool {{ value: Int }}\nfn inspect -> {result} = {expression}\npub effect fn main -> Unit with Console fails ConsoleError = println \"must-not-run\"\n")).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("run")
                .arg(&invalid)
                .args(["--profile", profile, "--diagnostic-format", "json"])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(2),
                "{profile}/{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains(diagnostic));
        }
    }
}

#[test]
fn standard_boolean_operations_keep_primitive_results_under_nominal_shadowing() {
    let fixture = Fixture::new();
    let path = fixture.0.join("operator-shadow-controls.ssrg");
    fs::write(
        &path,
        r#"
import * as arrays from "std/array"
struct Bool { value: Int }
fn nominal value: Bool -> Bool = value
fn nominalControl -> Int = { let original = Bool { value: 7 }; let forwarded = nominal original; forwarded.value }
fn choose value: Int -> Int = if value == 0 then 1 else 2
fn guard value: Int -> Int = match value { _ when value != 0 -> 10; _ -> 20 }
fn retained value: Int -> Int = { let predicate = value == 0; if predicate then 1 else 2 }
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ show $ choose 0
  println $ show $ choose 1
  println $ show $ guard 0
  println $ show $ guard 1
  println $ show $ [n | n <- [0, 1], n >= 1]
  println $ show $ `[n | n <- `[0, 1], n < 1]
  println $ show $ if (0 <= 0 && 1 > 0) || False then 3 else 4
  println $ show $ retained 0
  println $ show $ if !False then 5 else 6
  println $ show $ { let empty = arrays.isEmpty [1]; if empty then 7 else 8 }
  println $ show $ nominalControl ()
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&path)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"1\n2\n20\n10\n[1]\n`[0]\n3\n1\n5\n8\n7\n");
        assert!(output.stderr.is_empty());
        let invalid = fixture.0.join("operator-shadow-rejected.ssrg");
        fs::write(&invalid, "struct Bool { value: Int }\nfn inspect -> Int = if Bool { value: 0 } && True then 1 else 2\npub effect fn main -> Unit with Console fails ConsoleError = println \"must-not-run\"\n").unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&invalid)
            .args(["--profile", profile, "--diagnostic-format", "json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("call.argument-type-mismatch"));
    }
}

#[test]
fn unary_numeric_shadow_controls_execute_in_both_profiles() {
    let fixture = Fixture::new();
    let valid = fixture.0.join("numeric-shadow.ssrg");
    fs::write(
        fixture.0.join("numeric-probe.mjs"),
        "export const observe = value => String(value);\n",
    )
    .unwrap();
    fs::write(
        &valid,
        r#"
struct Int { value: Bool }
struct Float { value: Bool }
foreign "typescript" from "./numeric-probe.mjs" {
  pure fn observe<A> value: A -> String
}
fn inspect -> String = match (-1) { 0 -> "zero"; _ -> "negative" }
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ inspect ()
  println $ observe (-1.0)
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&valid)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"negative\n-1\n");
        assert!(output.stderr.is_empty());
        for name in ["Int", "Float"] {
            let invalid = fixture.0.join(format!("numeric-shadow-{name}.ssrg"));
            fs::write(&invalid, format!("struct {name} {{ value: Bool }}\nfn inspect value: {name} -> String = match (-value) {{ _ -> \"never\" }}\npub effect fn main -> Unit with Console fails ConsoleError = println \"must-not-run\"\n")).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("run")
                .arg(&invalid)
                .args(["--profile", profile, "--diagnostic-format", "json"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("unary.operand-type-mismatch"));
        }
    }
}

#[test]
fn composite_primitive_predicates_execute_and_nominal_fields_never_execute() {
    let fixture = Fixture::new();
    let valid = fixture.0.join("composite-primitive.ssrg");
    fs::write(&valid, r#"
struct Bool { value: Int }
fn identity<A> value: A -> A = value
fn fieldIdentity<A> value: { predicate: A } -> { predicate: A } = value
fn record -> Int = { let value = { predicate: True }; if value.predicate then 1 else 2 }
fn nested -> Int = { let value = { inner: { predicate: True } }; if value.inner.predicate then 1 else 2 }
fn spread -> Int = { let base = { predicate: Bool { value: 0 } }; let value = { ...base, predicate: True }; if value.predicate then 1 else 2 }
fn forwarded -> Int = { let value = fieldIdentity { predicate: True }; if value.predicate then 1 else 2 }
fn tuple -> Int = { let (predicate, ignored) = identity (True, 0); if predicate then 1 else 2 }
pub effect fn main -> Unit with Console fails ConsoleError = do {
  println $ show $ record ()
  println $ show $ nested ()
  println $ show $ spread ()
  println $ show $ forwarded ()
  println $ show $ tuple ()
}
"#).unwrap();
    for profile in ["development", "release"] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&valid)
            .args(["--profile", profile])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"1\n1\n1\n1\n1\n");
        assert!(output.stderr.is_empty());
        for (name, body) in [
            ("record", "{ let value = identity { predicate: Bool { value: 0 } }; if value.predicate then 1 else 2 }"),
            ("tuple", "{ let (predicate, ignored) = identity (Bool { value: 0 }, 0); if predicate then 1 else 2 }"),
            ("spread", "{ let base = { predicate: True }; let value = { ...base, predicate: Bool { value: 0 } }; if value.predicate then 1 else 2 }"),
        ] {
            let invalid = fixture.0.join(format!("composite-nominal-{name}.ssrg"));
            fs::write(&invalid, format!("struct Bool {{ value: Int }}\nfn identity<A> value: A -> A = value\nfn inspect -> Int = {body}\npub effect fn main -> Unit with Console fails ConsoleError = println \"must-not-run\"\n")).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_seseragi")).arg("run").arg(&invalid).args(["--profile",profile,"--diagnostic-format","json"]).output().unwrap();
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("if.condition-not-bool"));
        }
    }
}
