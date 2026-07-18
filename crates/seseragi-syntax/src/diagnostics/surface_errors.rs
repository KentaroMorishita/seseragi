use super::{ByteRange, Diagnostic, DiagnosticFix, DiagnosticSeverity, RelatedDiagnostic};
use crate::{
    SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr, SurfaceTemplatePart,
};
use std::collections::BTreeSet;

pub(super) fn diagnostics(
    declarations: &[SurfaceDecl],
    existing: &[Diagnostic],
) -> Vec<Diagnostic> {
    let mut spans = Vec::new();
    for declaration in declarations {
        match declaration {
            SurfaceDecl::Let {
                body: Some(body), ..
            }
            | SurfaceDecl::Fn {
                body: Some(body), ..
            }
            | SurfaceDecl::EffectFn {
                body: Some(body), ..
            } => collect_expression_errors(body, &mut spans),
            _ => {}
        }
    }

    let mut seen = BTreeSet::new();
    spans
        .into_iter()
        .filter(|range| seen.insert((range.start, range.end)))
        .filter(|range| !covered_by_existing_error(*range, existing))
        .map(expected_expression)
        .collect()
}

fn collect_expression_errors(expression: &SurfaceExpr, errors: &mut Vec<ByteRange>) {
    match expression {
        SurfaceExpr::Application {
            function, argument, ..
        } => {
            collect_expression_errors(function, errors);
            collect_expression_errors(argument, errors);
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
                    SurfaceComprehensionClause::Generator { source, .. } => source,
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
                if let Some(guard) = &arm.guard {
                    collect_expression_errors(guard, errors);
                }
                collect_expression_errors(&arm.body, errors);
            }
        }
        SurfaceExpr::Do { items, result, .. } => {
            for item in items {
                let value = match item {
                    SurfaceDoItem::Bind { value, .. }
                    | SurfaceDoItem::Let { value, .. }
                    | SurfaceDoItem::Expression { value, .. } => value,
                };
                collect_expression_errors(value, errors);
            }
            if let Some(result) = result {
                collect_expression_errors(result, errors);
            }
        }
        SurfaceExpr::Grouped { value, .. } => collect_expression_errors(value, errors),
        SurfaceExpr::Lambda { body, .. } => collect_expression_errors(body, errors),
        SurfaceExpr::Error { span } => errors.push(ByteRange {
            start: span.start,
            end: span.end,
        }),
        SurfaceExpr::Unit { .. }
        | SurfaceExpr::Integer { .. }
        | SurfaceExpr::String { .. }
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
        id: String::new(),
        code: "SES-P0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "parser.expected-expression".to_owned(),
        primary,
        related: Vec::<RelatedDiagnostic>::new(),
        fixes: Vec::<DiagnosticFix>::new(),
    }
}
