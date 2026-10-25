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
            "seseragi-program-wasm-{}-{}",
            std::process::id(),
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
            child: command.spawn().unwrap(),
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
fn build(source: &Path, destination: &Path, exports: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    command
        .args(["experimental", "build-wasm"])
        .arg(source)
        .arg("--out-dir")
        .arg(destination);
    for export in exports {
        command.args(["--export", export]);
    }
    command
}
fn image(path: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().into_string().unwrap(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect()
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

fn artifact_image(path: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, path: &Path, image: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(root, &entry.path(), image);
            } else {
                image.insert(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    visit(path, path, &mut result);
    result
}

fn standard_build(package: &Path, output: &Path, profile: &str) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    c.arg("build")
        .arg(package)
        .args(["--target", "web", "--profile", profile, "--out-dir"])
        .arg(output);
    c
}

#[test]
fn standard_web_build_owns_program_assets_types_and_the_selected_import_closure() {
    let root = root();
    let scratch = Scratch::new();
    let package = root.join("examples/spec/fixtures/rust-backend-projects/program-wasm-package");
    let selected = seseragi_project::load_local_project_module(
        &package,
        &seseragi_project::ModulePath::parse("calculation").unwrap(),
    )
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&selected).unwrap();
    let modules = core
        .compiled
        .order
        .iter()
        .map(|id| &core.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let native =
        seseragi_backend_rust::emit_rust_project(&modules, &core.entry_module, "probe").unwrap();
    fs::write(scratch.0.join("oracle.rs"), native.source).unwrap();
    let binary = scratch.0.join(if cfg!(windows) {
        "oracle.exe"
    } else {
        "oracle"
    });
    successful(bounded(
        Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "oracle", "-O"])
            .arg(scratch.0.join("oracle.rs"))
            .arg("-o")
            .arg(&binary),
        &scratch.0,
    ));
    let result = successful(bounded(
        Command::new(&binary)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "合計: 42|3|negative\n"
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    for (profile, value) in [
        ("development", seseragi_project::BuildProfile::Development),
        ("release", seseragi_project::BuildProfile::Release),
    ] {
        let site = scratch.0.join(profile);
        successful(bounded(
            &mut standard_build(&package, &site, profile),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        let program = &manifest["programs"][0];
        assert_eq!(program["hostImport"], "@seseragi/programs/calculation");
        assert_eq!(program["wasm"], "programs/calculation/program.wasm");
        let directory = site.join("programs/calculation");
        let receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("program-wasm.json")).unwrap())
                .unwrap();
        assert_eq!(program["module"], receipt["module"]);
        assert_eq!(receipt["modules"].as_array().unwrap().len(), 2);
        let expected = selected
            .graph()
            .topological_order()
            .unwrap()
            .iter()
            .map(seseragi_project::logical_module_id)
            .collect::<Vec<_>>();
        assert_eq!(receipt["modules"], serde_json::json!(expected));
        for name in [
            "program.wasm",
            "program.mjs",
            "program.d.mts",
            "program-wasm.json",
            "index.mjs",
            "index.d.mts",
        ] {
            assert!(manifest["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|file| file["path"] == format!("programs/calculation/{name}")));
        }
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(engine)
                    .arg(root.join("scripts/program-wasm-probe.mjs"))
                    .arg(&directory),
                &scratch.0,
            ));
            let proof: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(proof["calls"], 1000);
            assert_eq!(proof["normalManagedLiveAtClose"], 0);
        }
        // The generated virtual package declaration checks the original bridge,
        // without copying an implementation or weakening its declared types.
        let config = scratch.0.join(format!("types-{profile}.json"));
        fs::write(&config, serde_json::to_vec(&serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","lib":["ES2022","DOM"],"types":[],"skipLibCheck":true,"paths":{"@seseragi/programs/calculation":[site.join("programs/calculation/index.d.mts")] }},"files":[package.join("host/bridge.ts")]})).unwrap()).unwrap();
        successful(bounded(
            Command::new("node")
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("-p")
                .arg(&config),
            &scratch.0,
        ));
        let raw = scratch.0.join(format!("raw-{profile}"));
        fs::create_dir(&raw).unwrap();
        let compiled =
            seseragi_driver::compile_local_project_with_profile(&selected, Some(value)).unwrap();
        for module in compiled.compiled.modules.values() {
            let path = raw.join(&module.generated.metadata.outputs.typescript);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &module.generated.typescript).unwrap();
        }
        seseragi_runtime::stage_typescript_package(&raw).unwrap();
        let entry = &compiled.compiled.modules[&compiled.entry_module]
            .generated
            .metadata
            .outputs
            .typescript;
        fs::write(
            raw.join("oracle.ts"),
            format!("import {{ probe }} from './{entry}'; console.log(probe(undefined));\n"),
        )
        .unwrap();
        successful(bounded(
            Command::new("bun").current_dir(&raw).args([
                "build",
                "oracle.ts",
                "--target=node",
                "--outfile=oracle.mjs",
            ]),
            &scratch.0,
        ));
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(engine).arg(raw.join("oracle.mjs")),
                &scratch.0,
            ));
            assert_eq!(
                String::from_utf8(result.stdout).unwrap(),
                "合計: 42|3|negative\n"
            );
        }
    }
}

#[test]
fn failed_standard_program_build_preserves_the_complete_site() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_tree(
        &root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
        &package,
    );
    let site = scratch.0.join("site");
    successful(bounded(
        &mut standard_build(&package, &site, "development"),
        &scratch.0,
    ));
    let previous = artifact_image(&site);
    let source = package.join("src/calculation.ssrg");
    let original = fs::read_to_string(&source).unwrap();
    for replacement in [
        "pub fn label value: Ref<Int> -> Int = 1\n",
        "pub effect fn label -> Unit = ()\n",
        "pub fn label value: Unknown -> Int = 1\n",
        "pub fn missing value: Int -> Int = value\n",
    ] {
        fs::write(&source, replacement).unwrap();
        let result = bounded(
            &mut standard_build(&package, &site, "development"),
            &scratch.0,
        );
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("calculation"));
        assert_eq!(artifact_image(&site), previous);
    }
    fs::write(&source, original).unwrap();
    let result = bounded(
        standard_build(&package, &site, "development").env("PATH", ""),
        &scratch.0,
    );
    assert!(!result.status.success());
    assert_eq!(artifact_image(&site), previous);
    // An unused unsupported body in the selected closure must still fail
    // admission; exporting only pure functions does not license pruning it.
    fs::write(
        &source,
        format!(
            "{}\npub effect fn unsupported -> Unit = ()\n",
            fs::read_to_string(&source).unwrap()
        ),
    )
    .unwrap();
    let result = bounded(
        &mut standard_build(&package, &site, "development"),
        &scratch.0,
    );
    assert!(!result.status.success());
    assert_eq!(artifact_image(&site), previous);
    let original_source = fs::read_to_string(root().join(
        "examples/spec/fixtures/rust-backend-projects/program-wasm-package/src/calculation.ssrg",
    ))
    .unwrap();
    fs::write(&source, original_source).unwrap();
    let bad = scratch.0.join("bad");
    fs::create_dir_all(package.join("public/programs/calculation")).unwrap();
    fs::write(
        package.join("public/programs/calculation/program.wasm"),
        "host collision",
    )
    .unwrap();
    let toml = package.join("seseragi.toml");
    let original = fs::read_to_string(&toml).unwrap();
    fs::write(
        &toml,
        original.replace(
            "[web.programs.calculation]",
            "[web]\npublic='public'\n[web.programs.calculation]",
        ),
    )
    .unwrap();
    let lock = || {
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["lock", "update"])
                .arg(&package),
            &scratch.0,
        ));
    };
    lock();
    let result = bounded(
        &mut standard_build(&package, &bad, "development"),
        &scratch.0,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("collides"));
    assert!(!bad.exists());
    assert_eq!(artifact_image(&site), previous);
    fs::write(&toml, &original).unwrap();
    lock();
    // Web watch is covered in tests/dev.rs. A process manifest still refuses
    // dev before rustc, even when it also configures a valid pure program.
    fs::write(
        &toml,
        original.replace("target = \"web\"", "target = \"process\""),
    )
    .unwrap();
    lock();
    let result = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("dev")
            .arg(&package)
            .args(["--port", "0"])
            .env("PATH", ""),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("dev does not support the `process` target"));
    assert!(!package.join(".seseragi/dev").exists());
    fs::write(&toml, &original).unwrap();
    lock();
    let shadow = package.join("host/node_modules/@seseragi/programs");
    fs::create_dir_all(&shadow).unwrap();
    fs::write(
        shadow.join("package.json"),
        "{\"name\":\"@seseragi/programs\",\"type\":\"module\"}",
    )
    .unwrap();
    lock();
    let result = bounded(
        &mut standard_build(&package, &site, "development"),
        &scratch.0,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("reserved @seseragi/programs"));
    assert_eq!(artifact_image(&site), previous);
}

