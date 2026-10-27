use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn test_directory(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-dts-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_directory(&from, &to);
        } else {
            fs::copy(from, to).unwrap();
        }
    }
}

fn fixture(name: &str) -> PathBuf {
    repository_root()
        .join("examples/spec/fixtures/projects")
        .join(name)
}

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn converts_deprecation_through_the_product_writer_and_compiles_both_profiles() {
    let directory = tempfile::tempdir().unwrap();
    let package = directory.path().join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);
    let declaration = concat!(
        "// 元のAPI 🙂\n",
        "/** @deprecated Use current.\n * @since 9.9.9\n */\n",
        "export declare function old(value: string): string;\n",
        "export declare function current(value: string): string;\n",
        "/** @deprecated Use Modern. */\nexport interface Old { readonly value: string; }\n",
        "export declare namespace nested {\n",
        "  /** @deprecated Use current. */\n  export function old(value: string): string;\n",
        "}\n",
    );
    fs::write(package.join("host/index.d.ts"), declaration).unwrap();
    let output = run(&["dts", "convert", package.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = package.join(".seseragi/generated/fixture-api.ssrg");
    let source = fs::read_to_string(&generated).unwrap();
    let metadata: Value =
        serde_json::from_slice(&fs::read(generated.with_extension("binding.json")).unwrap())
            .unwrap();
    assert!(source.contains("deprecated \"Use current.\""));
    assert!(!source.contains("since"));
    let symbols = metadata["symbols"].as_array().unwrap();
    assert_eq!(
        symbols
            .iter()
            .filter(|symbol| symbol.get("deprecations").is_some())
            .count(),
        3
    );
    for symbol in symbols
        .iter()
        .filter(|symbol| symbol.get("deprecations").is_some())
    {
        let origin = &symbol["deprecations"][0]["origin"];
        assert_eq!(origin["source"], "host/index.d.ts");
        let start = origin["start"].as_u64().unwrap() as usize;
        let end = origin["end"].as_u64().unwrap() as usize;
        assert!(declaration[start..end].starts_with("/** @deprecated"));
    }
    let plain = source
        .replace("deprecated \"Use current.\" ", "")
        .replace("deprecated \"Use Modern.\" ", "");
    for profile in [
        seseragi_driver::BuildProfile::Development,
        seseragi_driver::BuildProfile::Release,
    ] {
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("generated.ssrg", "generated/api", &source)
                .with_profile(profile),
        )
        .unwrap();
        let original = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("generated.ssrg", "generated/api", &plain)
                .with_profile(profile),
        )
        .unwrap();
        assert_eq!(compiled.generated.typescript, original.generated.typescript);
        assert_eq!(
            compiled
                .typed_interface
                .exports
                .iter()
                .find(|export| export.name == "old")
                .unwrap()
                .deprecation
                .as_ref()
                .unwrap()
                .message,
            "Use current."
        );
    }
}

fn expected_conversion_stdout(package: &Path, entries: &[(&str, &str)]) -> String {
    let mut output = String::new();
    for (id, name) in entries {
        let report: Value = serde_json::from_slice(
            &fs::read(package.join(format!(".seseragi/generated/{name}.report.json"))).unwrap(),
        )
        .unwrap();
        let compatibility: seseragi_dts::SemanticCompatibilityReport =
            serde_json::from_value(report["compatibility"].clone()).unwrap();
        output.push_str(&format!(
            "compatibility for dts entry `{id}`:\n{}\n",
            compatibility.render_text()
        ));
    }
    for (id, name) in entries {
        output.push_str(&format!(
            "converted dts entry `{id}` to .seseragi/generated/{name}.ssrg\n"
        ));
    }
    output
}

