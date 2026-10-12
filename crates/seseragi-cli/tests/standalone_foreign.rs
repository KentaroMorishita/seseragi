//! Standalone host resources are staged from explicit source context, not cwd.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};
fn temporary() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "seseragi-standalone-host-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
fn cli(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn task_source(specifier: &str, call: bool) -> String {
    format!(
        r#"import * as effects from "std/effect"
foreign "typescript" from "{specifier}" {{
  task fn read value: String -> String
}}
type AppError deriving Show =
  | ConsoleFailure ConsoleError
  | ForeignFailure Js.Error
pub effect fn main -> Unit with Console fails AppError = do {{
  println "before" |> effects.mapError ConsoleFailure
  {}
}}
"#,
        if call {
            "value <- read \"x\" |> effects.mapError ForeignFailure\n  println value |> effects.mapError ConsoleFailure"
        } else {
            ""
        }
    )
}
#[test]
fn standalone_task_and_pure_stage_only_host_closure_and_relocate_on_both_hosts() {
    for mode in ["task", "pure"] {
        let root = temporary();
        let source_root = root.join("source");
        fs::create_dir_all(source_root.join("host/nested")).unwrap();
        let main = source_root.join("main.ssrg");
        let expected = if mode == "task" {
            fs::write(&main, task_source("./host/api.mjs", true)).unwrap();
            fs::write(source_root.join("host/api.mjs"),"import {label} from './nested/helper.mjs';\nconsole.log('host:load');\nexport function read(value) { console.log('host:call:'+value); return label+':'+value; }\n").unwrap();
            "before\nhost:load\nhost:call:x\nhost:x\n"
        } else {
            fs::write(&main,"foreign \"typescript\" from \"./host/api.mjs\" {\n  pure fn read value: String -> String\n}\npub effect fn main = println (read \"x\")\n").unwrap();
            fs::write(source_root.join("host/api.mjs"),"import {label} from './nested/helper.mjs';\nexport function read(value) { return label+':'+value; }\n").unwrap();
            "host:x\n"
        };
        fs::write(
            source_root.join("host/nested/helper.mjs"),
            "export const label = 'host';\n",
        )
        .unwrap();
        fs::write(
            source_root.join("host/unrelated-secret.txt"),
            "must never be staged",
        )
        .unwrap();
        let run = cli(&["run", main.to_str().unwrap()], &root);
        success(&run);
        assert_eq!(String::from_utf8_lossy(&run.stdout), expected, "{mode}");
        for profile in ["development", "release"] {
            let out = root.join(profile);
            success(&cli(
                &[
                    "build",
                    main.to_str().unwrap(),
                    "--profile",
                    profile,
                    "--target",
                    "process",
                    "--out-dir",
                    out.to_str().unwrap(),
                ],
                &root,
            ));
            assert!(out.join(".seseragi-foreign/host/api.mjs").is_file());
            assert!(out
                .join(".seseragi-foreign/host/nested/helper.mjs")
                .is_file());
            assert!(!out
                .join(".seseragi-foreign/host/unrelated-secret.txt")
                .exists());
            let generated = fs::read_to_string(out.join("main.ts")).unwrap();
            assert!(!generated.contains(source_root.to_str().unwrap()));
            if profile == "development" {
                let map: serde_json::Value =
                    serde_json::from_slice(&fs::read(out.join("main.ts.map")).unwrap()).unwrap();
                assert_eq!(map["sources"][0], "seseragi://single-file/main");
            }
        }
        fs::remove_dir_all(source_root).unwrap();
        for profile in ["development", "release"] {
            let out = root.join(format!("moved-{profile}"));
            fs::rename(root.join(profile), &out).unwrap();
            let artifact: serde_json::Value =
                serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for host in ["node", "bun"] {
                let run = Command::new(host)
                    .arg(out.join(artifact["entry"].as_str().unwrap()))
                    .current_dir(&root)
                    .output()
                    .unwrap();
                success(&run);
                assert_eq!(
                    String::from_utf8_lossy(&run.stdout),
                    expected,
                    "{mode}/{profile}/{host}"
                );
            }
        }
        fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn absent_task_hosts_remain_lazy_and_fail_at_the_typed_module_load_boundary() {
    let root = temporary();
    let main = root.join("main.ssrg");
    fs::write(&main, task_source("./missing.mjs", false)).unwrap();
    let unused = cli(&["run", main.to_str().unwrap()], &root);
    success(&unused);
    assert_eq!(String::from_utf8_lossy(&unused.stdout), "before\n");
    fs::write(&main, task_source("./missing.mjs", true)).unwrap();
    let invoked = cli(&["run", main.to_str().unwrap()], &root);
    assert!(!invoked.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&invoked.stdout),
        String::from_utf8_lossy(&invoked.stderr)
    );
    assert!(
        diagnostic.contains("ForeignFailure") && diagnostic.contains("ModuleLoad"),
        "{diagnostic}"
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn standalone_host_preflight_rejects_unbounded_and_unsupported_resource_paths() {
    for (specifier, host) in [
        (
            "./api.mjs",
            "export function read(value) { return import(value); }",
        ),
        (
            "../outside.mjs",
            "export function read(value) { return value; }",
        ),
        ("./unknown.bin", "not an executable host module"),
    ] {
        let root = temporary();
        let source_root = root.join("source");
        fs::create_dir(&source_root).unwrap();
        let main = source_root.join("main.ssrg");
        fs::write(&main, task_source(specifier, true)).unwrap();
        fs::write(source_root.join(specifier), host).unwrap();
        let run = cli(&["run", main.to_str().unwrap()], &root);
        assert!(!run.status.success());
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("SES-F0203"),
            "{run:?}"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn standalone_javascript_preserves_inherited_commonjs_scope_in_an_esm_artifact() {
    let root = temporary();
    fs::write(
        root.join("package.json"),
        r#"{"type":"commonjs","privateToken":"must-not-be-copied"}"#,
    )
    .unwrap();
    let source_root = root.join("source");
    fs::create_dir_all(&source_root).unwrap();
    let main = source_root.join("main.ssrg");
    fs::write(&main,"foreign \"typescript\" from \"./api.js\" {\n  pure fn read value: String -> String\n}\npub effect fn main = println (read \"x\")\n").unwrap();
    fs::write(source_root.join("api.js"),"const helper = require('./helper.cjs');\nexports.read = value => helper.label+':'+value;\n").unwrap();
    fs::write(
        source_root.join("helper.cjs"),
        "exports.label = 'commonjs';\n",
    )
    .unwrap();
    let run = cli(&["run", main.to_str().unwrap()], &root);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "commonjs:x\n");
    for profile in ["development", "release"] {
        let out = root.join(profile);
        success(&cli(
            &[
                "build",
                main.to_str().unwrap(),
                "--profile",
                profile,
                "--out-dir",
                out.to_str().unwrap(),
            ],
            &root,
        ));
        let scope = fs::read_to_string(out.join(".seseragi-foreign/package.json")).unwrap();
        assert!(
            scope.contains("commonjs") && !scope.contains("privateToken"),
            "{scope}"
        );
    }
    fs::remove_dir_all(source_root).unwrap();
    for profile in ["development", "release"] {
        let out = root.join(profile);
        let artifact: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for host in ["node", "bun"] {
            let run = Command::new(host)
                .arg(out.join(artifact["entry"].as_str().unwrap()))
                .current_dir(&root)
                .output()
                .unwrap();
            success(&run);
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                "commonjs:x\n",
                "{profile}/{host}"
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_host_typescript_file_named_main_cannot_overwrite_the_generated_main() {
    let root = temporary();
    let main = root.join("main.ssrg");
    fs::write(&main,"foreign \"typescript\" from \"./main.ts\" {\n  pure fn read value: String -> String\n}\npub effect fn main = println (read \"typed\")\n").unwrap();
    fs::write(
        root.join("main.ts"),
        "export function read(value: string): string { return value; }\n",
    )
    .unwrap();
    let run = cli(&["run", main.to_str().unwrap()], &root);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), "typed\n");
    let out = root.join("out");
    success(&cli(
        &[
            "build",
            main.to_str().unwrap(),
            "--out-dir",
            out.to_str().unwrap(),
        ],
        &root,
    ));
    assert_eq!(
        fs::read_to_string(out.join(".seseragi-foreign/main.ts")).unwrap(),
        fs::read_to_string(root.join("main.ts")).unwrap()
    );
    let generated = fs::read_to_string(out.join("main.ts")).unwrap();
    assert!(generated.contains("./.seseragi-foreign/main.ts"));
    let artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
    for host in ["node", "bun"] {
        let run = Command::new(host)
            .arg(out.join(artifact["entry"].as_str().unwrap()))
            .output()
            .unwrap();
        success(&run);
        assert_eq!(String::from_utf8_lossy(&run.stdout), "typed\n");
    }
    fs::remove_dir_all(root).unwrap();
}
