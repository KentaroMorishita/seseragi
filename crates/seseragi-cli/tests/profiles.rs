use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "seseragi-profiles-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}
fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn fixture(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(name)
            .join("src/main.ssrg"),
    )
    .unwrap()
}

#[test]
fn profiles_run_tail_recursion_and_effect_newtype_scenarios_in_fresh_processes() {
    for (name, expected) in [
        ("performance-release-shapes", "5000050001\n"),
        ("performance-stack-safety", "100000,5000050000\n"),
        ("effect-tail-recursion", "6\n"),
        ("performance-profile-equivalence", "Score 42, calls: 1\n"),
    ] {
        let temporary = Temporary::new();
        fs::write(temporary.0.join("main.ssrg"), fixture(name)).unwrap();
        let expectation_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(name)
            .join("project.expect.json");
        let expectation: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&expectation_path).unwrap()).unwrap();
        let profiles = expectation["differentialProfiles"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(|value| value.as_str().unwrap())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["development", "release"]);
        for profile in profiles {
            assert_eq!(
                success(cli(
                    &["run", "main.ssrg", "--profile", profile],
                    &temporary.0
                )),
                expected
            );
            let mut arguments = vec!["build", "main.ssrg", "--out-dir", profile];
            if profile == "release" && expectation["args"].is_array() {
                arguments.extend(
                    expectation["args"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_str().unwrap()),
                );
            } else {
                arguments.extend(["--profile", profile]);
            }
            success(cli(&arguments, &temporary.0));
            let inspection = temporary.0.join(format!("inspection-{profile}"));
            inspect_compiler_outputs(&temporary.0, &inspection, profile);
            let metadata: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(inspection.join("generated-module.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(metadata["profile"], profile);
            if name != "effect-tail-recursion" {
                check_types(&inspection);
            }
            if expectation["shapes"].is_array() && profile == "release" {
                let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
                success(
                    Command::new("bun")
                        .arg(root.join("scripts/release-shapes.ts"))
                        .arg(inspection.join("generated-module.json"))
                        .arg(&expectation_path)
                        .output()
                        .unwrap(),
                );
            }
            let output = Command::new("bun")
                .arg("run")
                .arg(product_entry(&temporary.0.join(profile)))
                .current_dir(temporary.0.join(profile))
                .output()
                .unwrap();
            assert_eq!(success(output), expected);
        }
    }
}

