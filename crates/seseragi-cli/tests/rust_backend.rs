use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(format!(
                "rust-cli-test-{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

#[test]
fn incomplete_instance_contracts_reject_before_emission_or_execution() {
    let scratch = Scratch::new();
    let artifacts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/spec/artifacts");
    for (schema, name, code) in [
        ("schema-1", "incomplete-trait-method", "SES-P0001"),
        ("schema-1", "incomplete-instance-method", "SES-P0001"),
        ("schema-1", "missing-instance-result", "SES-P0001"),
        ("schema-1", "trait-method-body", "SES-P0001"),
        (
            "semantic-diagnostics-schema-1",
            "missing-standard-instance-method",
            "SES-T0101",
        ),
        (
            "semantic-diagnostics-schema-1",
            "missing-custom-instance-method",
            "SES-T0101",
        ),
        (
            "semantic-diagnostics-schema-1",
            "wrong-standard-instance-signature",
            "SES-T0101",
        ),
        (
            "semantic-diagnostics-schema-1",
            "missing-generic-instance-method",
            "SES-T0101",
        ),
    ] {
        let source = scratch.0.join(format!("{name}-λ.ssrg"));
        fs::write(
            &source,
            fs::read_to_string(artifacts.join(schema).join(name).join("main.ssrg")).unwrap()
                + "\npub effect fn main = println \"must not run\"\n",
        )
        .unwrap();
        let rust = scratch.0.join(format!("{name}.rs"));
        let output = cli()
            .env("PATH", "")
            .args(["experimental", "emit-rust"])
            .arg(&source)
            .args(["--entry", "main"])
            .arg("--output")
            .arg(&rust)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "{name}: {output:?}"
        );
        assert!(output.stdout.is_empty());
        assert!(!rust.exists());
        assert!(!rust.with_extension("rust-backend.json").exists());
        let native_directory = scratch.0.join(format!("{name}-native"));
        let output = cli()
            .env("PATH", "")
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main"])
            .arg("--out-dir")
            .arg(&native_directory)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{name}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "{name}: {output:?}"
        );
        assert!(output.stdout.is_empty());
        assert!(!native_directory.exists());
        for profile in ["development", "release"] {
            let directory = scratch.0.join(format!("{name}-{profile}"));
            let output = cli()
                .env("PATH", "")
                .arg("build")
                .arg(&source)
                .args(["--profile", profile, "--out-dir"])
                .arg(&directory)
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(2),
                "{name}/{profile}: {output:?}"
            );
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(code),
                "{output:?}"
            );
            assert!(output.stdout.is_empty());
            assert!(!directory.exists());
            let output = cli()
                .env("PATH", "")
                .arg("run")
                .arg(&source)
                .args(["--profile", profile])
                .output()
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(2),
                "{name}/{profile}: {output:?}"
            );
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(code),
                "{output:?}"
            );
            assert!(output.stdout.is_empty());
        }
    }
}

#[test]
fn native_project_links_path_packages_dictionaries_and_ref_callbacks() {
    let scratch = Scratch::new();
    for (fixture, count) in [
        ("linked", 5),
        ("globals", 4),
        ("scalar-globals", 4),
        ("tuple-display", 3),
        ("array-display", 3),
        ("deprecation", 3),
    ] {
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/rust-backend-projects")
            .join(fixture);
        for entry in ["main", "probe"] {
            let directory = scratch.0.join(fixture).join(entry);
            let built = cli()
                .args(["experimental", "build-native"])
                .arg(&project)
                .args(["--entry", entry, "--out-dir"])
                .arg(&directory)
                .output()
                .unwrap();
            assert!(
                built.status.success(),
                "{}",
                String::from_utf8_lossy(&built.stderr)
            );
            let executable = directory.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            });
            let native = Command::new(executable)
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1")
                .output()
                .unwrap();
            assert!(
                native.status.success(),
                "{}",
                String::from_utf8_lossy(&native.stderr)
            );
            let expected = fs::read(project.join(if entry == "main" {
                "expected.stdout"
            } else {
                "probe.stdout"
            }))
            .unwrap();
            assert_eq!(native.stdout, expected);
            if entry == "main" {
                assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));
                for profile in ["development", "release"] {
                    let ts = cli()
                        .arg("run")
                        .arg(&project)
                        .args(["--profile", profile])
                        .output()
                        .unwrap();
                    assert!(
                        ts.status.success(),
                        "{}",
                        String::from_utf8_lossy(&ts.stderr)
                    );
                    assert_eq!(native.stdout, ts.stdout, "{profile}");
                }
            }
            let metadata: serde_json::Value =
                serde_json::from_slice(&fs::read(directory.join("rust-backend.json")).unwrap())
                    .unwrap();
            assert_eq!(metadata["modules"].as_array().unwrap().len(), count);
            assert!(metadata["entry"]
                .as_str()
                .unwrap()
                .ends_with(&format!("::main::{entry}")));
        }
    }
}

