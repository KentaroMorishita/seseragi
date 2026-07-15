use super::*;

#[test]
fn rejects_missing_project_provider_metadata_and_output_specifier() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
    let identity = format!("Show<{canonical}>");
    let failure = external("InputError", canonical, provider_module);
    let typed = interface_with_exports(
        Vec::new(),
        vec![imported_show_instance(canonical, Some(provider_module))],
    );
    let entry = generated_module("artifact/example", json!([]));
    let provider_interface = provider_type_interface(provider_module, "InputError", canonical, 0);
    let missing_provider = ProjectFailureRendererCatalog::new(
        [("artifact/example".to_owned(), &entry)],
        [(provider_module.to_owned(), &provider_interface)],
        [
            ("artifact/example".to_owned(), "./main.ts".to_owned()),
            (provider_module.to_owned(), "./provider.ts".to_owned()),
        ],
    );
    let error =
        super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &missing_provider)
            .unwrap_err();
    assert!(error.contains("has no compiled generated metadata"));

    let provider = generated_module(
        provider_module,
        json!([generated_imported_show(
            canonical,
            &identity,
            "showInputError"
        )]),
    );
    let missing_output = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &provider_interface)],
        [("artifact/example".to_owned(), "./main.ts".to_owned())],
    );
    let error =
        super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &missing_output)
            .unwrap_err();
    assert!(error.contains("has no staged TypeScript output specifier"));
}
#[test]
fn rejects_missing_or_ambiguous_provider_dictionary_metadata() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
    let identity = format!("Show<{canonical}>");
    let failure = external("InputError", canonical, provider_module);
    let typed = interface_with_exports(
        Vec::new(),
        vec![imported_show_instance(canonical, Some(provider_module))],
    );
    let entry = generated_module("artifact/example", json!([]));
    let provider_interface = provider_type_interface(provider_module, "InputError", canonical, 0);
    for (instances, expected) in [
        (json!([]), "missing dictionary metadata"),
        (
            json!([
                generated_imported_show(canonical, &identity, "showInputError1"),
                generated_imported_show(canonical, &identity, "showInputError2")
            ]),
            "ambiguous dictionary metadata",
        ),
    ] {
        let provider = generated_module(provider_module, instances);
        let catalog = ProjectFailureRendererCatalog::new(
            [
                ("artifact/example".to_owned(), &entry),
                (provider_module.to_owned(), &provider),
            ],
            [(provider_module.to_owned(), &provider_interface)],
            [
                ("artifact/example".to_owned(), "./main.ts".to_owned()),
                (provider_module.to_owned(), "./provider.ts".to_owned()),
            ],
        );
        let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
            .unwrap_err();
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn rejects_a_provider_dictionary_with_the_wrong_concrete_identity() {
    let canonical = "fixture/errors::provider::InputError";
    let provider_module = "fixture/errors::provider";
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
            "Show<fixture/wrong::InputError>",
            "showInputError"
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
            ("artifact/example".to_owned(), "./main.ts".to_owned()),
            (provider_module.to_owned(), "./provider.ts".to_owned()),
        ],
    );

    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains("dictionary identity mismatch"));
    assert!(error.contains("Show<fixture/wrong::InputError>"));
}

#[test]
fn rejects_missing_or_mismatched_provider_type_ownership() {
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
            "showInputError"
        )]),
    );
    let wrapper_specifiers = [
        ("artifact/example".to_owned(), "./main.ts".to_owned()),
        (provider_module.to_owned(), "./provider.ts".to_owned()),
    ];

    let missing = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        Vec::<(String, &TypedModuleInterface)>::new(),
        wrapper_specifiers.clone(),
    );
    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &missing)
        .unwrap_err();
    assert!(error.contains("has no final typed interface"));

    let mut wrong_module = provider_type_interface(provider_module, "InputError", canonical, 0);
    wrong_module.module = "fixture/errors::wrong".to_owned();
    let catalog = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &wrong_module)],
        wrapper_specifiers.clone(),
    );
    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains("typed interface module mismatch"));

    let wrong_export = provider_type_interface(provider_module, "WrongError", canonical, 0);
    let catalog = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &wrong_export)],
        wrapper_specifiers,
    );
    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains("does not publicly own monomorphic type export InputError"));

    let wrong_canonical = provider_type_interface(
        provider_module,
        "InputError",
        "fixture/errors::provider::OtherError",
        0,
    );
    let catalog = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &wrong_canonical)],
        [
            ("artifact/example".to_owned(), "./main.ts".to_owned()),
            (provider_module.to_owned(), "./provider.ts".to_owned()),
        ],
    );
    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains(canonical));
    assert!(error.contains("does not publicly own monomorphic type export InputError"));
}

#[test]
fn rejects_a_generic_provider_type_export_for_a_monomorphic_external_failure() {
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
            "showInputError"
        )]),
    );
    let mut provider_interface =
        provider_type_interface(provider_module, "InputError", canonical, 1);
    provider_interface.exports[0]
        .scheme
        .type_parameters
        .push(seseragi_syntax::TypeParameter::value("error"));
    let catalog = ProjectFailureRendererCatalog::new(
        [
            ("artifact/example".to_owned(), &entry),
            (provider_module.to_owned(), &provider),
        ],
        [(provider_module.to_owned(), &provider_interface)],
        [
            ("artifact/example".to_owned(), "./main.ts".to_owned()),
            (provider_module.to_owned(), "./provider.ts".to_owned()),
        ],
    );

    let error = super::super::project::resolve(&typed, &failure, &entry, "./main.ts", &catalog)
        .unwrap_err();
    assert!(error.contains("does not publicly own monomorphic type export InputError"));
}
