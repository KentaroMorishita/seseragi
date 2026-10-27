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
    if let Some(evidence) = std::env::var_os("SESERAGI_HKT_PROJECTION_EVIDENCE") {
        let capture = PathBuf::from(evidence)
            .join(scratch.file_name().unwrap())
            .join(directory.strip_prefix(scratch).unwrap());
        fs::create_dir_all(&capture).unwrap();
        let captured = files
            .iter()
            .map(|file| {
                let destination = capture.join(file.strip_prefix(directory).unwrap());
                fs::create_dir_all(destination.parent().unwrap()).unwrap();
                fs::copy(file, &destination).unwrap();
                destination
            })
            .collect::<Vec<_>>();
        let mut captured_config = config.clone();
        captured_config["files"] = serde_json::json!(captured);
        fs::write(
            capture.join("tsconfig.json"),
            serde_json::to_vec(&captured_config).unwrap(),
        )
        .unwrap();
    }
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

fn check_project(fixture_name: &str, prefix: &str, consumer: &str) {
    use seseragi_driver::{compile_project, BuildProfile, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let scratch = Scratch::new();
    let fixture = root()
        .join("examples/spec/fixtures/rust-backend-projects")
        .join(fixture_name);
    let expected = fs::read(fixture.join("expected.stdout")).unwrap();
    let project = scratch.0.join("project");
    copy_project(&fixture, &project);
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&project),
        &scratch.0,
    ));
    for (profile, value) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let mut graph = ModuleGraph::new();
        for (name, edges) in [
            ("model", vec![]),
            (
                "facade",
                vec![("./model".to_owned(), format!("{prefix}::model"))],
            ),
            (
                "main",
                vec![("./facade".to_owned(), format!("{prefix}::facade"))],
            ),
        ] {
            graph
                .add_module(format!("{prefix}::{name}"), edges)
                .unwrap();
        }
        let inputs = ["model", "facade", "main"].map(|name| {
            ProjectModuleInput::new(
                format!("{name}.ssrg"),
                format!("{prefix}::{name}"),
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
        fs::write(raw.join("consumer.ts"), consumer).unwrap();
        strict(&raw, typescript_files(&raw), &scratch.0);
        let product = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&project)
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
            .arg(&project)
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
fn hkt_data_aliases_and_fields_keep_raw_strict_types_and_the_same_ts_native_results() {
    check_project(
        "hkt-data-projection",
        "fixture/data",
        r#"
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
"#,
    );
}

#[test]
fn opaque_hkt_alias_factories_keep_real_values_and_private_brands_across_facades() {
    check_project(
        "hkt-opaque-alias-factory",
        "fixture/opaque-hkt",
        r#"
import { create, reveal, scoped, constrained, type PublicWrapped, type PublicChain } from "./facade.js";
import { intShow } from "@seseragi/runtime/show";
type Maybe<A> = { readonly tag: "Nothing" } | { readonly tag: "Just"; readonly value: A };
interface MaybeKind { readonly args: readonly [unknown]; readonly type: Maybe<this["args"][0]> }
const factory: (value: Maybe<number>) => PublicChain<MaybeKind, number> = create<MaybeKind, number>;
const hidden: PublicChain<MaybeKind, number> = factory({ tag: "Just", value: 42 });
const kept: PublicWrapped<MaybeKind, number> = hidden;
const actual: Maybe<number> = reveal<MaybeKind, number>(kept);
const throughScoped: Maybe<number> = scoped<MaybeKind, number>(actual);
const withEvidence: Maybe<number> = constrained<MaybeKind, number>(actual)(intShow);
// @ts-expect-error A checked factory input keeps the constructor's payload type.
create<MaybeKind, number>({ tag: "Just", value: "wrong" });
// @ts-expect-error The factory's actual value does not expose its private field.
hidden.value;
// @ts-expect-error A fabricated object lacks the canonical opaque brand.
const forged: PublicChain<MaybeKind, number> = {};
// @ts-expect-error The private brand retains the alias's success payload.
const wrong: PublicChain<MaybeKind, string> = hidden;
// @ts-expect-error A value kind does not satisfy the alias's constructor binder.
type WrongKind = PublicChain<number, number>;
"#,
    );
}

fn check_original_source(source: &str, expected: &[u8]) {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = Scratch::new();
    let path = scratch.0.join("main.ssrg");
    // The text fixtures retain the exact Issue/formal-release source bytes,
    // including historical formatting. Feed those bytes to the compiler.
    fs::write(&path, source).unwrap();
    for (profile, value) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let raw = scratch.0.join(format!("raw-{profile}"));
        fs::create_dir(&raw).unwrap();
        let compiled = compile_module(
            CompileInput::new("main.ssrg", "fixture/hkt-original", source).with_profile(value),
        )
        .unwrap();
        fs::write(raw.join("main.ts"), compiled.generated.typescript).unwrap();
        strict(&raw, vec![raw.join("main.ts")], &scratch.0);
        let product = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&path)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&product),
            &scratch.0,
        ));
        if profile == "development" {
            strict(
                &product,
                vec![product.join("entry.ts"), product.join("main.ts")],
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
            .arg(&path)
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
fn issue894_original_pattern_source_is_raw_strict_and_runs_as_just42() {
    check_original_source(
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/hkt-adt-pattern-issue894.ssrg.txt"
        ),
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/hkt-adt-pattern-issue894.stdout"
        ),
    );
}

#[test]
fn issue908_formal63_original_saved_method_is_raw_strict_and_runs_as_true() {
    check_original_source(
        include_str!("../../../examples/spec/fixtures/rust-backend/hkt-holder-issue908.ssrg.txt"),
        include_bytes!("../../../examples/spec/fixtures/rust-backend/hkt-holder-issue908.stdout"),
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
    let loaded = seseragi_project::load_local_project(&project).unwrap();
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let raw = scratch.0.join(format!("raw-{profile}"));
        fs::create_dir(&raw).unwrap();
        let compiled =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
        for module in compiled.compiled.modules.values() {
            let relative = Path::new(&module.generated.metadata.outputs.typescript);
            assert!(
                !relative.is_absolute()
                    && relative
                        .components()
                        .all(|part| matches!(part, std::path::Component::Normal(_)))
            );
            let path = raw.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &module.generated.typescript).unwrap();
        }
        strict(&raw, typescript_files(&raw), &scratch.0);
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