fn copy_project(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_project(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn write_project_lock(project: &Path) {
    let lock = seseragi_project::generate_lockfile(project).unwrap();
    fs::write(
        project.join("seseragi.lock"),
        seseragi_project::write_lockfile(&lock),
    )
    .unwrap();
}

#[test]
fn native_project_rejects_imported_boundaries_and_stale_locks_without_outputs() {
    let scratch = Scratch::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend-projects/linked");
    for kind in [
        "private",
        "opaque",
        "unsupported",
        "polymorphic-global",
        "stale",
        "missing",
    ] {
        let project = scratch.0.join(kind);
        copy_project(&fixture, &project);
        let library = project.join("vendor/model/src/lib.ssrg");
        let main = project.join("src/main.ssrg");
        let (expected, source, needle) = match kind {
            "private" => {
                fs::write(&main, "import { privateValue } from \"model\"\npub fn probe -> Int = privateValue ()\n").unwrap();
                ("SES-N0104", Some(main.clone()), "privateValue")
            }
            "opaque" => {
                fs::write(&library, "pub opaque struct Secret { value: Int }\npub fn secret -> Secret = Secret { value: 1 }\n").unwrap();
                fs::write(
                    &main,
                    "import { secret } from \"model\"\npub fn probe -> Int = (secret ()).value\n",
                )
                .unwrap();
                ("error[", Some(main.clone()), "value")
            }
            "unsupported" | "polymorphic-global" => {
                let addition = if kind == "unsupported" {
                    "\npub fn unsupported -> Float = 2.0 ** 3.0\n"
                } else {
                    "\nlet initial = \\value -> value\n"
                };
                fs::write(&library, fs::read_to_string(&library).unwrap() + addition).unwrap();
                (
                    "SES-RUST-0001",
                    Some(library.clone()),
                    if kind == "unsupported" {
                        "2.0 ** 3.0"
                    } else {
                        "initial"
                    },
                )
            }
            _ => ("SES-K0102", None, ""),
        };
        if kind != "stale" && kind != "missing" {
            write_project_lock(&project);
        } else if kind == "stale" {
            fs::write(
                &library,
                fs::read_to_string(&library).unwrap() + "\n// changed source\n",
            )
            .unwrap();
        } else {
            fs::remove_file(project.join("seseragi.lock")).unwrap();
        }
        let directory = scratch.0.join(format!("out-{kind}"));
        let output = cli()
            .args(["experimental", "build-native"])
            .arg(&project)
            .args(["--entry", "probe", "--out-dir"])
            .arg(&directory)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{kind}");
        let diagnostics = String::from_utf8(output.stderr).unwrap();
        assert!(diagnostics.contains(expected), "{kind}: {diagnostics}");
        if let Some(source) = source {
            assert!(
                diagnostics.contains(source.to_str().unwrap()),
                "{kind}: {diagnostics}"
            );
            assert!(
                diagnostics.contains(needle) || diagnostics.contains(".."),
                "{kind}: {diagnostics}"
            );
            if kind == "unsupported" {
                let prefix = format!("{}:", source.display());
                let range = diagnostics
                    .split(&prefix)
                    .nth(1)
                    .unwrap()
                    .split(':')
                    .next()
                    .unwrap();
                let (start, end) = range.split_once("..").unwrap();
                let bytes = fs::read(&source).unwrap();
                assert_eq!(
                    &bytes[start.parse::<usize>().unwrap()..end.parse::<usize>().unwrap()],
                    needle.as_bytes()
                );
            }
        }
        assert!(
            !directory.exists(),
            "unsupported project created outputs: {kind}"
        );
    }
}

#[test]
fn native_cli_and_ts_backend_agree_on_canonical_fixtures() {
    let scratch = Scratch::new();
    for name in [
        "ownership",
        "shared-values",
        "recursive-adt",
        "structural-record",
        "higher-kinded",
        "nested-dictionary",
        "tail-recursion",
        "tail-capture",
        "tail-evidence",
        "let-shadowing",
        "scalar-tail-loop",
        "scalar-tail-operators",
        "scalar-global",
        "char-unicode",
        "integer-literals",
        "float-values",
        "list-core",
        "list-tail",
        "tuple-display",
        "tuple-nominal",
        "array-display",
        "array-nominal",
        "instance-contracts",
        "deprecation",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../examples/spec/fixtures/rust-backend/{name}.ssrg"
        ));
        let destination = scratch.0.join(name);
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "probe", "--out-dir"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let executable = destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let native = Command::new(&executable).env("PATH", "").output().unwrap();
        assert!(
            native.status.success(),
            "{}",
            String::from_utf8_lossy(&native.stderr)
        );
        assert!(
            native.stderr.is_empty(),
            "experimental metrics must not leak into ordinary output"
        );
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(destination.join("rust-backend.json")).unwrap())
                .unwrap();
        assert_eq!(metadata["backend"], "rust");
        assert_eq!(metadata["stability"], "experimental");
        assert_eq!(metadata["module"], "experimental/main");
        let source = format!(
            "{}\npub effect fn main = println (show (probe ()))\n",
            fs::read_to_string(&path).unwrap()
        );
        let oracle_path = scratch.0.join(format!("{name}-oracle.ssrg"));
        fs::write(&oracle_path, source).unwrap();
        for profile in ["development", "release"] {
            let ts = cli()
                .arg("run")
                .arg(&oracle_path)
                .args(["--profile", profile])
                .output()
                .unwrap();
            assert!(
                ts.status.success(),
                "{name}/{profile}: {}",
                String::from_utf8_lossy(&ts.stderr)
            );
            assert_eq!(native.stdout, ts.stdout, "{name}/{profile}");
        }
    }
}

