use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-basic-standard-{}-{}",
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
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

struct Case<'a> {
    name: &'a str,
    stdout: &'a [u8],
    input: Option<&'a [u8]>,
    output: Option<&'a [u8]>,
    exit: i32,
    failure: Option<&'a str>,
    defect: bool,
}
impl<'a> Case<'a> {
    fn success(stdout: &'a [u8]) -> Self {
        Self {
            name: "success",
            stdout,
            input: None,
            output: None,
            exit: 0,
            failure: None,
            defect: false,
        }
    }
}

fn compare(source: &Path, cases: &[Case<'_>]) {
    let scratch = Scratch::new();
    let original = source.is_file().then(|| fs::read(source).unwrap());
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    for (name, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let out = scratch.0.join(name);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", name, "--out-dir"])
                .arg(&out),
            &scratch.0,
        ));
        let mut files = if source.is_dir() {
            let loaded = seseragi_project::load_local_project(source).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                    .unwrap();
            assert_eq!(compiled.compiled.modules.len(), 2);
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|c| c == std::path::Component::ParentDir)
                );
                let raw = out.join("raw").join(relative);
                fs::create_dir_all(raw.parent().unwrap()).unwrap();
                fs::write(&raw, &module.generated.typescript).unwrap();
                files.push(raw);
            }
            files
        } else {
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    "basic-text-math.ssrg",
                    "basic/text",
                    std::str::from_utf8(original.as_ref().unwrap()).unwrap(),
                )
                .with_profile(profile),
            )
            .unwrap();
            let raw = out.join("raw-original.ts");
            fs::write(&raw, compiled.generated.typescript).unwrap();
            vec![raw]
        };
        if name == "development" {
            files.push(out.join("entry.ts"));
        }
        let config = serde_json::json!({"compilerOptions": {
            "strict": true, "noEmit": true, "skipLibCheck": false,
            "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
            "allowImportingTsExtensions": true, "types": ["bun", "node"],
            "typeRoots": [root().join("node_modules/@types")], "baseUrl": root(),
            "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
            "lib": ["ES2023", "DOM", "DOM.Iterable"]
        }, "files": files});
        let config_path = out.join("raw-tsconfig.json");
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&config_path),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            programs.push((
                format!("{name}-{engine}"),
                runtime(engine, &scratch.0),
                Some(out.join(manifest["entry"].as_str().unwrap())),
                false,
            ));
        }
    }
    assert_eq!(programs.len(), 5);
    for (label, executable, argument, is_native) in programs {
        for case in cases {
            let label = format!("{label}-{}", case.name);
            let directory = scratch.0.join(&label);
            fs::create_dir(&directory).unwrap();
            if let Some(input) = case.input {
                fs::write(directory.join("input.txt"), input).unwrap();
                fs::write(directory.join("output.txt"), b"old output\n").unwrap();
            }
            let mut command = Command::new(&executable);
            command.current_dir(&directory).env("PATH", "");
            if let Some(argument) = &argument {
                command.arg(argument);
            }
            if is_native {
                command.env("SESERAGI_NATIVE_METRICS", "1");
            }
            let result = bounded(&mut command, &directory);
            assert_eq!(result.status.code(), Some(case.exit), "{label}: {result:?}");
            assert_eq!(result.stdout, case.stdout, "{label}");
            let stderr = String::from_utf8_lossy(&result.stderr);
            if is_native {
                assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
                if case.defect {
                    assert!(stderr.contains("deliberate"), "{label}: {stderr}");
                }
            } else if case.defect {
                assert_eq!(stderr, "seseragi: runtime defect\n", "{label}");
            } else if case.failure.is_none() {
                assert!(stderr.is_empty(), "{label}: {stderr}");
            }
            if let Some(failure) = case.failure {
                assert!(stderr.contains(failure), "{label}: {stderr}");
            }
            if let Some(input) = case.input {
                assert_eq!(
                    fs::read(directory.join("input.txt")).unwrap(),
                    input,
                    "{label}"
                );
                assert_eq!(
                    fs::read(directory.join("output.txt")).unwrap(),
                    case.output.unwrap_or(b"old output\n"),
                    "{label}"
                );
            }
        }
    }
    if let Some(original) = original {
        assert_eq!(fs::read(source).unwrap(), original);
    }
}

