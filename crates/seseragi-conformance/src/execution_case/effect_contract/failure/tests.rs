use serde_json::json;
use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{
    ByteSpan, InterfaceExport, InterfaceInstance, InterfaceScheme, InterfaceType, Visibility,
};

use super::resolve_effect_entry_contract;
use crate::execution_case::effect_contract::model::{
    DictionaryImport, EffectEntryContract, FailureRenderer,
};

fn named(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn type_export(name: &str) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("artifact/example::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility: Visibility::Public,
        declaration_kind: Some("type".to_owned()),
        declaration: ByteSpan { start: 0, end: 8 },
        scheme: InterfaceScheme {
            type_parameters: Vec::new(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: 0,
            },
        },
        representation: None,
    }
}

fn show_instance(type_ref: InterfaceType) -> InterfaceInstance {
    InterfaceInstance {
        trait_name: "Show".to_owned(),
        type_parameters: Vec::new(),
        head: InterfaceType::Apply {
            constructor: "Show".to_owned(),
            arguments: vec![type_ref],
        },
        constraints: Vec::new(),
        origin: ByteSpan { start: 0, end: 8 },
    }
}

fn interface_with_exports(
    exports: Vec<InterfaceExport>,
    instances: Vec<InterfaceInstance>,
) -> TypedModuleInterface {
    TypedModuleInterface {
        schema: 1,
        stage: "typed-interface".to_owned(),
        module: "artifact/example".to_owned(),
        source: "main.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports,
        operators: Vec::new(),
        instances,
    }
}

fn interface(instances: Vec<InterfaceInstance>) -> TypedModuleInterface {
    interface_with_exports(vec![type_export("AppError")], instances)
}

fn generated_instances(instances: serde_json::Value) -> serde_json::Value {
    json!({
        "schema": 1,
        "module": "artifact/example",
        "instances": instances,
    })
}

fn generated_show(dictionary_export: &str) -> serde_json::Value {
    json!({
        "trait": "Show",
        "head": { "kind": "reference", "name": "AppError", "arguments": [] },
        "typeIdentity": "artifact/example::AppError",
        "dictionaryExport": dictionary_export,
    })
}

#[test]
fn leaves_never_without_a_dictionary_contract() {
    let typed = interface(Vec::new());
    let generated = generated_instances(json!([]));

    assert_eq!(
        resolve_effect_entry_contract(&typed, &named("Never"), &generated).unwrap(),
        EffectEntryContract {
            failure_renderer: FailureRenderer::Never,
        }
    );
}

#[test]
fn resolves_a_unique_selected_show_dictionary_from_generated_metadata() {
    let typed = interface(vec![show_instance(named("AppError"))]);
    let generated = generated_instances(json!([generated_show("__ssrg$instance$Show$0")]));

    assert_eq!(
        resolve_effect_entry_contract(&typed, &named("AppError"), &generated).unwrap(),
        EffectEntryContract {
            failure_renderer: FailureRenderer::Show {
                dictionary: DictionaryImport {
                    module: "./main.ts".to_owned(),
                    export: "__ssrg$instance$Show$0".to_owned(),
                },
            },
        }
    );
}

#[test]
fn resolves_standard_show_dictionaries_without_local_instance_metadata() {
    let typed = interface_with_exports(Vec::new(), Vec::new());
    let generated = generated_instances(json!([]));

    for (failure, export) in [
        ("String", "stringShow"),
        ("ConsoleError", "consoleErrorShow"),
        ("StdinError", "stdinErrorShow"),
    ] {
        assert_eq!(
            resolve_effect_entry_contract(&typed, &named(failure), &generated).unwrap(),
            EffectEntryContract {
                failure_renderer: FailureRenderer::Show {
                    dictionary: DictionaryImport {
                        module: "@seseragi/runtime/show".to_owned(),
                        export: export.to_owned(),
                    },
                },
            }
        );
    }
}

#[test]
fn local_export_shadows_a_standard_failure_type_spelling() {
    let local = interface_with_exports(vec![type_export("ConsoleError")], Vec::new());
    let generated = generated_instances(json!([]));

    let error =
        resolve_effect_entry_contract(&local, &named("ConsoleError"), &generated).unwrap_err();

    assert!(error.contains("requires a selected Show instance"));
    assert!(!error.contains("@seseragi/runtime/show"));
}

#[test]
fn requires_exactly_one_typed_interface_show_instance() {
    let generated = generated_instances(json!([generated_show("showAppError")]));
    let missing =
        resolve_effect_entry_contract(&interface(Vec::new()), &named("AppError"), &generated)
            .unwrap_err();
    assert!(missing.contains("requires a selected Show instance"));

    let duplicate = resolve_effect_entry_contract(
        &interface(vec![
            show_instance(named("AppError")),
            show_instance(named("AppError")),
        ]),
        &named("AppError"),
        &generated,
    )
    .unwrap_err();
    assert!(duplicate.contains("expected exactly one"));
}

#[test]
fn requires_exactly_one_matching_generated_dictionary() {
    let typed = interface(vec![show_instance(named("AppError"))]);
    let missing =
        resolve_effect_entry_contract(&typed, &named("AppError"), &generated_instances(json!([])))
            .unwrap_err();
    assert!(missing.contains("missing the Show dictionary"));

    let duplicate = resolve_effect_entry_contract(
        &typed,
        &named("AppError"),
        &generated_instances(json!([
            generated_show("showAppError1"),
            generated_show("showAppError2")
        ])),
    )
    .unwrap_err();
    assert!(duplicate.contains("duplicate Show dictionaries"));
}

#[test]
fn rejects_dictionary_exports_that_cannot_be_imported_as_typescript_names() {
    let typed = interface(vec![show_instance(named("AppError"))]);
    let generated = generated_instances(json!([generated_show("bad-name;inject()")]));

    let error = resolve_effect_entry_contract(&typed, &named("AppError"), &generated).unwrap_err();

    assert!(error.contains("invalid dictionaryExport"));
}
