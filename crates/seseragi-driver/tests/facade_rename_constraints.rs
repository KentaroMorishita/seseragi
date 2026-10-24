use seseragi_driver::{compile_project, CompiledProject, ProjectModuleInput};
use seseragi_project::{ModuleGraph, ModuleLinkTarget};
use std::collections::BTreeMap;

const MODEL_ID: &str = "fixture881/model::lib";
const FACADE_ID: &str = "fixture881/app::facade";
const MAIN_ID: &str = "fixture881/app::main";
const MODEL: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/local-dictionaries/vendor/model/src/lib.ssrg"
);
const MAIN: &str = include_str!(
    "../../../examples/spec/fixtures/rust-backend-projects/local-dictionaries/src/main.ssrg"
);

fn sources(box_name: &str, trait_name: &str) -> (String, String) {
    (
        format!("pub import {{Box as {box_name}, Render as {trait_name}, rendered, boxed}} from \"model\"\n"),
        MAIN.replace("Box as Foreign", &format!("{box_name} as Foreign"))
            .replace("Render as Display", &format!("{trait_name} as Display")),
    )
}

fn project(
    facade: &str,
    main: &str,
    profile: seseragi_project::BuildProfile,
) -> Result<CompiledProject, seseragi_driver::ProjectCompileError> {
    project_with_model(MODEL, facade, main, profile)
}

fn project_with_model(
    model: &str,
    facade: &str,
    main: &str,
    profile: seseragi_project::BuildProfile,
) -> Result<CompiledProject, seseragi_driver::ProjectCompileError> {
    let mut graph = ModuleGraph::new();
    graph.add_module(MODEL_ID.to_owned(), []).unwrap();
    graph
        .add_module(
            FACADE_ID.to_owned(),
            [("model".to_owned(), MODEL_ID.to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            MAIN_ID.to_owned(),
            [("./facade".to_owned(), FACADE_ID.to_owned())],
        )
        .unwrap();
    compile_project(
        graph,
        [
            ProjectModuleInput::new("model/lib.ssrg", MODEL_ID, model, "dist/model/lib.js")
                .with_package_scope("fixture881/model")
                .with_profile(profile),
            ProjectModuleInput::new("app/facade.ssrg", FACADE_ID, facade, "dist/app/facade.js")
                .with_package_scope("fixture881/app")
                .with_profile(profile),
            ProjectModuleInput::new("app/main.ssrg", MAIN_ID, main, "dist/app/main.js")
                .with_package_scope("fixture881/app")
                .with_profile(profile),
        ],
    )
}

// Opt-in stage capture is diagnostic evidence only. The acceptance tests still
// call the strict project compiler and cannot lower a recovered/error module.
fn capture_stages(label: &str, facade: &str, main: &str) {
    let Ok(directory) = std::env::var("SESERAGI_FACADE881_CAPTURE") else {
        return;
    };
    let analyze = |source_name: &str,
                   module: &str,
                   source: &str,
                   targets: BTreeMap<String, ModuleLinkTarget>| {
        let linked = seseragi_project::link_module(
            seseragi_syntax::parse_unlinked_module_interface(source_name, module, source),
            &targets,
        )
        .unwrap();
        seseragi_semantics::analyze_linked_module(
            seseragi_syntax::parse_diagnostics(source_name, source),
            linked,
            source,
        )
        .unwrap()
    };
    let model = analyze("model/lib.ssrg", MODEL_ID, MODEL, BTreeMap::new());
    let facade_module = analyze(
        "app/facade.ssrg",
        FACADE_ID,
        facade,
        BTreeMap::from([(
            "model".to_owned(),
            ModuleLinkTarget::external(model.typed_interface.clone().into_link_interface()),
        )]),
    );
    let target = ModuleLinkTarget::same_package(
        seseragi_syntax::parse_unlinked_module_interface("app/facade.ssrg", FACADE_ID, facade)
            .header,
        facade_module.typed_interface.clone().into_link_interface(),
    )
    .unwrap();
    let linked = seseragi_project::link_module(
        seseragi_syntax::parse_unlinked_module_interface("app/main.ssrg", MAIN_ID, main),
        &BTreeMap::from([
            ("./facade".to_owned(), target),
            (
                "std/ref".to_owned(),
                seseragi_project::standard_module_target("std/ref").unwrap(),
            ),
        ]),
    )
    .unwrap();
    let resolved = seseragi_semantics::resolve_linked_module(linked.clone(), main);
    let analyzed = seseragi_semantics::analyze_linked_module_recovering(
        seseragi_syntax::parse_diagnostics("app/main.ssrg", main),
        linked,
        main,
    )
    .unwrap();
    let output = std::path::Path::new(&directory);
    std::fs::create_dir_all(output).unwrap();
    std::fs::write(
        output.join(format!("{label}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "diagnosticRecoveryOnly": true,
            "modelInterface": model.typed_interface,
            "facadeInterface": facade_module.typed_interface,
            "mainResolved": resolved,
            "mainTyped": analyzed.typed_hir,
            "mainInterface": analyzed.typed_interface,
            "diagnostics": analyzed.diagnostics
        }))
        .unwrap(),
    )
    .unwrap();
}

fn verify(label: &str, box_name: &str, trait_name: &str) {
    let (facade, main) = sources(box_name, trait_name);
    capture_stages(label, &facade, &main);
    for profile in [
        seseragi_project::BuildProfile::Development,
        seseragi_project::BuildProfile::Release,
    ] {
        let compiled = project(&facade, &main, profile)
            .unwrap_or_else(|error| panic!("{label} {profile:?}: {error:#?}"));
        let exports = &compiled.modules[FACADE_ID].typed_interface.exports;
        for (local, original, namespace) in
            [(box_name, "Box", "type"), (trait_name, "Render", "trait")]
        {
            let export = exports
                .iter()
                .find(|export| export.name == local && export.namespace == namespace)
                .unwrap();
            let expected = if namespace == "trait" {
                format!("{MODEL_ID}::trait({original})")
            } else {
                format!("{MODEL_ID}::{original}")
            };
            assert_eq!(export.symbol, expected);
            assert_eq!(export.scheme.type_parameters.len(), 1);
        }
        let rendered = exports
            .iter()
            .find(|export| export.name == "rendered")
            .unwrap();
        let model_rendered = compiled.modules[MODEL_ID]
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "rendered")
            .unwrap();
        assert_eq!(rendered.call_arity, model_rendered.call_arity);
        assert_eq!(
            rendered.scheme.constraints[0].trait_identity.as_deref(),
            Some(format!("{MODEL_ID}::trait(Render)").as_str())
        );
        assert!(compiled.modules[MAIN_ID]
            .typed_hir
            .external_type_bindings
            .iter()
            .any(|binding| binding.spelling == "Foreign"
                && binding.canonical == format!("{MODEL_ID}::Box")));
        let retained = compiled.modules[MAIN_ID]
            .typed_hir
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                seseragi_semantics::TypedDecl::Fn {
                    symbol, parameters, ..
                } if symbol == &format!("{MAIN_ID}::retained") => Some(parameters),
                _ => None,
            })
            .unwrap();
        assert_eq!(retained.len(), 1);
    }
}