#[test]
fn multiple_programs_keep_path_package_identity_and_initialization() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_tree(
        &root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
        &package,
    );
    let dependency = scratch.0.join("dependency");
    fs::create_dir_all(dependency.join("src")).unwrap();
    fs::write(dependency.join("seseragi.toml"), "[package]\nname='fixture/program-wasm-dependency'\nversion='0.0.0'\nlanguage='^0.1.0'\n[exports]\n'.'='calculation'\n").unwrap();
    fs::write(
        dependency.join("src/calculation.ssrg"),
        "let initial = 0\npub fn zero -> Int = initial\n",
    )
    .unwrap();
    fs::write(package.join("src/constants.ssrg"), "import { zero } from \"math\"\npub let offset: Int = zero ()\npub fn get -> Int = offset\n").unwrap();
    let manifest = package.join("seseragi.toml");
    let original = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, format!("{original}\n[dependencies]\nmath={{package='fixture/program-wasm-dependency',path='../dependency'}}\n[web.programs.constants]\nmodule='constants'\nexports=['get']\n")).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let site = scratch.0.join("site");
    successful(bounded(
        &mut standard_build(&package, &site, "release"),
        &scratch.0,
    ));
    let image = artifact_image(&site);
    let manifest: serde_json::Value =
        serde_json::from_slice(&image["artifact-manifest.json"]).unwrap();
    assert_eq!(manifest["programs"].as_array().unwrap().len(), 2);
    let calc: serde_json::Value =
        serde_json::from_slice(&image["programs/calculation/program-wasm.json"]).unwrap();
    assert_eq!(calc["modules"].as_array().unwrap().len(), 3);
    let selected = seseragi_project::load_local_project_module(
        &package,
        &seseragi_project::ModulePath::parse("calculation").unwrap(),
    )
    .unwrap();
    let expected = selected
        .graph()
        .topological_order()
        .unwrap()
        .iter()
        .map(seseragi_project::logical_module_id)
        .collect::<Vec<_>>();
    assert_eq!(calc["modules"], serde_json::json!(expected));
    fs::write(scratch.0.join("multi.mjs"), "import assert from 'node:assert/strict'; import {readFile} from 'node:fs/promises'; import {loadProgram} from './site/programs/calculation/index.mjs'; import {loadProgram as loadConstants} from './site/programs/constants/index.mjs'; const p=await loadProgram(await readFile(new URL('./site/programs/calculation/program.wasm',import.meta.url))); const q=await loadConstants(await readFile(new URL('./site/programs/constants/program.wasm',import.meta.url))); const prefix=p.exports.label('合計: ');const cb=prefix(40);prefix.dispose();assert.equal(cb(2),'合計: 42');assert.equal(q.exports.get(undefined),0);p.dispose();assert.equal(q.exports.get(undefined),0);q.dispose();assert.equal(p.ownership().managedLiveAtClose,0);assert.equal(q.ownership().managedLiveAtClose,0);console.log('two instances roots0');").unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(engine).arg(scratch.0.join("multi.mjs")),
            &scratch.0,
        ));
        assert_eq!(
            String::from_utf8(result.stdout).unwrap().trim(),
            "two instances roots0"
        );
    }
    // Imported initialization is observed, including its first-stage Defect.
    fs::write(
        dependency.join("src/calculation.ssrg"),
        "let initial = 9007199254740991 + 1\npub fn zero -> Int = initial\n",
    )
    .unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    successful(bounded(
        &mut standard_build(&package, &site, "release"),
        &scratch.0,
    ));
    fs::write(scratch.0.join("initial-defect.mjs"), "import assert from 'node:assert/strict'; import {readFile} from 'node:fs/promises';import {loadProgram,ProgramWasmDefect} from './site/programs/calculation/index.mjs';await assert.rejects(()=>readFile(new URL('./site/programs/calculation/program.wasm',import.meta.url)).then(bytes=>loadProgram(bytes)),ProgramWasmDefect);console.log('imported initializer Defect');").unwrap();
    for engine in ["bun", "node"] {
        successful(bounded(
            Command::new(engine).arg(scratch.0.join("initial-defect.mjs")),
            &scratch.0,
        ));
    }
}

#[cfg(unix)]
#[test]
fn changing_checked_ui_during_bundle_cannot_publish_a_mixed_site() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_tree(
        &root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-package"),
        &package,
    );
    let site = scratch.0.join("site");
    successful(bounded(
        &mut standard_build(&package, &site, "development"),
        &scratch.0,
    ));
    let previous = artifact_image(&site);
    let bun = successful(bounded(
        Command::new("bun").args(["-e", "console.log(process.execPath)"]),
        &scratch.0,
    ));
    let bun = String::from_utf8(bun.stdout).unwrap().trim().to_owned();
    let shim = scratch.0.join("shim");
    fs::create_dir(&shim).unwrap();
    fn quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
    fs::write(shim.join("bun"), format!("#!/bin/sh\n{} \"$@\"\nstatus=$?\nif [ \"$status\" = 0 ] && [ \"$1\" = build ]; then printf '\\n' >> {}; fi\nexit $status\n", quote(&bun), quote(package.join("src/main.ssrg").to_str().unwrap()))).unwrap();
    fs::set_permissions(shim.join("bun"), fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(shim).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let result = bounded(
        standard_build(&package, &site, "development").env("PATH", path),
        &scratch.0,
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("source changed during program WASM build"));
    assert_eq!(artifact_image(&site), previous);
}

