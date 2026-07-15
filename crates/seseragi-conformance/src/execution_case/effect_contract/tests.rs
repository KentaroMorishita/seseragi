use std::collections::BTreeMap;

use seseragi_lowering::GeneratedModule;
use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{
    ByteSpan, InterfaceExport, InterfaceInstance, InterfaceScheme, InterfaceType, Visibility,
};

use super::{
    compare_required_environment, effect_environment, validate_effect_entry_contract,
    validate_effect_entry_contract_in_memory, DictionaryImport, EffectEntryContract,
    FailureRenderer,
};

fn named(name: &str) -> InterfaceType {
    InterfaceType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn effect_entry(environment: InterfaceType) -> InterfaceType {
    InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(InterfaceType::Named {
            name: "Effect".to_owned(),
            arguments: vec![environment, named("Never"), named("Unit")],
        }),
    }
}

#[test]
fn extracts_canonical_effect_environment_fields_order_insensitively() {
    let entry = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![
            seseragi_syntax::InterfaceRecordField {
                name: "stdin".to_owned(),
                optional: false,
                type_ref: named("Stdin"),
            },
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
        ],
    });

    assert_eq!(
        effect_environment(&entry, "main").unwrap(),
        BTreeMap::from([
            ("console".to_owned(), "Console".to_owned()),
            ("stdin".to_owned(), "Stdin".to_owned()),
        ])
    );
}

#[test]
fn follows_curried_parameters_to_the_effect_result() {
    let entry = InterfaceType::Function {
        parameter: Box::new(named("String")),
        result: Box::new(effect_entry(InterfaceType::Record {
            closed: true,
            fields: Vec::new(),
        })),
    };

    assert!(effect_environment(&entry, "parse").unwrap().is_empty());
}

#[test]
fn rejects_non_effect_results_and_open_environments() {
    let pure = InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(named("Unit")),
    };
    assert!(effect_environment(&pure, "main")
        .unwrap_err()
        .contains("Effect<R, E, A>"));

    let open = effect_entry(InterfaceType::Record {
        closed: false,
        fields: Vec::new(),
    });
    assert!(effect_environment(&open, "main")
        .unwrap_err()
        .contains("closed record"));
}

#[test]
fn rejects_optional_and_duplicate_environment_fields() {
    let optional = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![seseragi_syntax::InterfaceRecordField {
            name: "console".to_owned(),
            optional: true,
            type_ref: named("Console"),
        }],
    });
    assert!(effect_environment(&optional, "main")
        .unwrap_err()
        .contains("cannot be optional"));

    let duplicate = effect_entry(InterfaceType::Record {
        closed: true,
        fields: vec![
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
            seseragi_syntax::InterfaceRecordField {
                name: "console".to_owned(),
                optional: false,
                type_ref: named("Console"),
            },
        ],
    });
    assert!(effect_environment(&duplicate, "main")
        .unwrap_err()
        .contains("duplicate field console"));
}

#[test]
fn compares_required_fields_by_exact_name_and_type() {
    let compiler = BTreeMap::from([("console".to_owned(), "Console".to_owned())]);
    assert!(compare_required_environment(&compiler, &compiler, "main").is_ok());

    let misspelled = BTreeMap::from([("console".to_owned(), "console".to_owned())]);
    let error = compare_required_environment(&compiler, &misspelled, "main").unwrap_err();
    assert!(error.contains("typed interface Effect R"));
    assert!(error.contains("Console"));
}

#[test]
fn requires_a_typed_interface_reference_for_effect_execution() {
    let error = validate_effect_entry_contract(
        std::path::Path::new("."),
        &serde_json::json!({ "entry": { "module": "example", "export": "main" } }),
        "main",
        "./main.ts",
    )
    .unwrap_err();

    assert!(error.contains("entry.typedInterface is required"));
}

