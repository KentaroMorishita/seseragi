use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_lowering::{emit_typescript_module, GeneratedBundle};

#[test]
fn public_notices_survive_both_artifacts_without_changing_standard_mappings() {
    let source = concat!(
        "// 元のAPI 🙂\n",
        "pub deprecated \"Use current\" since \"0.61.0\" fn old value: Int -> Int = value\n",
        "pub deprecated \"Use Modern\" type Legacy = | Legacy\n",
        "deprecated \"Internal\" fn hidden -> Int = 1\n",
        "pub trait Render<A> { deprecated \"Use render\" fn old value: A -> String }\n",
        "pub struct Box { pub value: Int }\n",
        "impl Box { pub deprecated \"Use read\" fn old self: Box -> Int = self.value }\n",
        "pub foreign \"typescript\" from \"./host.mjs\" {\n",
        "  deprecated \"Use host current\" pure fn oldHost value: Int -> Int\n",
        "}\n",
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("λ/api.ssrg", "artifact/api", source).with_profile(profile),
        )
        .unwrap();
        let notices = &compiled.generated.metadata.deprecations;
        assert_eq!(notices.len(), 5, "{notices:#?}");
        assert_eq!(notices, &compiled.generated.source_map.deprecations);
        assert!(notices.iter().all(|entry| entry.name != "hidden"));
        assert_eq!(
            notices
                .iter()
                .filter(|entry| entry.name == "Legacy")
                .count(),
            1
        );
        let old = notices
            .iter()
            .find(|entry| entry.identity == "artifact/api::old")
            .unwrap();
        assert_eq!(old.namespace, "value");
        assert!(old.owner.is_none());
        assert_eq!(old.deprecation.since.as_deref(), Some("0.61.0"));
        assert!(notices
            .iter()
            .any(|entry| entry.identity == "artifact/api::trait(Render)::old"
                && entry.owner.as_deref() == Some("artifact/api::trait(Render)")));
        assert!(notices
            .iter()
            .any(|entry| entry.identity == "artifact/api::Box::old"
                && entry.owner.as_deref() == Some("artifact/api::Box")));
        for notice in notices {
            assert_eq!(notice.deprecation.source, "λ/api.ssrg");
            assert!(
                source[notice.deprecation.origin.start..notice.deprecation.origin.end]
                    .starts_with("deprecated \"")
            );
        }
        let raw = emit_typescript_module(compiled.typescript_ir.clone(), source);
        assert_eq!(compiled.generated.typescript, raw.typescript);
        let mut standard_map = serde_json::to_value(&compiled.generated.source_map).unwrap();
        assert!(standard_map
            .as_object_mut()
            .unwrap()
            .remove("x_seseragi_deprecations")
            .is_some());
        assert_eq!(standard_map, serde_json::to_value(raw.source_map).unwrap());
        let json = serde_json::to_string(&compiled.generated).unwrap();
        let restored: GeneratedBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, compiled.generated);
    }
}

#[test]
fn empty_notices_keep_older_metadata_and_source_map_wire_shape() {
    let compiled = compile_module(CompileInput::new(
        "plain.ssrg",
        "artifact/plain",
        "pub fn answer -> Int = 42\n",
    ))
    .unwrap();
    let metadata = serde_json::to_value(&compiled.generated.metadata).unwrap();
    let source_map = serde_json::to_value(&compiled.generated.source_map).unwrap();
    assert!(metadata.get("deprecations").is_none());
    assert!(source_map.get("x_seseragi_deprecations").is_none());
    let restored: GeneratedBundle =
        serde_json::from_value(serde_json::to_value(&compiled.generated).unwrap()).unwrap();
    assert!(restored.metadata.deprecations.is_empty());
    assert!(restored.source_map.deprecations.is_empty());
}

#[test]
fn path_package_reexports_keep_canonical_provider_notice_in_both_artifacts() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/deprecation"
    ))
    .unwrap();
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let project =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        let mut provider_sources = std::collections::BTreeSet::new();
        let mut facade_notices = 0;
        for (module, compiled) in &project.compiled.modules {
            assert_eq!(
                compiled.generated.metadata.deprecations,
                compiled.generated.source_map.deprecations
            );
            for notice in &compiled.generated.metadata.deprecations {
                if notice.identity.ends_with("::historical") {
                    assert!(notice.identity.contains("deprecation-model"));
                    assert_eq!(notice.deprecation.message, "Use modern");
                    assert!(notice.deprecation.source.ends_with("lib.ssrg"));
                    provider_sources.insert(notice.deprecation.source.clone());
                    if module.ends_with("facade") {
                        facade_notices += 1;
                    }
                }
            }
        }
        assert_eq!(provider_sources.len(), 1);
        assert!(facade_notices > 0);
    }
}