#[test]
fn checked_program_runs_on_bun_and_node_and_preserves_owned_lifetimes() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm.ssrg");
    let output = scratch.0.join("program");
    let other = scratch.0.join("different-contract");
    successful(bounded(
        build(
            &source,
            &output,
            &[
                "label",
                "validate",
                "enabled",
                "optional",
                "transport",
                "staged",
                "ready",
                "deep",
            ],
        )
        .current_dir(&scratch.0),
        &scratch.0,
    ));
    successful(bounded(
        build(&source, &other, &["label"]).current_dir(&scratch.0),
        &scratch.0,
    ));
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert!(
        metadata["rustc"]["version"]
            .as_str()
            .unwrap()
            .starts_with("rustc 1.97.1 "),
        "{metadata}"
    );
    assert_eq!(metadata["exports"][0]["sourceArity"], 2);
    assert_eq!(metadata["exports"][5]["sourceArity"], 1);
    // Both ordinary targets execute the same calculation body. Only the TS
    // process entry is appended; the WASM boundary does not accept Console.
    let oracle = scratch.0.join("oracle.ssrg");
    let original = fs::read_to_string(&source).unwrap();
    fs::write(
        &oracle,
        format!("{original}\npub effect fn main = println (probe ())\n"),
    )
    .unwrap();
    for profile in ["development", "release"] {
        let directory = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&oracle)
                .args(["--profile", profile, "--out-dir"])
                .arg(&directory),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = directory.join(manifest["entry"].as_str().unwrap());
        let profile_value = if profile == "development" {
            seseragi_project::BuildProfile::Development
        } else {
            seseragi_project::BuildProfile::Release
        };
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                source.to_str().unwrap(),
                "proof/program-wasm",
                &original,
            )
            .with_profile(profile_value),
        )
        .unwrap();
        fs::write(directory.join("raw.ts"), raw.generated.typescript).unwrap();
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true},"files":["raw.ts"]});
        fs::write(
            directory.join("raw-tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(directory.join("raw-tsconfig.json")),
            &scratch.0,
        ));
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(&entry)
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(
                String::from_utf8(result.stdout).unwrap(),
                "合計: 42|3|negative\n"
            );
        }
    }
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "probe", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let native = native.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let result = successful(bounded(
        Command::new(&native)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "合計: 42|3|negative\n"
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    for engine in ["bun", "node"] {
        let engine = runtime(engine, &scratch.0);
        let result = successful(bounded(
            Command::new(engine)
                .arg(root().join("scripts/program-wasm-probe.mjs"))
                .arg(&output)
                .arg(&other)
                .current_dir(&scratch.0)
                .env("PATH", ""),
            &scratch.0,
        ));
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["normalManagedLiveAtClose"], 0);
        assert_eq!(report["engineStackTrap"], "retired");
        assert_eq!(report["recovery"], true);
    }
    fs::write(
        scratch.0.join("types.mts"),
        r#"
import {loadProgram, type OwnedCallback} from "./program/program.mjs";
const program=await loadProgram(new Uint8Array());
const prefix=program.exports.label("合計: ");
const f:OwnedCallback<number,string>=prefix(40);
const text:string=f(2);
f.dispose();
const result=program.exports.validate(2);
if(result.tag==="Left") { const message:string=result.value; }
else { const value:number=result.value; }
const pair:readonly [boolean,undefined]=program.exports.enabled(true);
// @ts-expect-error Int is a number, not a String
f("wrong");
// @ts-expect-error source Unit is distinct from the returned function's Unit
program.exports.ready(1);
// @ts-expect-error result variants must be narrowed
const wrong:string=result.value;
program.dispose();
"#,
    )
    .unwrap();
    fs::write(scratch.0.join("tsconfig.json"),r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"]},"files":["types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
    let ui = scratch.0.join("ui-source");
    fs::create_dir_all(ui.join("program")).unwrap();
    for name in ["main.ssrg", "bridge.ts"] {
        fs::copy(
            root()
                .join("examples/spec/fixtures/rust-backend-projects/program-wasm-ui")
                .join(name),
            ui.join(name),
        )
        .unwrap();
    }
    for (name, bytes) in image(&output) {
        fs::write(ui.join("program").join(name), bytes).unwrap();
    }
    let web = scratch.0.join("web");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(ui.join("main.ssrg"))
            .args(["--target", "web", "--out-dir"])
            .arg(&web),
        &scratch.0,
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(web.join("artifact-manifest.json")).unwrap()).unwrap();
    assert!(web.join(manifest["entry"].as_str().unwrap()).is_file());
    // Check the handwritten host boundary against generated declarations too.
    fs::write(scratch.0.join("tsconfig-host.json"),r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"]},"files":["ui-source/bridge.ts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig-host.json")),
        &scratch.0,
    ));
}

#[test]
fn links_a_package_through_the_same_checked_core_graph() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(package.join("seseragi.toml"),"[package]\nname=\"fixture/program-wasm\"\nversion=\"0.0.0\"\nlanguage=\">=0.1.0 <0.2.0\"\n[run]\nentry=\"main\"\ntarget=\"process\"\n").unwrap();
    fs::write(
        package.join("src/lib.ssrg"),
        "pub fn label value: Int -> String = `合計: ${value + 1}`\n",
    )
    .unwrap();
    fs::write(
        package.join("src/main.ssrg"),
        "import { label } from \"./lib\"\npub fn selected value: Int -> String = label value\n",
    )
    .unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let output = scratch.0.join("program");
    successful(bounded(
        &mut build(&package, &output, &["selected"]),
        &scratch.0,
    ));
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(output.join("program-wasm.json")).unwrap()).unwrap();
    assert_eq!(metadata["modules"].as_array().unwrap().len(), 2);
    let inputs = metadata["sources"].as_array().unwrap();
    assert_eq!(inputs.len(), 3);
    let names = inputs
        .iter()
        .map(|input| {
            Path::new(input["path"].as_str().unwrap())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        ["main.ssrg", "lib.ssrg", "seseragi.toml"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    for input in inputs {
        use sha2::{Digest, Sha256};
        assert_eq!(
            input["sha256"],
            format!(
                "{:x}",
                Sha256::digest(fs::read(input["path"].as_str().unwrap()).unwrap())
            )
        );
    }
    fs::write(scratch.0.join("package-probe.mjs"),"import {readFile} from 'node:fs/promises'; import assert from 'node:assert/strict'; import {loadProgram} from './program/program.mjs'; const p=await loadProgram(await readFile(new URL('./program/program.wasm',import.meta.url))); assert.equal(p.exports.selected(41),'合計: 42'); p.dispose(); assert.equal(p.ownership().managedLiveAtClose,0); console.log('package roots0');").unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(scratch.0.join("package-probe.mjs"))
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(
            String::from_utf8(result.stdout).unwrap().trim(),
            "package roots0"
        );
    }
    let manifest = package.join("seseragi.toml");
    let configured = fs::read_to_string(&manifest).unwrap()
        + "\n[providers]\n\"std/fs::FileSystem\"=\"custom/storage#filesystem\"\n";
    fs::write(&manifest, configured).unwrap();
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&package),
        &scratch.0,
    ));
    let rejected = scratch.0.join("provider-wasm");
    let result = bounded(
        build(&package, &rejected, &["selected"]).env("PATH", ""),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("this program WASM ABI does not accept provider configuration"),
        "{result:?}"
    );
    assert!(result.stdout.is_empty());
    assert!(!rejected.exists());
}

#[test]
fn initialization_defect_is_an_opaque_program_defect_before_exports_escape() {
    let scratch = Scratch::new();
    let source = scratch.0.join("initialization.ssrg");
    fs::write(
        &source,
        "pub let initial: Int = 9007199254740991 + 1\npub fn selected value: Int -> Int = value\n",
    )
    .unwrap();
    let output = scratch.0.join("program");
    successful(bounded(
        &mut build(&source, &output, &["selected"]),
        &scratch.0,
    ));
    fs::write(scratch.0.join("initialize.mjs"),"import {readFile} from 'node:fs/promises'; import assert from 'node:assert/strict'; import {loadProgram,ProgramWasmDefect} from './program/program.mjs'; await assert.rejects(loadProgram(await readFile(new URL('./program/program.wasm',import.meta.url))), e=>e instanceof ProgramWasmDefect && e.message.includes('initialization') && Object.getPrototypeOf(e.cause)===Object.prototype); console.log('initialization rejected');").unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(scratch.0.join("initialize.mjs"))
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(
            String::from_utf8(result.stdout).unwrap().trim(),
            "initialization rejected"
        );
    }
}