#[test]
fn requires_unit_success_only_for_the_package_main_entry() {
    let effect = |success| InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(InterfaceType::Named {
            name: "Effect".to_owned(),
            arguments: vec![
                InterfaceType::Record {
                    closed: true,
                    fields: Vec::new(),
                },
                named("Never"),
                success,
            ],
        }),
    };
    let mut interface = TypedModuleInterface {
        schema: 1,
        stage: "typed-interface".to_owned(),
        module: "fixture/parse".to_owned(),
        source: "main.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports: vec![InterfaceExport {
            symbol: "fixture/parse::parse".to_owned(),
            namespace: "value".to_owned(),
            name: "parse".to_owned(),
            constructor_of: None,
            visibility: Visibility::Public,
            declaration_kind: Some("effect-function".to_owned()),
            declaration: ByteSpan { start: 0, end: 5 },
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: effect(named("String")),
            },
            methods: Vec::new(),
            representation: None,
        }],
        operators: Vec::new(),
        instances: Vec::new(),
    };
    let generated: GeneratedModule = serde_json::from_value(serde_json::json!({
        "schema": 1,
        "module": "fixture/parse",
        "target": "typescript-es2022",
        "runtime": {
            "identity": "@seseragi/runtime",
            "abiMajor": 1,
            "requirements": []
        },
        "exports": ["parse"],
        "outputs": { "typescript": "main.js", "sourceMap": "main.js.map" }
    }))
    .unwrap();

    assert!(validate_effect_entry_contract_in_memory(
        &interface,
        &generated,
        "parse",
        "./main.js",
        &BTreeMap::new(),
    )
    .is_ok());

    interface.exports[0].name = "main".to_owned();
    interface.exports[0].symbol = "fixture/parse::main".to_owned();
    assert!(validate_effect_entry_contract_in_memory(
        &interface,
        &generated,
        "main",
        "./main.js",
        &BTreeMap::new(),
    )
    .unwrap_err()
    .contains("success type A must be standard Unit"));
}

#[test]
fn validates_an_in_memory_entry_with_the_callers_exact_module_specifier() {
    let app_error = named("AppError");
    let entry_type = InterfaceType::Function {
        parameter: Box::new(named("Unit")),
        result: Box::new(InterfaceType::Named {
            name: "Effect".to_owned(),
            arguments: vec![
                InterfaceType::Record {
                    closed: true,
                    fields: Vec::new(),
                },
                app_error.clone(),
                named("Unit"),
            ],
        }),
    };
    let interface = TypedModuleInterface {
        schema: 1,
        stage: "typed-interface".to_owned(),
        module: "fixture/game::main".to_owned(),
        source: "src/main.ssrg".to_owned(),
        dependencies: Vec::new(),
        exports: vec![
            InterfaceExport {
                symbol: "fixture/game::main::AppError".to_owned(),
                namespace: "type".to_owned(),
                name: "AppError".to_owned(),
                constructor_of: None,
                visibility: Visibility::Public,
                declaration_kind: Some("type".to_owned()),
                declaration: ByteSpan { start: 0, end: 8 },
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: InterfaceType::TypeConstructor {
                        name: "AppError".to_owned(),
                        arity: 0,
                    },
                },
                methods: Vec::new(),
                representation: None,
            },
            InterfaceExport {
                symbol: "fixture/game::main::main".to_owned(),
                namespace: "value".to_owned(),
                name: "main".to_owned(),
                constructor_of: None,
                visibility: Visibility::Public,
                declaration_kind: Some("effect-function".to_owned()),
                declaration: ByteSpan { start: 9, end: 20 },
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: entry_type,
                },
                methods: Vec::new(),
                representation: None,
            },
        ],
        operators: Vec::new(),
        instances: vec![InterfaceInstance {
            identity: Some("Show<fixture/game::main::AppError>".to_owned()),
            provider_module: Some("fixture/game::main".to_owned()),
            trait_identity: Some("Show".to_owned()),
            argument_identities: vec!["fixture/game::main::AppError".to_owned()],
            type_identity: Some("fixture/game::main::AppError".to_owned()),
            trait_name: "Show".to_owned(),
            type_parameters: Vec::new(),
            head: InterfaceType::Apply {
                constructor: "Show".to_owned(),
                arguments: vec![app_error],
            },
            constraints: Vec::new(),
            origin: ByteSpan { start: 0, end: 8 },
        }],
    };
    let generated: GeneratedModule = serde_json::from_value(serde_json::json!({
        "schema": 1,
        "module": "fixture/game::main",
        "target": "typescript-es2022",
        "runtime": {
            "identity": "@seseragi/runtime",
            "abiMajor": 1,
            "requirements": []
        },
        "exports": ["main"],
        "instances": [{
            "identity": "Show<fixture/game::main::AppError>",
            "trait": "Show",
            "arguments": [{ "kind": "reference", "name": "AppError", "arguments": [] }],
            "head": { "kind": "reference", "name": "AppError", "arguments": [] },
            "typeIdentity": "fixture/game::main::AppError",
            "dictionaryExport": "__ssrg$instance$Show$0"
        }],
        "outputs": { "typescript": "main.js", "sourceMap": "main.js.map" }
    }))
    .unwrap();

    let contract = validate_effect_entry_contract_in_memory(
        &interface,
        &generated,
        "main",
        "./dist/game/main.js",
        &BTreeMap::new(),
    )
    .unwrap();

    assert_eq!(
        contract,
        EffectEntryContract {
            failure_renderer: FailureRenderer::Show {
                dictionary: DictionaryImport {
                    module: "./dist/game/main.js".to_owned(),
                    export: "__ssrg$instance$Show$0".to_owned(),
                },
            },
        }
    );
}
