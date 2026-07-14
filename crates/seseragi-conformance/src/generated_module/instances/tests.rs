use serde_json::json;

use super::check_generated_instances;

fn metadata(instances: serde_json::Value) -> serde_json::Value {
    json!({
        "exports": ["main"],
        "instances": instances,
    })
}

fn show_instance(type_identity: &str, dictionary_export: &str) -> serde_json::Value {
    json!({
        "identity": format!("Show<{type_identity}>"),
        "trait": "Show",
        "arguments": [{
          "kind": "reference",
          "name": "AppError",
          "arguments": [],
        }],
        "typeIdentity": type_identity,
        "dictionaryExport": dictionary_export,
    })
}

#[test]
fn accepts_absent_and_empty_instance_metadata() {
    assert!(check_generated_instances(&json!({ "exports": [] }), "").is_ok());
    assert!(check_generated_instances(&metadata(json!([])), "").is_ok());
}

#[test]
fn accepts_internal_dictionary_exports_backed_by_typescript_consts() {
    let dictionary = "__ssrg$instance$Show$0";
    let generated = metadata(json!([show_instance(
        "artifact/example::AppError",
        dictionary
    )]));
    let typescript = format!(
        "export const main = () => undefined;\nexport const {dictionary}: Show<AppError> = {{ show: value => value.tag }};\n"
    );

    check_generated_instances(&generated, &typescript).unwrap();
}

#[test]
fn accepts_multi_parameter_instance_heads_without_a_primary_type() {
    let dictionary = "__ssrg$instance$Add$0";
    let generated = metadata(json!([{
        "identity": "Add<Vector,Float,Vector>",
        "trait": "Add",
        "arguments": [
            { "kind": "reference", "name": "Vector", "arguments": [] },
            { "kind": "float" },
            { "kind": "reference", "name": "Vector", "arguments": [] }
        ],
        "dictionaryExport": dictionary,
    }]));
    let typescript = format!("export const {dictionary} = {{}};\n");

    check_generated_instances(&generated, &typescript).unwrap();
}

#[test]
fn rejects_malformed_instance_and_argument_shapes() {
    assert_eq!(
        check_generated_instances(&json!({ "instances": {} }), "").unwrap_err(),
        "generated module instances must be an array"
    );
    assert!(check_generated_instances(&metadata(json!([null])), "")
        .unwrap_err()
        .contains("instances[0] must be an object"));

    let missing_arguments = json!({
        "identity": "Show<artifact/example::AppError>",
        "trait": "Show",
        "typeIdentity": "artifact/example::AppError",
        "dictionaryExport": "showAppError",
    });
    assert!(
        check_generated_instances(&metadata(json!([missing_arguments])), "")
            .unwrap_err()
            .contains("arguments must be an array")
    );

    let missing_argument_kind = json!({
        "identity": "Show<artifact/example::AppError>",
        "trait": "Show",
        "arguments": [{}],
        "typeIdentity": "artifact/example::AppError",
        "dictionaryExport": "showAppError",
    });
    assert!(
        check_generated_instances(&metadata(json!([missing_argument_kind])), "")
            .unwrap_err()
            .contains("arguments[0].kind must be a non-empty string")
    );
}

#[test]
fn rejects_missing_string_fields() {
    for (field, expected) in [
        ("identity", ".identity must be a non-empty string"),
        ("trait", ".trait must be a non-empty string"),
        ("typeIdentity", ".typeIdentity must be a non-empty string"),
        (
            "dictionaryExport",
            ".dictionaryExport must be a non-empty string",
        ),
    ] {
        let mut instance = show_instance("artifact/example::AppError", "showAppError");
        instance.as_object_mut().unwrap().remove(field);
        let error = check_generated_instances(&metadata(json!([instance])), "").unwrap_err();
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn rejects_duplicate_instance_identities() {
    let generated = metadata(json!([
        show_instance("artifact/example::AppError", "showAppError1"),
        show_instance("artifact/example::AppError", "showAppError2")
    ]));
    let error = check_generated_instances(
        &generated,
        "export const showAppError1 = {};\nexport const showAppError2 = {};\n",
    )
    .unwrap_err();

    assert!(error.contains("duplicate instance metadata identity"));
}

#[test]
fn rejects_unsafe_or_reserved_dictionary_export_names() {
    for name in ["bad-name;inject()", "class"] {
        let generated = metadata(json!([show_instance("artifact/example::AppError", name)]));
        let error = check_generated_instances(&generated, "").unwrap_err();
        assert!(error.contains("compiler-safe TypeScript identifier"));
    }
}

#[test]
fn requires_an_exact_export_const_in_typescript() {
    let generated = metadata(json!([show_instance(
        "artifact/example::AppError",
        "showAppError"
    )]));

    let missing = check_generated_instances(
        &generated,
        "const showAppError = {};\nexport const showAppErrorOther = {};\n",
    )
    .unwrap_err();

    assert!(missing.contains("missing from TypeScript output"));
}

#[test]
fn keeps_dictionary_exports_out_of_source_public_exports() {
    let dictionary = "showAppError";
    let generated = json!({
        "exports": ["main", dictionary],
        "instances": [show_instance("artifact/example::AppError", dictionary)],
    });
    let error = check_generated_instances(
        &generated,
        "export const showAppError: Show<AppError> = { show: value => value.tag };\n",
    )
    .unwrap_err();

    assert!(error.contains("must not appear in source-public metadata.exports"));
}
