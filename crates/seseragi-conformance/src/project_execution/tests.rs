use super::model::load_project_execution_case;
use crate::execution::{Invocation, InvocationArgument};
use std::fs;
use std::path::PathBuf;

#[test]
fn loads_a_pure_project_execution_envelope() {
    let case = temp_case("valid");
    fs::write(
        case.join("execution.json"),
        r#"{
  "schema": 1,
  "kind": "pure-project-execution",
  "target": "node-process",
  "entry": { "module": "fixture/main", "export": "main" },
  "invocation": {
    "arguments": [{ "type": "String", "value": "rock" }],
    "pure": { "result": "json" }
  },
  "expected": {
    "runtimeRequirements": ["core.string"],
    "process": { "exitCode": 0 },
    "stdout": "stdout.txt",
    "stderr": "stderr.txt"
  }
}"#,
    )
    .unwrap();

    let loaded = load_project_execution_case(&case).unwrap();
    assert_eq!(loaded.entry_module, "fixture/main");
    assert_eq!(loaded.entry_export, "main");
    assert_eq!(
        loaded.invocation,
        Invocation::PureJson {
            arguments: vec![InvocationArgument::String("rock".to_owned())]
        }
    );
    fs::remove_dir_all(case).unwrap();
}

#[test]
fn rejects_effect_invocations_in_the_pure_schema() {
    let case = temp_case("invalid");
    fs::write(
        case.join("execution.json"),
        r#"{
  "schema": 1,
  "kind": "pure-project-execution",
  "target": "node-process",
  "entry": { "module": "fixture/main", "export": "main" },
  "invocation": {
    "argument": "Unit",
    "effect": { "cold": true, "rootScope": true }
  },
  "expected": {
    "runtimeRequirements": [],
    "process": { "exitCode": 0 },
    "stdout": "../stdout.txt",
    "stderr": "stderr.txt"
  }
}"#,
    )
    .unwrap();

    assert!(load_project_execution_case(&case)
        .unwrap_err()
        .contains("only supports pure"));
    fs::remove_dir_all(case).unwrap();
}

#[test]
fn rejects_unsafe_expected_output_paths() {
    let case = temp_case("invalid-output");
    fs::write(
        case.join("execution.json"),
        r#"{
  "schema": 1,
  "kind": "pure-project-execution",
  "target": "node-process",
  "entry": { "module": "fixture/main", "export": "main" },
  "invocation": {
    "argument": "Unit",
    "pure": { "result": "json" }
  },
  "expected": {
    "runtimeRequirements": [],
    "process": { "exitCode": 0 },
    "stdout": "../stdout.txt",
    "stderr": "stderr.txt"
  }
}"#,
    )
    .unwrap();

    assert!(load_project_execution_case(&case)
        .unwrap_err()
        .contains("expected.stdout must be a canonical relative path"));
    fs::remove_dir_all(case).unwrap();
}

fn temp_case(name: &str) -> PathBuf {
    let case = std::env::temp_dir().join(format!(
        "seseragi-project-execution-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(&case).unwrap();
    case
}
