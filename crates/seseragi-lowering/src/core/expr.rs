use crate::source_span;
use seseragi_semantics::{
    TypedCallEvidence, TypedDoStatement, TypedExpr, TypedInstanceEvidence, TypedParameter,
};

use super::decision::lower_match;
use super::types::lower_typed_type;
use super::{CoreCallEvidence, CoreExpr, CoreInstanceEvidence, CoreParameter, CoreStatement};

pub(super) fn lower_effect_body(source: &str, body: TypedExpr) -> CoreExpr {
    match body {
        TypedExpr::EffectCall {
            operation,
            effect,
            arguments,
            origin,
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            origin: source_span(source, origin),
        },
        TypedExpr::EffectInvoke {
            callee,
            effect,
            arguments,
            origin,
        } => CoreExpr::EffectInvoke {
            callee,
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => {
            let statements = statements
                .into_iter()
                .map(|statement| lower_effect_statement(source, statement))
                .collect::<Vec<_>>();
            if statements.is_empty() {
                lower_expr(source, *result)
            } else {
                CoreExpr::Sequence {
                    statements,
                    result: Box::new(lower_expr(source, *result)),
                    origin: source_span(source, origin),
                }
            }
        }
        expr => lower_expr(source, expr),
    }
}

pub(super) fn lower_expr(source: &str, expr: TypedExpr) -> CoreExpr {
    match expr {
        TypedExpr::Unit { origin, .. } => CoreExpr::Unit {
            origin: source_span(source, origin),
        },
        TypedExpr::Integer { value, origin, .. } => CoreExpr::Int64 {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::String { value, origin, .. } => CoreExpr::String {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Boolean { value, origin, .. } => CoreExpr::Boolean {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Variable {
            name,
            type_ref,
            origin,
        } => CoreExpr::Variable {
            name,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Call {
            callee,
            arguments,
            evidence,
            type_ref,
            origin,
        } => CoreExpr::Call {
            callee,
            arguments: lower_exprs(source, arguments),
            evidence: evidence.into_iter().map(lower_call_evidence).collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Tuple {
            elements,
            type_ref,
            origin,
        } => CoreExpr::Tuple {
            elements: lower_exprs(source, elements),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Array {
            elements,
            type_ref,
            origin,
        } => CoreExpr::Array {
            elements: lower_exprs(source, elements),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Binary {
            operator,
            left,
            right,
            type_ref,
            origin,
        } => CoreExpr::Binary {
            operator,
            left: Box::new(lower_expr(source, *left)),
            right: Box::new(lower_expr(source, *right)),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            origin,
        } => CoreExpr::If {
            condition: Box::new(lower_expr(source, *condition)),
            then_branch: Box::new(lower_expr(source, *then_branch)),
            else_branch: Box::new(lower_expr(source, *else_branch)),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Match {
            scrutinee,
            arms,
            exhaustive,
            type_ref,
            origin,
        } => lower_match(source, *scrutinee, arms, exhaustive, type_ref, origin),
        TypedExpr::EffectCall {
            operation,
            effect,
            arguments,
            origin,
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            origin: source_span(source, origin),
        },
        TypedExpr::EffectInvoke {
            callee,
            effect,
            arguments,
            origin,
        } => CoreExpr::EffectInvoke {
            callee,
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => CoreExpr::Sequence {
            statements: statements
                .into_iter()
                .map(|statement| lower_expr_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            origin: source_span(source, origin),
        },
    }
}

fn lower_call_evidence(evidence: TypedCallEvidence) -> CoreCallEvidence {
    CoreCallEvidence {
        constraint: super::instances::lower_constraint(evidence.constraint),
        evidence: match evidence.evidence {
            TypedInstanceEvidence::Local { identity } => CoreInstanceEvidence::Local { identity },
            TypedInstanceEvidence::Imported {
                identity,
                provider_module,
            } => CoreInstanceEvidence::Imported {
                identity,
                provider_module,
            },
            TypedInstanceEvidence::Standard { identity } => {
                CoreInstanceEvidence::Standard { identity }
            }
        },
    }
}

pub(super) fn lower_parameter(parameter: &TypedParameter) -> CoreParameter {
    match parameter {
        TypedParameter::ImplicitUnit { type_ref } => CoreParameter {
            id: "unit".to_owned(),
            kind: "implicit".to_owned(),
            type_ref: lower_typed_type(type_ref.clone()),
        },
        TypedParameter::Named { name, type_ref, .. } => CoreParameter {
            id: name.clone(),
            kind: "named".to_owned(),
            type_ref: lower_typed_type(type_ref.clone()),
        },
    }
}

fn lower_exprs(source: &str, expressions: Vec<TypedExpr>) -> Vec<CoreExpr> {
    expressions
        .into_iter()
        .map(|expression| lower_expr(source, expression))
        .collect()
}

fn lower_effect_statement(source: &str, statement: TypedDoStatement) -> CoreStatement {
    match statement {
        TypedDoStatement::Effect { value } => CoreStatement::Effect {
            value: lower_effect_body(source, value),
        },
        TypedDoStatement::PureLet {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::PureLet {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_expr(source, value),
            origin: source_span(source, origin),
        },
        TypedDoStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::Bind {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_effect_body(source, value),
            origin: source_span(source, origin),
        },
    }
}

fn lower_expr_statement(source: &str, statement: TypedDoStatement) -> CoreStatement {
    match statement {
        TypedDoStatement::Effect { value } => CoreStatement::Effect {
            value: lower_expr(source, value),
        },
        TypedDoStatement::PureLet {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::PureLet {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_expr(source, value),
            origin: source_span(source, origin),
        },
        TypedDoStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::Bind {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_expr(source, value),
            origin: source_span(source, origin),
        },
    }
}

fn lower_effect_operation(operation: &str) -> String {
    match operation {
        "std/prelude::readLine" => "stdin.readLine".to_owned(),
        "std/prelude::print" => "console.print".to_owned(),
        "std/prelude::println" => "console.println".to_owned(),
        "std/effect::succeed" => "effect.succeed".to_owned(),
        "std/effect::fail" => "effect.fail".to_owned(),
        "std/effect::mapError" => "effect.mapError".to_owned(),
        "std/effect::fromEither" => "effect.fromEither".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::lower_effect_operation;

    #[test]
    fn lowers_canonical_from_either_operation_name() {
        assert_eq!(
            lower_effect_operation("std/effect::fromEither"),
            "effect.fromEither"
        );
    }
}
