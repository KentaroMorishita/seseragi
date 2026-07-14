use crate::{
    lower_core_module_to_typescript_ir, lower_typed_module, TypeScriptInstanceImplementation,
    TypeScriptShowDictionaryReference, TypeScriptType,
};
use seseragi_semantics::type_module;

#[test]
fn lowers_derived_show_to_a_fully_resolved_render_plan() {
    let source = "\
pub type Detail deriving Show =
  | Detail String

pub type AppError deriving Show =
  | Wrapped Detail
  | EndOfInput
";
    let core = lower_typed_module(type_module("artifact/derived-show/main.ssrg", source));
    let typescript = lower_core_module_to_typescript_ir(core);

    assert_eq!(typescript.instances.len(), 2);
    assert_eq!(
        typescript.runtime_requirements,
        vec![
            "core.adt",
            "core.string",
            "core.show.dictionary",
            "core.string.show",
        ]
    );
    assert_eq!(
        typescript.type_imports,
        vec![crate::TypeScriptTypeImport {
            feature: "core.show.dictionary".to_owned(),
            local: "_ssrg_show_Show".to_owned(),
        }]
    );
    assert_eq!(
        typescript.imports,
        vec![crate::TypeScriptImport {
            feature: "core.string.show".to_owned(),
            local: "_ssrg_show_stringShow".to_owned(),
        }]
    );

    let detail = &typescript.instances[0];
    assert_eq!(detail.identity, "Show<artifact/derived-show::Detail>");
    assert_eq!(detail.trait_name, "Show");
    assert_eq!(detail.dictionary_export, "__ssrg$instance$Show$0");
    assert_eq!(
        detail.arguments,
        vec![TypeScriptType::Reference {
            name: "Detail".to_owned(),
            arguments: Vec::new(),
        }]
    );
    let TypeScriptInstanceImplementation::DerivedShow { adt_name, variants } =
        &detail.implementation;
    assert_eq!(adt_name, "Detail");
    let payload = variants[0].payload.as_ref().unwrap();
    assert_eq!(payload.type_ref, TypeScriptType::String);
    assert_eq!(
        payload.dictionary,
        TypeScriptShowDictionaryReference::Runtime {
            identity: "Show<std/prelude::String>".to_owned(),
            feature: "core.string.show".to_owned(),
            local: "_ssrg_show_stringShow".to_owned(),
        }
    );

    let app_error = &typescript.instances[1];
    assert_eq!(app_error.dictionary_export, "__ssrg$instance$Show$1");
    let TypeScriptInstanceImplementation::DerivedShow { adt_name, variants } =
        &app_error.implementation;
    assert_eq!(adt_name, "AppError");
    assert_eq!(variants[0].name, "Wrapped");
    assert_eq!(variants[0].tag, "Wrapped");
    assert_eq!(
        variants[0].payload.as_ref().unwrap().dictionary,
        TypeScriptShowDictionaryReference::Local {
            identity: "Show<artifact/derived-show::Detail>".to_owned(),
            dictionary_export: "__ssrg$instance$Show$0".to_owned(),
        }
    );
    assert!(variants[1].payload.is_none());
}

#[test]
fn emits_no_show_requirements_or_imports_without_instances() {
    let source = "\
pub type AppError =
  | EndOfInput
";
    let core = lower_typed_module(type_module("artifact/no-show/main.ssrg", source));
    let typescript = lower_core_module_to_typescript_ir(core);

    assert!(typescript.instances.is_empty());
    assert_eq!(typescript.runtime_requirements, vec!["core.adt"]);
    assert!(typescript.imports.is_empty());
    assert!(typescript.type_imports.is_empty());
}

#[test]
fn keeps_render_plan_dictionary_references_aligned_when_imports_are_freshened() {
    let source = "\
pub fn _ssrg_show_stringShow value: String -> String = value

pub type Detail deriving Show =
  | Detail String
";
    let core = lower_typed_module(type_module(
        "artifact/show-name-collision/main.ssrg",
        source,
    ));
    let typescript = lower_core_module_to_typescript_ir(core);

    assert_eq!(
        typescript.imports,
        vec![crate::TypeScriptImport {
            feature: "core.string.show".to_owned(),
            local: "_ssrg_show_stringShow_1".to_owned(),
        }]
    );
    let TypeScriptInstanceImplementation::DerivedShow { variants, .. } =
        &typescript.instances[0].implementation;
    assert_eq!(
        variants[0].payload.as_ref().unwrap().dictionary,
        TypeScriptShowDictionaryReference::Runtime {
            identity: "Show<std/prelude::String>".to_owned(),
            feature: "core.string.show".to_owned(),
            local: "_ssrg_show_stringShow_1".to_owned(),
        }
    );
}