#[test]
fn rejects_unproven_boundaries_before_rustc_and_preserves_existing_output() {
    let scratch = Scratch::new();
    for (index,source) in [
        "pub fn selected<A> value: A -> A = value",
        "pub fn selected value: Float -> Float = value",
        "pub fn selected value: (Int -> Int) -> Int = value 1",
        "fn selected value: Int -> Int = value",
        "type Int = | NominalInt\npub fn selected value: Int -> Int = value",
        "import * as effects from \"std/effect\"\npub fn selected value: Int -> Effect<{}, Never, Int> = effects.succeed (value + 1)",
    ].iter().enumerate() {
        let path=scratch.0.join(format!("invalid-{index}-雪.ssrg"));
        fs::write(&path,source).unwrap();
        let destination=scratch.0.join(format!("invalid-{index}"));
        let result=bounded(build(&path,&destination,&["selected"]).env("PATH",""),&scratch.0);
        assert_eq!(result.status.code(),Some(2),"{result:?}");
        assert!(result.stdout.is_empty());
        let error=String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("SES-RUST-0001"),"{error}");
        assert!(error.contains("ssrg:"),"{error}");
        assert!(!error.contains("could not start rustc"),"{error}");
        assert!(!destination.exists());
    }
    let source = scratch.0.join("source.ssrg");
    fs::write(&source, "pub fn selected value: Int -> Int = value + 1\n").unwrap();
    let destination = scratch.0.join("owned");
    successful(bounded(
        &mut build(&source, &destination, &["selected"]),
        &scratch.0,
    ));
    let before = image(&destination);
    let result = bounded(
        build(&source, &destination, &["selected"]).env("PATH", ""),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(2), "{result:?}");
    assert!(String::from_utf8_lossy(&result.stderr).contains("could not start rustc"));
    assert_eq!(image(&destination), before);
    // A valid rebuild replaces the complete artifact, never a partial directory.
    fs::write(&source, "pub fn selected value: Int -> Int = value + 2\n").unwrap();
    successful(bounded(
        &mut build(&source, &destination, &["selected"]),
        &scratch.0,
    ));
    assert_ne!(image(&destination)["program.wasm"], before["program.wasm"]);
    fs::write(scratch.0.join("rebuilt.mjs"),"import {readFile} from 'node:fs/promises'; import assert from 'node:assert/strict'; import {loadProgram} from './owned/program.mjs'; const p=await loadProgram(await readFile(new URL('./owned/program.wasm',import.meta.url))); assert.equal(p.exports.selected(40),42); p.dispose(); assert.equal(p.ownership().managedLiveAtClose,0); console.log('rebuilt body');").unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(scratch.0.join("rebuilt.mjs"))
            .env("PATH", ""),
        &scratch.0,
    ));
    fs::write(destination.join("program.mjs"), "user changes").unwrap();
    let edited = image(&destination);
    let result = bounded(&mut build(&source, &destination, &["selected"]), &scratch.0);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("modified program WASM output"));
    assert_eq!(image(&destination), edited);
    let unrelated = scratch.0.join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    fs::write(unrelated.join("keep.txt"), "keep").unwrap();
    let result = bounded(&mut build(&source, &unrelated, &["selected"]), &scratch.0);
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(
        fs::read_to_string(unrelated.join("keep.txt")).unwrap(),
        "keep"
    );
    assert!(!fs::read_dir(&scratch.0).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".seseragi-program-wasm-")));
}

#[test]
fn providerless_program_effects_run_cancel_and_release_on_bun_and_node() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm-effects.ssrg");
    let output = scratch.0.join("effect");
    successful(bounded(
        &mut build(
            &source,
            &output,
            &[
                "value",
                "rejection",
                "defect",
                "callback",
                "pending",
                "cleanup",
                "cancelCleanup",
                "saved",
                "staged",
                "transport",
                "late",
            ],
        ),
        &scratch.0,
    ));
    let former = scratch.0.join("former.ssrg");
    // This exact source was a pure-v1 rejection. It is now a checked positive
    // control; the negative suite retains an actual aborting arithmetic case.
    fs::write(&former, "import * as effects from \"std/effect\"\npub fn selected value: Int -> Effect<{}, Never, Int> = effects.succeed value").unwrap();
    let former_output = scratch.0.join("former");
    successful(bounded(
        &mut build(&former, &former_output, &["selected"]),
        &scratch.0,
    ));
    for engine in ["bun", "node"] {
        let engine = runtime(engine, &scratch.0);
        let result = successful(bounded(
            Command::new(engine)
                .arg(root().join("scripts/program-wasm-effect-probe.mjs"))
                .arg(&output)
                .arg(&former_output)
                .current_dir(&scratch.0)
                .env("PATH", ""),
            &scratch.0,
        ));
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["abi"], "seseragi-program-wasm-effect-v1");
        assert_eq!(report["callbacks"], 3);
        assert_eq!(report["cancelled"], 2);
        assert_eq!(report["disposeDefectObservable"], true);
        assert_eq!(report["unexpectedTrapAndProtocolRetired"], true);
        assert_eq!(report["normalManagedLiveAtClose"], 0);
    }
    fs::write(scratch.0.join("types.mts"), r#"
import {loadProgram, type OwnedEffect, type OwnedCallback} from "./effect/program.mjs";
const owner=await loadProgram(new Uint8Array());
const plan:OwnedEffect<string,string>=owner.exports.value("input");
const execution=plan.run(); execution.cancel();
const exit=await execution.result;
if(exit.tag==="Failure") { const error:string=exit.error; }
if(exit.tag==="Success") { const value:string=exit.value; }
if(exit.tag==="Defect") { const errors:readonly string[]=exit.messages; }
const stage:OwnedCallback<undefined,OwnedEffect<never,OwnedCallback<string,string>>>=owner.exports.staged("prefix");
const closing:Promise<void>=owner.dispose(); await closing;
// @ts-expect-error Effect needs explicit execution, not a sync String result
const wrong:string=owner.exports.value("input");
// @ts-expect-error source Unit is separate from the returned String callback
stage("wrong");
// @ts-expect-error result must be narrowed before selecting a success value
const value:string=exit.value;
"#).unwrap();
    fs::write(scratch.0.join("tsconfig.json"),r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false},"files":["types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn literal_unit_values_match_native_and_strict_ts_profiles() {
    let scratch = Scratch::new();
    let root = root();
    let cli = Path::new(env!("CARGO_BIN_EXE_seseragi"));
    let fixtures = root.join("examples/spec/fixtures/rust-backend");
    let engines = [runtime("bun", &scratch.0), runtime("node", &scratch.0)];
    for (name, entry, code) in [
        ("effect-unit-values", "main", 0),
        ("effect-unit-finalizers", "failed", 1),
        ("effect-unit-finalizers", "crash", 70),
    ] {
        let case = scratch.0.join(format!("{name}-{entry}"));
        fs::create_dir(&case).unwrap();
        let mut text = fs::read_to_string(fixtures.join(format!("{name}.ssrg"))).unwrap();
        if entry != "main" {
            text.push_str(&format!("\npub effect fn main = {entry} ()\n"));
        }
        let source = case.join("main.ssrg");
        fs::write(&source, &text).unwrap();
        let expected = if code == 0 {
            fs::read(fixtures.join(format!("{name}.stdout"))).unwrap()
        } else {
            vec![]
        };
        let native = case.join("native");
        successful(bounded(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let cwd = case.join("native-run");
        fs::create_dir(&cwd).unwrap();
        let result = bounded(
            Command::new(native.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }))
            .current_dir(&cwd)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{result:?}");
        assert_eq!(result.stdout, expected);
        assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
        if code != 0 {
            assert_eq!(
                fs::read(cwd.join("trace.txt")).unwrap(),
                b"acquire\nbody: ()\nrelease\n"
            );
        }
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let site = case.join(profile);
            successful(bounded(
                Command::new(cli)
                    .arg("build")
                    .arg(&source)
                    .args(["--target", "process", "--profile", profile, "--out-dir"])
                    .arg(&site),
                &scratch.0,
            ));
            let raw = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new("unit.ssrg", "proof/unit", &text)
                    .with_profile(value),
            )
            .unwrap();
            fs::write(site.join("raw.ts"), raw.generated.typescript).unwrap();
            let config = serde_json::json!({"compilerOptions":{"strict":true,"skipLibCheck":false,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","allowImportingTsExtensions":true,"lib":["ES2023","DOM","DOM.Iterable"],"types":["node"],"typeRoots":[root.join("node_modules/@types")],"baseUrl":root,"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]}},"files":["raw.ts"]});
            fs::write(
                site.join("unit-tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(bounded(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(site.join("unit-tsconfig.json")),
                &scratch.0,
            ));
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for (index, engine) in engines.iter().enumerate() {
                let cwd = case.join(format!("{profile}-run-{index}"));
                fs::create_dir(&cwd).unwrap();
                let result = bounded(
                    Command::new(engine)
                        .arg(site.join(manifest["entry"].as_str().unwrap()))
                        .current_dir(&cwd)
                        .env("PATH", ""),
                    &scratch.0,
                );
                assert_eq!(result.status.code(), Some(code), "{result:?}");
                assert_eq!(result.stdout, expected);
                if code != 0 {
                    assert_eq!(
                        fs::read(cwd.join("trace.txt")).unwrap(),
                        b"acquire\nbody: ()\nrelease\n"
                    );
                }
            }
        }
    }
}

