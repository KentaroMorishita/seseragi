use seseragi_driver::{compile_module, relocate_foreign_outputs, CompileInput};
use std::collections::BTreeMap;
#[test]
fn runtime_locator_relocation_retains_the_compiler_contract_and_source_provenance() {
    let source = "pub foreign \"typescript\" from \"./host/api.mjs\" {\n  task fn read value: String -> String\n}\n";
    let original = compile_module(CompileInput::new(
        "main.ssrg",
        "fixture/standalone-host",
        source,
    ))
    .unwrap();
    let relocated = relocate_foreign_outputs(
        &original,
        &BTreeMap::from([(
            "./host/api.mjs".into(),
            "./.seseragi-foreign/host/api.mjs".into(),
        )]),
    );
    assert_eq!(original.typed_interface, relocated.typed_interface);
    assert_eq!(original.typed_hir, relocated.typed_hir);
    assert_eq!(original.core_ir, relocated.core_ir);
    assert_eq!(original.generated.metadata, relocated.generated.metadata);
    assert_eq!(
        original.generated.source_map.sources,
        relocated.generated.source_map.sources
    );
    assert_eq!(
        original.generated.source_map.sources_content,
        relocated.generated.source_map.sources_content
    );
    assert!(relocated
        .generated
        .typescript
        .contains("import(\"./.seseragi-foreign/host/api.mjs\")"));
    assert!(!relocated
        .generated
        .typescript
        .contains("import(\"./host/api.mjs\")"));
}
