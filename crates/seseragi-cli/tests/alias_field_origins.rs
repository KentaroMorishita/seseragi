#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{fs, path::Path, process::Command};

const MODEL: &str = r#"
import * as regex from "std/regex"
import { RegexSpan } from "std/regex"
pub alias Span = { span: regex.RegexSpan }
pub alias Hit = { hit: regex.RegexMatch }
pub alias Problem = { error: regex.RegexCompileError }
pub struct Box<A> { value: A }
pub alias BoxedSpan = { wrapped: Box<regex.RegexSpan> }
pub fn make start:Int -> end:Int -> Span = {
  span: RegexSpan { start: start, end: end }
}
"#;

const MAIN: &str = r#"
import { Span, make } from "./model"
import { PublicSpan, Hit, Problem, BoxedSpan } from "./facade"
import * as api from "./facade"
pub fn offset value:Span -> Int = value.span.start
pub fn end value:PublicSpan -> Int = value.span.end
pub fn nested value:Hit -> Int = value.hit.span.start
pub fn failure value:Problem -> Int = value.error.offset
pub fn qualified value:api.PublicSpan -> Int = value.span.start
pub fn boxed value:BoxedSpan -> Int = value.wrapped.value.start
pub effect fn main = do {
  println (show (offset (make 0 0)))
  println (show (end (make 1 9)))
  println (show (qualified (make 9007199254740991 9007199254740991)))
}
"#;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn package(directory: &Path, source: &str) -> std::path::PathBuf {
    let project = directory.join("package");
    let src = project.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("model.ssrg"), MODEL).unwrap();
    fs::write(
        src.join("facade.ssrg"),
        "pub import { Span, Span as PublicSpan, Hit, Problem, BoxedSpan, make } from \"./model\"\n",
    )
    .unwrap();
    fs::write(
        src.join("shadow.ssrg"),
        "pub struct RegexSpan { start: String, end: Bool }\n",
    )
    .unwrap();
    fs::write(src.join("main.ssrg"), source).unwrap();
    fs::write(project.join("seseragi.toml"), "[package]\nname = \"fixture/alias-field-origins\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
    successful(bounded(
        cli().args(["lock", "update"]).arg(&project),
        directory,
    ));
    project
}

fn strict_typescript(project: &Path, directory: &Path, profile: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let loaded = seseragi_project::load_local_project(project).unwrap();
    let profile = if profile == "development" {
        seseragi_driver::BuildProfile::Development
    } else {
        seseragi_driver::BuildProfile::Release
    };
    let compiled =
        seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
    let mut files = Vec::new();
    for module in compiled.compiled.modules.values() {
        let relative = Path::new(&module.generated.metadata.outputs.typescript);
        assert!(!relative.is_absolute());
        assert!(!relative
            .components()
            .any(|part| part == std::path::Component::ParentDir));
        let file = directory.join("raw").join(relative);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, &module.generated.typescript).unwrap();
        files.push(file);
    }
    let config = directory.join("tsconfig.json");
    fs::write(
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
    successful(bounded(
        Command::new(runtime("node", directory))
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        directory,
    ));
}

#[test]
fn alias_fields_keep_defining_structs_without_consumer_imports() {
    for shadow in [false, true] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let source = if shadow {
            format!(
                "import * as regex from \"./shadow\"\nstruct RegexSpan {{ start: Bool }}\n{MAIN}\npub fn local value:regex.RegexSpan -> String = value.start\npub fn own value:RegexSpan -> Bool = value.start\n"
            )
            .replace("from \"./model\"", "from \"./facade\"")
        } else {
            MAIN.to_owned()
        };
        let project = package(directory, &source);
        let expected = b"0\n9\n9007199254740991\n";
        for profile in ["development", "release"] {
            let output = directory.join(profile);
            successful(bounded(
                cli()
                    .arg("build")
                    .arg(&project)
                    .args([
                        "--target",
                        "process",
                        "--source-map",
                        "omit",
                        "--profile",
                        profile,
                        "--out-dir",
                    ])
                    .arg(&output),
                directory,
            ));
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            strict_typescript(&project, &output, profile);
            for engine in ["bun", "node"] {
                let result = successful(bounded(
                    Command::new(runtime(engine, directory))
                        .arg(output.join(manifest["entry"].as_str().unwrap()))
                        .env("PATH", ""),
                    directory,
                ));
                assert_eq!(
                    result.stdout, expected,
                    "{profile}/{engine}/shadow={shadow}"
                );
                assert!(result.stderr.is_empty(), "{result:?}");
            }
        }
        let output = directory.join("native");
        successful(bounded(
            cli()
                .args(["experimental", "build-native"])
                .arg(&project)
                .args(["--entry", "main", "--out-dir"])
                .arg(&output),
            directory,
        ));
        let result = successful(bounded(
            Command::new(output.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            directory,
        ));
        assert_eq!(result.stdout, expected, "native/shadow={shadow}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    }
}

#[test]
fn alias_fields_reject_missing_fields_wrong_results_and_different_owners() {
    for wrong in [
        "pub fn wrong value:PublicSpan -> Int = value.span.missing\n",
        "pub fn wrong value:PublicSpan -> String = value.span.start\n",
        "pub fn wrong value:PublicSpan -> regex.RegexSpan = value.span\n",
        "pub fn wrong value:PublicSpan -> RegexSpan = value.span\n",
    ] {
        let scratch = tempfile::tempdir().unwrap();
        let directory = scratch.path();
        let source = format!(
            "import * as regex from \"./shadow\"\nstruct RegexSpan {{ start: Bool }}\n{MAIN}\n{wrong}"
        );
        let project = package(directory, &source);
        let ts_output = directory.join("ts");
        let ts = bounded(
            cli()
                .arg("build")
                .arg(&project)
                .args(["--target", "process", "--source-map", "omit", "--out-dir"])
                .arg(&ts_output),
            directory,
        );
        let native_output = directory.join("native");
        let native = bounded(
            cli()
                .args(["experimental", "build-native"])
                .arg(&project)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native_output),
            directory,
        );
        assert_eq!(ts.status.code(), Some(2), "{ts:?}");
        assert_eq!(native.status.code(), Some(2), "{native:?}");
        let diagnostic = String::from_utf8(ts.stderr).unwrap();
        assert_eq!(
            diagnostic.matches("error[SES-T0101]").count(),
            1,
            "{diagnostic}"
        );
        assert!(diagnostic.contains(wrong.trim()), "{diagnostic}");
        assert!(
            !diagnostic.contains("Record access on non record"),
            "{diagnostic}"
        );
        if wrong.contains("missing") {
            assert!(
                diagnostic.contains("has no field `missing`"),
                "{diagnostic}"
            );
        }
        assert_eq!(
            String::from_utf8(native.stderr).unwrap(),
            format!("seseragi: {diagnostic}\n")
        );
        assert!(!ts_output.exists());
        assert!(!native_output
            .join(format!("program{}", std::env::consts::EXE_SUFFIX))
            .exists());
    }
}
