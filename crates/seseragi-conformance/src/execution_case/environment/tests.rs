use super::{parse_environment_plan, HostAdapter};
use serde_json::json;

fn environment(
    required_fields: serde_json::Value,
    services: serde_json::Value,
) -> serde_json::Value {
    json!({
        "requiredEnvironment": {
            "kind": "record",
            "closed": true,
            "fields": required_fields
        },
        "hostEnvironment": {
            "closed": false,
            "services": services
        }
    })
}

#[test]
fn permits_omitted_environment_only_for_pure_invocations() {
    assert!(parse_environment_plan(&json!({}), false)
        .unwrap()
        .bindings()
        .is_empty());

    let error = parse_environment_plan(&json!({}), true).unwrap_err();
    assert!(error.contains("required for Effect"));
}

#[test]
fn creates_typed_bindings_for_known_host_adapters() {
    let plan = parse_environment_plan(
        &environment(
            json!([
                { "name": "console", "type": "Console" },
                { "name": "stdin", "type": "Stdin" }
            ]),
            json!([
                { "field": "console", "type": "Console", "adapter": "capture-console" },
                { "field": "stdin", "type": "Stdin", "adapter": "process-stdin" }
            ]),
        ),
        true,
    )
    .unwrap();

    assert_eq!(plan.bindings().len(), 2);
    assert_eq!(plan.bindings()[0].field(), "console");
    assert_eq!(plan.bindings()[0].adapter(), HostAdapter::CaptureConsole);
    assert_eq!(plan.bindings()[1].adapter(), HostAdapter::ProcessStdin);
}

#[test]
fn creates_typed_bindings_for_deterministic_failure_adapters() {
    let plan = parse_environment_plan(
        &environment(
            json!([
                { "name": "console", "type": "Console" },
                { "name": "stdin", "type": "Stdin" }
            ]),
            json!([
                { "field": "console", "type": "Console", "adapter": "fail-console" },
                { "field": "stdin", "type": "Stdin", "adapter": "fail-stdin" }
            ]),
        ),
        true,
    )
    .unwrap();

    assert_eq!(plan.bindings()[0].adapter(), HostAdapter::FailConsole);
    assert_eq!(plan.bindings()[1].adapter(), HostAdapter::FailStdin);
}

#[test]
fn requires_a_closed_record_and_unique_required_fields() {
    let mut open = environment(json!([]), json!([]));
    open["requiredEnvironment"]["closed"] = json!(false);
    assert!(parse_environment_plan(&open, true)
        .unwrap_err()
        .contains("must be closed"));

    let duplicate = environment(
        json!([
            { "name": "console", "type": "Console" },
            { "name": "console", "type": "Console" }
        ]),
        json!([]),
    );
    assert!(parse_environment_plan(&duplicate, true)
        .unwrap_err()
        .contains("duplicate field console"));
}

#[test]
fn rejects_duplicate_missing_and_mistyped_host_services() {
    let duplicate = environment(
        json!([{ "name": "console", "type": "Console" }]),
        json!([
            { "field": "console", "type": "Console", "adapter": "capture-console" },
            { "field": "console", "type": "Console", "adapter": "capture-console" }
        ]),
    );
    assert!(parse_environment_plan(&duplicate, true)
        .unwrap_err()
        .contains("duplicate service field console"));

    let missing = environment(json!([{ "name": "stdin", "type": "Stdin" }]), json!([]));
    assert!(parse_environment_plan(&missing, true)
        .unwrap_err()
        .contains("missing required service stdin"));

    let mistyped = environment(
        json!([{ "name": "console", "type": "Console" }]),
        json!([{
            "field": "console",
            "type": "Stdin",
            "adapter": "process-stdin"
        }]),
    );
    assert!(parse_environment_plan(&mistyped, true)
        .unwrap_err()
        .contains("required type is Console"));
}

#[test]
fn rejects_unknown_or_type_incompatible_adapters() {
    let unknown = environment(
        json!([{ "name": "console", "type": "Console" }]),
        json!([{
            "field": "console",
            "type": "Console",
            "adapter": "mystery-console"
        }]),
    );
    assert!(parse_environment_plan(&unknown, true)
        .unwrap_err()
        .contains("not supported"));

    let incompatible = environment(
        json!([{ "name": "console", "type": "Console" }]),
        json!([{
            "field": "console",
            "type": "Console",
            "adapter": "process-stdin"
        }]),
    );
    assert!(parse_environment_plan(&incompatible, true)
        .unwrap_err()
        .contains("expected Stdin"));

    let failing_incompatible = environment(
        json!([{ "name": "stdin", "type": "Stdin" }]),
        json!([{
            "field": "stdin",
            "type": "Stdin",
            "adapter": "fail-console"
        }]),
    );
    assert!(parse_environment_plan(&failing_incompatible, true)
        .unwrap_err()
        .contains("expected Console"));
}

#[test]
fn permits_extra_services_only_in_an_open_host_environment() {
    let mut value = environment(
        json!([]),
        json!([{
            "field": "console",
            "type": "Console",
            "adapter": "capture-console"
        }]),
    );
    parse_environment_plan(&value, true).unwrap();

    value["hostEnvironment"]["closed"] = json!(true);
    assert!(parse_environment_plan(&value, true)
        .unwrap_err()
        .contains("extra service field console"));
}