#[test]
fn imported_unit_projects_match_native_and_raw_strict_ts_profiles() {
    let scratch = Scratch::new();
    let root = root();
    let cli = Path::new(env!("CARGO_BIN_EXE_seseragi"));
    let fixtures = root.join("examples/spec/fixtures/rust-backend-projects");
    let engines = [runtime("bun", &scratch.0), runtime("node", &scratch.0)];
    for (fixture, entry, code) in [
        ("imported-unit-identity", "main", 0),
        ("imported-unit-reexport", "main", 0),
        ("imported-unit-package", "main", 0),
        ("imported-unit-package", "failed", 1),
        ("imported-unit-package", "crash", 70),
    ] {
        let case = scratch.0.join(format!("{fixture}-{entry}"));
        fs::create_dir(&case).unwrap();
        let package = case.join("package");
        copy_tree(&fixtures.join(fixture), &package);
        if entry != "main" {
            let source = package.join("src/main.ssrg");
            let original = fs::read_to_string(&source).unwrap();
            assert_eq!(original.matches("pub effect fn main =").count(), 1);
            let selected = original.replacen("pub effect fn main =", "effect fn ordinary =", 1);
            fs::write(
                source,
                format!("{selected}\npub effect fn main = {entry} ()\n"),
            )
            .unwrap();
        }
        successful(bounded(
            Command::new(cli).args(["lock", "update"]).arg(&package),
            &scratch.0,
        ));
        let expected = if code == 0 {
            fs::read(package.join("expected.stdout")).unwrap()
        } else {
            vec![]
        };
        let native = case.join("native");
        successful(bounded(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&package)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let cwd = case.join("native-run");
        fs::create_dir(&cwd).unwrap();
        let executed = bounded(
            Command::new(native.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }))
            .current_dir(&cwd)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(executed.status.code(), Some(code), "{executed:?}");
        assert_eq!(executed.stdout, expected);
        assert!(String::from_utf8_lossy(&executed.stderr).contains("managed-live=0"));
        if code != 0 {
            assert_eq!(
                fs::read(cwd.join("trace.txt")).unwrap(),
                b"acquire\nbody: ()\nrelease\n"
            );
        }
        let loaded = seseragi_project::load_local_project(&package).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let site = case.join(profile);
            successful(bounded(
                Command::new(cli)
                    .arg("build")
                    .arg(&package)
                    .args(["--target", "process", "--profile", profile, "--out-dir"])
                    .arg(&site),
                &scratch.0,
            ));
            // Check the original per-module TS for both profiles, independently
            // of whether the normal CLI artifact is bundled.
            let raw = case.join(format!("raw-{profile}"));
            fs::create_dir(&raw).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let file = raw.join(&module.generated.metadata.outputs.typescript);
                assert!(file.starts_with(&raw));
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"skipLibCheck":false,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","allowImportingTsExtensions":true,"lib":["ES2023","DOM","DOM.Iterable"],"types":["node"],"typeRoots":[root.join("node_modules/@types")],"baseUrl":root,"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]}},"files":files});
            fs::write(
                raw.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(bounded(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(raw.join("tsconfig.json")),
                &scratch.0,
            ));
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for (index, engine) in engines.iter().enumerate() {
                let cwd = case.join(format!("{profile}-run-{index}"));
                fs::create_dir(&cwd).unwrap();
                let executed = bounded(
                    Command::new(engine)
                        .arg(site.join(manifest["entry"].as_str().unwrap()))
                        .current_dir(&cwd)
                        .env("PATH", ""),
                    &scratch.0,
                );
                assert_eq!(executed.status.code(), Some(code), "{executed:?}");
                assert_eq!(executed.stdout, expected);
                if code != 0 {
                    assert_eq!(
                        fs::read(cwd.join("trace.txt")).unwrap(),
                        b"acquire\nbody: ()\nrelease\n"
                    );
                }
            }
        }
    }
}

#[test]
fn nominal_unit_is_not_an_anonymous_imported_effect_argument() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_tree(
        &root().join("examples/spec/fixtures/rust-backend-projects/imported-unit-reexport"),
        &package,
    );
    let source = package.join("src/main.ssrg");
    let text = fs::read_to_string(&source).unwrap();
    assert_eq!(text.matches("genuine argument").count(), 1);
    fs::write(&source, text.replace("genuine argument", "genuine owned")).unwrap();
    let cli = Path::new(env!("CARGO_BIN_EXE_seseragi"));
    successful(bounded(
        Command::new(cli).args(["lock", "update"]).arg(&package),
        &scratch.0,
    ));
    let refused = scratch.0.join("refused");
    let result = bounded(
        Command::new(cli)
            .args(["experimental", "build-native"])
            .arg(&package)
            .args(["--entry", "main", "--out-dir"])
            .arg(&refused),
        &scratch.0,
    );
    assert_eq!(result.status.code(), Some(2));
    let diagnostic = String::from_utf8(result.stderr).unwrap();
    assert!(
        diagnostic.contains("main.ssrg") && diagnostic.contains("SES-T"),
        "{diagnostic}"
    );
    assert!(!refused.exists());
}

#[test]
fn facade_generic_renames_keep_cold_ref_closures_and_raw_strict_ts() {
    let scratch = Scratch::new();
    let root = root();
    let cli = Path::new(env!("CARGO_BIN_EXE_seseragi"));
    let fixture = root.join("examples/spec/fixtures/rust-backend-projects/facade-generic-renames");
    let engines = [runtime("bun", &scratch.0), runtime("node", &scratch.0)];
    for (name, box_name, trait_name) in [
        ("control", "Box", "Render"),
        ("type", "ExportedBox", "Render"),
        ("trait", "Box", "ExportedTrait"),
        ("both", "ExportedBox", "ExportedTrait"),
    ] {
        let case = scratch.0.join(name);
        fs::create_dir(&case).unwrap();
        let package = case.join("package");
        copy_tree(&fixture, &package);
        fs::write(
            package.join("src/facade.ssrg"),
            format!("pub import {{Box as {box_name}, Render as {trait_name}, rendered, boxed}} from \"model\"\n"),
        ).unwrap();
        let main = package.join("src/main.ssrg");
        fs::write(
            &main,
            fs::read_to_string(&main)
                .unwrap()
                .replace("ExportedBox as Foreign", &format!("{box_name} as Foreign"))
                .replace(
                    "ExportedTrait as Display",
                    &format!("{trait_name} as Display"),
                ),
        )
        .unwrap();
        successful(bounded(
            Command::new(cli).args(["lock", "update"]).arg(&package),
            &scratch.0,
        ));
        for (entry, oracle) in [("main", "expected.stdout"), ("probe", "probe.stdout")] {
            let native = case.join(format!("native-{entry}"));
            successful(bounded(
                Command::new(cli)
                    .args(["experimental", "build-native"])
                    .arg(&package)
                    .args(["--entry", entry, "--out-dir"])
                    .arg(&native),
                &scratch.0,
            ));
            let cwd = case.join(format!("native-run-{entry}"));
            fs::create_dir(&cwd).unwrap();
            let executed = successful(bounded(
                Command::new(native.join(if cfg!(windows) {
                    "program.exe"
                } else {
                    "program"
                }))
                .current_dir(cwd)
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
                &scratch.0,
            ));
            assert_eq!(
                executed.stdout,
                fs::read(package.join(oracle)).unwrap(),
                "{name}/{entry}"
            );
            assert!(String::from_utf8_lossy(&executed.stderr).contains("managed-live=0"));
        }
        let loaded = seseragi_project::load_local_project(&package).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let site = case.join(profile);
            successful(bounded(
                Command::new(cli)
                    .arg("build")
                    .arg(&package)
                    .args(["--target", "process", "--profile", profile, "--out-dir"])
                    .arg(&site),
                &scratch.0,
            ));
            let raw = case.join(format!("raw-{profile}"));
            fs::create_dir(&raw).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let file = raw.join(&module.generated.metadata.outputs.typescript);
                assert!(file.starts_with(&raw));
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"skipLibCheck":false,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","allowImportingTsExtensions":true,"lib":["ES2023","DOM","DOM.Iterable"],"types":["node"],"typeRoots":[root.join("node_modules/@types")],"baseUrl":root,"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]}},"files":files});
            fs::write(
                raw.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(bounded(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(raw.join("tsconfig.json")),
                &scratch.0,
            ));
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for (index, engine) in engines.iter().enumerate() {
                let cwd = case.join(format!("{profile}-run-{index}"));
                fs::create_dir(&cwd).unwrap();
                let executed = successful(bounded(
                    Command::new(engine)
                        .arg(site.join(manifest["entry"].as_str().unwrap()))
                        .current_dir(cwd)
                        .env("PATH", ""),
                    &scratch.0,
                ));
                assert_eq!(
                    executed.stdout,
                    fs::read(package.join("expected.stdout")).unwrap(),
                    "{name}/{profile}/{index}"
                );
                assert!(executed.stderr.is_empty());
            }
        }
    }
}

