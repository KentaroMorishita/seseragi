use std::{fs, path::Path, process::Command};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

const OPERATIONS: &str = "unbounded, bounded, offer, take, tryOffer, tryTake, size, close";
const TRANSFER: &str = r#"
import * as effects from "std/effect"
effect fn transfer = do {
  q <- queue.unbounded ()
  empty <- queue.tryTake q
  queue.offer 1 q
  queue.offer 2 q
  count <- queue.size q
  first <- queue.take q
  queue.close q
  second <- queue.take q
  closed <- queue.tryTake q
  rejected <- queue.tryOffer 3 q
  failedOffer <- effects.attempt (queue.offer 4 q)
  failedTake <- effects.attempt (queue.take q)
  queue.close q
  succeed [show empty, show count, show first, show second, show closed,
    show rejected, show failedOffer, show failedTake]
}
pub effect fn main = do {
  outcome <- effects.attempt (transfer ())
  println (show outcome)
}
"#;

struct Fixture(tempfile::TempDir);
impl Fixture {
    fn new(import: &str, body: &str) -> Self {
        let directory = tempfile::Builder::new()
            .prefix("seseragi-queue-inference-")
            .tempdir()
            .unwrap();
        let root = directory.path();
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("seseragi.toml"), "[package]\nname = \"fixture/queue-inference\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
        fs::write(root.join("src/main.ssrg"), format!("{import}\n{body}")).unwrap();
        fs::write(
            root.join("src/api.ssrg"),
            format!("pub import {{ {OPERATIONS} }} from \"std/queue\"\n"),
        )
        .unwrap();
        fs::write(root.join("src/facade.ssrg"), "pub import { unbounded as create, bounded, offer as put, take, tryOffer, tryTake, size, close } from \"./api\"\n").unwrap();
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
        let native = seseragi_driver::compile_core_local_project(&loaded).unwrap();
        for (name, profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let out = self.path().join(name);
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(self.path())
                    .args(["--target", "process", "--profile", name, "--out-dir"])
                    .arg(&out),
                self.path(),
            ));
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                    .unwrap();
            let mut files = Vec::new();
            for (id, module) in &compiled.compiled.modules {
                assert_eq!(module.core_ir, native.compiled.modules[id].core_ir);
                let file = out
                    .join("raw")
                    .join(&module.generated.metadata.outputs.typescript);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
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
                let output = successful(bounded(
                    Command::new(engine)
                        .arg(out.join(manifest["entry"].as_str().unwrap()))
                        .env("PATH", ""),
                    self.path(),
                ));
                assert_eq!(output.stdout, expected, "{name}/{engine:?}");
                assert!(output.stderr.is_empty(), "{output:?}");
            }
        }
    }

    fn native_queue_is_unsupported(&self) {
        let out = self.path().join("native");
        let output = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(self.path())
                .args(["--entry", "main", "--out-dir"])
                .arg(&out),
            self.path(),
        );
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("SES-RUST-0001"), "{stderr}");
        assert!(stderr.contains("std/queue::unbounded"), "{stderr}");
        assert!(!stderr.contains("SES-T0101"), "{stderr}");
        assert!(!out.exists());
        assert!(!self.path().join(".seseragi/native").exists());
    }
}

#[test]
fn direct_and_named_reexport_queues_keep_fifo_and_closed_errors_in_bun_and_node() {
    for (import, renamed) in [
        ("import * as queue from \"std/queue\"", false),
        ("import * as queue from \"./api\"", false),
        ("import * as queue from \"./facade\"", true),
    ] {
        let body = if renamed {
            TRANSFER
                .replace("queue.unbounded", "queue.create")
                .replace("queue.offer", "queue.put")
        } else {
            TRANSFER.to_owned()
        };
        let fixture = Fixture::new(import, &body);
        fixture.typescript(b"Right [Right Nothing, 2, 1, 2, Left QueueClosed, Left QueueClosed, Left QueueClosed, Left QueueClosed]\n");
        fixture.native_queue_is_unsupported();
    }
}

