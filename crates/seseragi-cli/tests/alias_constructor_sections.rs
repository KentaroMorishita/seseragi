use seseragi_driver::{BuildProfile, CompileInput};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn strict(files: &[PathBuf], directory: &Path, scratch: &Path) {
    let config = serde_json::json!({"compilerOptions": {
        "strict": true, "noEmit": true, "skipLibCheck": false,
        "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
        "types": ["bun", "node"], "typeRoots": [root().join("node_modules/@types")],
        "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
        "lib": ["ES2023", "DOM", "DOM.Iterable"], "allowImportingTsExtensions": true
    }, "files": files});
    fs::write(
        directory.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(directory.join("tsconfig.json")),
        scratch,
    ));
}

fn check_program(source: &str, expected: &str, code: i32) {
    check_case(
        source,
        expected,
        code,
        None,
        (code == 70).then_some("body defect"),
    );
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

fn check_case(
    source: &str,
    expected: &str,
    code: i32,
    project: Option<&Path>,
    defect: Option<&str>,
) {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-alias-section-")
        .tempdir()
        .unwrap();
    let directory = scratch.path();
    let source_path = if let Some(project) = project {
        let copied = directory.join("project");
        copy_project(project, &copied);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["lock", "update"])
                .arg(&copied),
            directory,
        ));
        copied
    } else {
        let path = directory.join("main.ssrg");
        fs::write(&path, source).unwrap();
        path
    };
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source_path)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        directory,
    ));
    let result = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .current_dir(directory)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        directory,
    );
    assert_eq!(result.status.code(), Some(code), "{result:?}");
    assert_eq!(result.stdout, expected.as_bytes());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("managed-live=0"), "{stderr}");
    if let Some(defect) = defect {
        assert!(stderr.contains(defect), "{stderr}");
    }
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let output = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source_path)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&output),
            directory,
        ));
        let raw = if project.is_some() {
            let loaded = seseragi_project::load_local_project(&source_path).unwrap();
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                .unwrap()
                .compiled
                .modules
                .into_values()
                .collect::<Vec<_>>()
        } else {
            vec![seseragi_driver::compile_module(
                CompileInput::new("main.ssrg", "proof/alias-sections", source)
                    .with_profile(profile),
            )
            .unwrap()]
        };
        // Check original driver bytes, without rewriting operators, dictionaries,
        // annotations, or the already-built process entry.
        let mut files = vec![];
        for module in raw {
            let relative = Path::new(&module.generated.metadata.outputs.typescript);
            assert!(
                !relative.is_absolute()
                    && relative
                        .components()
                        .all(|component| matches!(component, std::path::Component::Normal(_)))
            );
            let path = output.join("raw").join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, module.generated.typescript).unwrap();
            files.push(path);
        }
        if label == "development" {
            files.push(output.join("entry.ts"));
        }
        strict(&files, &output, directory);
        for engine in ["bun", "node"] {
            let result = bounded(
                Command::new(runtime(engine, directory))
                    .arg(output.join("entry.js"))
                    .current_dir(&output)
                    .env("PATH", ""),
                directory,
            );
            assert_eq!(
                result.status.code(),
                Some(code),
                "{label}/{engine}: {result:?}"
            );
            assert_eq!(result.stdout, expected.as_bytes(), "{label}/{engine}");
            if code == 0 {
                assert!(result.stderr.is_empty(), "{result:?}");
            } else {
                assert_eq!(result.stderr, b"seseragi: runtime defect\n");
            }
        }
    }
}

#[test]
fn fixed_alias_sections_execute_the_same_checked_contract_on_all_targets() {
    check_program(
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/alias-constructor-sections.ssrg"
        ),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/alias-constructor-sections.stdout"
        ),
        0,
    );
    check_program(
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-section-evaluation.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/alias-section-evaluation.stdout"
        ),
        0,
    );
}

#[test]
fn opaque_alias_sections_keep_imported_owner_and_saved_source_arity() {
    let fixture =
        root().join("examples/spec/fixtures/rust-backend-projects/alias-constructor-sections");
    check_case("", include_str!("../../../examples/spec/fixtures/rust-backend-projects/alias-constructor-sections/expected.stdout"), 0, Some(&fixture), None);
}

