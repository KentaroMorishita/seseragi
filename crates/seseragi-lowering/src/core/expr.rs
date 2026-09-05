use crate::source_span;
use seseragi_semantics::{
    TypedBlockStatement, TypedCallEvidence, TypedComprehensionClause, TypedDoStatement, TypedExpr,
    TypedModulePatternBinding, TypedMonadDoStatement, TypedParameter, TypedPattern,
    TypedRecordValueItem, TypedTemplatePart,
};
use seseragi_syntax::{ByteSpan, Visibility};

use super::decision::{
    lower_match, lower_pattern_binding_plan, projection_expression, CorePatternBindingPlan,
};
use super::types::lower_typed_type;
use super::{
    CoreBinding, CoreCallEvidence, CoreComprehensionClause, CoreExpr, CoreMonadDoStatement,
    CoreParameter, CorePattern, CoreRecordValueItem, CoreStatement, CoreTemplatePart,
};

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
            evidence,
            origin,
        } => CoreExpr::EffectInvoke {
            callee,
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            evidence: evidence.into_iter().map(lower_call_evidence).collect(),
            origin: source_span(source, origin),
        },
        TypedExpr::Block {
            statements,
            result,
            origin,
            ..
        } => CoreExpr::Sequence {
            statements: statements
                .into_iter()
                .flat_map(|statement| lower_block_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => {
            let statements = statements
                .into_iter()
                .flat_map(|statement| lower_effect_statement(source, statement))
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

fn lower_block_statement(source: &str, statement: TypedBlockStatement) -> Vec<CoreStatement> {
    match statement {
        TypedBlockStatement::Let {
            pattern,
            value,
            origin,
        } => lower_pure_pattern_statements(source, pattern, value, origin),
        TypedBlockStatement::Function {
            name,
            type_parameters,
            constraints,
            constraint_identities,
            parameters,
            body,
            origin,
        } => vec![CoreStatement::LocalFunction {
            name,
            type_parameters,
            constraints: constraints
                .into_iter()
                .enumerate()
                .map(|(index, constraint)| {
                    super::instances::lower_constraint_with_identity(
                        constraint,
                        constraint_identities.get(index).cloned().flatten(),
                    )
                })
                .collect(),
            parameters: parameters.iter().map(lower_parameter).collect(),
            body: lower_expr(source, body),
            origin: source_span(source, origin),
        }],
    }
}

pub(super) fn lower_expr(source: &str, expr: TypedExpr) -> CoreExpr {
    match expr {
        TypedExpr::Unit { origin, .. } => CoreExpr::Unit {
            origin: source_span(source, origin),
        },
        TypedExpr::Integer { value, origin, .. } => CoreExpr::Integer {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Float { value, origin, .. } => CoreExpr::Float64 {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Char { value, origin, .. } => CoreExpr::Char {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::String { value, origin, .. } => CoreExpr::String {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Template { parts, origin, .. } => CoreExpr::Template {
            parts: parts
                .into_iter()
                .map(|part| lower_template_part(source, part))
                .collect(),
            origin: source_span(source, origin),
        },
        TypedExpr::Boolean { value, origin, .. } => CoreExpr::Boolean {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Variable {
            name,
            evidence,
            type_ref,
            origin,
        } => CoreExpr::Variable {
            name,
            evidence: evidence.into_iter().map(lower_call_evidence).collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            trait_dispatch,
            type_ref,
            origin,
        } => CoreExpr::Call {
            callee,
            arguments: lower_exprs(source, arguments),
            evidence: evidence.into_iter().map(lower_call_evidence).collect(),
            deferred_evidence_parameters: deferred_evidence_parameters
                .into_iter()
                .map(lower_typed_type)
                .collect(),
            deferred_evidence_type_constructor_parameters,
            trait_dispatch: trait_dispatch.map(|dispatch| super::CoreTraitDispatch {
                trait_identity: dispatch.trait_identity,
                method: dispatch.method,
            }),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Lambda {
            parameter,
            body,
            type_ref,
            origin,
        } => CoreExpr::Lambda {
            parameter: lower_parameter(&parameter),
            body: Box::new(lower_expr(source, *body)),
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
        TypedExpr::FieldAccess {
            receiver,
            field,
            type_ref,
            origin,
        } => CoreExpr::FieldAccess {
            receiver: Box::new(lower_expr(source, *receiver)),
            field,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::OptionalFieldAccess {
            receiver,
            field,
            type_ref,
            origin,
        } => CoreExpr::OptionalFieldAccess {
            receiver: Box::new(lower_expr(source, *receiver)),
            field,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Record {
            items,
            type_ref,
            origin,
        } => CoreExpr::Record {
            items: items
                .into_iter()
                .map(|item| match item {
                    TypedRecordValueItem::Field {
                        name,
                        value,
                        origin,
                    } => CoreRecordValueItem::Field {
                        name,
                        value: lower_expr(source, value),
                        origin: source_span(source, origin),
                    },
                    TypedRecordValueItem::Spread { value, origin } => CoreRecordValueItem::Spread {
                        value: lower_expr(source, value),
                        origin: source_span(source, origin),
                    },
                })
                .collect(),
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
        TypedExpr::List {
            elements,
            type_ref,
            origin,
        } => CoreExpr::List {
            elements: lower_exprs(source, elements),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::ArrayComprehension {
            element,
            clauses,
            type_ref,
            origin,
        } => CoreExpr::ArrayComprehension {
            element: Box::new(lower_expr(source, *element)),
            clauses: clauses
                .into_iter()
                .map(|clause| lower_comprehension_clause(source, clause))
                .collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::ListComprehension {
            element,
            clauses,
            type_ref,
            origin,
        } => CoreExpr::ListComprehension {
            element: Box::new(lower_expr(source, *element)),
            clauses: clauses
                .into_iter()
                .map(|clause| lower_comprehension_clause(source, clause))
                .collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Binary {
            operator,
            left,
            right,
            evidence,
            type_ref,
            origin,
        } => {
            if operator == "??" {
                return super::decision::lower_fallback(source, *left, *right, type_ref, origin);
            }
            let origin = source_span(source, origin);
            let type_ref = lower_typed_type(type_ref);
            if matches!(operator.as_str(), "&&" | "||") {
                let left = Box::new(lower_expr(source, *left));
                let right = Box::new(lower_expr(source, *right));
                let (then_branch, else_branch) = if operator == "&&" {
                    (
                        right,
                        Box::new(CoreExpr::Boolean {
                            value: false,
                            origin: origin.clone(),
                        }),
                    )
                } else {
                    (
                        Box::new(CoreExpr::Boolean {
                            value: true,
                            origin: origin.clone(),
                        }),
                        right,
                    )
                };
                CoreExpr::If {
                    condition: left,
                    then_branch,
                    else_branch,
                    type_ref,
                    origin,
                }
            } else {
                CoreExpr::Binary {
                    operator,
                    left: Box::new(lower_expr(source, *left)),
                    right: Box::new(lower_expr(source, *right)),
                    evidence: evidence.into_iter().map(lower_call_evidence).collect(),
                    type_ref,
                    origin,
                }
            }
        }
        TypedExpr::Unary {
            operator,
            operand,
            type_ref,
            origin,
        } => CoreExpr::Unary {
            operator,
            operand: Box::new(lower_expr(source, *operand)),
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
            evidence,
            origin,
        } => CoreExpr::EffectInvoke {
            callee,
            requirements: lower_typed_type(effect.environment),
            failure: lower_typed_type(effect.failure),
            success: lower_typed_type(effect.success),
            arguments: lower_exprs(source, arguments),
            evidence: evidence.into_iter().map(lower_call_evidence).collect(),
            origin: source_span(source, origin),
        },
        TypedExpr::Block {
            statements,
            result,
            origin,
            ..
        } => CoreExpr::Sequence {
            statements: statements
                .into_iter()
                .flat_map(|statement| lower_block_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => CoreExpr::Sequence {
            statements: statements
                .into_iter()
                .flat_map(|statement| lower_expr_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            origin: source_span(source, origin),
        },
        TypedExpr::MonadDo {
            statements,
            result,
            evidence,
            type_ref,
            origin,
        } => CoreExpr::MonadDo {
            statements: statements
                .into_iter()
                .flat_map(|statement| lower_monad_do_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            evidence: lower_call_evidence(evidence),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
    }
}

fn lower_template_part(source: &str, part: TypedTemplatePart) -> CoreTemplatePart {
    match part {
        TypedTemplatePart::Text { value, origin } => CoreTemplatePart::Text {
            value,
            origin: source_span(source, origin),
        },
        TypedTemplatePart::Interpolation {
            value,
            evidence,
            trait_identity,
            origin,
        } => CoreTemplatePart::Interpolation {
            value: lower_expr(source, *value),
            evidence: evidence.map(lower_call_evidence),
            trait_identity,
            origin: source_span(source, origin),
        },
    }
}

fn lower_comprehension_clause(
    source: &str,
    clause: TypedComprehensionClause,
) -> CoreComprehensionClause {
    match clause {
        TypedComprehensionClause::Generator {
            pattern,
            source: values,
            evidence,
            origin,
        } => CoreComprehensionClause::Generator {
            pattern: lower_pattern(source, pattern),
            source: lower_expr(source, values),
            evidence: lower_call_evidence(evidence),
            origin: source_span(source, origin),
        },
        TypedComprehensionClause::Guard { condition, origin } => CoreComprehensionClause::Guard {
            condition: lower_expr(source, condition),
            origin: source_span(source, origin),
        },
    }
}

fn lower_pattern(source: &str, pattern: TypedPattern) -> CorePattern {
    match pattern {
        TypedPattern::Integer {
            value,
            type_ref,
            origin,
        } => CorePattern::Integer {
            value,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Char {
            value,
            type_ref,
            origin,
        } => CorePattern::Char {
            value,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::String {
            value,
            type_ref,
            origin,
        } => CorePattern::String {
            value,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Boolean {
            value,
            type_ref,
            origin,
        } => CorePattern::Boolean {
            value,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Binding {
            name,
            type_ref,
            origin,
            ..
        } => CorePattern::Binding {
            name,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Wildcard { type_ref, origin } => CorePattern::Wildcard {
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Constructor {
            symbol,
            argument,
            type_ref,
            origin,
        } => CorePattern::Constructor {
            symbol,
            argument: argument.map(|argument| Box::new(lower_pattern(source, *argument))),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Tuple {
            elements,
            type_ref,
            origin,
        } => CorePattern::Tuple {
            elements: elements
                .into_iter()
                .map(|element| lower_pattern(source, element))
                .collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Array {
            elements,
            rest,
            type_ref,
            origin,
        } => CorePattern::Array {
            elements: elements
                .into_iter()
                .map(|element| lower_pattern(source, element))
                .collect(),
            rest: rest.map(|rest| Box::new(lower_pattern(source, *rest))),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::List {
            elements,
            rest,
            type_ref,
            origin,
        } => CorePattern::List {
            elements: elements
                .into_iter()
                .map(|element| lower_pattern(source, element))
                .collect(),
            rest: rest.map(|rest| Box::new(lower_pattern(source, *rest))),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Record {
            fields,
            type_ref,
            origin,
        } => CorePattern::Record {
            fields: fields
                .into_iter()
                .map(|field| super::CoreRecordPatternField {
                    name: field.name,
                    pattern: lower_pattern(source, field.pattern),
                    origin: source_span(source, field.origin),
                })
                .collect(),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedPattern::Invalid { origin } => CorePattern::Invalid {
            origin: source_span(source, origin),
        },
    }
}

fn lower_call_evidence(evidence: TypedCallEvidence) -> CoreCallEvidence {
    CoreCallEvidence {
        constraint: super::instances::lower_constraint(evidence.constraint),
        evidence: super::instances::lower_instance_evidence(evidence.evidence),
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

fn lower_pure_pattern_statements(
    source: &str,
    pattern: TypedPattern,
    value: TypedExpr,
    origin: ByteSpan,
) -> Vec<CoreStatement> {
    lower_core_pattern_statements(source, pattern, lower_expr(source, value), origin, false)
}

fn lower_bind_pattern_statements(
    source: &str,
    pattern: TypedPattern,
    value: CoreExpr,
    origin: ByteSpan,
) -> Vec<CoreStatement> {
    lower_core_pattern_statements(source, pattern, value, origin, true)
}

fn lower_core_pattern_statements(
    source: &str,
    pattern: TypedPattern,
    value: CoreExpr,
    origin: ByteSpan,
    bind: bool,
) -> Vec<CoreStatement> {
    let plan = lower_pattern_binding_plan(source, pattern);
    let lowered_origin = source_span(source, origin);
    if let Some(binding) = direct_binding(&plan) {
        return vec![if bind {
            CoreStatement::Bind {
                name: binding.name.clone(),
                type_ref: binding.type_ref.clone(),
                value,
                origin: lowered_origin,
            }
        } else {
            CoreStatement::PureLet {
                name: binding.name.clone(),
                type_ref: binding.type_ref.clone(),
                value,
                origin: lowered_origin,
            }
        }];
    }

    let temporary = pattern_temporary(origin);
    let mut statements = vec![if bind {
        CoreStatement::Bind {
            name: temporary.clone(),
            type_ref: plan.input_type.clone(),
            value,
            origin: lowered_origin.clone(),
        }
    } else {
        CoreStatement::PureLet {
            name: temporary.clone(),
            type_ref: plan.input_type.clone(),
            value,
            origin: lowered_origin.clone(),
        }
    }];
    statements.extend(plan.bindings.iter().map(|binding| CoreStatement::PureLet {
        name: binding.name.clone(),
        type_ref: binding.type_ref.clone(),
        value: projection_expression(&temporary, &plan, binding),
        origin: binding.origin.clone(),
    }));
    statements
}

fn lower_monad_pattern_statements(
    source: &str,
    pattern: TypedPattern,
    value: TypedExpr,
    origin: ByteSpan,
    bind: bool,
) -> Vec<CoreMonadDoStatement> {
    let plan = lower_pattern_binding_plan(source, pattern);
    let value = lower_expr(source, value);
    let lowered_origin = source_span(source, origin);
    if let Some(binding) = direct_binding(&plan) {
        return vec![if bind {
            CoreMonadDoStatement::Bind {
                name: binding.name.clone(),
                type_ref: binding.type_ref.clone(),
                value,
                origin: lowered_origin,
            }
        } else {
            CoreMonadDoStatement::PureLet {
                name: binding.name.clone(),
                type_ref: binding.type_ref.clone(),
                value,
                origin: lowered_origin,
            }
        }];
    }

    let temporary = pattern_temporary(origin);
    let mut statements = vec![if bind {
        CoreMonadDoStatement::Bind {
            name: temporary.clone(),
            type_ref: plan.input_type.clone(),
            value,
            origin: lowered_origin,
        }
    } else {
        CoreMonadDoStatement::PureLet {
            name: temporary.clone(),
            type_ref: plan.input_type.clone(),
            value,
            origin: lowered_origin,
        }
    }];
    statements.extend(
        plan.bindings
            .iter()
            .map(|binding| CoreMonadDoStatement::PureLet {
                name: binding.name.clone(),
                type_ref: binding.type_ref.clone(),
                value: projection_expression(&temporary, &plan, binding),
                origin: binding.origin.clone(),
            }),
    );
    statements
}

fn direct_binding(plan: &CorePatternBindingPlan) -> Option<&super::CoreDecisionBinding> {
    (plan.tests.is_empty() && plan.bindings.len() == 1 && plan.bindings[0].path.is_empty())
        .then(|| &plan.bindings[0])
}

fn pattern_temporary(origin: ByteSpan) -> String {
    format!("__ssrg$pattern${}", origin.start)
}

pub(super) fn lower_top_level_pattern_binding(
    source: &str,
    module: &str,
    bindings: Vec<TypedModulePatternBinding>,
    pattern: TypedPattern,
    value: TypedExpr,
    visibility: Visibility,
    origin: ByteSpan,
) -> Vec<CoreBinding> {
    let plan = lower_pattern_binding_plan(source, pattern);
    if direct_binding(&plan).is_some() && bindings.len() == 1 {
        return vec![CoreBinding {
            symbol: bindings[0].symbol.clone(),
            visibility,
            origin: source_span(source, origin),
            value: lower_expr(source, value),
        }];
    }

    let temporary = pattern_temporary(origin);
    let mut lowered = vec![CoreBinding {
        symbol: format!("{module}::{temporary}"),
        visibility: Visibility::Private,
        origin: source_span(source, origin),
        value: lower_expr(source, value),
    }];
    for binding in bindings {
        let Some(projection) = plan
            .bindings
            .iter()
            .find(|candidate| candidate.name == binding.name)
        else {
            continue;
        };
        lowered.push(CoreBinding {
            symbol: binding.symbol,
            visibility,
            origin: source_span(source, binding.origin),
            value: projection_expression(&temporary, &plan, projection),
        });
    }
    lowered
}

fn lower_effect_statement(source: &str, statement: TypedDoStatement) -> Vec<CoreStatement> {
    match statement {
        TypedDoStatement::Effect { value } => vec![CoreStatement::Effect {
            value: lower_effect_body(source, value),
        }],
        TypedDoStatement::PureLet {
            pattern,
            value,
            origin,
        } => lower_pure_pattern_statements(source, pattern, value, origin),
        TypedDoStatement::Bind {
            pattern,
            value,
            origin,
        } => {
            lower_bind_pattern_statements(source, pattern, lower_effect_body(source, value), origin)
        }
    }
}

fn lower_expr_statement(source: &str, statement: TypedDoStatement) -> Vec<CoreStatement> {
    match statement {
        TypedDoStatement::Effect { value } => vec![CoreStatement::Effect {
            value: lower_expr(source, value),
        }],
        TypedDoStatement::PureLet {
            pattern,
            value,
            origin,
        } => lower_pure_pattern_statements(source, pattern, value, origin),
        TypedDoStatement::Bind {
            pattern,
            value,
            origin,
        } => lower_bind_pattern_statements(source, pattern, lower_expr(source, value), origin),
    }
}

fn lower_monad_do_statement(
    source: &str,
    statement: TypedMonadDoStatement,
) -> Vec<CoreMonadDoStatement> {
    match statement {
        TypedMonadDoStatement::Expression { value } => vec![CoreMonadDoStatement::Expression {
            value: lower_expr(source, value),
        }],
        TypedMonadDoStatement::PureLet {
            pattern,
            value,
            origin,
        } => lower_monad_pattern_statements(source, pattern, value, origin, false),
        TypedMonadDoStatement::Bind {
            pattern,
            value,
            origin,
        } => lower_monad_pattern_statements(source, pattern, value, origin, true),
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
