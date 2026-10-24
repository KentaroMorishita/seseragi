use seseragi_lowering::{
    emit_typescript_module, lower_core_module_to_typescript_ir, lower_typed_module,
};

fn compile(source_name: &str, source: &str) -> seseragi_lowering::GeneratedBundle {
    let typed = seseragi_semantics::type_module(source_name, source);
    let core = lower_typed_module(typed);
    let typescript = lower_core_module_to_typescript_ir(core);
    emit_typescript_module(typescript, source)
}

#[test]
fn imports_runtime_error_types_used_by_adt_payloads() {
    let source =
        "pub type AppError =\n  | StdinFailure StdinError\n  | ConsoleFailure ConsoleError\n";

    let generated = compile("artifact/runtime-errors/main.ssrg", source);

    assert!(generated
        .typescript
        .contains("import { type StdinError as StdinError } from \"@seseragi/runtime/stdin\""));
    assert!(generated.typescript.contains(
        "import { type ConsoleError as ConsoleError } from \"@seseragi/runtime/console\""
    ));
    assert!(generated
        .typescript
        .contains("readonly tag: \"StdinFailure\"; readonly value: StdinError"));
    assert!(generated
        .typescript
        .contains("readonly tag: \"ConsoleFailure\"; readonly value: ConsoleError"));
    assert!(generated
        .metadata
        .runtime
        .requirements
        .contains(&"effect.stdin.error".to_owned()));
    assert!(generated
        .metadata
        .runtime
        .requirements
        .contains(&"effect.console.error".to_owned()));
}

#[test]
fn does_not_import_a_runtime_type_for_a_local_shadow() {
    let source =
        "pub type StdinError = | LocalStdinError\npub type AppError = | StdinFailure StdinError\n";

    let generated = compile("artifact/runtime-error-shadow/main.ssrg", source);

    assert!(!generated.typescript.contains("@seseragi/runtime/stdin"));
    assert!(!generated
        .metadata
        .runtime
        .requirements
        .contains(&"effect.stdin.error".to_owned()));
}

#[test]
fn public_alias_projection_collects_target_runtime_types_but_respects_own_binders() {
    let generated = compile(
        "artifact/alias-runtime-target/main.ssrg",
        "pub alias Input<A> = { value: A, error: StdinError }\n",
    );
    assert!(generated
        .typescript
        .contains("import { type StdinError as StdinError } from \"@seseragi/runtime/stdin\""));
    assert!(generated.typescript.contains("export type Input<A>"));
    assert!(generated
        .metadata
        .runtime
        .requirements
        .contains(&"effect.stdin.error".to_owned()));
    let bound = compile(
        "artifact/alias-runtime-binder/main.ssrg",
        "pub alias Identity<StdinError> = StdinError\n",
    );
    assert!(bound
        .typescript
        .contains("export type Identity<StdinError> = StdinError"));
    assert!(!bound.typescript.contains("@seseragi/runtime/stdin"));
    assert!(!bound
        .metadata
        .runtime
        .requirements
        .contains(&"effect.stdin.error".to_owned()));
}
