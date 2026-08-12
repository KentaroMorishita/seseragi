use super::{LogicalType, OperationKind, Portability, ProviderContract, ServiceRequirement};
use serde_json::{json, Value};
use std::path::PathBuf;

fn valid_contract() -> Value {
    json!({
        "schema": 1,
        "kind": "provider-contract",
        "identity": "std/clock::Clock",
        "version": { "major": 1, "minor": 0 },
        "requirement": { "field": "clock", "type": "std/clock::Clock" },
        "operations": [{
            "identity": "std/clock::Clock#now",
            "kind": "one-shot",
            "input": { "kind": "unit" },
            "success": { "kind": "named", "identity": "std/time::Instant" },
            "failure": { "kind": "never" },
            "portability": { "kind": "portable" },
            "summary": "Observe the current monotonic instant."
        }]
    })
}

fn parse(value: &Value) -> Result<ProviderContract, String> {
    ProviderContract::from_json(&value.to_string()).map_err(|error| error.to_string())
}

#[test]
fn reads_committed_contracts_as_typed_operation_metadata() {
    let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/artifacts/provider-contract-schema-1");
    for case in [
        "clock",
        "filesystem",
        "http-client",
        "http-server",
        "bun-http-extension",
    ] {
        let raw = std::fs::read_to_string(artifacts.join(case).join("contract.json")).unwrap();
        ProviderContract::from_json(&raw).unwrap();
    }

    let clock = ProviderContract::from_json(
        &std::fs::read_to_string(artifacts.join("clock/contract.json")).unwrap(),
    )
    .unwrap();
    let sleep = clock.operation("std/clock::Clock#sleep").unwrap();
    assert_eq!(sleep.kind, OperationKind::OneShot);
    assert_eq!(
        sleep.input,
        LogicalType::Named {
            identity: "std/time::Duration".to_owned(),
        }
    );
    assert_eq!(sleep.portability, Portability::Portable);
    assert!(clock.provides_requirement(&ServiceRequirement {
        field: "testClock".to_owned(),
        service: "std/clock::Clock".to_owned(),
    }));
}

#[test]
fn accepts_package_and_module_qualified_type_identities() {
    let mut contract = valid_contract();
    contract["identity"] = json!("acme/payments::service::Payments");
    contract["requirement"]["field"] = json!("payments");
    contract["requirement"]["type"] = json!("acme/payments::service::Payments");
    contract["operations"][0]["identity"] = json!("acme/payments::service::Payments#charge");
    contract["operations"][0]["success"] = json!({
        "kind": "named",
        "identity": "acme/payments::model::Receipt"
    });
    parse(&contract).unwrap();
}

#[test]
fn rejects_unknown_fields_at_every_closed_schema_level() {
    for pointer in [
        "",
        "/version",
        "/requirement",
        "/operations/0",
        "/operations/0/input",
    ] {
        let mut contract = valid_contract();
        contract
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("typescript".to_owned(), json!("Promise"));
        assert!(parse(&contract).unwrap_err().contains("unknown field"));
    }
}

#[test]
fn rejects_duplicate_operation_and_record_field_identities() {
    let mut contract = valid_contract();
    let duplicate = contract["operations"][0].clone();
    contract["operations"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert!(parse(&contract)
        .unwrap_err()
        .contains("operation identity is duplicated"));

    let mut contract = valid_contract();
    contract["operations"][0]["input"] = json!({
        "kind": "record",
        "fields": [
            { "name": "value", "type": { "kind": "primitive", "name": "int" } },
            { "name": "value", "type": { "kind": "primitive", "name": "int" } }
        ]
    });
    assert!(parse(&contract)
        .unwrap_err()
        .contains("field name is duplicated"));
}

#[test]
fn rejects_backend_specific_logical_types() {
    let mut contract = valid_contract();
    contract["operations"][0]["success"] = json!({
        "kind": "named",
        "identity": "typescript/promise::Promise"
    });
    assert!(parse(&contract)
        .unwrap_err()
        .contains("backend-specific namespace typescript"));
}

#[test]
fn rejects_portable_target_namespaces_and_mismatched_extensions() {
    let mut contract = valid_contract();
    contract["identity"] = json!("std/http/bun::BunHttpServer");
    contract["requirement"]["type"] = json!("std/http/bun::BunHttpServer");
    contract["operations"][0]["identity"] = json!("std/http/bun::BunHttpServer#upgradeWebSocket");
    assert!(parse(&contract)
        .unwrap_err()
        .contains("may not mark a target namespace as portable"));
    contract["operations"][0]["portability"] =
        json!({ "kind": "target-extension", "target": "node" });
    assert!(parse(&contract)
        .unwrap_err()
        .contains("must appear in the service module identity"));
}
