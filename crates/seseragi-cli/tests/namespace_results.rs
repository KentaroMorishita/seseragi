use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
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

fn strict_config(root: &Path, files: &[PathBuf]) -> serde_json::Value {
    serde_json::json!({"compilerOptions": {
        "strict": true, "skipLibCheck": false, "noEmit": true,
        "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
        "allowImportingTsExtensions": true, "types": ["bun", "node"],
        "typeRoots": [root.join("node_modules/@types")],
        "lib": ["ES2023", "DOM", "DOM.Iterable"],
        "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
    }, "files": files})
}

#[test]
fn namespace_constructor_results_keep_strict_types_through_saved_functions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-namespace-results-")
        .tempdir()
        .unwrap();
    let fixture = root.join("examples/spec/fixtures/rust-backend-projects/array-index");
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let engines = ["bun", "node"].map(|name| {
        let found = success(
            Command::new(if cfg!(windows) { "where" } else { "which" })
                .arg(name)
                .output()
                .unwrap(),
        );
        PathBuf::from(
            String::from_utf8(found.stdout)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
    });
    for label in [
        "namespace-facade",
        "namespace-direct",
        "direct-facade",
        "adt-comparison",
        "shadow-opaque",
        "initialization-defect",
    ] {
        let project = scratch.path().join(label).join("project");
        copy_project(&fixture, &project);
        let main = project.join("src/main.ssrg");
        let mut source = fs::read_to_string(&main).unwrap();
        if label == "namespace-direct" {
            source = source.replace("\"./facade\"", "\"model\"");
        }
        if label == "direct-facade" {
            source = source
                .replace("import * as model", "import { Box, read, higher, keep }")
                .replace("model.", "");
        }
        let mut expected = b"[box=42, Just box=7, Nothing]\n".to_vec();
        if label == "adt-comparison" {
            let model = project.join("vendor/model/src/lib.ssrg");
            let mut provider = fs::read_to_string(&model).unwrap();
            provider.push_str(concat!(
                "\npub type Ranked<A> = | Ranked A\n",
                "instance<A> Eq<Ranked<A>> where Eq<A> { fn eq left: Ranked<A> -> right: Ranked<A> -> Bool = match left { Ranked first -> match right { Ranked second -> first == second } } }\n",
                "instance<A> Ord<Ranked<A>> where Ord<A> { fn compare left: Ranked<A> -> right: Ranked<A> -> Ordering = match left { Ranked first -> match right { Ranked second -> compare first second } } }\n",
            ));
            fs::write(model, provider).unwrap();
            let facade = project.join("src/facade.ssrg");
            let mut exports = fs::read_to_string(&facade).unwrap();
            exports.push_str("pub import { Ranked } from \"model\"\n");
            fs::write(facade, exports).unwrap();
            source = concat!(
                "import * as model from \"./facade\"\n",
                "pub effect fn main = do {\n",
                "  let value: model.Ranked<Int> = model.Ranked 2\n",
                "  println (show (value == model.Ranked 2))\n",
                "  println (show (value < model.Ranked 3))\n",
                "}\n",
            )
            .into();
            expected = b"True\nTrue\n".to_vec();
        }
        if label == "shadow-opaque" {
            let model = project.join("vendor/model/src/lib.ssrg");
            let mut provider = fs::read_to_string(&model).unwrap();
            provider.push_str(concat!(
                "\npub opaque type Secret<A> = | Secret A\n",
                "pub fn seal<A> value: A -> Secret<A> = Secret value\n",
                "pub fn reveal<A> value: Secret<A> -> A = match value { Secret item -> item }\n",
                "pub fn pick<Box> value: Box -> (Int -> Box) = \\_: Int -> value\n",
            ));
            fs::write(model, provider).unwrap();
            let facade = project.join("src/facade.ssrg");
            let mut exports = fs::read_to_string(&facade).unwrap();
            exports.push_str("pub import { Secret, seal, reveal, pick } from \"model\"\n");
            fs::write(facade, exports).unwrap();
            source = concat!(
                "import * as model from \"./facade\"\n",
                "type Box<A> = | Local A\n",
                "instance<A> Show<Box<A>> where Show<A> { fn show value: Box<A> -> String = match value { Local item -> \"local=\" + show item } }\n",
                "pub effect fn main = do {\n",
                "  let saved = model.read [model.keep (model.Box 42)]\n",
                "  let callback = match saved 0 { Just value -> value; Nothing -> model.keep (model.Box 0) }\n",
                "  let secret = model.pick (model.seal (callback 1))\n",
                "  println (show (model.reveal (secret 2)))\n",
                "  println (show (Local 9))\n",
                "}\n",
            ).into();
            expected = b"box=42\nlocal=9\n".to_vec();
        }
        let code = if label == "initialization-defect" {
            let model = project.join("vendor/model/src/lib.ssrg");
            let mut provider = fs::read_to_string(&model).unwrap();
            provider.push_str("\nlet privateDefect: Int = 9007199254740991 + 1\n");
            fs::write(model, provider).unwrap();
            expected.clear();
            70
        } else {
            0
        };
        fs::write(main, &source).unwrap();
        success(
            Command::new(cli)
                .args(["lock", "update"])
                .arg(&project)
                .output()
                .unwrap(),
        );
        let native = scratch.path().join(format!("{label}-native"));
        success(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&project)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native)
                .output()
                .unwrap(),
        );
        let result = Command::new(native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .current_dir(scratch.path())
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        assert_eq!(
            result.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, expected);
        assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
        let loaded = seseragi_project::load_local_project(&project).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let site = scratch.path().join(format!("{label}-{profile}"));
            success(
                Command::new(cli)
                    .arg("build")
                    .arg(&project)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&site)
                    .output()
                    .unwrap(),
            );
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for file in manifest["files"].as_array().unwrap() {
                assert_eq!(
                    format!(
                        "{:x}",
                        Sha256::digest(
                            fs::read(site.join(file["path"].as_str().unwrap())).unwrap()
                        )
                    ),
                    file["sha256"].as_str().unwrap()
                );
            }
            let proof = scratch.path().join(format!("{label}-strict-{profile}"));
            fs::create_dir(&proof).unwrap();
            if profile == "development" {
                let files: Vec<_> = manifest["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|file| file["path"].as_str().unwrap())
                    .filter(|file| {
                        file.ends_with(".ts")
                            && !file.starts_with("node_modules/")
                            && !file.starts_with("runtime/")
                    })
                    .map(|file| site.join(file))
                    .collect();
                assert!(files.contains(&site.join("entry.ts")));
                fs::write(
                    proof.join("artifact.json"),
                    strict_config(&root, &files).to_string(),
                )
                .unwrap();
                success(
                    Command::new(&engines[1])
                        .arg(root.join("node_modules/typescript/bin/tsc"))
                        .arg("--project")
                        .arg(proof.join("artifact.json"))
                        .output()
                        .unwrap(),
                );
            } else {
                assert!(
                    !site.join("entry.ts").exists(),
                    "missing release raw output is not a strict pass"
                );
            }
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            let raw = proof.join("raw");
            fs::create_dir(&raw).unwrap();
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|part| part == std::path::Component::ParentDir)
                );
                let file = raw.join(relative);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            fs::write(
                proof.join("raw.json"),
                strict_config(&root, &files).to_string(),
            )
            .unwrap();
            success(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(proof.join("raw.json"))
                    .output()
                    .unwrap(),
            );
            for engine in &engines {
                let result = Command::new(engine)
                    .arg(site.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(scratch.path())
                    .env("PATH", "")
                    .output()
                    .unwrap();
                assert_eq!(result.status.code(), Some(code));
                assert_eq!(result.stdout, expected);
                if code == 0 {
                    assert!(result.stderr.is_empty());
                } else {
                    assert!(String::from_utf8_lossy(&result.stderr).contains("Int overflow"));
                }
            }
        }
        if label == "shadow-opaque" {
            for (suffix, expected_type) in [("wrong-owner", "Box"), ("wrong-opaque", "Secret")] {
                let wrong =
                    format!("{source}\npub fn wrong -> model.{expected_type}<Int> = Local 42\n");
                fs::write(project.join("src/main.ssrg"), wrong).unwrap();
                success(
                    Command::new(cli)
                        .args(["lock", "update"])
                        .arg(&project)
                        .output()
                        .unwrap(),
                );
                let site = scratch.path().join(suffix);
                let refused = Command::new(cli)
                    .arg("build")
                    .arg(&project)
                    .arg("--out-dir")
                    .arg(&site)
                    .output()
                    .unwrap();
                assert_eq!(refused.status.code(), Some(2));
                assert!(String::from_utf8_lossy(&refused.stderr).contains("SES-T0101"));
                assert!(!site.exists());
                let rust = scratch.path().join(format!("{suffix}.rs"));
                let refused = Command::new(cli)
                    .args(["experimental", "emit-rust"])
                    .arg(&project)
                    .args(["--entry", "main", "--output"])
                    .arg(&rust)
                    .output()
                    .unwrap();
                assert_eq!(refused.status.code(), Some(2));
                assert!(String::from_utf8_lossy(&refused.stderr).contains("SES-T0101"));
                assert!(!rust.exists());
            }
        }
    }
}
