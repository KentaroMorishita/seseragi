use super::linked_program;
use crate::{
    analyze_linked_module, resolve_linked_module, TypedDecl, TypedDoStatement, TypedExpr, TypedType,
};

const DOMAIN_SOURCE: &str = "pub effect fn prompt label: String -> Maybe<String>\nwith Stdin\nfails StdinError =\n  readLine ()\n";

#[test]
fn types_a_saturated_imported_effect_as_a_cold_do_bind_value() {
    let main_source = "import { prompt } from \"./domain\"\n\npub effect fn ask =\n  do {\n    value <- prompt \"hand\"\n    succeed value\n  }\n";
    let analyzed = analyze(main_source).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::EffectFn {
            body: TypedExpr::DoBlock { statements, .. },
            effect,
            ..
        } if matches!(
                statements.as_slice(),
                [TypedDoStatement::Bind {
                    type_ref,
                    value: TypedExpr::EffectInvoke {
                        callee,
                        effect,
                        arguments,
                        ..
                    },
                    ..
                }] if callee == "fixture/game::domain::prompt"
                    && arguments.len() == 1
                    && type_ref == &maybe_string()
                    && effect.success == maybe_string()
            )
            && effect.success == maybe_string()
    ));
}

#[test]
fn keeps_an_imported_effect_partial_application_as_a_typed_pure_call() {
    let main_source = "import { prompt } from \"./domain\"\n\npub effect fn ask = prompt\n";
    let linked = linked(main_source);
    let typed =
        crate::typed::typed_module_from_resolved(resolve_linked_module(linked, main_source));

    assert!(matches!(
        &typed.declarations[0],
        TypedDecl::EffectFn {
            body: TypedExpr::Call {
                callee,
                arguments,
                type_ref: TypedType::Function { result, .. },
                ..
            },
            ..
        } if callee == "fixture/game::domain::prompt"
            && arguments.is_empty()
            && matches!(
                result.as_ref(),
                TypedType::Named { name, arguments }
                    if name == "Effect" && arguments.len() == 3
            )
    ));
}

#[test]
fn reports_imported_effect_argument_and_arity_mismatches_as_call_diagnostics() {
    for (body, message_key) in [
        ("prompt 1", "call.argument-type-mismatch"),
        ("prompt \"hand\" \"extra\"", "call.arity-mismatch"),
    ] {
        let main_source =
            format!("import {{ prompt }} from \"./domain\"\n\npub effect fn ask = {body}\n");
        let diagnostics = analyze(&main_source).unwrap_err();
        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(diagnostics.diagnostics[0].message_key, message_key);
    }
}

fn analyze(
    main_source: &str,
) -> Result<crate::AnalyzedModule, seseragi_syntax::DiagnosticArtifact> {
    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked(main_source),
        main_source,
    )
}

fn linked(main_source: &str) -> seseragi_project::LinkedModule {
    linked_program(
        main_source,
        [("./domain", "fixture/game::domain", DOMAIN_SOURCE)],
    )
}

fn maybe_string() -> TypedType {
    TypedType::Named {
        name: "Maybe".to_owned(),
        arguments: vec![TypedType::Named {
            name: "String".to_owned(),
            arguments: Vec::new(),
        }],
    }
}