#[test]
fn literal_unit_program_effects_replay_and_finish_their_finalizers() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm-unit-values.ssrg");
    let output = scratch.0.join("unit");
    successful(bounded(
        &mut build(&source, &output, &["value", "scoped", "releaseCheck"]),
        &scratch.0,
    ));
    let probe = scratch.0.join("probe.mjs");
    fs::write(&probe, r#"
import assert from "node:assert/strict";
import {readFile} from "node:fs/promises";
import {loadProgram,ProgramWasmClosed} from "./unit/program.mjs";
const program=await loadProgram(await readFile(new URL("./unit/program.wasm",import.meta.url)));
let runs=0;
try {
  for(const [name,expected] of [["value",{tag:"Success",value:undefined}],["scoped",{tag:"Success",value:"雪🙂"}],["releaseCheck",{tag:"Defect",messages:["released"]}]]) {
    const plan=program.exports[name]("雪🙂");
    assert.equal(program.ownership().running,false);
    assert.equal(program.ownership().plans,1);
    for(let replay=0;replay<2;replay++) { assert.deepEqual(await plan.run().result,expected); runs++; }
    plan.dispose();
    assert.throws(()=>plan.run(),ProgramWasmClosed);
    assert.equal(program.ownership().plans,0);
    assert.equal(program.ownership().retired,false);
  }
} finally { await program.dispose(); }
assert.equal(program.ownership().managedLiveAtClose,0);
assert.equal(program.ownership().callbacks,0);
console.log(JSON.stringify({runs,managedLiveAtClose:0}));
"#).unwrap();
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(&probe)
                .env("PATH", ""),
            &scratch.0,
        ));
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["runs"], 6);
        assert_eq!(report["managedLiveAtClose"], 0);
    }
    fs::write(
        scratch.0.join("unit.mts"),
        r#"
import {loadProgram,type OwnedEffect} from "./unit/program.mjs";
const owner=await loadProgram(new Uint8Array());
const plan:OwnedEffect<never,undefined>=owner.exports.value("input");
// @ts-expect-error Unit is not a numeric success
const wrong:OwnedEffect<never,number>=owner.exports.value("input");
const exit=await plan.run().result;
if(exit.tag==="Success") { const value:undefined=exit.value; }
await owner.dispose();
"#,
    )
    .unwrap();
    fs::write(scratch.0.join("unit-tsconfig.json"), r#"{"compilerOptions":{"strict":true,"skipLibCheck":false,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"]},"files":["unit.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("unit-tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn providerless_program_effects_keep_the_same_body_on_ts_and_native() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm-effects.ssrg");
    let original = fs::read_to_string(&source).unwrap();
    let oracle = scratch.0.join("oracle.ssrg");
    fs::write(
        &oracle,
        format!(
            r#"{original}
pub effect fn oracle -> String fails String = do {{
  first <- value "cold"
  second <- rejection "typed" |> effects.recover (\message: String -> effects.succeed message)
  next <- callback "saved:"
  let rendered = next "closure"
  final <- late "scope"
  effects.succeed (first + "|" + second + "|" + rendered + "|" + final)
}}
effect fn unexpected message: String -> String = effects.die message
pub effect fn main = do {{
  answer <- oracle () |> effects.recover unexpected
  println answer
}}
"#
        ),
    )
    .unwrap();
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&oracle)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let binary = native.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let result = successful(bounded(
        Command::new(binary)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        "cold|typed|saved:closure|scope\n"
    );
    assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    // Both ordinary targets use the identical Console runner and calculation
    // bodies. The raw program WASM boundary itself remains providerless.
    for (profile, profile_value) in [
        ("development", seseragi_project::BuildProfile::Development),
        ("release", seseragi_project::BuildProfile::Release),
    ] {
        let directory = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&oracle)
                .args(["--profile", profile, "--out-dir"])
                .arg(&directory),
            &scratch.0,
        ));
        let raw = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "program-wasm-effects.ssrg",
                "proof/effects",
                &original,
            )
            .with_profile(profile_value),
        )
        .unwrap();
        fs::write(directory.join("raw.ts"), raw.generated.typescript).unwrap();
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":["raw.ts"]});
        fs::write(
            directory.join("tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(directory.join("tsconfig.json")),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = directory.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(&entry)
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(
                String::from_utf8(result.stdout).unwrap(),
                "cold|typed|saved:closure|scope\n"
            );
        }
    }
}

#[test]
fn standard_web_effect_build_owns_async_program_types_and_lifetimes() {
    let scratch = Scratch::new();
    let package =
        root().join("examples/spec/fixtures/rust-backend-projects/program-wasm-effect-package");
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            standard_build(&package, &site, profile).current_dir(&scratch.0),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["profile"], profile);
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        assert_eq!(
            manifest["programs"][0]["abi"],
            "seseragi-program-wasm-effect-v1"
        );
        assert_eq!(
            manifest["programs"][0]["hostImport"],
            "@seseragi/programs/calculation"
        );
        let directory = site.join("programs/calculation");
        for engine in ["bun", "node"] {
            for scenario in ["normal", "defect"] {
                let result = successful(bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(root().join("scripts/program-wasm-effect-package-probe.mjs"))
                        .arg(&directory)
                        .arg(scenario)
                        .current_dir(&scratch.0)
                        .env("PATH", ""),
                    &scratch.0,
                ));
                let proof: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
                assert_eq!(proof["roots"], 2);
                assert_eq!(proof["normalManagedLiveAtClose"], 0);
                assert_eq!(proof["cleanupDefect"], scenario == "defect");
                assert_eq!(proof["lateFactoryClosed"], true);
            }
        }
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false,"paths":{"@seseragi/programs/calculation":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let config_path = scratch.0.join(format!("{profile}-types.json"));
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path),
            &scratch.0,
        ));
    }
}

