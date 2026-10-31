use std::path::Path;
use std::process::Command;

#[test]
fn executes_spec_let_polymorphism_and_preserves_dictionaries() {
    for name in [
        "let-polymorphism",
        "let-polymorphism-constraints",
        "constrained-callable-alias",
        "constrained-operator-alias",
    ] {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../examples/spec/fixtures/compile/{name}.ssrg"));
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            std::fs::read_to_string(fixture.with_extension("stdout")).unwrap()
        );
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }
}

#[path = "support/generic_impl_process.rs"]
mod process;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn check_output(output: std::process::Output, expected: &[u8]) {
    let output = process::successful(output);
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty(), "{output:?}");
}

fn check_generated_typescript(
    input: &Path,
    profile: seseragi_driver::BuildProfile,
    directory: &Path,
    node: &Path,
) {
    let bundles = if input.is_dir() {
        let loaded = seseragi_project::load_local_project(input).unwrap();
        seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
            .unwrap()
            .compiled
            .modules
            .into_values()
            .map(|module| module.generated)
            .collect::<Vec<_>>()
    } else {
        vec![
            seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    input.to_str().unwrap(),
                    "fixture/polymorphic-globals",
                    &std::fs::read_to_string(input).unwrap(),
                )
                .with_profile(profile),
            )
            .unwrap()
            .generated,
        ]
    };
    let mut files = Vec::new();
    for bundle in bundles {
        let file = directory.join(bundle.metadata.outputs.typescript);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, bundle.typescript).unwrap();
        files.push(file);
    }
    let root = root();
    let config = directory.join("tsconfig.json");
    std::fs::write(
        &config,
        serde_json::json!({
            "compilerOptions": {
                "strict": true, "skipLibCheck": false, "noEmit": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun", "node"],
                "typeRoots": [root.join("node_modules/@types")],
                "lib": ["ES2023", "DOM", "DOM.Iterable"],
                "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
            }, "files": files
        })
        .to_string(),
    )
    .unwrap();
    process::successful(process::bounded(
        Command::new(node)
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        directory,
    ));
}

#[test]
fn polymorphic_globals_build_typecheck_and_run_at_several_types() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-polymorphic-globals-")
        .tempdir()
        .unwrap();
    let engines = ["bun", "node"].map(|name| process::runtime(name, scratch.path()));
    for name in [
        "polymorphic-global-absent",
        "polymorphic-global-values",
        "polymorphic-global-functions",
        "package",
    ] {
        let case = scratch.path().join(name);
        std::fs::create_dir(&case).unwrap();
        let (input, expected) = if name == "package" {
            let fixture =
                root().join("examples/spec/fixtures/ts-backend-projects/polymorphic-globals");
            let project = case.join("project");
            std::fs::create_dir_all(project.join("src")).unwrap();
            std::fs::copy(fixture.join("seseragi.toml"), project.join("seseragi.toml")).unwrap();
            for file in ["main.ssrg", "values.ssrg", "facade.ssrg"] {
                std::fs::copy(
                    fixture.join("src").join(file),
                    project.join("src").join(file),
                )
                .unwrap();
            }
            process::successful(process::bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .args(["lock", "update"])
                    .arg(&project),
                &case,
            ));
            (
                project,
                std::fs::read(fixture.join("expected.stdout")).unwrap(),
            )
        } else {
            let source = root().join(format!("examples/spec/fixtures/compile/{name}.ssrg"));
            let expected = std::fs::read(source.with_extension("stdout")).unwrap();
            (source, expected)
        };
        check_output(
            process::bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("run")
                    .arg(&input),
                &case,
            ),
            &expected,
        );
        for (label, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let out = case.join(label);
            process::successful(process::bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&input)
                    .args(["--profile", label, "--out-dir"])
                    .arg(&out),
                &case,
            ));
            check_generated_typescript(
                &input,
                profile,
                &case.join(format!("{label}-raw")),
                &engines[1],
            );
            for engine in &engines {
                check_output(
                    process::bounded(Command::new(engine).arg(out.join("entry.js")), &case),
                    &expected,
                );
            }
            if name == "polymorphic-global-values" {
                let probe_root = case.join("development");
                process::successful(process::bounded(
                    Command::new(&engines[0])
                        .args(["build", "--target=node", "--packages=external"])
                        .arg(case.join(format!("{label}-raw/main.ts")))
                        .arg("--outfile")
                        .arg(probe_root.join("sharing-module.mjs")),
                    &case,
                ));
                let probe = probe_root.join("sharing.mjs");
                std::fs::write(
                    &probe,
                    "import { intIdentity, stringIdentity } from './sharing-module.mjs';\n\
                     const first = intIdentity(undefined);\n\
                     const second = stringIdentity(undefined);\n\
                     if (typeof first !== 'function' || !Object.is(first, second)) {\n\
                       throw new Error('the polymorphic global function was recreated');\n\
                     }\n\
                     if (first(42) !== 42 || second('shared') !== 'shared') {\n\
                       throw new Error('the shared global function changed behavior');\n\
                     }\n",
                )
                .unwrap();
                for engine in &engines {
                    check_output(
                        process::bounded(Command::new(engine).arg(&probe), &case),
                        b"",
                    );
                }
            }
        }
        if name == "polymorphic-global-functions" {
            let out = case.join("native");
            process::successful(process::bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .args(["experimental", "build-native"])
                    .arg(&input)
                    .args(["--entry", "main", "--out-dir"])
                    .arg(&out),
                &case,
            ));
            check_output(
                process::bounded(
                    &mut Command::new(out.join(if cfg!(windows) {
                        "program.exe"
                    } else {
                        "program"
                    })),
                    &case,
                ),
                &expected,
            );
        }
    }
}

#[test]
fn polymorphic_values_preserve_surrounding_initializer_defects_and_order() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-polymorphic-initialization-")
        .tempdir()
        .unwrap();
    let source = scratch.path().join("main.ssrg");
    std::fs::write(
        &source,
        "let first: Int = 1 / 0\n\
         let absent = Nothing\n\
         let second: Int = 9007199254740991 + 1\n\
         pub effect fn main = println \"unexpected\"\n",
    )
    .unwrap();
    let engines = ["bun", "node"].map(|name| process::runtime(name, scratch.path()));
    let check_defect = |output: std::process::Output| {
        assert_eq!(output.status.code(), Some(70), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("runtime defect during module initialization"),
            "{error}"
        );
        assert!(error.contains("Seseragi Int division by zero"), "{error}");
        assert!(!error.contains("overflow"), "{error}");
    };
    check_defect(process::bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&source),
        scratch.path(),
    ));
    for (label, profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let out = scratch.path().join(label);
        process::successful(process::bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--profile", label, "--out-dir"])
                .arg(&out),
            scratch.path(),
        ));
        check_generated_typescript(
            &source,
            profile,
            &scratch.path().join(format!("{label}-raw")),
            &engines[1],
        );
        for engine in &engines {
            check_defect(process::bounded(
                Command::new(engine).arg(out.join("entry.js")),
                scratch.path(),
            ));
        }
    }
}
