use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_typescript_abi_case(case: &Path) -> Result<(), String> {
    let path = case.join("abi.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider TypeScript ABI: {error}"))?;
    check_provider_typescript_abi(&raw)
}

fn check_provider_typescript_abi(raw: &str) -> Result<(), String> {
    let abi: ProviderTypescriptAbi = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider TypeScript ABI: {error}"))?;
    if abi.schema != 1
        || abi.kind != "provider-runtime-abi"
        || abi.identity != "seseragi/provider-abi/typescript"
        || abi.abi_major != 1
        || abi.backend != "typescript"
    {
        return Err("provider TypeScript ABI envelope must identify ABI v1".to_owned());
    }
    check_value_mappings(&abi.values)?;
    check_presence(&abi.presence)?;
    check_call(&abi.call)?;
    check_defect(&abi.defect)?;
    check_handle(&abi.handle)?;
    check_examples(&abi.examples)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderTypescriptAbi {
    schema: u32,
    kind: String,
    identity: String,
    #[serde(rename = "abiMajor")]
    abi_major: u32,
    backend: String,
    values: Vec<ValueMapping>,
    presence: Presence,
    call: Call,
    defect: Defect,
    handle: Handle,
    examples: Vec<ProjectionExample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValueMapping {
    #[serde(rename = "logicalKind")]
    logical_kind: String,
    typescript: String,
    ownership: String,
    validation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presence {
    null: String,
    undefined: String,
    missing: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Call {
    entry: String,
    input: String,
    completion: String,
    result: ResultEnvelope,
    #[serde(rename = "synchronousThrow")]
    synchronous_throw: String,
    #[serde(rename = "promiseRejection")]
    promise_rejection: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultEnvelope {
    #[serde(rename = "tagField")]
    tag_field: String,
    success: OutcomeVariant,
    failure: OutcomeVariant,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutcomeVariant {
    tag: String,
    payload: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Defect {
    source: String,
    tag: String,
    payload: String,
    #[serde(rename = "metadataFields")]
    metadata_fields: Vec<String>,
    stages: Vec<String>,
    cause: String,
    #[serde(rename = "invalidValue")]
    invalid_value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Handle {
    shape: String,
    token: String,
    owner: String,
    service: String,
    #[serde(rename = "handleType")]
    handle_type: String,
    transfer: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionExample {
    capability: String,
    operation: String,
    input: String,
    success: String,
    failure: String,
}

fn check_value_mappings(mappings: &[ValueMapping]) -> Result<(), String> {
    let expected = [
        ("unit", "undefined", "value", "exact-undefined"),
        ("never", "never", "value", "uninhabited"),
        ("bool", "boolean", "value", "exact-boolean"),
        ("bytes", "Uint8Array", "snapshot-copy", "exact-uint8-array"),
        ("float", "number", "value", "exact-number"),
        (
            "int",
            "number",
            "value",
            "safe-integer-normalize-negative-zero",
        ),
        ("string", "string", "value", "exact-string"),
        (
            "array",
            "ReadonlyArray<unknown>",
            "snapshot-copy",
            "recursive-elements",
        ),
        (
            "record",
            "Readonly<Record<string, unknown>>",
            "snapshot-copy",
            "closed-own-properties",
        ),
        (
            "named",
            "unknown",
            "codec-owned",
            "registered-canonical-codec",
        ),
    ];
    if mappings.len() != expected.len() {
        return Err("provider TypeScript ABI must define every v1 logical value kind".to_owned());
    }
    let mut seen = BTreeSet::new();
    for mapping in mappings {
        if !seen.insert(mapping.logical_kind.as_str()) {
            return Err(format!(
                "provider TypeScript ABI logical value kind is duplicated: {}",
                mapping.logical_kind
            ));
        }
        let Some(expected_mapping) = expected
            .iter()
            .find(|expected| expected.0 == mapping.logical_kind)
        else {
            return Err(format!(
                "provider TypeScript ABI logical value kind is unknown: {}",
                mapping.logical_kind
            ));
        };
        if mapping.typescript != expected_mapping.1
            || mapping.ownership != expected_mapping.2
            || mapping.validation != expected_mapping.3
        {
            return Err(format!(
                "provider TypeScript ABI mapping is not canonical: {}",
                mapping.logical_kind
            ));
        }
    }
    Ok(())
}

fn check_presence(presence: &Presence) -> Result<(), String> {
    if presence.null != "invalid-unless-named-codec"
        || presence.undefined != "unit-only"
        || presence.missing != "invalid-required-record-field"
    {
        return Err(
            "provider TypeScript ABI null, undefined, and missing rules must stay distinct"
                .to_owned(),
        );
    }
    Ok(())
}

fn check_call(call: &Call) -> Result<(), String> {
    if call.entry != "readonly-object-export"
        || call.input != "single-encoded-value"
        || call.completion != "promise"
        || call.synchronous_throw != "bridge-defect"
        || call.promise_rejection != "bridge-defect"
        || call.result.tag_field != "kind"
        || call.result.success.tag != "success"
        || call.result.success.payload != "value"
        || call.result.failure.tag != "failure"
        || call.result.failure.payload != "failure"
    {
        return Err("provider TypeScript ABI call protocol is not canonical".to_owned());
    }
    Ok(())
}

fn check_defect(defect: &Defect) -> Result<(), String> {
    if defect.source != "bridge-only"
        || defect.tag != "defect"
        || defect.payload != "defect"
        || defect.metadata_fields
            != [
                "provider",
                "service",
                "operation",
                "stage",
                "message",
                "cause",
            ]
        || defect.stages != ["input", "call", "result"]
        || defect.cause != "preserved-unknown"
        || defect.invalid_value != "provider-boundary-defect"
    {
        return Err("provider TypeScript ABI defect bridge is not canonical".to_owned());
    }
    Ok(())
}

fn check_handle(handle: &Handle) -> Result<(), String> {
    if handle.shape != "readonly-branded-object"
        || handle.token != "opaque-host-object"
        || handle.owner != "provider-identity"
        || handle.service != "canonical-service-identity"
        || handle.handle_type != "canonical-type-identity"
        || handle.transfer != "forbidden-unless-contract-declares"
    {
        return Err("provider TypeScript ABI handle metadata is not canonical".to_owned());
    }
    Ok(())
}

fn check_examples(examples: &[ProjectionExample]) -> Result<(), String> {
    let mut capabilities = BTreeSet::new();
    for example in examples {
        if !capabilities.insert(example.capability.as_str()) {
            return Err(format!(
                "provider TypeScript ABI example is duplicated: {}",
                example.capability
            ));
        }
        if example.operation.is_empty()
            || example.input.is_empty()
            || example.success.is_empty()
            || example.failure.is_empty()
        {
            return Err("provider TypeScript ABI examples must define every projection".to_owned());
        }
    }
    let expected = BTreeSet::from([
        "clock",
        "filesystem",
        "http",
        "navigation",
        "postgresql",
        "storage",
    ]);
    if capabilities != expected {
        return Err("provider TypeScript ABI must cover every canonical projection".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_typescript_abi, check_provider_typescript_abi_case};
    use std::path::PathBuf;

    fn fixture() -> &'static str {
        include_str!(
            "../../../examples/spec/artifacts/provider-typescript-abi-schema-1/core/abi.json"
        )
    }

    #[test]
    fn accepts_committed_provider_typescript_abi() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-typescript-abi-schema-1/core");
        check_provider_typescript_abi_case(&case).unwrap();
    }

    #[test]
    fn rejects_unknown_fields() {
        let raw = fixture().replace("\"backend\":", "\"dynamic\": true, \"backend\":");
        assert!(check_provider_typescript_abi(&raw)
            .unwrap_err()
            .contains("unknown field `dynamic`"));
    }

    #[test]
    fn rejects_collapsed_presence_rules() {
        let raw = fixture().replace(
            "\"undefined\": \"unit-only\"",
            "\"undefined\": \"invalid-unless-named-codec\"",
        );
        assert!(check_provider_typescript_abi(&raw)
            .unwrap_err()
            .contains("must stay distinct"));
    }

    #[test]
    fn rejects_sync_return_and_exception_as_typed_failure() {
        let raw = fixture()
            .replace(
                "\"completion\": \"promise\"",
                "\"completion\": \"sync-or-promise\"",
            )
            .replace(
                "\"synchronousThrow\": \"bridge-defect\"",
                "\"synchronousThrow\": \"typed-failure\"",
            );
        assert!(check_provider_typescript_abi(&raw)
            .unwrap_err()
            .contains("call protocol"));
    }

    #[test]
    fn rejects_shared_mutable_bytes() {
        let raw = fixture().replace(
            "\"ownership\": \"snapshot-copy\",\n      \"validation\": \"exact-uint8-array\"",
            "\"ownership\": \"shared-view\",\n      \"validation\": \"exact-uint8-array\"",
        );
        assert!(check_provider_typescript_abi(&raw)
            .unwrap_err()
            .contains("mapping is not canonical: bytes"));
    }

    #[test]
    fn rejects_handle_transfer_without_contract() {
        let raw = fixture().replace(
            "\"transfer\": \"forbidden-unless-contract-declares\"",
            "\"transfer\": \"always\"",
        );
        assert!(check_provider_typescript_abi(&raw)
            .unwrap_err()
            .contains("handle metadata"));
    }
}