#[test]
fn consumer_alias_control_keeps_generic_constraints_and_cold_closures() {
    verify("control", "Box", "Render");
}

#[test]
fn facade_type_rename_keeps_generic_constraints_and_cold_closures() {
    verify("type", "ExportedBox", "Render");
}

#[test]
fn facade_trait_rename_keeps_generic_constraints_and_cold_closures() {
    verify("trait", "Box", "ExportedTrait");
}

#[test]
fn facade_type_and_trait_renames_keep_generic_constraints_and_cold_closures() {
    verify("both", "ExportedBox", "ExportedTrait");
}

#[test]
fn renamed_opaque_type_stays_distinct_from_local_types_and_scoped_parameters() {
    let model = "pub foreign \"typescript\" from \"./host.mjs\" {\n opaque type Box<A>\n pure fn keep<A> value: Box<A> -> Box<A>\n}\n";
    let facade = "pub import {Box as Token, keep} from \"model\"\n";
    let main = "import {Token as Foreign, keep} from \"./facade\"\nstruct Box {value: Int}\npub fn own value: Box -> Int = value.value\npub fn scoped<Box> value: Box -> Box = value\npub fn forward value: Foreign<Int> -> Foreign<Int> = keep<Int> value\n";
    for profile in [
        seseragi_project::BuildProfile::Development,
        seseragi_project::BuildProfile::Release,
    ] {
        let compiled = project_with_model(model, facade, main, profile).unwrap();
        let owner = &compiled.modules[MODEL_ID]
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "Box" && export.namespace == "type")
            .unwrap()
            .symbol;
        assert!(compiled.modules[MAIN_ID]
            .typed_hir
            .external_type_bindings
            .iter()
            .any(|binding| binding.spelling == "Foreign" && binding.canonical == *owner));
        let wrong = main.replace(
            "value: Foreign<Int> -> Foreign<Int>",
            "value: Box -> Foreign<Int>",
        );
        let error = project_with_model(model, facade, &wrong, profile).unwrap_err();
        let seseragi_driver::ProjectCompileError::Diagnostics { modules } = error else {
            panic!("expected a source diagnostic, received {error:#?}");
        };
        let artifact = &modules
            .iter()
            .find(|module| module.module == MAIN_ID)
            .unwrap()
            .diagnostics;
        assert_eq!(artifact.source, "app/main.ssrg");
        let diagnostic = artifact
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message_key == "call.argument-type-mismatch")
            .unwrap();
        assert_eq!(diagnostic.code, "SES-T0101");
        assert_eq!(
            diagnostic.primary.start as usize,
            wrong.rfind("value").unwrap()
        );
        assert_eq!(
            diagnostic.primary.end as usize,
            wrong.rfind("value").unwrap() + "value".len()
        );
    }
}
