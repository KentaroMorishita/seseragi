use super::{ByteRange, Diagnostic, DiagnosticFix, DiagnosticSeverity, RelatedDiagnostic};
use crate::{
    SurfaceBlockItem, SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr,
    SurfaceImplMember, SurfacePattern, SurfaceTemplatePart,
};
use std::collections::BTreeSet;

pub(super) fn diagnostics(
    declarations: &[SurfaceDecl],
    existing: &[Diagnostic],
) -> Vec<Diagnostic> {
    let mut spans = Errors::default();
    for declaration in declarations {
        match declaration {
            SurfaceDecl::Let { pattern, body, .. } => {
                collect_pattern_errors(pattern, &mut spans);
                if let Some(body) = body {
                    collect_expression_errors(body, &mut spans);
                }
            }
            SurfaceDecl::Fn {
                body: Some(body), ..
            }
            | SurfaceDecl::EffectFn {
                body: Some(body), ..
            }
            | SurfaceDecl::Operator {
                body: Some(body), ..
            } => collect_expression_errors(body, &mut spans),
            SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => {
                for method in methods {
                    if let Some(body) = &method.body {
                        collect_expression_errors(body, &mut spans);
                    }
                }
            }
            SurfaceDecl::Impl { members, .. } => {
                for member in members {
                    let body = match member {
                        SurfaceImplMember::Method { method, .. } => method.body.as_ref(),
                        SurfaceImplMember::Operator { body, .. } => body.as_ref(),
                    };
                    if let Some(body) = body {
                        collect_expression_errors(body, &mut spans);
                    }
                }
            }
            _ => {}
        }
    }

    let mut seen = BTreeSet::new();
    spans
        .expressions
        .into_iter()
        .map(expected_expression)
        .chain(spans.patterns.into_iter().map(expected_pattern))
        .filter(|diagnostic| seen.insert((diagnostic.primary.start, diagnostic.primary.end)))
        .filter(|diagnostic| !covered_by_existing_error(diagnostic.primary, existing))
        .collect()
}

#[derive(Default)]
struct Errors {
    expressions: Vec<ByteRange>,
    patterns: Vec<ByteRange>,
}

