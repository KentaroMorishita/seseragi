use super::model::{load_project_execution_case, ProjectExecutionKind};
use crate::execution::{Invocation, InvocationArgument};
use crate::execution_case::environment::HostAdapter;
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
fn loads_an_effect_project_execution_envelope_with_shared_observations() {
    let case = temp_case("effect");
    fs::write(
        case.join("execution.json"),
        r#"{
  "schema": 1,
  "kind": "effect-project-execution",
  "target": "node-process",
  "entry": { "module": "fixture/main", "export": "main" },
  "invocation": {
    "argument": "Unit",
    "effect": { "cold": true, "rootScope": true }
  },
  "requiredEnvironment": {
    "kind": "record",
    "closed": true,
    "fields": [{ "name": "console", "type": "Console" }]
  },
  "hostEnvironment": {
    "closed": false,
    "services": [{
      "field": "console",
      "type": "Console",
      "adapter": "capture-console"
    }]
  },
  "expected": {
    "exit": { "kind": "success", "value": "Unit" },
    "trace": [{
      "service": "console",
      "operation": "println",
      "arguments": ["hello"],
      "stdout": "hello\n"
    }],
    "runtimeRequirements": ["effect.console.println"],
    "process": { "exitCode": 0 },
    "stdout": "stdout.txt",
    "stderr": "stderr.txt"
  }
}"#,
    )
    .unwrap();

    let loaded = load_project_execution_case(&case).unwrap();
    assert_eq!(loaded.kind, ProjectExecutionKind::Effect);
    assert_eq!(loaded.invocation, super::model::unit_effect_invocation());
    assert_eq!(
        loaded
            .required_environment
            .get("console")
            .map(String::as_str),
        Some("Console")
    );
    assert_eq!(
        loaded.environment.bindings()[0].adapter(),
        HostAdapter::CaptureConsole
    );
    assert_eq!(
        loaded
            .expected_effect_exit
            .as_ref()
            .and_then(|exit| exit.get("kind"))
            .and_then(serde_json::Value::as_str),
        Some("success")
    );
    assert!(loaded.expected_operation_trace.is_some());
    fs::remove_dir_all(case).unwrap();
}

#[test]
fn requires_trace_when_effect_project_captures_console() {
    let case = temp_case("missing-trace");
    fs::write(
        case.join("execution.json"),
        r#"{
  "schema": 1,
  "kind": "effect-project-execution",
  "target": "node-process",
  "entry": { "module": "fixture/main", "export": "main" },
  "invocation": {
    "argument": "Unit",
    "effect": { "cold": true, "rootScope": true }
  },
  "requiredEnvironment": {
    "kind": "record",
    "closed": true,
    "fields": [{ "name": "console", "type": "Console" }]
  },
  "hostEnvironment": {
    "closed": false,
    "services": [{
      "field": "console",
      "type": "Console",
      "adapter": "capture-console"
    }]
  },
  "expected": {
    "exit": { "kind": "success", "value": "Unit" },
    "runtimeRequirements": ["effect.console.println"],
    "process": { "exitCode": 0 },
    "stdout": "stdout.txt",
    "stderr": "stderr.txt"
  }
}"#,
    )
    .unwrap();

    assert!(load_project_execution_case(&case)
        .unwrap_err()
        .contains("capture-console requires expected.trace"));
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
