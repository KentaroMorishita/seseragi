use seseragi_backend_rust::emit_rust_project;
use seseragi_lowering::CoreModule;

fn project() -> (Vec<CoreModule>, String) {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/linked"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    (
        core.compiled
            .order
            .iter()
            .map(|id| core.compiled.modules[id].core_ir.clone())
            .collect(),
        core.entry_module,
    )
}

#[test]
fn canonical_project_owners_and_entry_module_are_checked() {
    let (modules, entry) = project();
    let refs = modules.iter().collect::<Vec<_>>();
    let emitted = emit_rust_project(&refs, &entry, "probe").unwrap();
    assert_eq!(emitted.metadata.modules.len(), 5);
    assert_eq!(emitted.metadata.entry, format!("{entry}::probe"));
    assert!(emit_rust_project(&refs, &entry, "privateValue").is_err());
    let mut reversed = refs.clone();
    reversed.reverse();
    let error = emit_rust_project(&reversed, &entry, "probe").unwrap_err();
    assert!(error.origin.is_some());
    assert!(error.message.contains("dependency"));
    let mut duplicate = refs;
    duplicate.push(&modules[0]);
    assert!(emit_rust_project(&duplicate, &entry, "probe").is_err());
}

fn counterfeit_import(value: &mut serde_json::Value, kind: usize) -> usize {
    match value {
        serde_json::Value::Object(fields) => {
            if fields.get("kind").and_then(|v| v.as_str()) == Some("imported")
                && fields.contains_key("providerModule")
            {
                if kind == 0 {
                    fields.insert("providerModule".into(), "counterfeit/provider".into());
                } else {
                    fields.insert("kind".into(), "local".into());
                    fields.remove("providerModule");
                }
                1
            } else {
                fields
                    .values_mut()
                    .map(|v| counterfeit_import(v, kind))
                    .sum()
            }
        }
        serde_json::Value::Array(items) => {
            items.iter_mut().map(|v| counterfeit_import(v, kind)).sum()
        }
        _ => 0,
    }
}

#[test]
fn imported_evidence_cannot_be_rebound_to_a_name_or_a_local_factory() {
    let (modules, entry) = project();
    for kind in 0..2 {
        let mut encoded = serde_json::to_value(&modules).unwrap();
        for module in encoded.as_array_mut().unwrap() {
            for key in ["adts", "bindings", "functions"] {
                module
                    .as_object_mut()
                    .unwrap()
                    .entry(key)
                    .or_insert(serde_json::json!([]));
            }
        }
        assert!(counterfeit_import(&mut encoded, kind) > 0);
        let changed: Vec<CoreModule> = serde_json::from_value(encoded).unwrap();
        let error =
            emit_rust_project(&changed.iter().collect::<Vec<_>>(), &entry, "probe").unwrap_err();
        assert!(error.message.contains("provider owner"), "{error:?}");
        assert!(error.origin.is_some());
        assert!(error.module.is_some());
    }
}

#[test]
fn display_children_keep_provider_identity_and_instantiated_types() {
    for fixture in ["tuple-display", "array-display", "instance-alias"] {
        let loaded = seseragi_project::load_local_project(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/spec/fixtures/rust-backend-projects")
                .join(fixture),
        )
        .unwrap();
        let checked = seseragi_driver::compile_core_local_project(&loaded).unwrap();
        let modules = checked
            .compiled
            .order
            .iter()
            .map(|id| checked.compiled.modules[id].core_ir.clone())
            .collect::<Vec<_>>();
        emit_rust_project(
            &modules.iter().collect::<Vec<_>>(),
            &checked.entry_module,
            "main",
        )
        .unwrap();
        fn mutate(value: &mut serde_json::Value, kind: usize) -> usize {
            match value {
                serde_json::Value::Object(fields) => {
                    if fields.get("kind").and_then(|v| v.as_str()) == Some("imported")
                        && fields
                            .get("identity")
                            .and_then(|v| v.as_str())
                            .is_some_and(|v| {
                                v.starts_with("std/prelude::Show<") && v.contains("::Box<")
                            })
                    {
                        if kind == 0 {
                            fields.insert("providerModule".into(), "counterfeit/provider".into());
                        } else if kind == 1 {
                            fields.insert(
                                "typeArguments".into(),
                                serde_json::json!([{"kind":"named","name":"Bool","arguments":[]}]),
                            );
                        } else {
                            let child = fields["evidenceArguments"][0]["constraint"]
                                .as_object_mut()
                                .unwrap();
                            child.insert("traitIdentity".into(), "user::Show".into());
                        }
                        1
                    } else {
                        fields.values_mut().map(|v| mutate(v, kind)).sum()
                    }
                }
                serde_json::Value::Array(items) => items.iter_mut().map(|v| mutate(v, kind)).sum(),
                _ => 0,
            }
        }
        for kind in 0..3 {
            let mut encoded = serde_json::to_value(&modules).unwrap();
            for module in encoded.as_array_mut().unwrap() {
                for key in ["adts", "bindings", "functions"] {
                    module
                        .as_object_mut()
                        .unwrap()
                        .entry(key)
                        .or_insert(serde_json::json!([]));
                }
            }
            assert!(mutate(&mut encoded, kind) > 0);
            let changed: Vec<CoreModule> = serde_json::from_value(encoded).unwrap();
            let result = emit_rust_project(
                &changed.iter().collect::<Vec<_>>(),
                &checked.entry_module,
                "main",
            );
            let Err(error) = result else {
                panic!("{fixture}: counterfeit kind {kind} was accepted");
            };
            assert_eq!(error.code, "SES-RUST-0001");
            assert!(error.origin.is_some());
            assert!(error.module.is_some());
        }
    }
}

#[test]
fn initialization_uses_source_import_postorder_instead_of_checking_order() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/globals"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    let modules = core
        .compiled
        .order
        .iter()
        .map(|id| &core.compiled.modules[id].core_ir)
        .collect::<Vec<_>>();
    let prefix = core.entry_module.rsplit_once("::").unwrap().0;
    let left = format!("{prefix}::left");
    let lib = format!("{prefix}::lib");
    let position = |order: &[String], id: &str| order.iter().position(|m| m == id).unwrap();
    assert!(position(&core.compiled.order, &left) < position(&core.compiled.order, &lib));
    let artifact = emit_rust_project(&modules, &core.entry_module, "main").unwrap();
    assert!(
        position(&artifact.metadata.modules, &lib) < position(&artifact.metadata.modules, &left)
    );
    assert_eq!(
        artifact.metadata.modules,
        seseragi_lowering::core_module_initialization_order(&modules, &core.entry_module).unwrap()
    );
}
