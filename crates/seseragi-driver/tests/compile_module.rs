use seseragi_driver::{compile_module, CompileInput};
use seseragi_lowering::GeneratedModule;
use seseragi_semantics::TypedModuleInterface;

fn input<'source>(
    source_name: &'source str,
    module_id: &'source str,
    source: &'source str,
) -> CompileInput<'source> {
    CompileInput::new(source_name, module_id, source)
}

#[test]
fn compiles_a_valid_module_through_every_owned_stage() {
    let compiled = compile_module(input(
        "artifact/driver-basic/main.ssrg",
        "artifact/driver-basic",
        "pub let answer: Int = 42\n",
    ))
    .expect("valid module should compile");

    assert!(compiled.diagnostics.diagnostics.is_empty());
    assert_eq!(compiled.typed_hir.stage, "typed-hir");
    assert_eq!(compiled.typed_interface.stage, "typed-interface");
    assert_eq!(compiled.core_ir.stage, "core-ir");
    assert_eq!(compiled.typescript_ir.stage, "typescript-ir");
    assert_eq!(compiled.generated.metadata.module, "artifact/driver-basic");
    assert_eq!(compiled.generated.metadata.exports, vec!["answer"]);
    assert_eq!(
        compiled.generated.typescript,
        "export const answer: bigint = 42n;\n"
    );
}

#[test]
fn keeps_physical_source_name_independent_from_logical_module_identity() {
    let compiled = compile_module(input(
        "/tmp/seseragi-cache/entry.ssrg",
        "game/domain",
        "pub let answer: Int = 42\n",
    ))
    .expect("explicit module identity should compile");

    assert_eq!(
        compiled.diagnostics.source,
        "/tmp/seseragi-cache/entry.ssrg"
    );
    assert_eq!(compiled.typed_hir.source, "entry.ssrg");
    assert_eq!(compiled.typed_hir.module, "game/domain");
    assert_eq!(compiled.typed_interface.module, "game/domain");
    assert_eq!(compiled.core_ir.module, "game/domain");
    assert_eq!(compiled.typescript_ir.module, "game/domain");
    assert_eq!(compiled.generated.metadata.module, "game/domain");
}

#[test]
fn carries_selected_derived_show_evidence_into_generated_metadata() {
    let source = "pub type AppError deriving Show =\n  | EndOfInput\n";
    let compiled = compile_module(input(
        "artifact/driver-show/main.ssrg",
        "artifact/driver-show",
        source,
    ))
    .expect("supported deriving clause should compile");

    assert_eq!(compiled.typed_hir.instances.len(), 1);
    assert_eq!(compiled.typed_interface.instances.len(), 1);
    assert_eq!(compiled.core_ir.instances.len(), 1);
    assert_eq!(compiled.typescript_ir.instances.len(), 1);
    assert_eq!(compiled.generated.metadata.instances.len(), 1);
    assert_eq!(
        compiled.generated.metadata.instances[0].type_identity,
        "artifact/driver-show::AppError"
    );
    assert_eq!(
        compiled.generated.metadata.instances[0].dictionary_export,
        "__ssrg$instance$Show$0"
    );
}

#[test]
fn rejects_semantically_invalid_source_before_producing_outputs() {
    let diagnostics = compile_module(input(
        "artifact/driver-invalid/main.ssrg",
        "artifact/driver-invalid",
        "pub fn broken value: Int -> Int = missing\n",
    ))
    .expect_err("unresolved name must prevent emission");

    assert!(!diagnostics.diagnostics.is_empty());
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-N0001"));
}

#[test]
fn rejects_imports_until_a_project_resolver_can_link_them() {
    let diagnostics = compile_module(input(
        "entry.ssrg",
        "demo@1.2.3::game/domain",
        "import * as text from \"std/text\"\npub let answer: Int = 42\n",
    ))
    .expect_err("unlinked imports must prevent all later compiler outputs");

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0104");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "module.specifier-unresolved"
    );
}

#[test]
fn reports_unlinked_imports_in_source_order() {
    let diagnostics = compile_module(input(
        "entry.ssrg",
        "artifact/driver-imports",
        "import * as support from \"./support\"\nimport * as text from \"std/text\"\npub let answer: Int = 42\n",
    ))
    .expect_err("a single-module driver cannot resolve imports");

    assert_eq!(diagnostics.diagnostics.len(), 2);
    assert_eq!(diagnostics.diagnostics[0].id, "d1");
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0104");
    assert_eq!(diagnostics.diagnostics[0].primary.start, 0);
    assert_eq!(diagnostics.diagnostics[1].id, "d2");
    assert_eq!(diagnostics.diagnostics[1].code, "SES-N0104");
    assert!(diagnostics.diagnostics[0].primary.start < diagnostics.diagnostics[1].primary.start);
}

#[test]
fn does_not_cascade_import_diagnostics_after_a_parse_error() {
    let diagnostics = compile_module(input(
        "entry.ssrg",
        "artifact/driver-parse-error",
        "import * as text from \"std/text\"\npub let answer: Int =",
    ))
    .expect_err("invalid source must stop before module linking");

    assert!(!diagnostics.diagnostics.is_empty());
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code != "SES-N0104"));
}

#[test]
fn physical_directory_spelling_does_not_change_semantic_outputs() {
    let source = "pub fn identity value: Int -> Int = value\n";
    let first = compile_module(input(
        "/tmp/seseragi-cache-a/main.ssrg",
        "artifact/driver-identity",
        source,
    ))
    .expect("first physical source should compile");
    let second = compile_module(input(
        "/var/tmp/seseragi-cache-b/main.ssrg",
        "artifact/driver-identity",
        source,
    ))
    .expect("second physical source should compile");

    assert_ne!(first.diagnostics.source, second.diagnostics.source);
    assert_eq!(first.typed_hir, second.typed_hir);
    assert_eq!(first.typed_interface, second.typed_interface);
    assert_eq!(first.core_ir, second.core_ir);
    assert_eq!(first.typescript_ir, second.typescript_ir);
    assert_eq!(first.generated, second.generated);
}

#[test]
fn compiles_the_cumulative_phase_one_program_to_canonical_outputs() {
    const SOURCE: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    const EXPECTED_TYPESCRIPT: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ts");
    const EXPECTED_TYPED_INTERFACE: &str = include_str!(
        "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/typed-interface.json"
    );
    const EXPECTED_METADATA: &str = include_str!(
        "../../../examples/spec/artifacts/schema-1/rock-paper-scissors-cli/generated-module.json"
    );

    let compiled = compile_module(input(
        "artifact/rock-paper-scissors-cli/main.ssrg",
        "artifact/rock-paper-scissors-cli",
        SOURCE,
    ))
    .expect("the Phase 1 goal program should compile through the public driver");
    let expected_typed_interface: TypedModuleInterface =
        serde_json::from_str(EXPECTED_TYPED_INTERFACE).expect("typed interface fixture is valid");
    let expected_metadata: GeneratedModule =
        serde_json::from_str(EXPECTED_METADATA).expect("generated metadata fixture is valid");

    assert_eq!(compiled.typed_interface, expected_typed_interface);
    assert_eq!(compiled.generated.typescript, EXPECTED_TYPESCRIPT);
    assert_eq!(compiled.generated.metadata, expected_metadata);
}