#[test]
fn collection_normal_bodies_typecheck_and_match_fresh_bun_and_node() {
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for fixture in [
        "list-core",
        "list-tail",
        "tuple-display",
        "tuple-nominal",
        "array-display",
        "array-nominal",
    ] {
        let fixture_path = root.join(format!(
            "examples/spec/fixtures/rust-backend/{fixture}.ssrg"
        ));
        let source = fs::read_to_string(&fixture_path).unwrap();
        for (name, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new("main.ssrg", "lists/main", &source)
                    .with_profile(profile),
            )
            .unwrap();
            let directory = scratch.0.join(format!("{fixture}-{name}"));
            fs::create_dir_all(&directory).unwrap();
            let emitted = directory.join("generated.ts");
            fs::write(&emitted, compiled.generated.typescript).unwrap();
            let output = Command::new("bun")
                .arg(root.join("scripts/list-probe.ts"))
                .arg(&emitted)
                .arg(fixture_path.with_extension("stdout"))
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{fixture}/{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout)
                    .matches("\"normalBody\":true")
                    .count(),
                2
            );
        }
    }
}

#[test]
fn unicode_character_fixture_agrees_on_bun_and_node() {
    let scratch = Scratch::new();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/char-unicode.ssrg");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("main.ssrg", "chars/main", source)
                .with_profile(profile),
        )
        .unwrap();
        let directory = scratch.0.join(name);
        fs::create_dir_all(&directory).unwrap();
        let emitted = directory.join("generated.ts");
        fs::write(&emitted, compiled.generated.typescript).unwrap();
        let output = Command::new("bun")
            .arg(root.join("scripts/char-probe.ts"))
            .arg(&emitted)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout)
                .matches("\"unicodeCodePoints\":1114113")
                .count(),
            2
        );
    }
}

