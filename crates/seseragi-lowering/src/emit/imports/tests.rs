use super::render_source_imports;
use crate::{SourceSpan, TypeScriptModule, TypeScriptSourceImport, TypeScriptSourceImportBinding};

fn binding(type_only: bool) -> TypeScriptSourceImportBinding {
    TypeScriptSourceImportBinding {
        imported: "Label".into(),
        local: "Label".into(),
        source_local: "Label".into(),
        canonical: "owner/model::Label".into(),
        type_only,
        origin: SourceSpan {
            source: "facade.ssrg".into(),
            start: 0,
            end: 20,
        },
    }
}
fn lines(reexports: Vec<TypeScriptSourceImportBinding>) -> Vec<String> {
    render_source_imports(&TypeScriptModule {
        higher_kinds: None,
        closed_display_dictionaries: Vec::new(),
        schema: 1,
        stage: "typescript-ir".into(),
        module: "facade".into(),
        foreign_modules: vec![],
        runtime_requirements: vec![],
        imports: vec![],
        type_imports: vec![],
        source_imports: vec![TypeScriptSourceImport {
            reexports,
            module: "owner/model".into(),
            specifier: "./model.js".into(),
            runtime_edge: true,
            bindings: vec![],
            origin: binding(true).origin,
        }],
        adts: vec![],
        aliases: vec![],
        structs: vec![],
        instances: vec![],
        bindings: vec![],
        functions: vec![],
    })
}
#[test]
fn merged_reexports_keep_the_value_and_initialization_in_either_order() {
    for reexports in [
        vec![binding(true), binding(false)],
        vec![binding(false), binding(true)],
    ] {
        assert_eq!(
            lines(reexports),
            vec![
                "export { Label } from \"./model.js\"",
                "import \"./model.js\""
            ]
        );
    }
}
#[test]
fn type_only_reexports_and_distinct_aliases_or_identities_are_not_erased() {
    assert_eq!(
        lines(vec![binding(true)])[0],
        "export { type Label } from \"./model.js\""
    );
    let mut value = binding(false);
    value.local = "Renamed".into();
    assert_eq!(
        lines(vec![binding(true), value])[0],
        "export { type Label, Label as Renamed } from \"./model.js\""
    );
    let mut value = binding(false);
    value.canonical = "other/model::Label".into();
    assert!(lines(vec![binding(true), value])[0].contains("type Label, Label"));
    let mut value = binding(false);
    value.imported = "Other".into();
    assert!(lines(vec![binding(true), value])[0].contains("type Label, Other as Label"));
}
