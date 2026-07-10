use crate::typed::{analyze_pure_function, TypedResolution};
use seseragi_syntax::{lex, parse_diagnostics, Diagnostic, DiagnosticArtifact, SurfaceDecl, Token};

mod conditional;
mod effect;
mod function_body;
mod let_binding;
mod pure_call;
mod resolution;
mod type_labels;

pub fn semantic_diagnostics(source_name: impl Into<String>, source: &str) -> DiagnosticArtifact {
    let source_name = source_name.into();
    let mut artifact = parse_diagnostics(source_name.clone(), source);
    if !artifact.diagnostics.is_empty() {
        return artifact;
    }

    let resolved = crate::resolve_module(source_name, source);
    let has_effect_functions = resolved
        .declarations
        .iter()
        .any(|declaration| matches!(declaration, SurfaceDecl::EffectFn { .. }));
    let tokens = if has_effect_functions {
        lex(artifact.source.clone(), source).tokens
    } else {
        Vec::new()
    };
    let resolution = TypedResolution::new(&resolved);
    let mut diagnostics = Vec::new();

    for declaration in &resolved.declarations {
        collect_decl_diagnostics(declaration, &tokens, &resolution, &mut diagnostics);
    }
    resolution::collect_resolution_diagnostics(&resolved, &mut diagnostics);

    artifact.diagnostics = diagnostics
        .into_iter()
        .enumerate()
        .map(|(index, mut diagnostic)| {
            diagnostic.id = format!("d{}", index + 1);
            diagnostic
        })
        .collect();
    artifact
}