#[test]
fn effect_success_callbacks_return_cold_owned_effects() {
    let scratch = Scratch::new();
    let source =
        root().join("examples/spec/fixtures/rust-backend/program-wasm-effect-factory.ssrg");
    let output = scratch.0.join("factory");
    successful(bounded(
        &mut build(
            &source,
            &output,
            &[
                "create",
                "nested",
                "createChecked",
                "createDefect",
                "cleanupFactory",
                "pendingFactory",
            ],
        ),
        &scratch.0,
    ));
    for name in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(name, &scratch.0))
                .arg(root().join("scripts/program-wasm-effect-factory-probe.mjs"))
                .arg(&output)
                .env("PATH", ""),
            &scratch.0,
        ));
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["factoryReplay"], true);
        assert_eq!(report["detachedCallbacksAndPlans"], true);
        assert_eq!(report["nestedEffect"], true);
        assert_eq!(report["typedFailureAndLanguageDefect"], true);
        assert_eq!(report["maskedLifoCancellationAndDispose"], true);
        assert_eq!(report["normalManagedLiveAtClose"], 0);
        assert_eq!(report["freshInstance"], true);
    }
    fs::write(scratch.0.join("types.mts"), r#"
import {loadProgram, type OwnedEffect, type OwnedCallback} from "./factory/program.mjs";
const owner=await loadProgram(new Uint8Array());
const creation:OwnedEffect<never,OwnedCallback<string,OwnedEffect<never,string>>>=owner.exports.create("seed:");
const first=await creation.run().result;
if(first.tag==="Success") {
  const plan:OwnedEffect<never,string>=first.value("one");
  const value=await plan.run().result;
  if(value.tag==="Success") { const answer:string=value.value; }
  // @ts-expect-error callback arguments retain their checked String codec
  first.value(42);
  // @ts-expect-error a returned Effect remains cold until explicitly run
  const eager:string=first.value("one");
}
const checked:OwnedEffect<never,OwnedCallback<boolean,OwnedEffect<string,string>>>=owner.exports.createChecked("typed");
const nested:OwnedEffect<never,OwnedEffect<never,string>>=owner.exports.nested("nested");
const pending:OwnedEffect<never,OwnedCallback<undefined,OwnedEffect<never,undefined>>>=owner.exports.pendingFactory(undefined);
// @ts-expect-error the factory itself returns an Effect, not an eager callback
const eager:OwnedCallback<string,OwnedEffect<never,string>>=owner.exports.create("seed:");
await owner.dispose();
"#).unwrap();
    fs::write(scratch.0.join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false},"files":["types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
    for (index, source) in [
        "import * as refs from \"std/ref\"\nimport * as effects from \"std/effect\"\nimport { Ref } from \"std/ref\"\npub effect fn selected value: Int -> Ref<Int> = refs.make value",
        "import * as refs from \"std/ref\"\nimport * as effects from \"std/effect\"\nfn identity value: Int -> Int = value\npub effect fn selected value: Int -> Int = do { cell <- refs.make identity; saved <- refs.get cell; effects.succeed (saved value) }",
        "import * as effects from \"std/effect\"\npub effect fn selected callback: (Int -> Int) -> Int = effects.succeed (callback 1)",
        "import * as effects from \"std/effect\"\neffect fn add initial: Int -> delta: Int -> Int = effects.succeed (initial + delta)\npub effect fn selected initial: Int -> (Int -> Effect<{}, Never, Int>) = effects.succeed (add initial)",

    ].iter().enumerate() {
        let file = scratch.0.join(format!("unsupported-{index}.ssrg"));
        let destination = scratch.0.join(format!("unsupported-{index}"));
        fs::write(&file, source).unwrap();
        let result = bounded(build(&file, &destination, &["selected"]).env("PATH", ""), &scratch.0);
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("SES-RUST-0001"), "{error}");
        assert!(!error.contains("could not start rustc"), "{error}");
        if index == 3 {
            assert!(error.contains("arithmetic requires fallible evaluation"), "{error}");
        }
        assert!(!destination.exists());
    }
}

#[test]
fn effect_success_callback_bodies_match_raw_ts_and_native() {
    let scratch = Scratch::new();
    let original = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/program-wasm-effect-factory.ssrg"),
    )
    .unwrap();
    let normal = r#"
effect fn oracle -> String = do {
  let creation = create "seed:"
  first <- creation
  second <- creation
  one <- first "one"
  two <- first "two"
  other <- second "one"
  inner <- nested "nested"
  nestedValue <- inner
  checked <- createChecked "typed failure"
  failure <- checked True |> effects.recover (\message: String -> effects.succeed message)
  effects.succeed (one + "|" + two + "|" + other + "|" + nestedValue + "|" + failure)
}

pub effect fn main = do { answer <- oracle (); println answer }
"#;
    for (label, main, code, expected) in [
        ("normal", normal, 0, "seed:one|seed:two|seed:one|nested|typed failure\n"),
        ("defect", "pub effect fn main = do { next <- createDefect \"factory defect\"; answer <- next (); println answer }\n", 70, ""),
        ("cleanup", "pub effect fn main = do { finish <- cleanupFactory (); finish () }\n", 70, ""),
    ] {
        let source = scratch.0.join(format!("{label}.ssrg"));
        let text = format!("{original}\n{main}");
        fs::write(&source, &text).unwrap();
        let native = scratch.0.join(format!("{label}-native"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let result = bounded(
            Command::new(native.join(if cfg!(windows) { "program.exe" } else { "program" }))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{label}: {result:?}");
        assert_eq!(result.stdout, expected.as_bytes());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("managed-live=0"), "{error}");
        if label == "defect" {
            assert!(error.contains("factory defect"), "{error}");
        }
        if label == "cleanup" {
            assert_eq!(error.matches("inner released").count(), 1, "{error}");
            assert_eq!(error.matches("outer released").count(), 1, "{error}");
        }
        for (profile, value) in [
            ("development", seseragi_project::BuildProfile::Development),
            ("release", seseragi_project::BuildProfile::Release),
        ] {
            let output = scratch.0.join(format!("{label}-{profile}"));
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&source)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            ));
            let raw = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new("factory.ssrg", "proof/factory", &text)
                    .with_profile(value),
            ).unwrap();
            fs::write(output.join("raw.ts"), raw.generated.typescript).unwrap();
            let mut files = vec![output.join("raw.ts")];
            if profile == "development" {
                files.push(output.join("entry.ts"));
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":files});
            fs::write(output.join("tsconfig.json"), serde_json::to_vec(&config).unwrap()).unwrap();
            successful(bounded(
                Command::new(runtime("node", &scratch.0))
                    .arg(root().join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(output.join("tsconfig.json")),
                &scratch.0,
            ));
            let raw_runner = if label == "cleanup" {
                    // Process entry deliberately keeps Defects opaque. Inspect
                    // the same original raw TypeScript via its public runtime
                    // outcome instead, preserving both ordered cleanup errors.
                    fs::write(output.join("raw-check.mjs"), r##"
import assert from "node:assert/strict";
import { cleanupFactory } from "./raw.ts";
import { run } from "@seseragi/runtime/effect";
const creation=await run(cleanupFactory(undefined),{});
assert.equal(creation.kind,"success");
await assert.rejects(run(creation.value(undefined),{}),(error)=>{
  assert.equal(error.message,"inner released");
  assert.deepEqual(error.suppressed.map(value=>value.message),["outer released"]);
  return true;
});
console.log("raw cleanup checked");
"##).unwrap();
                    successful(bounded(
                        Command::new(runtime("bun", &scratch.0))
                            .arg("build")
                            .arg(output.join("raw-check.mjs"))
                            .args(["--target", "node", "--outfile"])
                            .arg(output.join("raw-check.js")),
                        &scratch.0,
                    ));

                Some(output.join("raw-check.js"))
            } else {
                None
            };
            for engine in ["bun", "node"] {
                let result = bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(output.join("entry.js"))
                        .env("PATH", ""),
                    &scratch.0,
                );
                assert_eq!(result.status.code(), Some(code), "{label}/{profile}: {result:?}");
                assert_eq!(result.stdout, expected.as_bytes());
                let error = String::from_utf8_lossy(&result.stderr);
                if code == 0 {
                    assert!(error.is_empty(), "{error}");
                } else {
                    assert!(error.contains("seseragi: runtime defect"), "{error}");
                }
                if let Some(runner) = &raw_runner {
                    let raw_result = successful(bounded(
                        Command::new(runtime(engine, &scratch.0))
                            .arg(runner)
                            .env("PATH", ""),
                        &scratch.0,
                    ));
                    assert_eq!(raw_result.stdout, b"raw cleanup checked\n");
                    assert!(raw_result.stderr.is_empty(), "{raw_result:?}");
                }
            }
        }
    }
}

#[test]
fn effect_success_factory_standard_web_package_keeps_original_bridge_types() {
    let scratch = Scratch::new();
    let package = root()
        .join("examples/spec/fixtures/rust-backend-projects/program-wasm-effect-factory-package");
    assert_eq!(
        fs::read(package.join("src/calculation.ssrg")).unwrap(),
        fs::read(
            root().join("examples/spec/fixtures/rust-backend/program-wasm-effect-factory.ssrg")
        )
        .unwrap(),
    );
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            standard_build(&package, &site, profile).current_dir(&scratch.0),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["profile"], profile);
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        assert_eq!(
            manifest["programs"][0]["abi"],
            "seseragi-program-wasm-effect-v1"
        );
        assert_eq!(
            manifest["programs"][0]["hostImport"],
            "@seseragi/programs/calculation"
        );
        let directory = site.join("programs/calculation");
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(root().join("scripts/program-wasm-effect-factory-probe.mjs"))
                    .arg(&directory)
                    .arg("package")
                    .env("PATH", ""),
                &scratch.0,
            ));
            let proof: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(proof["factoryReplay"], true);
            assert_eq!(proof["detachedCallbacksAndPlans"], true);
            assert_eq!(proof["maskedLifoCancellationAndDispose"], true);
            assert_eq!(proof["normalManagedLiveAtClose"], 0);
        }
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false,"paths":{"@seseragi/programs/calculation":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let config_path = scratch.0.join(format!("{profile}-types.json"));
        fs::write(&config_path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config_path),
            &scratch.0,
        ));
    }
}

