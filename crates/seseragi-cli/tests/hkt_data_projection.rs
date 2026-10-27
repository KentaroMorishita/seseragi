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
            "seseragi-hkt-data-projection-{}-{}",
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

fn strict(directory: &Path, files: Vec<PathBuf>, scratch: &Path) {
    let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"],"allowImportingTsExtensions":true},"files":files});
    fs::write(
        directory.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("-p")
            .arg(directory.join("tsconfig.json")),
        scratch,
    ));
}

fn typescript_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(typescript_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "ts") {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn hkt_data_aliases_and_fields_keep_raw_strict_types_and_the_same_ts_native_results() {
    use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let scratch = Scratch::new();
    let fixture = root().join("examples/spec/fixtures/rust-backend-projects/hkt-data-projection");
    let expected = fs::read(fixture.join("expected.stdout")).unwrap();
    for (profile, value) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let mut graph = ModuleGraph::new();
        for (name, edges) in [
            ("model", vec![]),
            (
                "facade",
                vec![("./model".to_owned(), "fixture/data::model".to_owned())],
            ),
            (
                "main",
                vec![("./facade".to_owned(), "fixture/data::facade".to_owned())],
            ),
        ] {
            graph
                .add_module(format!("fixture/data::{name}"), edges)
                .unwrap();
        }
        let inputs = ["model", "facade", "main"].map(|name| {
            ProjectModuleInput::new(
                format!("{name}.ssrg"),
                format!("fixture/data::{name}"),
                fs::read_to_string(fixture.join(format!("src/{name}.ssrg"))).unwrap(),
                format!("dist/{name}.js"),
            )
            .with_profile(value)
        });
        let raw = scratch.0.join(format!("raw-{profile}"));
        fs::create_dir(&raw).unwrap();
        let compiled = compile_project(graph, inputs).unwrap();
        for (id, module) in &compiled.modules {
            fs::write(
                raw.join(format!("{}.ts", id.rsplit("::").next().unwrap())),
                &module.generated.typescript,
            )
            .unwrap();
        }
        fs::write(raw.join("consumer.ts"), r#"
import { Parcel, Pack, StateT } from "./facade.js";
import type { Sealed, $ssrg$kind$Plain } from "./model.js";
type Maybe<A> = { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A };
interface MaybeKind { readonly args: readonly [unknown]; readonly type: Maybe<this["args"][0]> }
const parcel: Pack<MaybeKind, number> = Parcel<MaybeKind, number>({ tag: "Just", value: 42 });
if (parcel.value.tag === "Just") { const number: number = parcel.value.value; }
declare const original: Sealed<MaybeKind, number>;
const alias: Sealed<$ssrg$kind$Plain, number> = original;
const transparent: Sealed<MaybeKind, number> = alias;
const state: StateT<string, MaybeKind, number> = (input) => ({ tag: "Just", value: [42, input] });
// @ts-expect-error A wrong payload must remain a TypeScript error.
const wrong: Pack<MaybeKind, number> = { tag: "Box", value: { tag: "Just", value: "wrong" } };
// @ts-expect-error A value kind does not satisfy a unary constructor binder.
type WrongKind = Pack<number, number>;
// @ts-expect-error Two-argument kinds do not satisfy a unary constructor binder.
type WrongArity = Pack<{ readonly args: readonly [unknown, unknown]; readonly type: never }, number>;
// @ts-expect-error The HKT alias keeps the state and success payload.
const wrongState: StateT<string, MaybeKind, number> = (input) => ({ tag: "Just", value: [false, input] });
"#).unwrap();
        strict(&raw, typescript_files(&raw), &scratch.0);
        let product = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&fixture)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&product),
            &scratch.0,
        ));
        // Release distribution is bundled JavaScript; its original compiler
        // TypeScript and negative consumer were checked above before bundling.
        if profile == "development" {
            strict(
                &product,
                typescript_files(&product.join("dist")),
                &scratch.0,
            );
        }
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(product.join("entry.js"))
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{profile}/{engine}: {output:?}");
        }
    }
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&fixture)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let output = successful(bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    ));
    assert_eq!(output.stdout, expected);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
        "{output:?}"
    );
}

#[test]
fn type_only_hkt_alias_imports_keep_dependency_initialization_defects() {
    let scratch = Scratch::new();
    let fixture = root().join("examples/spec/fixtures/rust-backend-projects/hkt-data-projection");
    let project = scratch.0.join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::copy(fixture.join("seseragi.toml"), project.join("seseragi.toml")).unwrap();
    let mut model = fs::read_to_string(fixture.join("src/model.ssrg")).unwrap();
    model.push_str("\nlet privateDefect: Int = 9007199254740991 + 1\n");
    fs::write(project.join("src/model.ssrg"), model).unwrap();
    fs::copy(
        fixture.join("src/facade.ssrg"),
        project.join("src/facade.ssrg"),
    )
    .unwrap();
    fs::write(
        project.join("src/main.ssrg"),
        "import { Pack } from \"./facade\"\npub fn keep value: Pack<Maybe, Int> -> Pack<Maybe, Int> = value\npub effect fn main = do { println \"unreachable\" }\n",
    )
    .unwrap();
    let cli = env!("CARGO_BIN_EXE_seseragi");
    successful(bounded(
        Command::new(cli).args(["lock", "update"]).arg(&project),
        &scratch.0,
    ));
    for profile in ["development", "release"] {
        let product = scratch.0.join(profile);
        successful(bounded(
            Command::new(cli)
                .arg("build")
                .arg(&project)
                .args(["--profile", profile, "--out-dir"])
                .arg(&product),
            &scratch.0,
        ));
        for engine in ["bun", "node"] {
            let output = bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(product.join("entry.js"))
                    .env("PATH", ""),
                &scratch.0,
            );
            assert_eq!(
                output.status.code(),
                Some(70),
                "{profile}/{engine}: {output:?}"
            );
            assert!(output.stdout.is_empty(), "{profile}/{engine}: {output:?}");
        }
    }
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(cli)
            .args(["experimental", "build-native"])
            .arg(&project)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let output = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        &scratch.0,
    );
    assert_eq!(output.status.code(), Some(70), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}