#[test]
fn root_manifest_profile_is_overridden_without_rewriting_the_lock() {
    let temporary = Temporary::new();
    fs::create_dir(temporary.0.join("src")).unwrap();
    fs::write(
        temporary.0.join("src/main.ssrg"),
        "pub effect fn main = println \"profile\"\n",
    )
    .unwrap();
    fs::write(temporary.0.join("seseragi.toml"), "[package]\nname = \"fixture/profile\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[build]\nprofile = \"release\"\n").unwrap();
    success(cli(&["lock", "update", "."], &temporary.0));
    let lock = fs::read(temporary.0.join("seseragi.lock")).unwrap();
    for (options, expected) in [
        (vec![], "release"),
        (vec!["--profile", "development"], "development"),
    ] {
        let mut args = vec!["build", ".", "--out-dir", "dist"];
        args.extend(options);
        success(cli(&args, &temporary.0));
        let marker: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(temporary.0.join("dist/.seseragi-build.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(marker["profile"], expected);
        assert_eq!(fs::read(temporary.0.join("seseragi.lock")).unwrap(), lock);
    }
    let invalid = cli(&["build", ".", "--profile", "fast"], &temporary.0);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("unknown profile"));
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn imported_dictionaries_and_foreign_boundaries_preserve_observable_results() {
    for name in [
        "imported-derived-json-codecs",
        "imported-derived-structural",
        "imported-derived-show-debug",
        "foreign-pure-load",
        "foreign-task-load",
        "foreign-failure-phases",
    ] {
        let temporary = Temporary::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(name);
        copy_tree(&source.join("src"), &temporary.0.join("src"));
        if source.join("host").is_dir() {
            copy_tree(&source.join("host"), &temporary.0.join("host"));
        }
        let manifest = fs::read_to_string(source.join("seseragi.toml"))
            .unwrap()
            .replace(">=0.1.0 <0.2.0", ">=0.1.0")
            .replace("test-js", "process");
        fs::write(temporary.0.join("seseragi.toml"), manifest).unwrap();
        success(cli(&["lock", "update", "."], &temporary.0));
        let development = cli(&["run", ".", "--profile", "development"], &temporary.0);
        let release = cli(&["run", ".", "--profile", "release"], &temporary.0);
        assert!(
            development.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&development.stderr)
        );
        assert_eq!(
            release.status.code(),
            development.status.code(),
            "{name}: {}",
            String::from_utf8_lossy(&release.stderr)
        );
        assert_eq!(release.stdout, development.stdout, "{name}");
        assert_eq!(release.stderr, development.stderr, "{name}");
    }
}

#[test]
fn local_and_generic_self_tail_calls_use_constant_stack() {
    let temporary = Temporary::new();
    fs::write(
        temporary.0.join("main.ssrg"),
        r#"
fn repeat<A> current: Int -> value: A -> A =
  if current == 0 then value else repeat (current - 1) value
fn local start: Int -> Int = {
  fn loop<A> current: Int -> value: A -> A =
    match current {
      0 -> value
      _ -> loop (current - 1) value
    }
  loop start 42
}
pub effect fn main = do {
  println (show (repeat 100000 42))
  println (show (local 100000))
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        assert_eq!(
            success(cli(
                &["run", "main.ssrg", "--profile", profile],
                &temporary.0
            )),
            "42\n42\n"
        );
    }
}

#[test]
fn nested_newtypes_and_effect_pattern_binds_preserve_payloads() {
    let temporary = Temporary::new();
    fs::write(
        temporary.0.join("main.ssrg"),
        r#"
import * as effects from "std/effect"
newtype Inner = Int
newtype Outer = Inner
newtype Box<A> deriving Show = A
type Container =
  | Wrapped Outer
  | Empty
fn unwrap value: Container -> Int = match value {
  Wrapped (Outer (Inner number)) -> number
  Empty -> 0
}
pub effect fn main = do {
  Outer (Inner number) <- effects.succeed (Outer (Inner 42))
  println (show number)
  println (show (unwrap (Wrapped (Outer (Inner 43)))))
  println (show (Box 44))
}
"#,
    )
    .unwrap();
    for profile in ["development", "release"] {
        assert_eq!(
            success(cli(
                &["run", "main.ssrg", "--profile", profile],
                &temporary.0
            )),
            "42\n43\nBox 44\n"
        );
    }
}

fn check_types(output: &Path) {
    success(
        Command::new("bun")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../scripts/check-generated-types.ts"),
            )
            .arg(output)
            .output()
            .unwrap(),
    );
}

#[test]
fn imported_newtype_codecs_are_strictly_typed_in_both_profiles() {
    let temporary = Temporary::new();
    fs::create_dir(temporary.0.join("src")).unwrap();
    fs::write(
        temporary.0.join("seseragi.toml"),
        r#"[package]
name = "fixture/profile-codecs"
version = "0.0.0"
language = ">=0.1.0"
[run]
entry = "main"
target = "process"
"#,
    )
    .unwrap();
    fs::write(
        temporary.0.join("src/domain.ssrg"),
        r#"
import * as json from "std/json"
pub newtype UserId deriving Show, Eq, Ord, Hash, JsonEncode, JsonDecode = Int
"#,
    )
    .unwrap();
    fs::write(
        temporary.0.join("src/main.ssrg"),
        r#"
import * as json from "std/json"
import { UserId } from "./domain"
fn decode text: String -> Either<json.JsonReadError, UserId> = json.decodeString text
pub effect fn main = do {
  let value = UserId 42
  println (show value)
  println (json.encodeString value)
  match decode "42" {
    Left _ -> println "failure"
    Right decoded -> println (show decoded)
  }
}
"#,
    )
    .unwrap();
    success(cli(&["lock", "update", "."], &temporary.0));
    for profile in ["development", "release"] {
        assert_eq!(
            success(cli(&["run", ".", "--profile", profile], &temporary.0)),
            "UserId 42\n42\nUserId 42\n"
        );
        success(cli(
            &["build", ".", "--profile", profile, "--out-dir", profile],
            &temporary.0,
        ));
        let inspection = temporary.0.join(format!("inspection-{profile}"));
        inspect_compiler_outputs(&temporary.0, &inspection, profile);
        check_types(&inspection);
        assert_eq!(
            success(
                Command::new("bun")
                    .arg(product_entry(&temporary.0.join(profile)))
                    .current_dir(temporary.0.join(profile))
                    .output()
                    .unwrap()
            ),
            "UserId 42\n42\nUserId 42\n"
        );
    }
}

fn product_entry(directory: &Path) -> String {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("artifact-manifest.json")).unwrap())
            .unwrap();
    manifest["entry"].as_str().unwrap().to_owned()
}

// Compiler shape/type checks inspect the compiler stage independently of the
// application packaging contract, which no longer publishes TypeScript.
fn inspect_compiler_outputs(package: &Path, output: &Path, profile: &str) {
    let profile = seseragi_project::BuildProfile::parse(profile).unwrap();
    let modules = if package.join("seseragi.toml").exists() {
        let loaded = seseragi_project::load_local_project(package).unwrap();
        seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
            .unwrap()
            .compiled
            .modules
            .into_values()
            .collect::<Vec<_>>()
    } else {
        let source = fs::read_to_string(package.join("main.ssrg")).unwrap();
        vec![seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("main.ssrg", "single-file/main", &source)
                .with_profile(profile),
        )
        .unwrap()]
    };
    fs::create_dir_all(output).unwrap();
    seseragi_runtime::stage_typescript_package(output).unwrap();
    for module in modules {
        let path = output.join(
            module
                .generated
                .metadata
                .outputs
                .typescript
                .trim_start_matches("./"),
        );
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &module.generated.typescript).unwrap();
        let metadata = if package.join("seseragi.toml").exists() {
            path.with_extension("generated-module.json")
        } else {
            output.join("generated-module.json")
        };
        fs::write(
            metadata,
            serde_json::to_vec(&module.generated.metadata).unwrap(),
        )
        .unwrap();
        fs::write(
            output.join(&module.generated.metadata.outputs.source_map),
            serde_json::to_vec(&module.generated.source_map).unwrap(),
        )
        .unwrap();
    }
}
