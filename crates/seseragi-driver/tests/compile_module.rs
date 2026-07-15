use seseragi_driver::{
    compile_linked_module, compile_module, plan_typescript_outputs, CompileInput,
    LinkedCompileError, TypeScriptModuleOutput,
};
use seseragi_lowering::{
    GeneratedModule, TypeScriptFunction, TypeScriptLoweringError, TypeScriptOutputPlan,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_semantics::{TypedDecl, TypedModuleInterface};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

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
fn compiles_array_literals_through_every_owned_stage() {
    const SOURCE: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/array-literal/main.ssrg");
    const EXPECTED_TYPESCRIPT: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/array-literal/main.ts");
    let compiled = compile_module(input(
        "artifact/driver-array/main.ssrg",
        "artifact/driver-array",
        SOURCE,
    ))
    .expect("typed array literals should compile");

    assert!(compiled.diagnostics.diagnostics.is_empty());
    assert!(serde_json::to_string(&compiled.typed_hir)
        .unwrap()
        .contains("\"kind\":\"array\""));
    assert!(serde_json::to_string(&compiled.core_ir)
        .unwrap()
        .contains("\"kind\":\"array\""));
    assert!(serde_json::to_string(&compiled.typescript_ir)
        .unwrap()
        .contains("\"kind\":\"array\""));
    assert_eq!(compiled.generated.typescript, EXPECTED_TYPESCRIPT);
}

#[test]
fn compiles_a_higher_order_parameter_call_through_every_owned_stage() {
    let compiled = compile_module(input(
        "artifact/driver-higher-order/main.ssrg",
        "artifact/driver-higher-order",
        concat!(
            "pub fn apply f: (Int -> Int) -> value: Int -> Int = f value\n",
            "pub fn increment value: Int -> Int = value + 1\n",
            "pub fn example value: Int -> Int = apply increment value\n",
        ),
    ))
    .expect("higher-order parameter application should compile");

    assert!(compiled.diagnostics.diagnostics.is_empty());
    assert_eq!(
        compiled.generated.typescript,
        concat!(
            "import { add as _ssrg_int64_add } from \"@seseragi/runtime/int64\"\n",
            "\n",
            "export const apply = (f: (argument: bigint) => bigint) => (value: bigint) => f(value)\n",
            "export const increment = (value: bigint) => _ssrg_int64_add(value, 1n)\n",
            "export const example = (value: bigint) => apply(increment)(value)\n",
        )
    );
}

#[test]
fn instantiates_a_generic_higher_order_function() {
    const SOURCE: &str = include_str!(
        "../../../examples/spec/artifacts/schema-1/generic-higher-order-call/main.ssrg"
    );
    const EXPECTED_TYPESCRIPT: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/generic-higher-order-call/main.ts");
    let compiled = compile_module(input(
        "artifact/driver-generic-higher-order/main.ssrg",
        "artifact/driver-generic-higher-order",
        SOURCE,
    ))
    .expect("generic higher-order application should compile");

    assert!(compiled.diagnostics.diagnostics.is_empty());
    assert!(compiled
        .generated
        .typescript
        .contains("export const apply = <A, B,>(f: (argument: A) => B)"));
    assert!(compiled
        .generated
        .typescript
        .contains("apply(increment)(value)"));
    assert_eq!(compiled.generated.typescript, EXPECTED_TYPESCRIPT);
}

#[test]
fn passes_an_arithmetic_operator_as_a_function_value() {
    const SOURCE: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/operator-reference/main.ssrg");
    const EXPECTED_TYPESCRIPT: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/operator-reference/main.ts");
    let compiled = compile_module(input(
        "artifact/driver-operator-reference/main.ssrg",
        "artifact/driver-operator-reference",
        SOURCE,
    ))
    .expect("arithmetic operator reference should compile");

    assert!(compiled.diagnostics.diagnostics.is_empty());
    assert_eq!(compiled.generated.typescript, EXPECTED_TYPESCRIPT);
}

#[test]
fn preserves_constraint_arguments_through_owned_ir_stages() {
    const SOURCE: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/constraint-arguments/main.ssrg");
    let compiled = compile_module(input(
        "artifact/constraint-arguments/main.ssrg",
        "artifact/constraint-arguments",
        SOURCE,
    ))
    .expect("structured constraints should compile");

    let TypedDecl::Fn { scheme, .. } = &compiled.typed_hir.declarations[0] else {
        panic!("expected a typed function");
    };
    assert_eq!(scheme.constraints[0].name, "Reducible");
    assert_eq!(scheme.constraints[0].arguments.len(), 2);
    assert_eq!(
        compiled.core_ir.functions[0].constraints[0].arguments.len(),
        2
    );
    let TypeScriptFunction::ConstFunction { constraints, .. } =
        &compiled.typescript_ir.functions[0];
    assert_eq!(constraints[0].name, "Reducible");
    assert_eq!(constraints[0].arguments.len(), 2);
}

#[test]
fn compiles_standard_array_reduce_with_selected_instance_evidence() {
    const SOURCE: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/array-reduce/main.ssrg");
    const EXPECTED_TYPESCRIPT: &str =
        include_str!("../../../examples/spec/artifacts/schema-1/array-reduce/main.ts");
    let compiled = compile_module(input(
        "artifact/array-reduce/main.ssrg",
        "artifact/array-reduce",
        SOURCE,
    ))
    .expect("standard Array reduction should compile");

    let core = serde_json::to_string(&compiled.core_ir).unwrap();
    assert!(core.contains("std/array::Reducible"));
    assert_eq!(
        compiled.typescript_ir.runtime_requirements,
        vec![
            "core.unit",
            "core.bool",
            "core.array.reduce",
            "core.int64",
            "core.int64.add",
        ]
    );
    assert_eq!(compiled.generated.typescript, EXPECTED_TYPESCRIPT);
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
        compiled.generated.metadata.instances[0]
            .type_identity
            .as_deref(),
        Some("artifact/driver-show::AppError")
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

#[test]
fn compiles_a_linked_module_through_planned_typescript_output() {
    let domain_source = "pub fn increment value: Int -> Int = value + 1\n";
    let main_source =
        "import { increment as next } from \"./domain\"\n\npub fn run value: Int -> Int = next value\n";
    let linked = linked_module(main_source, domain_source);
    let plan = plan_typescript_outputs(
        "dist/game/main.js",
        [TypeScriptModuleOutput::new(
            "fixture/game::domain",
            "dist/game/domain.js",
        )],
    )
    .unwrap();

    let compiled = compile_linked_module(linked, main_source, &plan)
        .expect("linked module and complete output plan should compile");

    assert_eq!(compiled.typed_hir.module_dependencies.len(), 1);
    assert_eq!(compiled.core_ir.module_dependencies.len(), 1);
    assert_eq!(compiled.typescript_ir.source_imports.len(), 1);
    assert!(compiled
        .generated
        .typescript
        .contains("import { increment as next } from \"./domain.js\""));
    assert!(compiled
        .generated
        .typescript
        .contains("export const run = (value: bigint) => next(value)"));
}

#[test]
fn classifies_a_missing_linked_output_specifier_as_a_plan_error() {
    let domain_source = "pub fn increment value: Int -> Int = value + 1\n";
    let main_source =
        "import { increment } from \"./domain\"\n\npub fn run value: Int -> Int = increment value\n";
    let linked = linked_module(main_source, domain_source);

    let error = compile_linked_module(linked, main_source, &TypeScriptOutputPlan::default())
        .expect_err("linked emission requires one output specifier per dependency module");

    assert_eq!(
        error,
        LinkedCompileError::TypeScriptPlan(TypeScriptLoweringError::MissingOutputSpecifier {
            module: "fixture/game::domain".to_owned(),
            source_specifier: "./domain".to_owned(),
        })
    );
}

fn linked_module(main_source: &str, domain_source: &str) -> seseragi_project::LinkedModule {
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let interface = compile_module(input("domain.ssrg", "fixture/game::domain", domain_source))
        .expect("dependency module should compile through the public driver")
        .typed_interface
        .into_link_interface();
    let target = ModuleLinkTarget::same_package(domain.header, interface).unwrap();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap()
}
