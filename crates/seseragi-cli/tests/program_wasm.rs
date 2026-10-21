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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
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
    assert_eq!(metadata["sources"].as_array().unwrap().len(), 2);
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
            .contains("pure program WASM ABI does not accept provider configuration"),
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
        "import * as effects from \"std/effect\"\npub fn selected value: Int -> Effect<{}, Never, Int> = effects.succeed value",
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
