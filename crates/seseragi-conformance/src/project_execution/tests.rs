use super::model::{load_project_execution_case, ProjectExecutionKind};
use crate::execution::{Invocation, InvocationArgument};
use crate::execution_case::environment::HostAdapter;
use std::fs;
use std::path::PathBuf;

#[path = "tests/discovery.rs"]
mod discovery;

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

#[test]
fn rejects_unknown_fields_at_every_project_execution_value_boundary() {
    let boundaries = [
        ("invocation", "/invocation", "invocation"),
        ("effect-mode", "/invocation/effect", "invocation.effect"),
        (
            "typed-argument",
            "/invocation/arguments/0",
            "invocation.arguments[0]",
        ),
        (
            "required-environment",
            "/requiredEnvironment",
            "requiredEnvironment",
        ),
        (
            "required-field",
            "/requiredEnvironment/fields/0",
            "requiredEnvironment.fields[0]",
        ),
        ("host-environment", "/hostEnvironment", "hostEnvironment"),
        (
            "host-service",
            "/hostEnvironment/services/0",
            "hostEnvironment.services[0]",
        ),
        ("exit", "/expected/exit", "expected.exit"),
    ];

    for (name, pointer, label) in boundaries {
        assert_unknown_nested_field(name, pointer, label);
    }
}

#[test]
fn validates_trace_event_keys_instead_of_only_counting_them() {
    let case = temp_case("unknown-trace-event");
    let mut descriptor = valid_effect_descriptor();
    let event = descriptor
        .pointer_mut("/expected/trace/0")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    event.remove("operation");
    event.insert("surprise".to_owned(), serde_json::json!(true));
    fs::write(
        case.join("execution.json"),
        serde_json::to_vec_pretty(&descriptor).unwrap(),
    )
    .unwrap();

    let error = load_project_execution_case(&case).unwrap_err();
    assert!(error.contains("expected.trace[0] has unknown field `surprise`"));
    fs::remove_dir_all(case).unwrap();
}

fn assert_unknown_nested_field(name: &str, pointer: &str, label: &str) {
    let case = temp_case(&format!("unknown-{name}"));
    let mut descriptor = valid_effect_descriptor();
    descriptor
        .pointer_mut(pointer)
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .insert("surprise".to_owned(), serde_json::json!(true));
    fs::write(
        case.join("execution.json"),
        serde_json::to_vec_pretty(&descriptor).unwrap(),
    )
    .unwrap();

    let error = load_project_execution_case(&case).unwrap_err();
    assert!(
        error.contains(&format!("{label} has unknown field `surprise`")),
        "unexpected error for {label}: {error}"
    );
    fs::remove_dir_all(case).unwrap();
}

fn valid_effect_descriptor() -> serde_json::Value {
    serde_json::json!({
        "schema": 1,
        "kind": "effect-project-execution",
        "target": "node-process",
        "entry": { "module": "fixture/main", "export": "main" },
        "invocation": {
            "arguments": [{ "type": "Unit" }],
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
    })
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
