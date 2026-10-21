use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-file-transform-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}
fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn runtime(name: &str) -> PathBuf {
    let output = successful(
        Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(name)
            .output()
            .unwrap(),
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}
struct Program {
    name: String,
    executable: PathBuf,
    argument: Option<PathBuf>,
}
impl Program {
    fn run(&self, cwd: &Path) -> Output {
        let mut command = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            command.arg(argument);
        }
        let output = command
            .current_dir(cwd)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1")
            .output()
            .unwrap();
        if self.argument.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {:?}",
                self.name,
                output
            );
        }
        output
    }
}
fn programs(source: &Path, directory: &Path) -> Vec<Program> {
    let native = directory.join("native");
    successful(
        cli()
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native)
            .output()
            .unwrap(),
    );
    let mut programs = vec![Program {
        name: "native".into(),
        executable: native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }),
        argument: None,
    }];
    for profile in ["development", "release"] {
        let dist = directory.join(profile);
        successful(
            cli()
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&dist)
                .output()
                .unwrap(),
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for name in ["bun", "node"] {
            programs.push(Program {
                name: format!("{profile}-{name}"),
                executable: runtime(name),
                argument: Some(dist.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}
fn copy_package(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_package(&entry.path(), &target);
        } else if entry.file_name() != "seseragi.lock" {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn file_transform_runs_from_source_and_path_package_with_typed_failures() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/filesystem"),
        &package,
    );
    successful(
        cli()
            .args(["lock", "update"])
            .arg(&package)
            .output()
            .unwrap(),
    );
    for (name, source) in [
        (
            "source",
            root().join("examples/spec/fixtures/rust-backend/filesystem-transform.ssrg"),
        ),
        ("package", package.clone()),
    ] {
        for program in programs(&source, &scratch.0.join(name)) {
            for scenario in ["success", "missing", "invalid-utf8", "write-directory"] {
                let cwd = scratch
                    .0
                    .join(format!("{name}-{}-{scenario}", program.name));
                fs::create_dir(&cwd).unwrap();
                let input = if scenario == "invalid-utf8" {
                    &[0xc3, 0xa9, 0xff][..]
                } else {
                    "初期値\n42\n".as_bytes()
                };
                if scenario != "missing" {
                    fs::write(cwd.join("input.txt"), input).unwrap();
                }
                let old =
                    b"old output that must survive a read failure and be truncated on success";
                if scenario == "write-directory" {
                    fs::create_dir(cwd.join("output.txt")).unwrap();
                } else {
                    fs::write(cwd.join("output.txt"), old).unwrap();
                }
                let result = program.run(&cwd);
                let stderr = String::from_utf8_lossy(&result.stderr);
                assert!(
                    result.stdout.is_empty(),
                    "{name}/{}/{}",
                    program.name,
                    scenario
                );
                if scenario == "success" {
                    assert!(result.status.success(), "{name}/{}: {stderr}", program.name);
                    assert_eq!(
                        fs::read(cwd.join("output.txt")).unwrap(),
                        [b"processed:\n".as_slice(), input].concat()
                    );
                } else {
                    assert_eq!(
                        result.status.code(),
                        Some(1),
                        "{name}/{}/{scenario}: {stderr}",
                        program.name
                    );
                    let error = match scenario {
                        "missing" => "FileNotFound",
                        "invalid-utf8" => "InvalidUtf8 { offset: 2 }",
                        _ => "IsADirectory",
                    };
                    assert!(
                        stderr.contains(error),
                        "{name}/{}/{scenario}: {stderr}",
                        program.name
                    );
                    assert!(!stderr.contains("ProviderPackageDefect"));
                    if scenario != "write-directory" {
                        assert_eq!(fs::read(cwd.join("output.txt")).unwrap(), old);
                    }
                }
                if scenario != "missing" {
                    assert_eq!(fs::read(cwd.join("input.txt")).unwrap(), input);
                }
            }
        }
    }
    // The native host implementation cannot execute a JavaScript provider.
    // Reject a package selection instead of silently using another provider.
    let manifest = package.join("seseragi.toml");
    let text = fs::read_to_string(&manifest).unwrap()
        + "\n[providers]\n\"std/fs::FileSystem\"=\"custom/storage#filesystem\"\n";
    fs::write(&manifest, text).unwrap();
    successful(
        cli()
            .args(["lock", "update"])
            .arg(&package)
            .output()
            .unwrap(),
    );
    for operation in ["emit-rust", "build-native"] {
        let out = scratch.0.join(operation);
        let result = cli()
            .args(["experimental", operation])
            .arg(&package)
            .args([
                "--entry",
                "main",
                if operation == "emit-rust" {
                    "--output"
                } else {
                    "--out-dir"
                },
            ])
            .arg(&out)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("SES-RUST-0001"));
        assert!(String::from_utf8_lossy(&result.stderr)
            .contains("native filesystem provider selection"));
        assert!(!out.exists());
    }
}

#[test]
fn file_effects_are_cold_repeatable_and_finalize_on_failure_and_defect() {
    let scratch = Scratch::new();
    let original =
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-effects.ssrg");
    for scenario in ["success", "failure", "defect", "create-new"] {
        let source = match scenario {
            "failure" => original.replace("append \"body\\n\"", "effects.fail \"body-failure\""),
            "defect" => {
                original.replace("append \"body\\n\"", "append (show (9007199254740991 + 1))")
            }
            "create-new" => original.replace("fs.Append", "fs.CreateNew"),
            _ => original.into(),
        };
        let path = scratch.0.join(format!("{scenario}.ssrg"));
        fs::write(&path, source).unwrap();
        for program in programs(&path, &scratch.0.join(scenario)) {
            let cwd = scratch.0.join(format!("{scenario}-{}", program.name));
            fs::create_dir(&cwd).unwrap();
            let result = program.run(&cwd);
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(result.stdout.is_empty());
            let expected = match scenario {
                "success" => "repeat\nrepeat\nacquire\nbody\nrelease\n",
                "create-new" => "repeat\n",
                _ => "repeat\nrepeat\nacquire\nrelease\n",
            };
            assert_eq!(
                fs::read_to_string(cwd.join("trace.txt")).unwrap(),
                expected,
                "{scenario}/{}: {stderr}",
                program.name
            );
            assert_eq!(
                result.status.code(),
                Some(match scenario {
                    "success" => 0,
                    "defect" => 70,
                    _ => 1,
                }),
                "{scenario}/{}: {stderr}",
                program.name
            );
            if scenario != "success" {
                assert!(
                    stderr.contains(match scenario {
                        "failure" => "body-failure",
                        "defect" =>
                            if program.argument.is_none() {
                                "overflow"
                            } else {
                                "defect"
                            },
                        _ => "FileAlreadyExists",
                    }),
                    "{scenario}/{}: {stderr}",
                    program.name
                );
            }
        }
    }
}

#[test]
fn portable_path_operations_match_ts_without_using_host_path_normalization() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/path-portable.ssrg");
    for program in programs(&source, &scratch.0.join("path")) {
        let result = successful(program.run(&scratch.0));
        assert_eq!(
            String::from_utf8(result.stdout).unwrap(),
            include_str!("../../../examples/spec/fixtures/rust-backend/path-portable.stdout"),
            "{}",
            program.name
        );
    }
}

#[test]
fn filesystem_generated_typescript_is_strict_in_both_profiles() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = Scratch::new();
    let mut files = Vec::new();
    for name in [
        "filesystem-transform",
        "filesystem-effects",
        "filesystem-async",
        "path-portable",
    ] {
        let source = fs::read_to_string(
            root().join(format!("examples/spec/fixtures/rust-backend/{name}.ssrg")),
        )
        .unwrap();
        for (profile_name, profile) in [
            ("development", BuildProfile::Development),
            ("release", BuildProfile::Release),
        ] {
            let compiled = compile_module(
                CompileInput::new("main.ssrg", "single-file/main", &source).with_profile(profile),
            )
            .unwrap();
            let path = scratch.0.join(format!("{name}-{profile_name}.ts"));
            fs::write(&path, compiled.generated.typescript).unwrap();
            files.push(path);
        }
    }
    let project = seseragi_project::load_local_project(
        root().join("examples/spec/fixtures/rust-backend-projects/filesystem"),
    )
    .unwrap();
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled =
            seseragi_driver::compile_local_project_with_profile(&project, Some(profile)).unwrap();
        for module in compiled.compiled.modules.values() {
            let path = scratch
                .0
                .join("project")
                .join(name)
                .join(&module.generated.metadata.outputs.typescript);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, &module.generated.typescript).unwrap();
            files.push(path);
        }
    }
    let config = scratch.0.join("tsconfig.json");
    fs::write(&config, serde_json::to_vec(&serde_json::json!({
        "compilerOptions": {"noEmit":true,"strict":true,"skipLibCheck":true,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","typeRoots":[root().join("node_modules/@types")],"paths":{"@seseragi/runtime/*":[root().join("runtime/ts/src/*").to_string_lossy()]}},
        "files":files,
    })).unwrap()).unwrap();
    successful(
        Command::new(runtime("node"))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config)
            .output()
            .unwrap(),
    );
}

#[cfg(unix)]
#[test]
fn filesystem_matches_std_wrapper_lexical_normalization_before_host_access() {
    let scratch = Scratch::new();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-transform.ssrg")
            .replace(
                "paths.child \"input.txt\" (paths.current ())",
                "paths.parse \"link/../input.txt\"",
            )
            .replace(
                "paths.child \"output.txt\" (paths.current ())",
                "paths.parse \"link/../output.txt\"",
            );
    let path = scratch.0.join("symlink.ssrg");
    fs::write(&path, source).unwrap();
    for program in programs(&path, &scratch.0.join("symlink")) {
        let cwd = scratch.0.join(&program.name);
        fs::create_dir(&cwd).unwrap();
        fs::create_dir_all(cwd.join("nested/inside")).unwrap();
        std::os::unix::fs::symlink("nested/inside", cwd.join("link")).unwrap();
        fs::write(cwd.join("input.txt"), "lexical target").unwrap();
        fs::write(cwd.join("nested/input.txt"), "symlink target").unwrap();
        fs::write(cwd.join("nested/output.txt"), "untouched symlink output").unwrap();
        successful(program.run(&cwd));
        assert_eq!(
            fs::read_to_string(cwd.join("output.txt")).unwrap(),
            "processed:\nlexical target",
            "{}",
            program.name
        );
        assert_eq!(
            fs::read_to_string(cwd.join("input.txt")).unwrap(),
            "lexical target"
        );
        assert_eq!(
            fs::read_to_string(cwd.join("nested/output.txt")).unwrap(),
            "untouched symlink output"
        );
    }
}
