use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-deprecated-operators-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove the owned operator acceptance directory");
    }
}

struct OwnedChild {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl OwnedChild {
    fn start(command: &mut Command, directory: &Path) -> Self {
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        command
            .stdout(Stdio::from(File::create(&stdout).unwrap()))
            .stderr(Stdio::from(File::create(&stderr).unwrap()));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self {
            child: {
                let child = command.spawn().unwrap();
                println!("owned operator child PID {}", child.id());
                child
            },
            stdout,
            stderr,
        }
    }
    fn wait(&mut self, timeout: Duration) -> Output {
        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "deadline for owned PID {}\n{}",
                self.child.id(),
                fs::read_to_string(&self.stderr).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        self.child.wait().unwrap();
        println!("waited operator child PID {}", self.child.id());
        Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            #[cfg(unix)]
            unsafe {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // Only this still-live child's dedicated process group.
                kill(-(self.child.id() as i32), 15);
                // Do not reap the group leader during the grace interval. Its
                // reserved PID prevents group-ID reuse before the final kill;
                // compiler descendants may outlive a terminated parent.
                std::thread::sleep(Duration::from_secs(2));
                kill(-(self.child.id() as i32), 9);
            }
            #[cfg(windows)]
            {
                if let Ok(mut terminator) = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    let until = Instant::now() + Duration::from_secs(5);
                    while Instant::now() < until {
                        if terminator.try_wait().ok().flatten().is_some() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let _ = terminator.kill();
                    let _ = terminator.wait();
                }
            }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
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
fn bounded(command: &mut Command, directory: &Path) -> Output {
    OwnedChild::start(command, directory).wait(Duration::from_secs(120))
}
fn runtime(name: &str, directory: &Path) -> PathBuf {
    let output = successful(bounded(
        Command::new(if cfg!(windows) { "where" } else { "which" }).arg(name),
        directory,
    ));
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}

fn root() -> PathBuf {
    // Node24 cannot open script entries through Windows verbatim paths from
    // fs::canonicalize. Cargo's manifest path is absolute; use lexical parents.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root above the crate")
        .to_path_buf()
}

const CLAUSE: &str = "deprecated \"Use combine\" since \"0.61.0\"";

fn copy_project(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let output = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_project(&entry.path(), &output);
        } else {
            fs::copy(entry.path(), output).unwrap();
        }
    }
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn strict_original(
    project: &seseragi_driver::CompiledLocalProject,
    output: &Path,
    scratch: &Path,
    node: &Path,
) {
    fs::create_dir(output).unwrap();
    let mut files = Vec::new();
    for module in project.compiled.modules.values() {
        let relative = Path::new(&module.generated.metadata.outputs.typescript);
        assert!(!relative.is_absolute());
        assert!(!relative
            .components()
            .any(|component| component == std::path::Component::ParentDir));
        let file = output.join(relative);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, &module.generated.typescript).unwrap();
        files.push(file);
    }
    assert_eq!(files.len(), 3);
    let config = serde_json::json!({"compilerOptions": {
        "strict": true, "noEmit": true, "skipLibCheck": false,
        "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
        "allowImportingTsExtensions": true, "types": ["node", "bun"],
        "typeRoots": [root().join("node_modules/@types")],
        "lib": ["ES2023", "DOM", "DOM.Iterable"],
        "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
    }, "files": files});
    fs::write(output.join("tsconfig.json"), config.to_string()).unwrap();
    successful(bounded(
        Command::new(node)
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(output.join("tsconfig.json")),
        scratch,
    ));
}

fn rust_source(project: &seseragi_project::LoadedLocalProject) -> String {
    let core = seseragi_driver::compile_core_local_project(project).unwrap();
    seseragi_backend_rust::emit_rust_project(
        &core
            .compiled
            .modules
            .values()
            .map(|module| &module.core_ir)
            .collect::<Vec<_>>(),
        &core.entry_module,
        "main",
    )
    .unwrap()
    .source
}

#[test]
fn path_package_operator_notices_preserve_original_strict_ts_native_and_stage_lifetimes() {
    let scratch = Scratch::new();
    let engines = [runtime("bun", &scratch.0), runtime("node", &scratch.0)];
    let fixture = root().join("examples/spec/fixtures/rust-backend-projects/deprecation-operators");
    let success_output = fs::read(fixture.join("expected.stdout")).unwrap();
    for (label, code, expected) in [
        ("success", 0, success_output.as_slice()),
        ("defect", 70, b"acquired\nsaved\nreleased\n".as_slice()),
    ] {
        let parent = scratch.0.join(label);
        fs::create_dir(&parent).unwrap();
        let annotated = parent.join("annotated");
        copy_project(&fixture, &annotated);
        if code == 70 {
            let provider = annotated.join("vendor/model/src/lib.ssrg");
            let source = fs::read_to_string(&provider).unwrap();
            assert_eq!(source.matches("self.value + other.value").count(), 2);
            // Only the selected Box body fails; the Other control is intact.
            fs::write(
                provider,
                source.replacen("self.value + other.value", "9007199254740991 + 1", 1),
            )
            .unwrap();
            fs::write(annotated.join("src/main.ssrg"), concat!(
                "// 呼出し時期 🙂\n",
                "import { Box as Container } from \"./facade\"\n",
                "import * as effects from \"std/effect\"\n",
                "effect fn release _value: Unit -> Unit = effects.recover (\\_: ConsoleError -> effects.succeed ()) (println \"released\")\n",
                "pub effect fn main = effects.scoped $ do {\n",
                " effects.acquireRelease (do { println \"acquired\"; effects.succeed () }) release\n",
                " let saved: Container -> Container = (+) (Container { value: 40 })\n",
                " println \"saved\"\n",
                " let result = saved (Container { value: 2 })\n",
                " println (\"unreachable: \" + show result.value)\n",
                "}\n",
            )).unwrap();
        }
        let ordinary = parent.join("ordinary");
        copy_project(&annotated, &ordinary);
        let provider = ordinary.join("vendor/model/src/lib.ssrg");
        let source = fs::read_to_string(&provider).unwrap();
        assert_eq!(source.matches(CLAUSE).count(), 1);
        fs::write(provider, source.replace(CLAUSE, "")).unwrap();
        for project in [&annotated, &ordinary] {
            successful(bounded(
                cli().args(["lock", "update"]).arg(project),
                &scratch.0,
            ));
        }
        let noticed = seseragi_project::load_local_project(&annotated).unwrap();
        let plain = seseragi_project::load_local_project(&ordinary).unwrap();
        // Compare the original self-contained native program as well as TS.
        // No emitter, evidence, parameter list, or body is rewritten for this proof.
        assert_eq!(rust_source(&noticed), rust_source(&plain), "{label}");
        let lint = bounded(
            cli().arg("lint").arg(&annotated).args([
                "--diagnostic-format",
                "json",
                "--deny-warnings",
            ]),
            &scratch.0,
        );
        assert_eq!(lint.status.code(), Some(1), "{lint:?}");
        assert!(lint.stdout.is_empty());
        let report: serde_json::Value = serde_json::from_slice(&lint.stderr).unwrap();
        let deprecated = report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|document| document["diagnostics"]["diagnostics"].as_array().unwrap())
            .filter(|warning| warning["code"] == "SES-L0201")
            .collect::<Vec<_>>();
        assert_eq!(
            deprecated.len(),
            if code == 0 { 3 } else { 1 },
            "{report:#}"
        );
        for warning in deprecated {
            assert!(warning.to_string().contains("Use combine"));
            assert!(warning.to_string().contains("vendor/model/src/lib.ssrg"));
        }
        for (name, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&noticed, Some(profile))
                    .unwrap();
            let original =
                seseragi_driver::compile_local_project_with_profile(&plain, Some(profile)).unwrap();
            assert_eq!(
                compiled.compiled.modules.len(),
                original.compiled.modules.len()
            );
            for (id, module) in &compiled.compiled.modules {
                assert_eq!(
                    module.generated.typescript, original.compiled.modules[id].generated.typescript,
                    "{id}/{name}/{label}"
                );
            }
            strict_original(
                &compiled,
                &parent.join(format!("{name}-strict")),
                &scratch.0,
                &engines[1],
            );
            let output = parent.join(name);
            successful(bounded(
                cli()
                    .arg("build")
                    .arg(&annotated)
                    .args(["--profile", name, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            ));
            for engine in &engines {
                let result = bounded(
                    Command::new(engine)
                        .arg(output.join("entry.js"))
                        .current_dir(&output),
                    &scratch.0,
                );
                assert_eq!(
                    result.status.code(),
                    Some(code),
                    "{label}/{name}: {result:?}"
                );
                assert_eq!(result.stdout, expected, "{label}/{name}");
                if code == 0 {
                    assert!(result.stderr.is_empty(), "{result:?}");
                } else {
                    assert!(
                        String::from_utf8_lossy(&result.stderr)
                            .contains("seseragi: runtime defect"),
                        "{result:?}"
                    );
                }
            }
        }
        let native = parent.join("native");
        successful(bounded(
            cli()
                .args(["experimental", "build-native"])
                .arg(&annotated)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let result = bounded(
            Command::new(native.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }))
            .current_dir(&parent)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{result:?}");
        assert_eq!(result.stdout, expected, "{label}/native");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("managed-live=0"),
            "{result:?}"
        );
        if code == 70 {
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("Seseragi Int overflow"),
                "{result:?}"
            );
        }
    }
}