#[test]
fn named_reexport_preserves_explicit_queue_effects_and_rejects_wrong_elements() {
    for import in [
        "import * as queue from \"std/queue\"",
        "import * as queue from \"./api\"",
    ] {
        let fixture = Fixture::new(
            import,
            "import * as effects from \"std/effect\"\npub effect fn main = do { q <- queue.unbounded<Int> (); effects.attempt (queue.offer 1 q); succeed () }\n",
        );
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("lint")
                .arg(fixture.path()),
            fixture.path(),
        ));
        fixture.typescript(b"");
        fixture.native_queue_is_unsupported();
        fs::write(fixture.path().join("src/main.ssrg"), format!("{import}\npub effect fn main = do {{ q <- queue.unbounded<Int> (); queue.offer \"bad\" q }}\n")).unwrap();
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["lock", "update"])
                .arg(fixture.path()),
            fixture.path(),
        ));
        let mut errors = Vec::new();
        for native in [false, true] {
            let out = fixture.path().join(if native {
                "rejected-native"
            } else {
                "rejected-ts"
            });
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
            let output = bounded(command.arg("--out-dir").arg(&out), fixture.path());
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("error[SES-T0101]"), "{stderr}");
            assert!(
                stderr.contains("expected Queue<String>, received Queue<Int>"),
                "{stderr}"
            );
            assert!(!stderr.contains("SES-RUST-0001"), "{stderr}");
            assert!(!out.exists());
            errors.push(
                stderr
                    .strip_prefix("seseragi: ")
                    .unwrap_or(&stderr)
                    .trim_end()
                    .to_owned(),
            );
        }
        assert_eq!(errors[0], errors[1]);
    }
}

#[test]
fn bounded_queues_keep_capacity_empty_and_invalid_capacity_results_after_reexport() {
    for import in [
        "import * as queue from \"std/queue\"",
        "import * as queue from \"./api\"",
    ] {
        let fixture = Fixture::new(
            import,
            r#"
import * as effects from "std/effect"
effect fn capacity = do {
  q <- queue.bounded 1
  accepted <- queue.tryOffer 7 q
  full <- queue.tryOffer 8 q
  first <- queue.tryTake q
  empty <- queue.tryTake q
  queue.close q
  closed <- queue.tryTake q
  succeed [show accepted, show full, show first, show empty, show closed]
}
pub effect fn main = do {
  values <- effects.attempt (capacity ())
  println (show values)
  zero <- effects.attempt (queue.bounded<Int> 0)
  negative <- effects.attempt (queue.bounded<Int> (-1))
  println (match zero { Left error -> show error; Right _ -> "unexpected success" })
  println (match negative { Left error -> show error; Right _ -> "unexpected success" })
}
"#,
        );
        fixture.typescript(b"Right [Right True, Right False, Right Just 7, Right Nothing, Left QueueClosed]\nNonPositiveCapacity 0\nNonPositiveCapacity -1\n");
    }
}

#[test]
fn do_inference_for_generic_effect_results_matches_ts_and_native() {
    let fixture = Fixture::new(
        "",
        r#"
effect fn empty<A> = { let values: Array<A> = []; succeed values }
pub effect fn main = do {
  ints <- empty ()
  strings <- empty ()
  let a: Array<Int> = ints
  let b: Array<String> = strings
  println (show a)
  println (show b)
}
"#,
    );
    fixture.typescript(b"[]\n[]\n");
    let out = fixture.path().join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(fixture.path())
            .args(["--entry", "main", "--out-dir"])
            .arg(&out),
        fixture.path(),
    ));
    let output = successful(bounded(
        Command::new(out.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        fixture.path(),
    ));
    assert_eq!(output.stdout, b"[]\n[]\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}
