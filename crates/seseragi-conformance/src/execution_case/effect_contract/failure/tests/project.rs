pub(super) use super::super::project::resolve;
use super::*;

mod provider;

fn external(name: &str, canonical: &str, provider_module: &str) -> InterfaceType {
    InterfaceType::ExternalNamed {
        name: name.to_owned(),
        canonical: canonical.to_owned(),
        provider_module: provider_module.to_owned(),
        provider_export: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn imported_show_instance(canonical: &str, provider_module: Option<&str>) -> InterfaceInstance {
    InterfaceInstance {
        identity: Some(format!("Show<{canonical}>")),
        provider_module: provider_module.map(str::to_owned),
        type_identity: Some(canonical.to_owned()),
        trait_name: "Show".to_owned(),
        type_parameters: Vec::new(),
        // The structured type identity, rather than this provider-local
        // spelling, selects transitive evidence in the consumer.
        head: InterfaceType::Apply {
            constructor: "Show".to_owned(),
            arguments: vec![named("InputError")],
        },
        constraints: Vec::new(),
        origin: ByteSpan { start: 0, end: 8 },
    }
}

fn generated_imported_show(
    canonical: &str,
    identity: &str,
    dictionary_export: &str,
) -> serde_json::Value {
    json!({
        "identity": identity,
        "trait": "Show",
        "head": { "kind": "reference", "name": "InputError", "arguments": [] },
        "typeIdentity": canonical,
        "dictionaryExport": dictionary_export,
    })
}

fn provider_type_interface(
    provider_module: &str,
    provider_export: &str,
    canonical: &str,
    arity: u32,
) -> TypedModuleInterface {
    let mut export = type_export(provider_export);
    export.symbol = canonical.to_owned();
    export.scheme.type_ref = InterfaceType::TypeConstructor {
        name: provider_export.to_owned(),
        arity,
    };
    let mut interface = interface_with_exports(vec![export], Vec::new());
    interface.module = provider_module.to_owned();
    interface.source = "provider.ssrg".to_owned();
    interface
}

#[test]
fn resolves_a_transitive_show_provider_from_structured_final_evidence() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
    let identity = format!("Show<{canonical}>");
    let failure = external("InputError", canonical, provider_module);
    let typed = interface_with_exports(
        Vec::new(),
        vec![imported_show_instance(canonical, Some(provider_module))],
    );
    let entry = generated_module("artifact/example", json!([]));
    let provider = generated_module(
        provider_module,
        json!([generated_imported_show(
            canonical,
            &identity,
            "__ssrg$instance$Show$0"
        )]),
    );
    let provider_interface = provider_type_interface(provider_module, "InputError", canonical, 0);
    let catalog = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &provider_interface)],
        [
            (
                "artifact/example".to_owned(),
                "./dist/app/main.ts".to_owned(),
            ),
            (
                provider_module.to_owned(),
                "./dist/errors/provider.ts".to_owned(),
            ),
        ],
    );

    assert_eq!(
        super::super::project::resolve(&typed, &failure, &entry, "./dist/app/main.ts", &catalog,)
            .unwrap(),
        EffectEntryContract {
            failure_renderer: FailureRenderer::Show {
                dictionary: DictionaryImport {
                    module: "./dist/errors/provider.ts".to_owned(),
                    export: "__ssrg$instance$Show$0".to_owned(),
                },
            },
        }
    );
}

#[test]
fn rejects_missing_or_ambiguous_structured_project_show_evidence() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
    let failure = external("InputError", canonical, provider_module);
    let entry = generated_module("artifact/example", json!([]));
    let empty_catalog = ProjectFailureRendererCatalog::new(
        [("artifact/example".to_owned(), &entry)],
        Vec::<(String, &TypedModuleInterface)>::new(),
        [("artifact/example".to_owned(), "./main.ts".to_owned())],
    );

    let missing = interface_with_exports(Vec::new(), Vec::new());
    let error =
        super::super::project::resolve(&missing, &failure, &entry, "./main.ts", &empty_catalog)
            .unwrap_err();
    assert!(error.contains("exactly one selected Show evidence; found none"));

    let evidence = imported_show_instance(canonical, Some(provider_module));
    let ambiguous = interface_with_exports(Vec::new(), vec![evidence.clone(), evidence]);
    let error =
        super::super::project::resolve(&ambiguous, &failure, &entry, "./main.ts", &empty_catalog)
            .unwrap_err();
    assert!(error.contains("exactly one selected Show evidence; found 2"));

    let missing_provider =
        interface_with_exports(Vec::new(), vec![imported_show_instance(canonical, None)]);
    let error = super::super::project::resolve(
        &missing_provider,
        &failure,
        &entry,
        "./main.ts",
        &empty_catalog,
    )
    .unwrap_err();
    assert!(error.contains("missing providerModule"));

    let mut missing_type_identity = imported_show_instance(canonical, Some(provider_module));
    missing_type_identity.type_identity = None;
    let missing_type_identity = interface_with_exports(Vec::new(), vec![missing_type_identity]);
    let error = super::super::project::resolve(
        &missing_type_identity,
        &failure,
        &entry,
        "./main.ts",
        &empty_catalog,
    )
    .unwrap_err();
    assert!(error.contains("missing typeIdentity"));

    for identity in [None, Some(String::new())] {
        let mut missing_identity = imported_show_instance(canonical, Some(provider_module));
        missing_identity.identity = identity;
        let missing_identity = interface_with_exports(Vec::new(), vec![missing_identity]);
        let error = super::super::project::resolve(
            &missing_identity,
            &failure,
            &entry,
            "./main.ts",
            &empty_catalog,
        )
        .unwrap_err();
        assert!(error.contains("missing its final concrete identity"));
    }
}

#[test]
fn rejects_generic_or_conditional_external_show_evidence() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
    let failure = external("InputError", canonical, provider_module);
    let entry = generated_module("artifact/example", json!([]));
    let catalog = ProjectFailureRendererCatalog::new(
        [("artifact/example".to_owned(), &entry)],
        Vec::<(String, &TypedModuleInterface)>::new(),
        [("artifact/example".to_owned(), "./main.ts".to_owned())],
    );

    let mut generic = imported_show_instance(canonical, Some(provider_module));
    generic.type_parameters.push("error".to_owned());
    let generic = interface_with_exports(Vec::new(), vec![generic]);
    let error = super::super::project::resolve(&generic, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains("is generic; execution requires concrete evidence"));

    let mut conditional = imported_show_instance(canonical, Some(provider_module));
    conditional
        .constraints
        .push(seseragi_syntax::InterfaceConstraint {
            name: "Show<payload>".to_owned(),
            arguments: Vec::new(),
        });
    let conditional = interface_with_exports(Vec::new(), vec![conditional]);
    let error =
        super::super::project::resolve(&conditional, &failure, &entry, "./main.ts", &catalog)
            .unwrap_err();
    assert!(error.contains("is conditional; execution requires constraint-free evidence"));
}
