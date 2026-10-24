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
fn public_aliases_keep_raw_strict_exports_owners_and_type_only_initialization() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-public-aliases-")
        .tempdir()
        .unwrap();
    let fixture = root.join("examples/spec/fixtures/rust-backend-projects/public-alias-projection");
    let expected = fs::read(fixture.join("expected.stdout")).unwrap();
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
    for (label, code, expected) in [
        ("values", 0, expected.as_slice()),
        ("type-only-defect", 70, b"".as_slice()),
    ] {
        let project = scratch.path().join(label).join("project");
        copy_project(&fixture, &project);
        if code == 70 {
            let model = project.join("vendor/model/src/lib.ssrg");
            let mut source = fs::read_to_string(&model).unwrap();
            source.push_str("\nlet privateDefect: Int = 9007199254740991 + 1\n");
            fs::write(model, source).unwrap();
            fs::write(project.join("src/main.ssrg"), "import { MyUnit } from \"./facade\"\npub effect fn main = do { let value: MyUnit = (); println (show value) }\n").unwrap();
        }
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
            let mut facade = None;
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
                if module.generated.metadata.module.ends_with("::facade") {
                    facade = Some(relative.to_owned());
                }
                if label == "values"
                    && module
                        .generated
                        .typescript
                        .contains("export type Identity<Int>")
                {
                    assert!(module
                        .generated
                        .typescript
                        .contains("export type Identity<Int> = Int"));
                    assert!(module
                        .generated
                        .typescript
                        .contains("export type UnitField<Unit>"));
                    assert!(module
                        .generated
                        .typescript
                        .contains("export type Wrapper<A>"));
                    assert!(module.generated.typescript.contains("export type Chain<A>"));
                    assert!(module
                        .generated
                        .typescript
                        .contains("export type MyUnit = undefined"));
                    assert!(!module
                        .generated
                        .metadata
                        .exports
                        .iter()
                        .any(|name| matches!(
                            name.as_str(),
                            "Wrapper"
                                | "Chain"
                                | "MyUnit"
                                | "Identity"
                                | "UnitField"
                                | "Items"
                                | "KeptSecret"
                        )));
                }
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
            if label == "values" {
                let facade = facade.unwrap().to_string_lossy().replace('\\', "/");
                let witness = raw.join("opaque-witness.ts");
                fs::write(&witness, format!("import {{ type PublicSecret }} from \"./{facade}\";\nconst forged: PublicSecret = {{}};\n")).unwrap();
                files.push(witness);
                fs::write(
                    proof.join("opaque.json"),
                    strict_config(&root, &files).to_string(),
                )
                .unwrap();
                let refused = Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(proof.join("opaque.json"))
                    .output()
                    .unwrap();
                assert_eq!(refused.status.code(), Some(2));
                let diagnostic = String::from_utf8(refused.stdout).unwrap();
                assert!(
                    diagnostic.contains("TS2741") && diagnostic.contains("__ssrg$brand$Secret"),
                    "{diagnostic}"
                );
            }
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
    }
    let wrong = scratch.path().join("wrong-owner");
    copy_project(&fixture, &wrong);
    let main = wrong.join("src/main.ssrg");
    let mut source = fs::read_to_string(&main).unwrap();
    source.push_str("\npub fn wrong unit: MyUnit -> PublicWrapper<Int> = Local 42\n");
    fs::write(main, source).unwrap();
    success(
        Command::new(cli)
            .args(["lock", "update"])
            .arg(&wrong)
            .output()
            .unwrap(),
    );
    let site = scratch.path().join("wrong-site");
    let refused = Command::new(cli)
        .arg("build")
        .arg(&wrong)
        .arg("--out-dir")
        .arg(&site)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("SES-T0101"));
    assert!(!site.exists());
    let rust = scratch.path().join("wrong.rs");
    let refused = Command::new(cli)
        .args(["experimental", "emit-rust"])
        .arg(&wrong)
        .args(["--entry", "main", "--output"])
        .arg(&rust)
        .output()
        .unwrap();
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("SES-T0101"));
    assert!(!rust.exists());
}