#[test]
fn executes_every_dts_fixture_through_the_cli_product_route() {
    for (name, output) in [
        ("dts-basic-conversion", "fixture-api"),
        ("dts-callback-during-call", "callback-api"),
        ("dts-declaration-merge", "merge-api"),
        ("dts-generated-name", "naming-api"),
        ("dts-namespace-runtime", "analytics"),
        ("dts-overload-selection", "parser-api"),
    ] {
        let directory = test_directory(name);
        let package = directory.join("package");
        copy_directory(&fixture(name), &package);
        let converted = run(&["dts", "convert", package.to_str().unwrap()]);
        assert_eq!(
            converted.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&converted.stderr)
        );
        assert_eq!(
            fs::read_to_string(
                package
                    .join(".seseragi/generated")
                    .join(format!("{output}.ssrg"))
            )
            .unwrap(),
            fs::read_to_string(package.join("expected").join(format!("{output}.ssrg"))).unwrap(),
            "{name}"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    for (name, code) in [
        ("dts-callback-missing-release", "SES-F0102"),
        ("dts-unsupported-any", "SES-F0101"),
    ] {
        let directory = test_directory(name);
        let package = directory.join("package");
        copy_directory(&fixture(name), &package);
        let converted = run(&["dts", "convert", package.to_str().unwrap()]);
        assert_eq!(converted.status.code(), Some(1), "{name}");
        assert!(String::from_utf8_lossy(&converted.stderr).contains(code));
        assert!(!package.join(".seseragi/generated").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn builds_generated_foreign_bindings_with_imported_big_int_types() {
    let directory = test_directory("imported-types");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);
    let manifest = package.join("seseragi.toml");
    fs::write(
        &manifest,
        format!(
            "{}\n[run]\nentry = \"main\"\ntarget = \"process\"\n",
            fs::read_to_string(&manifest).unwrap()
        ),
    )
    .unwrap();
    fs::create_dir_all(package.join("src")).unwrap();
    fs::create_dir_all(package.join("host/node_modules")).unwrap();
    fs::write(
        package.join("src/main.ssrg"),
        "import { fetchName } from \"gen/fixture-api\"\npub effect fn main = println \"ok\"\n",
    )
    .unwrap();

    for command in [
        vec!["dts", "convert"],
        vec!["lock", "update"],
        vec!["build"],
    ] {
        let mut arguments = command;
        arguments.push(package.to_str().unwrap());
        let output = run(&arguments);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn converts_a_configured_entry_and_writes_the_three_deterministic_artifacts() {
    let directory = test_directory("success");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);

    let first = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        expected_conversion_stdout(&package, &[("api", "fixture-api")])
    );

    let generated = package.join(".seseragi/generated");
    assert_eq!(
        fs::read_to_string(generated.join("fixture-api.ssrg")).unwrap(),
        fs::read_to_string(package.join("expected/fixture-api.ssrg")).unwrap()
    );
    let metadata: Value =
        serde_json::from_slice(&fs::read(generated.join("fixture-api.binding.json")).unwrap())
            .unwrap();
    assert_eq!(metadata["schema"], 1);
    assert_eq!(metadata["kind"], "seseragi-typescript-binding");
    assert_eq!(metadata["entry"], "api");
    assert_eq!(metadata["specifier"], "fixture-api");
    assert_eq!(metadata["hostModule"]["specifier"], "fixture-api");
    assert_eq!(metadata["hostModule"]["exactIdentity"], "fixture-api@1.0.0");
    assert_eq!(metadata["evaluation"], "task");
    assert_eq!(metadata["symbols"].as_array().unwrap().len(), 4);
    assert_eq!(metadata["generator"]["name"], "seseragi-dts");
    assert_eq!(metadata["inputDigest"].as_str().unwrap().len(), 64);
    assert_eq!(metadata["settingsDigest"].as_str().unwrap().len(), 64);

    let report: Value =
        serde_json::from_slice(&fs::read(generated.join("fixture-api.report.json")).unwrap())
            .unwrap();
    assert_eq!(report["schema"], 1);
    assert_eq!(report["entry"], "api");
    assert_eq!(report["added"].as_array().unwrap().len(), 4);
    assert_eq!(report["changed"].as_array().unwrap().len(), 0);
    assert_eq!(report["removed"].as_array().unwrap().len(), 0);
    assert_eq!(report["unsupported"].as_array().unwrap().len(), 0);

    let before = fs::read(generated.join("fixture-api.binding.json")).unwrap();
    let second = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "api",
    ]);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(
        fs::read(generated.join("fixture-api.binding.json")).unwrap(),
        before
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn converts_all_entries_or_only_the_selected_entry_in_stable_id_order() {
    let directory = test_directory("entry-selection");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);
    fs::write(
        package.join("host/extra.d.ts"),
        "export declare function label(value: string): string;\n",
    )
    .unwrap();
    fs::write(
        package.join("seseragi.bindings.toml"),
        concat!(
            "schema = 1\n\n",
            "[entries.alpha]\n",
            "declaration = \"host/extra.d.ts\"\n",
            "specifier = \"extra-api\"\n",
            "output = \"extra-api\"\n",
            "evaluation = \"task\"\n\n",
            "[entries.api]\n",
            "declaration = \"host/index.d.ts\"\n",
            "specifier = \"fixture-api\"\n",
            "output = \"fixture-api\"\n",
            "evaluation = \"task\"\n",
        ),
    )
    .unwrap();
    let host_manifest: Value =
        serde_json::from_slice(&fs::read(package.join("host/package.json")).unwrap()).unwrap();
    let mut host_manifest = host_manifest.as_object().unwrap().clone();
    host_manifest.insert("name".to_owned(), Value::String("host-app".to_owned()));
    host_manifest.insert(
        "dependencies".to_owned(),
        serde_json::json!({ "extra-api": "1.0.0", "fixture-api": "1.0.0" }),
    );
    fs::write(
        package.join("host/package.json"),
        serde_json::to_vec_pretty(&host_manifest).unwrap(),
    )
    .unwrap();
    for package_name in ["extra-api", "fixture-api"] {
        let module = package.join("host/node_modules").join(package_name);
        fs::create_dir_all(&module).unwrap();
        fs::write(
            module.join("package.json"),
            format!("{{\"name\":\"{package_name}\",\"version\":\"1.0.0\"}}\n"),
        )
        .unwrap();
    }

    let selected = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "api",
    ]);
    assert_eq!(selected.status.code(), Some(0));
    let generated = package.join(".seseragi/generated");
    assert!(generated.join("fixture-api.ssrg").is_file());
    assert!(!generated.join("extra-api.ssrg").exists());

    let all = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(
        all.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&all.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&all.stdout),
        expected_conversion_stdout(&package, &[("alpha", "extra-api"), ("api", "fixture-api")])
    );
    assert!(generated.join("extra-api.ssrg").is_file());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reports_precise_conversion_errors_without_replacing_any_entry_artifact() {
    let directory = test_directory("atomic-error");
    let package = directory.join("package");
    copy_directory(&fixture("dts-unsupported-any"), &package);
    let generated = package.join(".seseragi/generated");
    fs::create_dir_all(&generated).unwrap();
    for suffix in ["ssrg", "binding.json", "report.json"] {
        fs::write(
            generated.join(format!("unsafe-api.{suffix}")),
            b"previous\n",
        )
        .unwrap();
    }

    let output = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let human = String::from_utf8_lossy(&output.stdout);
    assert!(
        human.contains("compatibility for dts entry `api`"),
        "{human}"
    );
    assert!(human.contains("policy-required"), "{human}");
    assert!(human.contains("projection-not-published"), "{human}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("host/index.d.ts:38:41: error[SES-F0101]"));
    assert!(stderr.contains("`any` requires an explicit unsafe fallback"));
    for suffix in ["ssrg", "binding.json", "report.json"] {
        assert_eq!(
            fs::read(generated.join(format!("unsafe-api.{suffix}"))).unwrap(),
            b"previous\n"
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_unknown_entries_and_stale_bindings_before_build_output() {
    let directory = test_directory("stale");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);

    let unknown = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "missing",
    ]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("binding entry `missing` does not exist")
    );

    assert_eq!(
        run(&["dts", "convert", package.to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(
        package.join("src/main.ssrg"),
        "pub effect fn main -> Unit with Console fails ConsoleError = println \"ready\"\n",
    )
    .unwrap();
    fs::write(
        package.join("seseragi.toml"),
        fs::read_to_string(package.join("seseragi.toml")).unwrap()
            + "\n[run]\nentry = \"main\"\ntarget = \"process\"\n",
    )
    .unwrap();
    let lock = run(&["lock", "update", package.to_str().unwrap()]);
    assert_eq!(
        lock.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    fs::write(
        package.join("host/index.d.ts"),
        fs::read_to_string(package.join("host/index.d.ts")).unwrap()
            + "\nexport declare function newlyAdded(): string;\n",
    )
    .unwrap();

    let artifact = directory.join("artifact");
    let build = run(&[
        "build",
        package.to_str().unwrap(),
        "--out-dir",
        artifact.to_str().unwrap(),
    ]);
    assert_eq!(build.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(stderr.contains("SES-F0103"));
    assert!(stderr.contains("run `seseragi dts convert`"));
    assert!(!artifact.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn runs_a_generated_binding_through_the_normal_foreign_typescript_pipeline() {
    let directory = test_directory("dts-namespace-runtime-pipeline");
    let package = directory.join("package");
    copy_directory(&fixture("dts-namespace-runtime"), &package);

    let converted = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(
        converted.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    let lock = run(&["lock", "update", package.to_str().unwrap()]);
    assert_eq!(
        lock.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let executed = run(&["run", package.to_str().unwrap()]);
    assert_eq!(
        executed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(
        fs::read_to_string(package.join("expected.stdout")).unwrap(),
        String::from_utf8_lossy(&executed.stdout)
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn explicit_json_policy_runs_a_separate_checked_api_and_rejects_stale_policy() {
    let directory = test_directory("semantic-policy");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);
    let manifest_path = package.join("seseragi.toml");
    fs::write(
        &manifest_path,
        format!(
            "{}\n[run]\nentry='main'\ntarget='process'\n",
            fs::read_to_string(&manifest_path).unwrap()
        ),
    )
    .unwrap();
    let settings_path = package.join("seseragi.bindings.toml");
    let settings = format!(
        "{}\nprojection_policy='host/user.policy.json'\n",
        fs::read_to_string(&settings_path).unwrap()
    );
    fs::write(&settings_path, &settings).unwrap();
    let declaration = "export type User = { readonly name: string; readonly active: boolean }; export declare function touch(): void;";
    fs::write(package.join("host/index.d.ts"), declaration).unwrap();
    let host = package.join("host/node_modules/fixture-api");
    fs::create_dir_all(&host).unwrap();
    fs::write(
        host.join("package.json"),
        r#"{"name":"fixture-api","version":"1.0.0","type":"module","exports":"./index.js"}"#,
    )
    .unwrap();
    let marker = directory.join("host-executed.txt");
    let marker_literal = serde_json::to_string(marker.to_str().unwrap()).unwrap();
    fs::write(host.join("index.js"), format!("import fs from 'node:fs'; fs.appendFileSync({marker_literal}, 'module loaded\\n'); export function touch() {{ fs.appendFileSync({marker_literal}, 'function called\\n'); }}")).unwrap();
    let host_manifest = fs::read_to_string(package.join("host/package.json")).unwrap();
    let mut interpreted = seseragi_dts::convert_entry(&seseragi_dts::ConversionInput {
        settings: &settings,
        entry: "api",
        declaration,
        host: seseragi_dts::HostMetadata {
            manifest: &host_manifest,
            resolved_package: None,
        },
        previous_metadata: None,
    })
    .unwrap();
    let entry = seseragi_dts::parse_bindings(&settings)
        .unwrap()
        .entries
        .remove("api")
        .unwrap();
    let manifest =
        seseragi_project::parse_manifest(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let artifacts = [
        seseragi_project::RuntimeTarget::Node,
        seseragi_project::RuntimeTarget::Bun,
        seseragi_project::RuntimeTarget::Browser,
    ]
    .into_iter()
    .filter_map(|target| {
        seseragi_project::resolve_foreign_runtime_artifact_for_target(
            &package,
            manifest.foreign_typescript.as_ref(),
            &package.join(".seseragi/generated/fixture-api.ssrg"),
            "fixture-api",
            target,
        )
        .ok()
        .map(|artifact| (target, artifact))
    })
    .collect();
    seseragi_dts::attach_runtime_linkage(&mut interpreted, &entry, &artifacts).unwrap();
    let graph = interpreted.semantic.unwrap();
    let policy = serde_json::json!({"schema":1,"kind":"seseragi-semantic-projection-policy","producer":{"name":"explicit-user-policy","version":"1"},"graphDigest":graph.digest().unwrap(),"artifactIdentity":graph.identity,"policies":[{"selector":{"kind":"symbol","symbol":"User"},"functionName":"decodeUser","mapping":{"kind":"json-snapshot"}}]});
    fs::write(package.join("host/user.policy.json"), policy.to_string()).unwrap();
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(package.join("src/main.ssrg"),r#"import * as json from "std/json"
import * as effects from "std/effect"
import {touch} from "gen/fixture-api"
import {decodeUser} from "gen/fixture-api/semantic"
fn input text: String -> json.Json = match json.parse text { Left _ -> json.JsonNull; Right value -> value }
fn decoded text: String -> String = match decodeUser (input text) { Left _ -> "rejected"; Right user -> user.name }
type AppError deriving Show = | ConsoleFailure ConsoleError | ForeignFailure Js.Error
pub effect fn main -> Unit with Console fails AppError = do {
  touch () |> effects.mapError ForeignFailure
  println (decoded "{\"name\":\"Ada\",\"active\":true}") |> effects.mapError ConsoleFailure
  println (decoded "{\"name\":3,\"active\":true}") |> effects.mapError ConsoleFailure
}

"#).unwrap();
    let converted = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(
        converted.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    assert!(String::from_utf8_lossy(&converted.stdout).contains("applied-semantic-json-policy"));
    let locked = run(&["lock", "update", package.to_str().unwrap()]);
    assert!(
        locked.status.success(),
        "{}",
        String::from_utf8_lossy(&locked.stderr)
    );
    let executed = run(&["run", package.to_str().unwrap()]);
    assert_eq!(
        executed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "Ada\nrejected\n");
    assert_eq!(
        fs::read_to_string(&marker).unwrap(),
        "module loaded\nfunction called\n"
    );
    for profile in ["development", "release"] {
        let artifact = directory.join(profile);
        let built = run(&[
            "build",
            package.to_str().unwrap(),
            "--profile",
            profile,
            "--out-dir",
            artifact.to_str().unwrap(),
        ]);
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
    }
    fs::remove_file(&marker).unwrap();
    let mut stale = policy;
    stale["graphDigest"] = serde_json::json!("0".repeat(64));
    fs::write(package.join("host/user.policy.json"), stale.to_string()).unwrap();
    for arguments in [
        vec!["run"],
        vec!["build"],
        vec!["build", "--profile", "release"],
        vec!["lock", "update"],
    ] {
        let mut arguments = arguments;
        let artifact = directory.join("stale-artifact");
        if arguments[0] == "build" {
            arguments.extend(["--out-dir", artifact.to_str().unwrap()]);
        }
        arguments.push(package.to_str().unwrap());
        let rejected = run(&arguments);
        assert!(
            !rejected.status.success(),
            "{arguments:?} executed a stale checked API"
        );
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains("semantic policy"),
            "{}",
            String::from_utf8_lossy(&rejected.stderr)
        );
        assert!(!String::from_utf8_lossy(&rejected.stdout).contains("Ada"));
        assert!(!artifact.exists());
        assert!(
            !marker.exists(),
            "stale policy executed a host module or function"
        );
    }
    let failed = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("SES-F0105"));
    fs::remove_file(package.join("host/user.policy.json")).unwrap();
    let missing = run(&["run", package.to_str().unwrap()]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("semantic policy"));
    assert!(missing.stdout.is_empty());
    assert!(!marker.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn conversion_never_overwrites_another_entries_policy_or_partially_publishes() {
    for selected in [false, true] {
        let directory = test_directory("authored-policy-collision");
        let package = directory.join("package");
        copy_directory(&fixture("dts-basic-conversion"), &package);
        let settings_path = package.join("seseragi.bindings.toml");
        let settings = fs::read_to_string(&settings_path).unwrap() + "\n[entries.other]\ndeclaration='host/user.d.ts'\nspecifier='./data.js'\noutput='user'\nprojection_policy='.seseragi/generated/fixture-api.binding.json'\n";
        fs::write(&settings_path, &settings).unwrap();
        let declaration = "export type User = { readonly name: string };";
        fs::write(package.join("host/user.d.ts"), declaration).unwrap();
        let graph =
            seseragi_dts::interpret_declaration(declaration, "host/user.d.ts", &settings).unwrap();
        let policy = serde_json::json!({"schema":1,"kind":"seseragi-semantic-projection-policy","producer":{"name":"explicit-authored-policy","version":"1"},"graphDigest":graph.digest().unwrap(),"artifactIdentity":graph.identity,"policies":[{"selector":{"kind":"symbol","symbol":"User"},"functionName":"decodeUser","mapping":{"kind":"json-snapshot"}}]}).to_string();
        let authored = package.join(".seseragi/generated/fixture-api.binding.json");
        fs::create_dir_all(authored.parent().unwrap()).unwrap();
        fs::write(&authored, &policy).unwrap();
        let mut arguments = vec!["dts", "convert", package.to_str().unwrap()];
        if selected {
            arguments.extend(["--entry", "api"]);
        }
        let rejected = run(&arguments);
        assert_eq!(rejected.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("collides with authored input"));
        assert!(rejected.stdout.is_empty());
        assert_eq!(fs::read_to_string(&authored).unwrap(), policy);
        for output in [
            "fixture-api.ssrg",
            "fixture-api.report.json",
            "user.ssrg",
            "user.binding.json",
        ] {
            assert!(!package.join(".seseragi/generated").join(output).exists());
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