#[test]
fn float_normal_bodies_typecheck_and_match_bun_node_across_binary64_values() {
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/float-values.ssrg");
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "floats.ssrg",
        "floats/main",
        source,
    ))
    .unwrap();
    let artifact = seseragi_backend_rust::emit_rust_module(&core.core_ir, "probe").unwrap();
    let display = format!(
        "v_{}_direct",
        "floats/main::display"
            .bytes()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    // Invoke the unchanged generated callable. Only the standalone test entry
    // is replaced, so the compiler's Float body and runtime are the product's.
    let harness = r#"
fn main() {
 let mut state=0x5eed6123_u64; let mut digest=0xcbf29ce484222325_u64;
 let mut cases=0;
 let mut check=|bits| {
  let text=<DISPLAY>(vec![],V::Float(f64::from_bits(bits))).render();
  for byte in text.bytes().chain(std::iter::once(0)) {
   digest=(digest ^ byte as u64).wrapping_mul(0x100000001b3);
  }
  cases+=1;
 };
 for _ in 0..100000 { state^=state<<13;state^=state>>7;state^=state<<17;check(state); }
 for exponent in -1074..=1023 {
  let bits=if exponent < -1022 {1_u64 << (exponent+1074)} else {((exponent+1023) as u64)<<52};
  check(bits);check(bits+1);check(bits-1);
 }
 clear_dictionaries();assert_eq!(LIVE.with(Cell::get),0);
 println!("{{\"formatCases\":{},\"digest\":\"{:x}\"}}",cases,digest);
}
"#
    .replace("<DISPLAY>", &display);
    let main = artifact.source.rfind("fn main()").unwrap();
    let rust = scratch.0.join("format.rs");
    fs::write(&rust, artifact.source[..main].to_owned() + &harness).unwrap();
    let executable = scratch.0.join(if cfg!(windows) {
        "format.exe"
    } else {
        "format"
    });
    let compiled = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&executable)
        .arg(&rust)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let native = Command::new(&executable).env("PATH", "").output().unwrap();
    assert!(
        native.status.success(),
        "{}",
        String::from_utf8_lossy(&native.stderr)
    );
    let expected: serde_json::Value = serde_json::from_slice(&native.stdout).unwrap();
    assert_eq!(expected["formatCases"], 106294);
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("floats.ssrg", "floats/main", source)
                .with_profile(profile),
        )
        .unwrap();
        let directory = scratch.0.join(name);
        fs::create_dir_all(&directory).unwrap();
        let emitted = directory.join("generated.ts");
        fs::write(&emitted, compiled.generated.typescript).unwrap();
        let output = Command::new("bun")
            .arg(root.join("scripts/float-probe.ts"))
            .arg(&emitted)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let lines = String::from_utf8(output.stdout).unwrap();
        let lines = lines.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        for line in lines {
            let (_, json) = line.split_once(": ").unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(json).unwrap(),
                expected,
                "{name}: {line}"
            );
        }
    }
}

#[test]
fn integer_literal_fixture_typechecks_and_executes_on_bun_and_node() {
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/integer-literals.ssrg");
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../examples/spec/fixtures/rust-backend/integer-literals.stdout"
    ))
    .unwrap();
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("numbers.ssrg", "numbers/main", source)
                .with_profile(profile),
        )
        .unwrap();
        let directory = scratch.0.join(name);
        fs::create_dir_all(&directory).unwrap();
        let emitted = directory.join("generated.ts");
        fs::write(&emitted, compiled.generated.typescript).unwrap();
        let checked = Command::new("bun")
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .args([
                "--noEmit",
                "--strict",
                "--skipLibCheck",
                "--target",
                "ES2022",
                "--module",
                "ESNext",
                "--moduleResolution",
                "bundler",
            ])
            .arg(&emitted)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            checked.status.success(),
            "{name}: {}{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        );
        let harness = directory.join("harness.ts");
        fs::write(
            &harness,
            "import { probe } from './generated'; console.log(JSON.stringify(probe()));\n",
        )
        .unwrap();
        let module = directory.join("harness.mjs");
        let built = Command::new("bun")
            .arg("build")
            .arg(&harness)
            .args(["--target", "node", "--outfile"])
            .arg(&module)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        for runtime in ["bun", "node"] {
            let output = Command::new(runtime).arg(&module).output().unwrap();
            assert!(
                output.status.success(),
                "{name}/{runtime}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty());
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
                expected,
                "{name}/{runtime}"
            );
        }
    }
}

#[test]
fn invalid_numeric_spellings_reject_before_rust_emission_or_execution() {
    let scratch = Scratch::new();
    for (index, raw) in [
        "0b102",
        "0o78",
        "0xFFz",
        "0x_FF",
        "1__0",
        "42suffix",
        "0x1.fp+2",
        "0x20000000000000",
        "-0b100000000000000000000000000000000000000000000000000000",
    ]
    .iter()
    .enumerate()
    {
        let source = scratch.0.join(format!("bad-{index}.ssrg"));
        fs::write(&source, format!("pub fn probe -> Int = {raw}\n")).unwrap();
        let destination = scratch.0.join(format!("bad-{index}.rs"));
        let output = cli()
            .env("PATH", "")
            .args(["experimental", "emit-rust"])
            .arg(&source)
            .args(["--entry", "probe", "--output"])
            .arg(&destination)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{raw}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("SES-P0203"),
            "{raw}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty(), "{raw}");
        assert!(!destination.exists(), "{raw}");
        assert!(
            !destination.with_extension("rust-backend.json").exists(),
            "{raw}"
        );
    }
}

