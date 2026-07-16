use crate::typed::MonadDoIssue;
use seseragi_syntax::{ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic};

use super::type_labels::type_label;

pub(super) fn collect_monad_do_diagnostic(
    issue: Option<&MonadDoIssue>,
    function_span: ByteSpan,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(issue) = issue else {
        return;
    };
    let (message_key, primary, message) = match issue {
        MonadDoIssue::ResultTypeNotMonadic { expression, actual } => (
            "do.result-type-not-monadic",
            *expression,
            format!(
                "do block expected a unary monadic result type, received {}",
                type_label(actual)
            ),
        ),
        MonadDoIssue::ConstructorMismatch {
            expression,
            expected,
            actual,
        } => (
            "do.monad-constructor-mismatch",
            *expression,
            format!(
                "all do expressions must use {}; received {}",
                type_label(expected),
                type_label(actual)
            ),
        ),
        MonadDoIssue::RefutableBindPattern { pattern } => (
            "do.refutable-bind-pattern",
            *pattern,
            "do bind patterns must be irrefutable; use an exhaustive match after binding"
                .to_owned(),
        ),
        MonadDoIssue::UnsupportedBindPattern { pattern } => (
            "do.binding-pattern-not-supported",
            *pattern,
            "this irrefutable binding pattern is not connected to Monad do lowering yet".to_owned(),
        ),
        MonadDoIssue::MissingFinalExpression { do_block } => (
            "do.missing-final-expression",
            *do_block,
            "do block requires a final monadic expression".to_owned(),
        ),
    };
    diagnostics.push(Diagnostic {
        id: String::new(),
        code: "SES-T0101".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: message_key.to_owned(),
        primary: ByteRange {
            start: primary.start,
            end: primary.end,
        },
        related: vec![RelatedDiagnostic {
            message,
            primary: ByteRange {
                start: function_span.start,
                end: function_span.end,
            },
        }],
        fixes: Vec::new(),
    });
}

#[cfg(test)]
mod tests {
    use crate::semantic_diagnostics;
    use seseragi_syntax::ByteRange;

    const PRELUDE: &str = r#"trait Functor<F<_>> {
  fn map<A, B> f: (A -> B) -> value: F<A> -> F<B>
}
trait Applicative<F<_>> where Functor<F> {
  fn pure<A> value: A -> F<A>
}
trait Monad<M<_>> where Applicative<M> {
  fn flatMap<A, B> f: (A -> M<B>) -> value: M<A> -> M<B>
}
instance Functor<Maybe> {
  fn map<A, B> f: (A -> B) -> value: Maybe<A> -> Maybe<B> =
    match value {
      Nothing -> Nothing
      Just item -> Just $ f item
    }
}
instance Applicative<Maybe> {
  fn pure<A> value: A -> Maybe<A> = Just value
}
instance Monad<Maybe> {
  fn flatMap<A, B> f: (A -> Maybe<B>) -> value: Maybe<A> -> Maybe<B> =
    match value {
      Nothing -> Nothing
      Just item -> f item
    }
}
"#;

    #[test]
    fn reports_a_refutable_monad_bind_pattern_at_the_pattern() {
        let source = format!(
            "{PRELUDE}fn broken value: Maybe<Int> -> Maybe<Int> = do {{ Just item <- value; pure item }}\n"
        );
        let artifact = semantic_diagnostics("refutable-monad-do.ssrg", &source);
        let start = source.rfind("Just item").unwrap();

        assert_eq!(artifact.diagnostics.len(), 1, "{:#?}", artifact.diagnostics);
        assert_eq!(artifact.diagnostics[0].code, "SES-T0101");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "do.refutable-bind-pattern"
        );
        assert_eq!(
            artifact.diagnostics[0].primary,
            ByteRange {
                start,
                end: start + "Just item".len()
            }
        );
    }

    #[test]
    fn reports_a_different_constructor_inside_monad_do() {
        let source = format!(
            "{PRELUDE}fn broken value: Maybe<Int> -> Maybe<Int> = do {{ value; Right 42 }}\n"
        );
        let artifact = semantic_diagnostics("mixed-monad-do.ssrg", &source);
        let start = source.rfind("Right 42").unwrap();

        assert_eq!(artifact.diagnostics.len(), 1, "{:#?}", artifact.diagnostics);
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "do.monad-constructor-mismatch"
        );
        assert_eq!(
            artifact.diagnostics[0].primary,
            ByteRange {
                start,
                end: start + "Right 42".len()
            }
        );
    }

    #[test]
    fn reports_a_missing_final_monadic_expression() {
        let source = format!(
            "{PRELUDE}fn broken value: Maybe<Int> -> Maybe<Int> = do {{ item <- value }}\n"
        );
        let artifact = semantic_diagnostics("unfinished-monad-do.ssrg", &source);
        let start = source.rfind("do {").unwrap();

        assert_eq!(artifact.diagnostics.len(), 1, "{:#?}", artifact.diagnostics);
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "do.missing-final-expression"
        );
        assert_eq!(artifact.diagnostics[0].primary.start, start);
    }
}
