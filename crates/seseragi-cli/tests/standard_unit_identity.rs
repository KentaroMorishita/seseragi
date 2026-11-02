use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Fixture(tempfile::TempDir);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = tempfile::Builder::new()
            .prefix("seseragi-standard-unit-")
            .tempdir()
            .unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("seseragi.toml"), "[package]\nname = \"fixture/standard-unit\"\nversion = \"0.0.0\"\nlanguage = \"^0.1.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
        fs::write(root.join("src/main.ssrg"), source).unwrap();
        fs::write(root.join("src/model.ssrg"), "pub alias StandardUnit = Unit\npub fn standard -> StandardUnit = ()\npub type Token = | FakeUnit\npub alias NominalUnit = Token\n").unwrap();
        fs::write(root.join("src/facade.ssrg"), "pub import { StandardUnit, standard as done, Token as Unit, FakeUnit, NominalUnit } from \"./model\"\n").unwrap();
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["lock", "update"])
                .arg(root),
            root,
        ));
        Self(directory)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn typescript(&self, expected: &[u8]) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let engines = ["bun", "node"].map(|name| runtime(name, self.path()));
        let loaded = seseragi_project::load_local_project(self.path()).unwrap();
        for (name, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let out = self.path().join(name);
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(self.path())
                    .args([
                        "--target",
                        "process",
                        "--profile",
                        name,
                        "--source-map",
                        "omit",
                        "--out-dir",
                    ])
                    .arg(&out),
                self.path(),
            ));
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                    .unwrap();
            let raw = out.join("raw");
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(!relative.is_absolute());
                assert!(!relative
                    .components()
                    .any(|part| part == std::path::Component::ParentDir));
                let file = raw.join(relative);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            if name == "development" {
                files.push(out.join("entry.ts"));
            }
            let config = serde_json::json!({"compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun", "node"],
                "typeRoots": [root.join("node_modules/@types")], "baseUrl": root,
                "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023", "DOM", "DOM.Iterable"]
            }, "files": files});
            let config_path = out.join("tsconfig.json");
            fs::write(&config_path, config.to_string()).unwrap();
            successful(bounded(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(&config_path),
                self.path(),
            ));
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for engine in &engines {
                let result = successful(bounded(
                    Command::new(engine)
                        .arg(out.join(manifest["entry"].as_str().unwrap()))
                        .env("PATH", ""),
                    self.path(),
                ));
                assert_eq!(result.stdout, expected, "{name}/{engine:?}");
                assert!(result.stderr.is_empty(), "{result:?}");
            }
        }
    }

    fn native(&self) -> PathBuf {
        let out = self.path().join("native");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(self.path())
                .args(["--entry", "main", "--out-dir"])
                .arg(&out),
            self.path(),
        ));
        out.join(format!("program{}", std::env::consts::EXE_SUFFIX))
    }
}

#[test]
fn queue_nominal_unit_arguments_fail_before_either_target_emits_outputs() {
    for (declarations, value) in [
        ("type Unit = | FakeUnit", "FakeUnit"),
        ("type Token = | FakeUnit", "FakeUnit"),
        ("type Token = | FakeUnit\nalias Unit = Token", "FakeUnit"),
        ("import { Unit, FakeUnit } from \"./facade\"", "FakeUnit"),
        (
            "import { NominalUnit as Unit, FakeUnit } from \"./facade\"",
            "FakeUnit",
        ),
    ] {
        let source = format!("import * as queue from \"std/queue\"\n{declarations}\nimport * as effects from \"std/effect\"\npub effect fn main = do {{ queue.unbounded<Int> {value}; effects.succeed () }}\n");
        let fixture = Fixture::new(&source);
        for native in [false, true] {
            let out = fixture.path().join(if native { "native" } else { "ts" });
            let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
            if native {
                command
                    .args(["experimental", "build-native"])
                    .arg(fixture.path())
                    .args(["--entry", "main"]);
            } else {
                command
                    .arg("build")
                    .arg(fixture.path())
                    .args(["--target", "process"]);
            }
            let result = bounded(command.arg("--out-dir").arg(&out), fixture.path());
            assert_eq!(result.status.code(), Some(2), "{source}\n{result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
            let stderr = String::from_utf8(result.stderr).unwrap();
            assert!(stderr.contains("error[SES-T0101]"), "{source}\n{stderr}");
            assert!(
                stderr.contains("Argument type does not match the parameter type"),
                "{stderr}"
            );
            assert!(!stderr.contains("SES-RUST-0001"), "{stderr}");
            assert!(!out.exists());
            assert!(!fixture.path().join(".seseragi/native").exists());
        }
    }
}

#[test]
fn queue_accepts_standard_unit_under_shadow_in_strict_ts_and_both_engines() {
    let fixture = Fixture::new("import * as queue from \"std/queue\"\ntype Unit = | FakeUnit\nimport * as effects from \"std/effect\"\npub effect fn main = do { queue.unbounded<Int> (); effects.succeed () }\n");
    fixture.typescript(b"");
    let out = fixture.path().join("native");
    let result = bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(fixture.path())
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        fixture.path(),
    );
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("SES-RUST-0001"), "{stderr}");
    assert!(stderr.contains("std/queue::unbounded"), "{stderr}");
    assert!(!stderr.contains("SES-T0101"), "{stderr}");
    assert!(!out.exists());
}

#[test]
fn standard_unit_aliases_and_saved_calls_match_ts_and_native_execution() {
    for (declarations, binding) in [
        (
            "import { StandardUnit } from \"./facade\"\ntype Unit = | FakeUnit\nfn done -> StandardUnit = ()",
            "let unit: StandardUnit = done ()",
        ),
        (
            "import { Unit, StandardUnit, done } from \"./facade\"",
            "let unit: StandardUnit = done ()",
        ),
        (
            "import { StandardUnit, done } from \"./facade\"\nalias Unit = StandardUnit",
            "let unit: Unit = done ()",
        ),
    ] {
        let source = format!("import * as array from \"std/array\"\nimport * as list from \"std/list\"\nimport * as effects from \"std/effect\"\n{declarations}\npub effect fn main = do {{\n  {binding}\n  effects.yieldNow unit\n  let saved = array.empty<Int>\n  println (show (saved unit))\n  println (show (array.length (array.empty<Unit> ())))\n  println (show (list.length (list.empty<Int> unit)))\n}}\n");
        let fixture = Fixture::new(&source);
        let expected = b"[]\n0\n0\n";
        fixture.typescript(expected);
        let result = successful(bounded(
            Command::new(fixture.native())
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            fixture.path(),
        ));
        assert_eq!(result.stdout, expected);
        assert!(String::from_utf8_lossy(&result.stderr).contains("managed-live=0"));
    }
}