fn collect_decl_diagnostics(
    declaration: &SurfaceDecl,
    tokens: &[Token],
    resolution: &TypedResolution<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let SurfaceDecl::Let {
        type_ref,
        body,
        span,
        ..
    } = declaration
    {
        let_binding::collect_let_binding_diagnostics(
            type_ref.as_ref(),
            body.as_ref(),
            *span,
            resolution,
            diagnostics,
        );
        return;
    }

    if let SurfaceDecl::Fn {
        parameters,
        return_type,
        body,
        span,
        ..
    } = declaration
    {
        let analysis = analyze_pure_function(body.as_ref(), parameters, return_type, resolution);
        conditional::collect_conditional_diagnostics(
            analysis.conditional_issue.as_ref(),
            *span,
            diagnostics,
        );
        function_body::collect_function_body_diagnostics(
            analysis.function_body_issue.as_ref(),
            *span,
            diagnostics,
        );
        pure_call::collect_pure_function_diagnostics(&analysis, *span, diagnostics);
        return;
    }

    effect::collect_effect_fn_diagnostics(declaration, tokens, diagnostics);
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::ByteRange;

    #[test]
    fn reports_compact_effect_body_that_is_not_effect() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-not-effect/main.ssrg",
            "pub effect fn greet name: String = name\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].id, "d1");
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.compact-body-not-effect"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 35, end: 39 }
        );
        assert_eq!(diagnostics.diagnostics[0].related.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 39 }
        );
    }

    #[test]
    fn reports_unknown_compact_do_statement() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-not-effect/main.ssrg",
            "pub effect fn greet =\n  do { name }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 29, end: 33 }
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 35 }
        );
    }

    #[test]
    fn reports_unknown_compact_do_statement_after_known_statement() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-late-not-effect/main.ssrg",
            "pub effect fn greet =\n  do { println \"hello\" name }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 45, end: 49 }
        );
    }

    #[test]
    fn reports_contract_clause_in_compact_effect_fn() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-with-clause/main.ssrg",
            "pub effect fn main with Console =\n  println \"hello\"\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.compact-contract-clause"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 19, end: 23 }
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].primary,
            ByteRange { start: 0, end: 51 }
        );
    }

    #[test]
    fn reports_do_bind_without_final_monadic_expression() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-bind/main.ssrg",
            "pub effect fn main =\n  do { line <- readLine () }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-P0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.do-missing-final-expression"
        );
    }

    #[test]
    fn accepts_do_bind_followed_by_final_monadic_expression() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-do-bind-result/main.ssrg",
            "pub effect fn main =\n  do {\n    line <- readLine ()\n    succeed ()\n  }\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn reports_multiple_non_never_failures_in_compact_effect() {
        let diagnostics = semantic_diagnostics(
            "artifact/effect-compact-failure-conflict/main.ssrg",
            "pub effect fn main =\n  do { readLine () println \"done\" }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-E0001");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.compact-failure-conflict"
        );
        assert_eq!(diagnostics.diagnostics[0].related.len(), 2);
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "operation can fail with StdinError"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[1].message,
            "operation can fail with ConsoleError"
        );
    }

    #[test]
    fn reports_unresolved_name_in_pure_function_body() {
        let diagnostics = semantic_diagnostics(
            "artifact/unknown-pure-name/main.ssrg",
            "pub fn useMissing value: Int -> Int = missing\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
        assert_eq!(diagnostics.diagnostics[0].message_key, "name.unresolved");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 38, end: 45 }
        );
    }

    #[test]
    fn accepts_top_level_binding_in_pure_function_body() {
        let diagnostics = semantic_diagnostics(
            "artifact/top-level-binding/main.ssrg",
            "pub let answer: Int = 42\npub fn answerValue unit: Unit -> Int = answer\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn reports_function_value_return_type_mismatch_instead_of_arity() {
        let diagnostics = semantic_diagnostics(
            "artifact/function-value-reference/main.ssrg",
            "fn source unit: Unit -> Int = 1\nfn alias unit: Unit -> Int = source\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "function.return-type-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 61, end: 67 }
        );
    }

    #[test]
    fn accepts_saturated_top_level_pure_function_call() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-diagnostics/main.ssrg",
            "fn identity value: Int -> Int = value\nfn use value: Int -> Int = identity value\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn treats_too_few_arguments_as_partial_application() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-too-few/main.ssrg",
            "fn add left: Int -> right: Int -> Int = left + right\nfn use value: Int -> Int = add value\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "function.return-type-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "declared Int, body produces function"
        );
    }

    #[test]
    fn reports_known_function_with_too_many_arguments_as_arity_mismatch() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-too-many/main.ssrg",
            "fn identity value: Int -> Int = value\nfn use value: Int -> Int = identity value 1\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "call.arity-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "expected 1 argument, received 2"
        );
    }

    #[test]
    fn reports_known_function_argument_type_mismatch() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-type-mismatch/main.ssrg",
            "fn identity value: Int -> Int = value\nfn use unit: Unit -> Int = identity \"wrong\"\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "argument 1 expected Int, received String"
        );
        assert_ne!(diagnostics.diagnostics[0].message_key, "name.unresolved");
    }

    #[test]
    fn reports_payload_constructor_argument_type_mismatch() {
        let diagnostics = semantic_diagnostics(
            "artifact/constructor-type-mismatch/main.ssrg",
            "type Label = | Present String\nfn invalid unit: Unit -> Label = Present 1\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "argument 1 expected String, received Int"
        );
    }

    #[test]
    fn reports_nullary_constructor_overapplication() {
        let diagnostics = semantic_diagnostics(
            "artifact/constructor-arity/main.ssrg",
            "type Hand = | Rock\nfn invalid unit: Unit -> Hand = Rock ()\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "call.arity-mismatch"
        );
        assert_eq!(
            diagnostics.diagnostics[0].related[0].message,
            "expected 0 arguments, received 1"
        );
    }

    #[test]
    fn reports_only_the_unknown_argument_of_a_known_function() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-unknown-argument/main.ssrg",
            "fn identity value: Int -> Int = value\nfn use unit: Unit -> Int = identity missing\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
        assert_eq!(diagnostics.diagnostics[0].message_key, "name.unresolved");
        assert_eq!(
            diagnostics.diagnostics[0].primary,
            ByteRange { start: 74, end: 81 }
        );
    }

    #[test]
    fn accepts_nested_conditional_as_a_typed_call_argument() {
        let diagnostics = semantic_diagnostics(
            "artifact/pure-call-expression-argument/main.ssrg",
            "fn identity value: Int -> Int = value\nfn use unit: Unit -> Int = identity (if True then 1 else 2)\n",
        );

        assert!(diagnostics.diagnostics.is_empty());
    }

    #[test]
    fn reports_non_effect_value_in_explicit_do_bind() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-do-bind/main.ssrg",
            "effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { ignored <- missing }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.bind-value-not-effect"
        );
    }

    #[test]
    fn reports_non_effect_statement_in_explicit_do_block() {
        let diagnostics = semantic_diagnostics(
            "artifact/invalid-explicit-do/main.ssrg",
            "effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do { missing }\n",
        );

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "effect.do-statement-not-effect"
        );
    }
}
