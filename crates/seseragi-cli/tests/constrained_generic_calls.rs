use std::{fs, path::Path, process::Command};

#[path = "support/generic_impl_process.rs"]
mod process;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
}

fn check_typescript(input: &Path, directory: &Path, expected: &str, calls: &[&str]) {
    let engines = ["bun", "node"].map(|name| process::runtime(name, directory));
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let bundles = if input.is_dir() {
            let loaded = seseragi_project::load_local_project(input).unwrap();
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(build_profile))
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
                        "fixture/constrained-generics",
                        &fs::read_to_string(input).unwrap(),
                    )
                    .with_profile(build_profile),
                )
                .unwrap()
                .generated,
            ]
        };
        let raw = directory.join(format!("{profile}-raw"));
        let mut files = Vec::new();
        let mut typescript = String::new();
        for bundle in bundles {
            let file = raw.join(bundle.metadata.outputs.typescript);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, &bundle.typescript).unwrap();
            typescript.push_str(&bundle.typescript);
            files.push(file);
        }
        for call in calls {
            assert!(typescript.contains(call), "{profile}: {call}\n{typescript}");
        }
        let config = raw.join("tsconfig.json");
        fs::write(
            &config,
            serde_json::json!({
                "compilerOptions": {
                    "strict": true, "skipLibCheck": false, "noEmit": true,
                    "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                    "allowImportingTsExtensions": true, "types": ["bun", "node"],
                    "typeRoots": [root().join("node_modules/@types")],
                    "lib": ["ES2023", "DOM", "DOM.Iterable"],
                    "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
                },
                "files": files
            })
            .to_string(),
        )
        .unwrap();
        process::successful(process::bounded(
            Command::new(&engines[1])
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config),
            directory,
        ));
        let output = directory.join(profile);
        process::successful(process::bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(input)
                .args(["--profile", profile, "--out-dir"])
                .arg(&output),
            directory,
        ));
        for engine in &engines {
            let result = process::successful(process::bounded(
                Command::new(engine).arg(output.join("entry.js")),
                directory,
            ));
            assert_eq!(result.stdout, expected.as_bytes());
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn constrained_generic_call_reifies_inferred_type_before_first_curried_argument() {
    let scratch = tempfile::tempdir().unwrap();
    let input = root().join("crates/seseragi-cli/tests/fixtures/constrained-generic-calls.ssrg");
    check_typescript(&input, scratch.path(), "4242\n", &["render<number>(\"\")"]);
    let output = scratch.path().join("native");
    process::successful(process::bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        scratch.path(),
    ));
    let result = process::successful(process::bounded(
        &mut Command::new(output.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        })),
        scratch.path(),
    ));
    assert_eq!(result.stdout, b"4242\n");
    assert!(result.stderr.is_empty(), "{result:?}");
}

#[test]
fn constrained_generic_calls_preserve_partials_callbacks_and_type_parameter_order() {
    let scratch = tempfile::tempdir().unwrap();
    let input =
        root().join("crates/seseragi-cli/tests/fixtures/constrained-generic-call-variants.ssrg");
    check_typescript(
        &input,
        scratch.path(),
        "direct:42\nexplicit:42\nprefix:42\nsaved:42\ncallable:42\nreturned:42\nreturned-direct:42\ncombined:42:ok\ngeneric:42\nlocal:42\ncallback-direct:42\ncallback:42\ncustom:42:42\n7\n7\neffect:42\n",
        &[
            "render<number>(\"direct:\")",
            "render<number>(\"prefix:\")",
            "render<number>(\"saved:\")",
            "render<A>(prefix)",
            "render<T>(\"generic:\")",
            "combine<number, string>(\"combined:\")",
            "callback<number>(\"callback-direct:\")",
            "callback<number>(\"callback:\")",
            "labeled<number>(\"custom:\")",
            "transform<string, number, number>(choose)",
            "renderEffect<number>(\"effect:\")",
        ],
    );
}

#[test]
fn imported_constrained_generic_calls_reify_inferred_types() {
    let scratch = tempfile::tempdir().unwrap();
    let project = scratch.path().join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("seseragi.toml"),
        "[package]\nname=\"fixture/constrained-generics\"\nversion=\"0.0.0\"\nlanguage=\">=0.1.0 <0.2.0\"\n[run]\nentry=\"main\"\ntarget=\"process\"\n",
    )
    .unwrap();
    fs::write(
        project.join("src/render.ssrg"),
        include_str!("fixtures/constrained-generic-calls.ssrg")
            .split("pub effect fn main")
            .next()
            .unwrap()
            .replace("type Renderer", "pub type Renderer")
            .replace("fn render", "pub fn render"),
    )
    .unwrap();
    fs::write(
        project.join("src/main.ssrg"),
        "import { Renderer, render } from \"./render\"\npub effect fn main = { let saved: Renderer<Int> -> Int -> String = render \"\"; println (saved (Renderer (\\value: Int -> show value)) 42) }\n",
    )
    .unwrap();
    process::successful(process::bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&project),
        scratch.path(),
    ));
    check_typescript(&project, scratch.path(), "4242\n", &["<number>(\"\")"]);
}
