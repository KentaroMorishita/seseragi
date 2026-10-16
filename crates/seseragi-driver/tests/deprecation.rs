use seseragi_driver::{compile_module, format_module, BuildProfile, CompileInput};

#[test]
fn deprecation_changes_metadata_without_changing_ts_or_curried_abi() {
    let body = "pub fn historical<A> value: A -> Int = 41\npub fn probe -> Int = historical True\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let plain =
            compile_module(CompileInput::new("main.ssrg", "app/main", body).with_profile(profile))
                .unwrap();
        for prefix in [
            "pub deprecated \"Use replacement\" ",
            "pub deprecated \"Use replacement\" since \"0.4.1\"\n",
        ] {
            let source = body.replacen("pub ", prefix, 1);
            let compiled = compile_module(
                CompileInput::new("main.ssrg", "app/main", &source).with_profile(profile),
            )
            .unwrap();
            assert_eq!(compiled.generated.typescript, plain.generated.typescript);
            assert_eq!(
                compiled.core_ir.functions.len(),
                plain.core_ir.functions.len()
            );
            let export = compiled
                .typed_interface
                .exports
                .iter()
                .find(|e| e.name == "historical")
                .unwrap();
            assert_eq!(export.symbol, "app/main::historical");
            assert_eq!(
                export.scheme,
                plain
                    .typed_interface
                    .exports
                    .iter()
                    .find(|e| e.name == "historical")
                    .unwrap()
                    .scheme
            );
            assert_eq!(
                export.deprecation.as_ref().unwrap().message,
                "Use replacement"
            );
            let formatted = format_module("main.ssrg", &source).unwrap();
            let second = format_module("main.ssrg", &formatted.text).unwrap();
            assert!(!second.changed, "{}", second.text);
            let after = compile_module(
                CompileInput::new("main.ssrg", "app/main", &formatted.text).with_profile(profile),
            )
            .unwrap();
            assert_eq!(after.generated.typescript, compiled.generated.typescript);
        }
    }
}

#[test]
fn imported_and_reexported_metadata_keep_defining_source_and_identity() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/deprecation"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let project =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        assert_eq!(project.compiled.modules.len(), 3);
        for (owner, module) in &project.compiled.modules {
            assert_eq!(module.core_ir, core.compiled.modules[owner].core_ir);
            let label_reexports = module
                .typescript_ir
                .source_imports
                .iter()
                .flat_map(|group| &group.reexports)
                .filter(|binding| binding.imported == "Label")
                .collect::<Vec<_>>();
            if !label_reexports.is_empty() {
                assert_eq!(label_reexports.len(), 2);
                assert!(label_reexports.iter().any(|b| b.type_only));
                assert!(label_reexports.iter().any(|b| !b.type_only));
                assert!(!module.generated.typescript.contains("type Label, Label"));
            }
            for export in &module.typed_interface.exports {
                if export.symbol.ends_with("::cold") {
                    assert_eq!(
                        export.deprecation.as_ref().unwrap().message,
                        "Use modernCold"
                    );
                }
                if export.symbol.ends_with("::historical") {
                    let metadata = export.deprecation.as_ref().unwrap();
                    assert_eq!(metadata.message, "Use modern");
                    assert_eq!(metadata.since.as_deref(), Some("0.4.1"));
                    assert!(metadata.source.ends_with("lib.ssrg"), "{metadata:?}");
                    assert!(export.symbol.contains("deprecation-model"));
                }
                if export.constructor_of.is_some() {
                    assert!(
                        export.deprecation.is_none(),
                        "Constructors do not inherit type deprecation: {export:?}"
                    );
                }
            }
        }
    }
}