#[test]
fn alias_sections_keep_child_evidence_cold_replay_and_cleanup_on_failure_or_defect() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-section-effects.ssrg");
    assert_eq!(source.matches("body \"ready\" released").count(), 1);
    check_program(
        source,
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-section-effects.stdout"),
        0,
    );
    check_program(
        &source.replace("body \"ready\" released", "body \"fail\" released"),
        "acquired\ncreated\ncold=10\nreleased\nFailure: body failure\nreleased: 1\n",
        0,
    );
    check_program(
        &source.replace("body \"ready\" released", "body \"defect\" released"),
        "acquired\ncreated\ncold=10\nreleased\n",
        70,
    );
}

#[test]
fn alias_section_selection_prefix_and_payload_evaluate_once_in_source_order() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-alias-section-trace-")
        .tempdir()
        .unwrap();
    let directory = scratch.path();
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/alias-section-evaluation.ssrg");
    let events = [
        "selected",
        "prefix",
        "payload",
        "bodyValue",
        "payload",
        "bodyValue",
    ];
    let core = seseragi_driver::compile_core_module(CompileInput::new(
        "trace.ssrg",
        "proof/alias-section-trace",
        source,
    ))
    .unwrap()
    .core_ir;
    let id = |name: &str| {
        format!(
            "v_{}",
            format!("proof/alias-section-trace::{name}")
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        )
    };
    let mut rust = seseragi_backend_rust::emit_rust_module(&core, "probe")
        .unwrap()
        .source;
    rust.truncate(rust.rfind("fn main()").unwrap());
    // Only source helper entries are observed. Selected instance bodies,
    // argument staging, and dictionary application remain original output.
    for name in ["selected", "prefix", "payload", "bodyValue"] {
        let start = rust.find(&format!("fn {}_direct(", id(name))).unwrap();
        let body = start + rust[start..].find("-> V {").unwrap() + "-> V {".len();
        rust.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({name:?})); "),
        );
    }
    rust.push_str(&format!("\nthread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}\nfn main() {{ let result = {}(vec![]).apply(V::Unit); assert_eq!(result.render(), \"saved=42|saved=42\"); TRACE.with(|trace| assert_eq!(&*trace.borrow(), &{events:?})); drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0); }}", id("probe")));
    let file = directory.join("trace.rs");
    let binary = directory.join(format!("trace{}", std::env::consts::EXE_SUFFIX));
    fs::write(&file, rust).unwrap();
    successful(bounded(
        Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--edition=2021", "-O", "-o"])
            .arg(&binary)
            .arg(&file),
        directory,
    ));
    successful(bounded(Command::new(&binary).env("PATH", ""), directory));
    let expected = serde_json::json!({"value":"saved=42|saved=42", "trace":events});
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let output = directory.join(label);
        fs::create_dir(&output).unwrap();
        let original = seseragi_driver::compile_module(
            CompileInput::new("trace.ssrg", "proof/alias-section-trace", source)
                .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(output.join("original.ts"), &original).unwrap();
        let mut traced = original;
        for name in ["selected", "prefix", "payload", "bodyValue"] {
            let start = traced.find(&format!("const {name} =")).unwrap();
            let body = start + traced[start..].find("=>").unwrap() + 2;
            let line_end = body + traced[body..].find('\n').unwrap();
            let end = if traced[..line_end].ends_with(';') {
                line_end - 1
            } else {
                line_end
            };
            assert!(!traced[body..end].trim_start().starts_with('{'));
            traced.insert_str(end, ")");
            traced.insert_str(body, &format!(" ($trace.push({name:?}), "));
        }
        traced.push_str("\nconst $trace: string[] = [];\nconsole.log(JSON.stringify({value: probe(undefined), trace: $trace}));\n");
        fs::write(output.join("traced.ts"), traced).unwrap();
        strict(
            &[output.join("original.ts"), output.join("traced.ts")],
            &output,
            directory,
        );
        seseragi_runtime::stage_typescript_package(&output).unwrap();
        successful(bounded(
            Command::new(runtime("bun", directory))
                .current_dir(&output)
                .args([
                    "build",
                    "traced.ts",
                    "--target=node",
                    "--outfile=traced.mjs",
                ]),
            directory,
        ));
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(output.join("traced.mjs"))
                    .current_dir(&output)
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
                expected,
                "{label}/{engine}"
            );
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
}