#[test]
fn scalar_tail_loops_preserve_primitive_traces_on_bun_and_node() {
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for fixture in ["scalar-tail-loop", "scalar-global"] {
        let source = fs::read_to_string(root.join(format!(
            "examples/spec/fixtures/rust-backend/{fixture}.ssrg"
        )))
        .unwrap();
        for (name, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new("main.ssrg", "scalar/main", &source)
                    .with_profile(profile),
            )
            .unwrap();
            let directory = scratch.0.join(fixture).join(name);
            fs::create_dir_all(&directory).unwrap();
            let emitted = directory.join("generated.ts");
            fs::write(&emitted, compiled.generated.typescript).unwrap();
            let output = Command::new("bun")
                .arg(root.join("scripts/scalar-tail-probe.ts"))
                .arg(&emitted)
                .args(if fixture == "scalar-global" {
                    vec!["globals"]
                } else {
                    vec![]
                })
                .current_dir(&root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{name}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&output.stdout)
                    .matches("\"cases\":69")
                    .count(),
                2
            );
        }
    }
}

#[test]
fn emission_does_not_launch_rustc_and_invalid_targets_write_nothing() {
    let scratch = Scratch::new();
    let source_path = scratch.0.join("pure.ssrg");
    fs::write(&source_path, "pub fn answer unit: Unit -> Int = 42\n").unwrap();
    let output_path = scratch.0.join("generated.rs");
    let output = cli()
        .env("PATH", "")
        .args(["experimental", "emit-rust"])
        .arg(&source_path)
        .args(["--entry", "answer", "--output"])
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());
    for (name, source) in [
        ("float-power", "pub fn answer -> Float = 1.5 ** 2.0\n"),
        ("float-named-power", "import * as floats from \"std/float\"\npub fn answer -> Float = floats.power 2.0 1.5\n"),
        ("float-parse", "import * as floats from \"std/float\"\npub fn answer -> Either<floats.FloatParseError, Float> = floats.parse \"1.5\"\n"),
        ("float-rounding", "import * as floats from \"std/float\"\nimport * as numbers from \"std/number\"\npub fn answer -> Float = floats.roundIntegral numbers.HalfEven 1.5\n"),
        ("float-convert", "import * as floats from \"std/float\"\nimport * as numbers from \"std/number\"\npub fn answer -> Either<floats.FloatConversionError, Int> = floats.toInt numbers.HalfEven 1.5\n"),
        ("hash-char", "pub fn answer -> Int = hash 'λ'\n"),
        ("list-eq", "pub fn answer -> Bool = `[1] == `[1]\n"),
        ("list-sort", "import * as lists from \"std/list\"\npub fn answer -> List<Int> = lists.sort `[2, 1]\n"),
        ("list-comprehension", "pub fn answer -> List<Int> = `[n | n <- `[1, 2]]\n"),
        ("list-functor", "pub fn answer -> List<Int> = map (\\n: Int -> n + 1) `[1, 2]\n"),
    ] {
        let bad_path = scratch.0.join(format!("{name}.ssrg"));
        fs::write(&bad_path, source).unwrap();
        let bad_output = scratch.0.join(format!("{name}.rs"));
        let bad = cli()
            .args(["experimental", "emit-rust"])
            .arg(&bad_path)
            .args(["--entry", "answer", "--output"])
            .arg(&bad_output)
            .output()
            .unwrap();
        assert_eq!(bad.status.code(), Some(2));
        let diagnostic = String::from_utf8_lossy(&bad.stderr);
        assert!(diagnostic.contains("SES-RUST-0001"), "{name}: {diagnostic}");
        assert!(
            diagnostic.contains(&format!("{name}.ssrg:")),
            "{diagnostic}"
        );
        assert!(!bad_output.exists());
        assert!(!bad_output.with_extension("rust-backend.json").exists());
    }
    let original = fs::read(&source_path).unwrap();
    let collision = cli()
        .args(["experimental", "emit-rust"])
        .arg(&source_path)
        .args(["--entry", "answer", "--output"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(collision.status.code(), Some(2));
    assert_eq!(fs::read(source_path).unwrap(), original);
}

#[test]
fn float_does_not_acquire_eq_ord_or_hash_through_native_emission() {
    let scratch = Scratch::new();
    for (name, source) in [
        ("eq", "pub fn answer -> Bool = 1.0 == 1.0\n"),
        ("ord", "pub fn answer -> Bool = 1.0 < 2.0\n"),
        ("hash", "pub fn answer -> Int = hash 1.0\n"),
    ] {
        let path = scratch.0.join(format!("{name}.ssrg"));
        fs::write(&path, source).unwrap();
        let output = scratch.0.join(format!("{name}.rs"));
        let result = cli()
            .env("PATH", "")
            .args(["experimental", "emit-rust"])
            .arg(&path)
            .args(["--entry", "answer", "--output"])
            .arg(&output)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        assert!(diagnostic.contains("SES-T0201"), "{name}: {diagnostic}");
        assert!(
            diagnostic.contains(&format!("{name}.ssrg:")),
            "{diagnostic}"
        );
        assert!(!output.exists());
        assert!(!output.with_extension("rust-backend.json").exists());
    }
}

#[test]
fn native_effect_cli_preserves_ts_results_scopes_and_typed_payloads() {
    let scratch = Scratch::new();
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/spec");
    for (name, relative, expected, exit) in [
        (
            "deprecation-effects",
            "fixtures/rust-backend/deprecation-effects.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/deprecation-effects.stdout"),
            0,
        ),
        (
            "array-effects",
            "fixtures/rust-backend/array-effects.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/array-effects.stdout"),
            0,
        ),
        (
            "array-defect",
            "fixtures/rust-backend/array-defect.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/array-defect.stdout"),
            70,
        ),
        (
            "tuple-effects",
            "fixtures/rust-backend/tuple-effects.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/tuple-effects.stdout"),
            0,
        ),
        (
            "tuple-defect",
            "fixtures/rust-backend/tuple-defect.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/tuple-defect.stdout"),
            70,
        ),
        (
            "list-effects",
            "fixtures/rust-backend/list-effects.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/list-effects.stdout"),
            0,
        ),
        (
            "resource-lesson",
            "lessons/18-resource-scope.ssrg",
            "released: True\n",
            0,
        ),
        (
            "ref-cycle",
            "fixtures/rust-backend-research/ref-cycle.ssrg",
            "answer: 42\n",
            0,
        ),
        (
            "cold-effect",
            "fixtures/rust-backend-research/cold-effect.ssrg",
            "before: 0\nafter: 2\n",
            0,
        ),
        (
            "nested-scope",
            "fixtures/rust-backend-research/nested-scope.ssrg",
            "order: 3241\n",
            0,
        ),
        (
            "escaped-callback",
            "fixtures/rust-backend-research/escaped-callback.ssrg",
            "escaped: 42\n",
            0,
        ),
        (
            "failure-scope",
            "fixtures/rust-backend-research/failure-scope.ssrg",
            "failure released: 42\n",
            0,
        ),
        (
            "deep-effect",
            "fixtures/rust-backend-research/deep-effect.ssrg",
            "deep: 42\n",
            0,
        ),
        (
            "typed-payload",
            "fixtures/rust-backend/typed-effect-failure.ssrg",
            "typed payload: 42\n",
            0,
        ),
        (
            "ref-modify",
            "fixtures/rust-backend/ref-modify.ssrg",
            "modify: 41, 42\n",
            0,
        ),
        (
            "effect-evidence",
            "fixtures/rust-backend/effect-evidence.ssrg",
            "42\nbox 42\n",
            0,
        ),
        (
            "char-console",
            "fixtures/rust-backend/char-console.ssrg",
            "🙂\nλ\n",
            0,
        ),
        (
            "char-effects",
            "fixtures/rust-backend/char-effects.ssrg",
            "chars: λ -> 🙂\nrecovered: 128578\n🙂\n'λ'\nreleased: λ\n",
            0,
        ),
        (
            "float-effects",
            "fixtures/rust-backend/float-effects.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/float-effects.stdout"),
            0,
        ),
        (
            "float-console",
            "fixtures/rust-backend/float-console.ssrg",
            include_str!("../../../examples/spec/fixtures/rust-backend/float-console.stdout"),
            0,
        ),
        (
            "release-defect",
            "fixtures/rust-backend-research/release-callback-timing.ssrg",
            "body before release defect\n",
            70,
        ),
        (
            "console-values",
            "fixtures/rust-backend/console-values.ssrg",
            "true\n42\nhello\nundefined\nTrue\n",
            0,
        ),
        (
            "qualified-console",
            "fixtures/rust-backend/qualified-console.ssrg",
            "True\n",
            0,
        ),
        (
            "unhandled-failure",
            "fixtures/rust-backend/unhandled-effect-failure.ssrg",
            "",
            1,
        ),
    ] {
        let path = fixtures.join(relative);
        let destination = scratch.0.join(name);
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let program = destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        });
        let native = Command::new(&program)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        assert_eq!(
            native.status.code(),
            Some(exit),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&native.stdout), expected, "{name}");
        assert!(
            String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        let ts = cli().arg("run").arg(&path).output().unwrap();
        assert_eq!(
            ts.status.code(),
            Some(exit),
            "{name}: {}",
            String::from_utf8_lossy(&ts.stderr)
        );
        assert_eq!(native.stdout, ts.stdout, "{name}");
        if exit != 70 {
            let native_error = String::from_utf8_lossy(&native.stderr)
                .lines()
                .filter(|line| !line.starts_with("native managed-live="))
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(
                native_error,
                String::from_utf8_lossy(&ts.stderr).trim_end(),
                "{name}"
            );
        }
        if name == "cold-effect" {
            let ordinary = Command::new(&program)
                .env("PATH", "")
                .env_remove("SESERAGI_NATIVE_METRICS")
                .output()
                .unwrap();
            assert!(ordinary.status.success());
            assert!(ordinary.stderr.is_empty());
            assert_eq!(ordinary.stdout, ts.stdout);
        }
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(destination.join("rust-backend.json")).unwrap())
                .unwrap();
        assert_eq!(metadata["abi"], "rust-erased-sync-v2");
        assert_eq!(metadata["entryKind"], "effect");
    }
}

