#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn staging_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "seseragi-test-runner-integration-{}",
        std::process::id()
    ))
}

#[test]
fn reports_parallel_cases_in_discovery_order_and_replays_the_seed() {
    let directory = staging_directory();
    if directory.exists() {
        fs::remove_dir_all(&directory).unwrap();
    }
    fs::create_dir_all(&directory).unwrap();
    seseragi_runtime::stage_typescript_package(&directory).unwrap();
    let entry = directory.join("entry.ts");
    fs::write(
        &entry,
        r#"
import { milliseconds } from "@seseragi/runtime/clock"
import {
  fail,
  runTestModules,
  skip,
  suite,
  test,
  timeout,
} from "@seseragi/runtime/test"

const duration = milliseconds(2)
if (duration.tag !== "Right") throw new Error("duration fixture failed")
const tests = suite("suite", [
  test("slow", async () => { await new Promise((resolve) => setTimeout(resolve, 20)) }),
  test("fast", () => undefined),
  test("seed", async (environment) => {
    const value = await environment.random.nextInt({ cancelled: false })
    await environment.console.println(String(value))
  }),
  skip("later", test("skipped", () => undefined)),
  test("typed", fail("boom")),
  test("defect", () => { throw new Error("kaboom") }),
  timeout(duration.value, test("timed", () => new Promise(() => undefined))),
  timeout(duration.value, test("leak", (_environment, context) => {
    context?.onCancel(() => new Promise(() => undefined))
    return new Promise(() => undefined)
  })),
])

process.exitCode = await runTestModules(
  [{ name: "module", tests }],
  { jobs: 4, timeoutMs: 100, cleanupGraceMs: 10, seed: 42 }
)
"#,
    )
    .unwrap();

    let run = || {
        Command::new("bun")
            .arg("run")
            .arg(&entry)
            .current_dir(&directory)
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stdout = String::from_utf8(first.stdout).unwrap();
    assert_eq!(
        stdout,
        concat!(
            "PASS module::suite::slow\n",
            "PASS module::suite::fast\n",
            "PASS module::suite::seed\n",
            "SKIP module::suite::skipped -- later\n",
            "FAIL module::suite::typed\n",
            "FAIL module::suite::defect\n",
            "FAIL module::suite::timed\n",
            "FAIL module::suite::leak\n",
            "3 passed; 4 failed; 1 skipped\n",
        )
    );
    let stderr = String::from_utf8(first.stderr).unwrap();
    assert!(stderr.contains("module::suite::typed: boom"));
    assert!(stderr.contains("module::suite::defect: defect: kaboom"));
    assert!(stderr.contains("module::suite::timed: timed out after 2 ms"));
    assert!(stderr.contains("module::suite::leak: resource leak after timeout"));

    fs::remove_dir_all(directory).unwrap();
}