#[test]
fn private_ref_effect_callbacks_retain_state_and_release_the_instance() {
    let scratch = Scratch::new();
    let source = root().join("examples/spec/fixtures/rust-backend/program-wasm-private-ref.ssrg");
    let destination = scratch.0.join("cells");
    successful(bounded(
        &mut build(
            &source,
            &destination,
            &["create", "reader", "unit", "failing", "snapshot", "session"],
        ),
        &scratch.0,
    ));
    for engine in ["bun", "node"] {
        let result = successful(bounded(
            Command::new(runtime(engine, &scratch.0))
                .arg(root().join("scripts/program-wasm-private-ref-probe.mjs"))
                .arg(&destination)
                .env("PATH", ""),
            &scratch.0,
        ));
        assert_eq!(result.stdout, b"private cells: cold independent state, retained plans, Failure/Defect/cancel, LIFO cleanup and roots released\n");
        assert!(result.stderr.is_empty(), "{result:?}");
    }
    fs::write(scratch.0.join("types.mts"), r#"
import {loadProgram, type OwnedEffect, type OwnedCallback} from "./cells/program.mjs";
const owner=await loadProgram(new Uint8Array());
const creation:OwnedEffect<never,OwnedCallback<string,OwnedEffect<never,string>>>=owner.exports.create("seed:");
const session:OwnedEffect<never,OwnedCallback<string,OwnedCallback<boolean,OwnedCallback<boolean,OwnedCallback<boolean,OwnedEffect<string,string>>>>>>=owner.exports.session("state");
// @ts-expect-error private Ref does not become a host argument
owner.exports.create({value:"seed:"});
// @ts-expect-error state is owned by the returned callback and remains cold
const eager:string=owner.exports.create("seed:");
const value:OwnedEffect<never,number>=owner.exports.snapshot(42);
await owner.dispose();
"#).unwrap();
    fs::write(scratch.0.join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"NodeNext","moduleResolution":"NodeNext","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false},"files":["types.mts"]}"#).unwrap();
    successful(bounded(
        Command::new(runtime("node", &scratch.0))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(scratch.0.join("tsconfig.json")),
        &scratch.0,
    ));
}

#[test]
fn private_ref_bodies_match_raw_ts_and_native() {
    let scratch = Scratch::new();
    let original = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/program-wasm-private-ref.ssrg"),
    )
    .unwrap();
    let normal = r#"
effect fn oracle -> String fails String = do {
  let creation = create "seed:"
  first <- creation
  second <- creation
  one <- first "one"
  two <- first "two"
  other <- second "one"
  state <- session "state"
  initial <- state ":ok" False False False
  failed <- state ":bad" True False False |> effects.recover (\message: String -> effects.succeed message)
  after <- state "" False False False
  read <- reader "雪🙂"
  text <- read ()
  unit ()
  effects.succeed (one + "|" + two + "|" + other + "|" + initial + "|" + failed + "|" + after + "|" + text)
}
effect fn unexpected message: String -> String = effects.die message
pub effect fn main = do {
  answer <- oracle () |> effects.recover unexpected
  println answer
  number <- snapshot 42
  println number
}
"#;
    let defect = r#"
effect fn oracle -> String fails String = do {
  state <- session "state"
  state ":die" False True False
}
effect fn unexpected message: String -> String = effects.die message
pub effect fn main = do { answer <- oracle () |> effects.recover unexpected; println answer }
"#;

    for (label, main, code, expected) in [
        ("normal", normal, 0, "seed:one|seed:onetwo|seed:one|state:ok|typed failure|state:ok:inner:outer:bad:inner:outer|雪🙂\n42\n"),
        ("defect", defect, 70, ""),
    ] {
        let source = scratch.0.join(format!("{label}.ssrg"));
        let text = format!("{original}\n{main}");
        fs::write(&source, &text).unwrap();
        let native = scratch.0.join(format!("{label}-native"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let result = bounded(
            Command::new(native.join(if cfg!(windows) { "program.exe" } else { "program" }))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{label}: {result:?}");
        assert_eq!(result.stdout, expected.as_bytes());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(error.contains("managed-live=0"), "{error}");
        if label == "defect" { assert!(error.contains("explicit defect"), "{error}"); }
        for (profile, value) in [("development", seseragi_project::BuildProfile::Development), ("release", seseragi_project::BuildProfile::Release)] {
            let output = scratch.0.join(format!("{label}-{profile}"));
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi")).arg("build").arg(&source)
                    .args(["--profile", profile, "--out-dir"]).arg(&output),
                &scratch.0,
            ));
            let raw = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new("private-ref.ssrg", "proof/private-ref", &text).with_profile(value),
            ).unwrap();
            fs::write(output.join("raw.ts"), raw.generated.typescript).unwrap();
            let mut files=vec![output.join("raw.ts")];
            if profile == "development" { files.push(output.join("entry.ts")); }
            let config=serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":files});
            fs::write(output.join("tsconfig.json"),serde_json::to_vec(&config).unwrap()).unwrap();
            successful(bounded(Command::new(runtime("node", &scratch.0)).arg(root().join("node_modules/typescript/bin/tsc")).arg("--project").arg(output.join("tsconfig.json")),&scratch.0));
            for engine in ["bun", "node"] {
                let result=bounded(Command::new(runtime(engine,&scratch.0)).arg(output.join("entry.js")).env("PATH",""),&scratch.0);
                assert_eq!(result.status.code(),Some(code),"{label}/{profile}: {result:?}");
                assert_eq!(result.stdout,expected.as_bytes());
                let error=String::from_utf8_lossy(&result.stderr);
                if code==0 { assert!(error.is_empty(),"{error}"); }
                else { assert!(error.contains("seseragi: runtime defect"),"{error}"); }
            }
        }
    }
}

#[test]
fn private_ref_standard_web_package_keeps_the_typed_ui_bridge() {
    let scratch = Scratch::new();
    let package = root()
        .join("examples/spec/fixtures/rust-backend-projects/program-wasm-private-ref-package");
    assert_eq!(
        fs::read(package.join("src/calculation.ssrg")).unwrap(),
        fs::read(root().join("examples/spec/fixtures/rust-backend/program-wasm-private-ref.ssrg"))
            .unwrap()
    );
    for profile in ["development", "release"] {
        let site = scratch.0.join(profile);
        successful(bounded(
            standard_build(&package, &site, profile).current_dir(&scratch.0),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["profile"], profile);
        assert_eq!(manifest["programs"].as_array().unwrap().len(), 1);
        assert_eq!(
            manifest["programs"][0]["abi"],
            "seseragi-program-wasm-effect-v1"
        );
        assert_eq!(
            manifest["programs"][0]["hostImport"],
            "@seseragi/programs/calculation"
        );
        let directory = site.join("programs/calculation");
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(root().join("scripts/program-wasm-private-ref-probe.mjs"))
                    .arg(&directory)
                    .arg("package")
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(result.stdout,b"private cells: cold independent state, retained plans, Failure/Defect/cancel, LIFO cleanup and roots released\n");
            assert!(result.stderr.is_empty(), "{result:?}");
        }
        let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":[],"lib":["ES2022","DOM"],"skipLibCheck":false,"paths":{"@seseragi/programs/calculation":[directory.join("index.d.mts")]}},"files":[package.join("host/bridge.ts")]});
        let path = scratch.0.join(format!("{profile}-types.json"));
        fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(&path),
            &scratch.0,
        ));
    }
}