#[test]
fn basic_text_and_math_match_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-math.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math.stdout"
        ))],
    );
}

#[test]
fn basic_bytes_match_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-bytes.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-bytes.stdout"
        ))],
    );
}

#[test]
fn basic_array_matches_original_raw_strict_ts_and_native() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array.stdout"
        ))],
    );
}

#[test]
fn basic_array_releases_saved_readers_after_cold_replay_and_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-array-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-array-effects.stdout"
        ))],
    );
}

#[test]
fn basic_array_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-array-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("array-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n128\ncold\n128\n256\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_array_path_package_preserves_generic_stages_and_captured_callbacks() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-array"),
        &[Case::success(b"[2, 2]:3:4:42\n")],
    );
}

#[test]
fn basic_bytes_path_package_keeps_canonical_buffers_and_saved_readers() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend-projects/basic-bytes"),
        &[Case::success(b"[0, 128, 255]:0:128:255:-1\nbad:256\n")],
    );
}

#[test]
fn basic_bytes_release_saved_buffers_after_cold_replay_and_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-bytes-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-bytes-effects.stdout"
        ))],
    );
}

#[test]
fn basic_bytes_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-bytes-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("bytes-defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout: b"created\nacquired\ncold\n128\ncold\n128\n510\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

#[test]
fn basic_standard_calls_remain_cold_and_release_saved_functions_after_typed_failure() {
    compare(
        &root().join("examples/spec/fixtures/rust-backend/basic-text-math-effects.ssrg"),
        &[Case::success(include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/basic-text-math-effects.stdout"
        ))],
    );
}

#[test]
fn basic_standard_defect_runs_the_finalizer_and_releases_every_root() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/basic-text-math-effects.ssrg");
    assert_eq!(source.matches("effects.fail \"typed\"").count(), 1);
    let path = scratch.0.join("defect.ssrg");
    fs::write(
        &path,
        source.replace("effects.fail \"typed\"", "effects.die \"deliberate\""),
    )
    .unwrap();
    compare(
        &path,
        &[Case {
            name: "defect",
            stdout:
                b"created\nacquired\ncold\n9.0|16.0\nTrue\ncold\n9.0|16.0\nTrue\n8.0\nreleased\n",
            exit: 70,
            defect: true,
            ..Case::success(b"")
        }],
    );
}

fn copy_package(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let output = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_package(&entry.path(), &output);
        } else {
            fs::copy(entry.path(), output).unwrap();
        }
    }
}

#[test]
fn standard_text_math_path_package_transforms_files_and_preserves_failed_outputs() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/text-math-csv"),
        &package,
    );
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let lock = fs::read(package.join("seseragi.lock")).unwrap();
    let cases = [
        Case {
            name: "valid",
            input: Some("\u{85} 9m, 16m ,#skip,,25m\u{3000}".as_bytes()),
            output: Some(b"3.0\n4.0\n5.0\n"),
            ..Case::success(b"")
        },
        Case {
            name: "domain",
            input: Some(b"-1m,0m"),
            output: Some(b"NaN\n0.0\n"),
            ..Case::success(b"")
        },
        Case {
            name: "parse",
            input: Some(b"9m,not-a-float"),
            exit: 1,
            failure: Some("NumberFailure"),
            ..Case::success(b"")
        },
        Case {
            name: "utf8",
            input: Some(b"\xff"),
            exit: 1,
            failure: Some("TextFailure"),
            ..Case::success(b"")
        },
    ];
    compare(&package, &cases);
    assert_eq!(fs::read(package.join("seseragi.lock")).unwrap(), lock);
}