fn collect_pattern_errors(pattern: &SurfacePattern, errors: &mut Errors) {
    match pattern {
        SurfacePattern::Error { span } => errors.patterns.push(ByteRange {
            start: span.start,
            end: span.end,
        }),
        SurfacePattern::Constructor { argument, .. } => {
            if let Some(argument) = argument {
                collect_pattern_errors(argument, errors);
            }
        }
        SurfacePattern::Tuple { elements, .. } => {
            for element in elements {
                collect_pattern_errors(element, errors);
            }
        }
        SurfacePattern::Array { elements, rest, .. }
        | SurfacePattern::List { elements, rest, .. } => {
            for element in elements {
                collect_pattern_errors(element, errors);
            }
            if let Some(rest) = rest {
                collect_pattern_errors(rest, errors);
            }
        }
        SurfacePattern::Record { fields, .. } | SurfacePattern::Struct { fields, .. } => {
            for field in fields {
                collect_pattern_errors(&field.pattern, errors);
            }
        }
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Char { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Name { .. }
        | SurfacePattern::Wildcard { .. } => {}
    }
}

fn collect_expression_errors(expression: &SurfaceExpr, errors: &mut Errors) {
    match expression {
        SurfaceExpr::Application {
            function, argument, ..
        }
        | SurfaceExpr::Index {
            receiver: function,
            index: argument,
            ..
        } => {
            collect_expression_errors(function, errors);
            collect_expression_errors(argument, errors);
        }
        SurfaceExpr::Prefix { operand, .. } => collect_expression_errors(operand, errors),
        SurfaceExpr::Assignment { target, value, .. } => {
            collect_expression_errors(target, errors);
            collect_expression_errors(value, errors);
        }
        SurfaceExpr::Member { receiver, .. } => collect_expression_errors(receiver, errors),
        SurfaceExpr::Tuple { elements, .. }
        | SurfaceExpr::Array { elements, .. }
        | SurfaceExpr::List { elements, .. } => {
            for element in elements {
                collect_expression_errors(element, errors);
            }
        }
        SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
            for item in items {
                collect_expression_errors(item.value(), errors);
            }
        }
        SurfaceExpr::Template { parts, .. } => {
            for part in parts {
                if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                    collect_expression_errors(value, errors);
                }
            }
        }
        SurfaceExpr::ArrayComprehension {
            element, clauses, ..
        }
        | SurfaceExpr::ListComprehension {
            element, clauses, ..
        } => {
            collect_expression_errors(element, errors);
            for clause in clauses {
                let expression = match clause {
                    SurfaceComprehensionClause::Generator {
                        pattern, source, ..
                    } => {
                        collect_pattern_errors(pattern, errors);
                        source
                    }
                    SurfaceComprehensionClause::Guard { condition, .. } => condition,
                };
                collect_expression_errors(expression, errors);
            }
        }
        SurfaceExpr::Binary { left, right, .. } => {
            collect_expression_errors(left, errors);
            collect_expression_errors(right, errors);
        }
        SurfaceExpr::InfixChain { first, steps, .. } => {
            collect_expression_errors(first, errors);
            for step in steps {
                collect_expression_errors(&step.operand, errors);
            }
        }
        SurfaceExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expression_errors(condition, errors);
            collect_expression_errors(then_branch, errors);
            collect_expression_errors(else_branch, errors);
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_expression_errors(scrutinee, errors);
            for arm in arms {
                collect_pattern_errors(&arm.pattern, errors);
                if let Some(guard) = &arm.guard {
                    collect_expression_errors(guard, errors);
                }
                collect_expression_errors(&arm.body, errors);
            }
        }
        SurfaceExpr::Block { items, result, .. } => {
            for item in items {
                let value = match item {
                    SurfaceBlockItem::Let { pattern, value, .. } => {
                        collect_pattern_errors(pattern, errors);
                        value
                    }
                    SurfaceBlockItem::Function { value, .. } => value,
                };
                collect_expression_errors(value, errors);
            }
            collect_expression_errors(result, errors);
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                let value = match item {
                    SurfaceDoItem::Bind { pattern, value, .. }
                    | SurfaceDoItem::Let { pattern, value, .. } => {
                        collect_pattern_errors(pattern, errors);
                        value
                    }
                    SurfaceDoItem::Expression { value, .. } => value,
                };
                collect_expression_errors(value, errors);
            }
            if let Some(result) = result {
                collect_expression_errors(result, errors);
            }
        }
        SurfaceExpr::Grouped { value, .. } => collect_expression_errors(value, errors),
        SurfaceExpr::Lambda { body, .. } => collect_expression_errors(body, errors),
        SurfaceExpr::EffectfulFor {
            pattern,
            source,
            body,
            ..
        } => {
            collect_pattern_errors(pattern, errors);
            collect_expression_errors(source, errors);
            collect_expression_errors(body, errors);
        }
        SurfaceExpr::Error { span } => errors.expressions.push(ByteRange {
            start: span.start,
            end: span.end,
        }),
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::Float { .. }
        | SurfaceExpr::String { .. }
        | SurfaceExpr::Char { .. }
        | SurfaceExpr::Boolean { .. }
        | SurfaceExpr::Name { .. } => {}
    }
}

fn covered_by_existing_error(range: ByteRange, existing: &[Diagnostic]) -> bool {
    existing.iter().any(|diagnostic| {
        diagnostic.severity == DiagnosticSeverity::Error
            && ((diagnostic.primary.start <= range.start && diagnostic.primary.end >= range.end)
                || (diagnostic.primary.start >= range.start && diagnostic.primary.end <= range.end))
    })
}

fn expected_expression(primary: ByteRange) -> Diagnostic {
    Diagnostic {
        type_difference: None,
        id: String::new(),
        code: "SES-P0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "parser.expected-expression".to_owned(),
        primary,
        related: Vec::<RelatedDiagnostic>::new(),
        fixes: Vec::<DiagnosticFix>::new(),
    }
}

fn expected_pattern(primary: ByteRange) -> Diagnostic {
    let mut diagnostic = expected_expression(primary);
    diagnostic.message_key = "parser.expected-pattern".to_owned();
    diagnostic
}