#[test]
fn unsupported_async_operation_has_a_source_range_and_writes_nothing() {
    let scratch = Scratch::new();
    let path = scratch.0.join("async.ssrg");
    fs::write(
        &path,
        "import * as effects from \"std/effect\"\npub effect fn main = effects.yieldNow ()\n",
    )
    .unwrap();
    let output = scratch.0.join("unsupported.rs");
    let result = cli()
        .env("PATH", "")
        .args(["experimental", "emit-rust"])
        .arg(&path)
        .args(["--entry", "main", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&result.stderr);
    assert_eq!(result.status.code(), Some(2));
    assert!(
        error.contains("SES-RUST-0001") && error.contains("yieldNow"),
        "{error}"
    );
    assert!(error.contains("async.ssrg:"), "{error}");
    assert!(!output.exists());
    assert!(!output.with_extension("rust-backend.json").exists());
}

#[test]
fn native_globals_share_initialization_diagnostics_and_write_nothing() {
    let scratch = Scratch::new();
    for (index, source) in [
        "// λ🙂\nlet first: Int = later\nlet later: Int = 42\npub fn probe -> Int = first\n",
        "// λ🙂\nfn read -> Int = value\nlet value: Int = read ()\npub fn probe -> Int = value\n",
        "// λ🙂\nfn read -> Int = later\nlet first: Int = read ()\nlet later: Int = 42\npub fn probe -> Int = first\n",
    ].iter().enumerate() {
        let path = scratch.0.join(format!("invalid-{index}.ssrg"));
        fs::write(&path, source).unwrap();
        let output_path = scratch.0.join(format!("invalid-{index}.rs"));
        let output = cli().args(["experimental", "emit-rust"]).arg(&path)
            .args(["--entry", "probe", "--output"]).arg(&output_path).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(!output_path.exists());
        assert!(String::from_utf8_lossy(&output.stderr).contains("SES-N0201"));
        for profile in ["development", "release"] {
            let ts = cli().arg("run").arg(&path).args(["--profile", profile]).output().unwrap();
            assert_eq!(ts.status.code(), Some(2));
            assert!(ts.stdout.is_empty());
            assert!(String::from_utf8_lossy(&ts.stderr).contains("SES-N0201"));
        }
    }
}

#[test]
fn native_globals_initialize_eagerly_before_entry_and_release_partial_roots() {
    let scratch = Scratch::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend-projects/globals");
    for kind in [
        "dependency",
        "source",
        "siblings",
        "unused-import",
        "unreferenced",
    ] {
        let project = scratch.0.join(kind);
        copy_project(&fixture, &project);
        let library = project.join("vendor/model/src/lib.ssrg");
        let main = project.join("src/main.ssrg");
        let first = "\nlet firstDefect: Int = 9007199254740991 + 1\n";
        let second = "\nlet secondDefect: Int = 9007199254740991 * 9007199254740991\n";
        match kind {
            "dependency" => {
                fs::write(&library, fs::read_to_string(&library).unwrap() + first).unwrap();
                fs::write(&main, fs::read_to_string(&main).unwrap() + second).unwrap();
            }
            "source" => {
                fs::write(&main, fs::read_to_string(&main).unwrap() + first + second).unwrap()
            }
            "siblings" => {
                let left = project.join("src/left.ssrg");
                let lib = project.join("src/lib.ssrg");
                fs::write(&left, fs::read_to_string(&left).unwrap() + first).unwrap();
                fs::write(&lib, fs::read_to_string(&lib).unwrap() + second).unwrap();
            }
            "unused-import" => {
                fs::write(&library, fs::read_to_string(&library).unwrap() + first).unwrap();
                fs::write(&main,"import { reader } from \"model\"\npub effect fn main = println \"must not run\"\n").unwrap();
            }
            _ => fs::write(project.join("src/unreferenced.ssrg"), first).unwrap(),
        }
        write_project_lock(&project);
        let directory = scratch.0.join(format!("out-{kind}"));
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&project)
            .args(["--entry", "main", "--out-dir"])
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{kind}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let native = Command::new(directory.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        let code = if kind == "unreferenced" { 0 } else { 70 };
        assert_eq!(native.status.code(), Some(code), "{kind}");
        if code == 0 {
            assert_eq!(
                native.stdout,
                fs::read(fixture.join("expected.stdout")).unwrap()
            );
        } else {
            assert!(native.stdout.is_empty());
        }
        let errors = String::from_utf8_lossy(&native.stderr);
        if code != 0 {
            let first_defect = if kind == "siblings" {
                "Seseragi Defect: Int overflow"
            } else {
                "Seseragi Defect: Seseragi Int overflow"
            };
            assert!(errors.contains(first_defect), "{kind}: {errors}");
        }
        assert!(errors.contains("managed-live=0"), "{kind}: {errors}");
        for profile in ["development", "release"] {
            let ts = cli()
                .arg("run")
                .arg(&project)
                .args(["--profile", profile])
                .output()
                .unwrap();
            assert_eq!(
                ts.status.code(),
                Some(code),
                "{kind}: {}",
                String::from_utf8_lossy(&ts.stderr)
            );
            assert_eq!(ts.stdout, native.stdout);
            if code != 0 {
                assert!(String::from_utf8_lossy(&ts.stderr).contains("Int overflow"));
            }
            // The release application-pruning pass and bundler must retain
            // eager initialization even when no imported/global name is used.
            let output = scratch.0.join(format!("{kind}-ts-{profile}"));
            let built = cli()
                .arg("build")
                .arg(&project)
                .args(["--profile", profile, "--out-dir"])
                .arg(&output)
                .output()
                .unwrap();
            assert!(
                built.status.success(),
                "{}",
                String::from_utf8_lossy(&built.stderr)
            );
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for runtime in ["bun", "node"] {
                let ts = Command::new(runtime)
                    .arg(output.join(manifest["entry"].as_str().unwrap()))
                    .output()
                    .unwrap();
                assert_eq!(
                    ts.status.code(),
                    Some(code),
                    "{kind}/{profile}/{runtime}: {}",
                    String::from_utf8_lossy(&ts.stderr)
                );
                assert_eq!(ts.stdout, native.stdout, "{kind}/{profile}/{runtime}");
                if code != 0 {
                    assert!(String::from_utf8_lossy(&ts.stderr).contains("module initialization"));
                }
            }
        }
    }
}

#[test]
fn invalid_deprecation_prefixes_fail_before_output_or_execution() {
    let scratch = Scratch::new();
    for (index, clause) in [
        "deprecated 41",
        "deprecated \"\"",
        "deprecated \"old\" since \"01.2.3\"",
        "deprecated \"old\";",
    ]
    .iter()
    .enumerate()
    {
        let source = scratch.0.join(format!("deprecated-{index}-λ.ssrg"));
        fs::write(&source, format!("// λ\n{clause}\nfn historical -> Int = 41\npub effect fn main = println \"must not run\"\n")).unwrap();
        let rust = scratch.0.join(format!("deprecated-{index}.rs"));
        let result = cli()
            .env("PATH", "")
            .args(["experimental", "emit-rust"])
            .arg(&source)
            .args(["--entry", "main", "--output"])
            .arg(&rust)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("SES-P0001"));
        assert!(!rust.exists());
        assert!(!rust.with_extension("rust-backend.json").exists());
        for profile in ["development", "release"] {
            let directory = scratch.0.join(format!("deprecated-{index}-{profile}"));
            let result = cli()
                .env("PATH", "")
                .arg("build")
                .arg(&source)
                .args(["--profile", profile, "--out-dir"])
                .arg(&directory)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(2), "{result:?}");
            assert!(result.stdout.is_empty());
            assert!(!directory.exists());
            let result = cli()
                .env("PATH", "")
                .arg("run")
                .arg(&source)
                .args(["--profile", profile, "--diagnostic-format", "json"])
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(2));
            assert!(result.stdout.is_empty());
            let json: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
            assert!(json.to_string().contains("SES-P0001"));
        }
    }
}
