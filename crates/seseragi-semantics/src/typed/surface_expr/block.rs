use std::collections::BTreeMap;

use crate::{
    SymbolId, SymbolKind, SymbolNamespace, TypedBlockStatement, TypedConstraint, TypedExpr,
    TypedType,
};
use seseragi_syntax::{ByteSpan, SurfaceBlockItem, SurfaceExpr};

use super::pattern::type_pattern_binding;
use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::typed_parameters_from_surface;
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{
    application_argument_type_from_expr, inferred_type_from_expr, typed_type_contains_hole,
};

pub(super) fn type_block(
    items: &[SurfaceBlockItem],
    result: &SurfaceExpr,
    origin: ByteSpan,
    base_context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_block_with(items, result, origin, base_context, type_surface_expression)
}

pub(crate) fn type_block_with(
    items: &[SurfaceBlockItem],
    result: &SurfaceExpr,
    origin: ByteSpan,
    base_context: &PureExpressionContext<'_>,
    mut type_body: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let mut locals = BTreeMap::<SymbolId, SemanticValueType>::new();
    let mut statements = Vec::new();
    let mut merged = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Unit {
            type_ref: named_type("Unit"),
            origin,
        },
        SemanticTypeKey::Other,
    );

    let mut active_rec_group = None;
    let mut recursive_context = None;
    for item in items {
        if let SurfaceBlockItem::Function {
            rec_group: Some(group),
            ..
        } = item
        {
            if active_rec_group != Some(*group) {
                for member in items {
                    if let SurfaceBlockItem::Function {
                        name_span,
                        rec_group: Some(member_group),
                        parameters,
                        return_type,
                        effect,
                        ..
                    } = member
                    {
                        if member_group != group {
                            continue;
                        }
                        let parameters =
                            typed_parameters_from_surface(parameters, base_context.resolution);
                        let mut result = base_context
                            .semantic_value_from_type_ref(return_type)
                            .type_ref;
                        if let Some(effect) = effect {
                            result = crate::typed::type_ref::effect_value_type(
                                &crate::typed::effect::typed_effect_from_surface(
                                    &Some(return_type.clone()),
                                    &effect.requirements,
                                    effect.failure.as_ref(),
                                    false,
                                    &merged.value,
                                    base_context.resolution,
                                ),
                            );
                        }
                        if let Some(symbol) =
                            base_context.declaration_symbol(*name_span, SymbolKind::Function)
                        {
                            locals.insert(
                                symbol,
                                SemanticValueType {
                                    type_ref: function_type(&parameters, result),
                                    key: SemanticTypeKey::Other,
                                },
                            );
                        }
                    }
                }
                let members: BTreeMap<_, _> = items
                    .iter()
                    .filter_map(|member| {
                        let SurfaceBlockItem::Function {
                            name_span,
                            rec_group: Some(member_group),
                            ..
                        } = member
                        else {
                            return None;
                        };
                        if member_group != group {
                            return None;
                        }
                        let symbol =
                            base_context.declaration_symbol(*name_span, SymbolKind::Function)?;
                        Some((
                            symbol,
                            base_context
                                .with_locals(locals.clone())
                                .callable_value(symbol)?,
                        ))
                    })
                    .collect();
                recursive_context = members
                    .keys()
                    .next()
                    .copied()
                    .map(|first| super::recursion::RecursiveContext::new(members, first));
                active_rec_group = Some(*group);
            }
        }

        match item {
            SurfaceBlockItem::Let {
                pattern,
                type_ref,
                value,
                span,
            } => {
                let context = base_context.with_locals(locals.clone());
                let binding = type_pattern_binding(
                    pattern,
                    type_ref.as_ref(),
                    value,
                    &context,
                    &mut type_body,
                );
                merged.pure_call_issue = merged.pure_call_issue.take().or(binding.mismatch);
                if binding.pattern.is_refutable() {
                    merged.pure_call_issue = merged.pure_call_issue.take().or(Some(
                        PureCallIssue::RefutableBindingPattern {
                            pattern: pattern.span(),
                            surface: "block let",
                        },
                    ));
                }
                locals.extend(binding.pattern.locals.clone());
                statements.push(TypedBlockStatement::Let {
                    pattern: binding.pattern.typed,
                    value: binding.expression.value.clone(),
                    origin: *span,
                });
                merged.match_issues.extend(binding.pattern.issues);
                merged.merge_issues_from(binding.expression);
            }
            SurfaceBlockItem::Function {
                rec_group,
                name,
                name_span,
                type_parameters,
                parameters,
                return_type,
                effect,
                constraints,
                value,
                span,
            } => {
                let typed_parameters =
                    typed_parameters_from_surface(parameters, base_context.resolution);
                let declaration =
                    effect
                        .as_ref()
                        .map(|effect| seseragi_syntax::SurfaceDecl::EffectFn {
                            visibility: seseragi_syntax::Visibility::Private,
                            name: name.clone(),
                            name_span: *name_span,
                            type_parameters: type_parameters.clone(),
                            parameters: parameters.clone(),
                            inferred_contract: effect.inferred,
                            return_type: (!effect.inferred).then(|| return_type.clone()),
                            requirements: effect.requirements.clone(),
                            failure: effect.failure.clone(),
                            constraints: constraints.clone(),
                            body: Some(value.clone()),
                            span: *span,
                        });
                let mut expected = base_context.semantic_value_from_type_ref(return_type);
                if let Some(effect) = effect {
                    let contract = crate::typed::effect::typed_effect_from_surface(
                        &Some(return_type.clone()),
                        &effect.requirements,
                        effect.failure.as_ref(),
                        false,
                        &merged.value,
                        base_context.resolution,
                    );
                    expected = base_context.semantic_value_from_typed_type(
                        &crate::typed::type_ref::effect_value_type(&contract),
                    );
                }
                let local_function_type =
                    function_type(&typed_parameters, expected.type_ref.clone());
                let mut function_locals = locals.clone();
                if let Some(symbol) =
                    base_context.declaration_symbol(*name_span, SymbolKind::Function)
                {
                    function_locals.insert(
                        symbol,
                        SemanticValueType {
                            type_ref: local_function_type.clone(),
                            key: SemanticTypeKey::Other,
                        },
                    );
                    locals.insert(
                        symbol,
                        SemanticValueType {
                            type_ref: local_function_type,
                            key: SemanticTypeKey::Other,
                        },
                    );
                }
                function_locals.extend(base_context.parameter_locals(&typed_parameters));
                let scoped_evidence =
                    crate::typed::scoped_call_evidence(constraints, base_context.resolution);
                let mut context = base_context
                    .with_locals(function_locals)
                    .with_expected(Some(expected.clone()));
                if rec_group.is_some() {
                    if let (Some(group), Some(symbol)) = (
                        &recursive_context,
                        base_context.declaration_symbol(*name_span, SymbolKind::Function),
                    ) {
                        context.recursive_groups.push(group.for_member(symbol));
                    }
                }
                if !constraints.is_empty() {
                    context = context.with_evidence_parameters(scoped_evidence);
                }
                let analysis = if let Some(declaration) = &declaration {
                    let body = crate::typed::effect_body::analyze_effect_body_in_context(
                        value,
                        &context,
                        base_context.resolution,
                    );
                    let typed_body = body.value.clone();
                    let issues = crate::typed::effect_analysis::validate_effect_function(
                        declaration,
                        &[],
                        base_context.resolution,
                        body,
                    );
                    let mut analysis = SurfaceExpressionAnalysis::valid(typed_body);
                    if let Some(issue) = issues.into_iter().next() {
                        analysis.pure_call_issue = Some(PureCallIssue::LocalEffect {
                            issue: Box::new(issue),
                            function: *span,
                        });
                    }
                    if let Some(effect) = effect.as_ref().filter(|effect| effect.inferred) {
                        let contract = crate::typed::effect::typed_effect_from_surface(
                            &None,
                            &effect.requirements,
                            effect.failure.as_ref(),
                            true,
                            &analysis.value,
                            base_context.resolution,
                        );
                        expected = base_context.semantic_value_from_typed_type(
                            &crate::typed::type_ref::effect_value_type(&contract),
                        );
                    }
                    if let Some(symbol) =
                        base_context.declaration_symbol(*name_span, SymbolKind::Function)
                    {
                        locals.insert(
                            symbol,
                            SemanticValueType {
                                type_ref: function_type(
                                    &typed_parameters,
                                    expected.type_ref.clone(),
                                ),
                                key: SemanticTypeKey::Other,
                            },
                        );
                    }
                    analysis
                } else {
                    type_surface_expression(value, &context)
                };
                let actual = SemanticValueType {
                    type_ref: inferred_type_from_expr(&analysis.value),
                    key: analysis.semantic_type.clone(),
                };
                if effect.is_none()
                    && !typed_type_contains_hole(&expected.type_ref)
                    && !typed_type_contains_hole(&actual.type_ref)
                    && !semantic_values_are_compatible(&expected, &actual)
                {
                    merged.pure_call_issue = merged.pure_call_issue.take().or(Some(
                        PureCallIssue::LocalFunctionBodyTypeMismatch {
                            body: value.span(),
                            expected: expected.type_ref.clone(),
                            actual: actual.type_ref,
                        },
                    ));
                }
                statements.push(TypedBlockStatement::Function {
                    rec_group: *rec_group,
                    return_type: rec_group.map(|_| expected.type_ref.clone()),
                    effect: effect.as_ref().and_then(|_| {
                        crate::typed::type_ref::effect_from_value_type(&expected.type_ref)
                    }),
                    name: name.clone(),
                    type_parameters: type_parameters.clone(),
                    constraints: constraints
                        .iter()
                        .map(|constraint| TypedConstraint {
                            name: constraint.name.clone(),
                            arguments: constraint
                                .arguments
                                .iter()
                                .map(|argument| {
                                    base_context.semantic_value_from_type_ref(argument).type_ref
                                })
                                .collect(),
                        })
                        .collect(),
                    constraint_identities: constraints
                        .iter()
                        .map(|constraint| {
                            base_context
                                .resolution
                                .target(constraint.name_span, SymbolNamespace::Trait)
                                .and_then(|target| base_context.resolution.symbol(target))
                                .and_then(|symbol| symbol.canonical.clone())
                        })
                        .collect(),
                    parameters: typed_parameters,
                    body: analysis.value.clone(),
                    origin: *span,
                });
                merged.merge_issues_from(analysis);
            }
        }
    }

    let result_analysis = type_body(
        result,
        &base_context
            .with_locals(locals)
            .with_expected(base_context.expected().cloned()),
    );
    let type_ref = application_argument_type_from_expr(&result_analysis.value);
    let semantic_type = result_analysis.semantic_type.clone();
    let result_value = result_analysis.value.clone();
    merged.merge_issues_from(result_analysis);
    // A recursive group shares one lexical environment. Other declarations get
    // subsequent scopes so later shadowing cannot change already-bound closures.
    let isolate_declarations = items.iter().any(|item| {
        matches!(
            item,
            SurfaceBlockItem::Function {
                effect: Some(_),
                ..
            } | SurfaceBlockItem::Function {
                rec_group: Some(_),
                ..
            }
        )
    });
    merged.value = if isolate_declarations {
        let mut groups: Vec<Vec<TypedBlockStatement>> = Vec::new();
        let mut previous_group = None;
        for statement in statements {
            let group = match &statement {
                TypedBlockStatement::Function { rec_group, .. } => *rec_group,
                _ => None,
            };
            if group.is_some() && previous_group == group {
                groups.last_mut().unwrap().push(statement);
            } else {
                groups.push(vec![statement]);
            }
            previous_group = group;
        }
        groups
            .into_iter()
            .rev()
            .fold(result_value, |result, statements| TypedExpr::Block {
                statements,
                result: Box::new(result),
                type_ref: type_ref.clone(),
                origin,
            })
    } else {
        TypedExpr::Block {
            statements,
            result: Box::new(result_value),
            type_ref,
            origin,
        }
    };
    merged.semantic_type = semantic_type;
    merged
}

fn function_type(parameters: &[crate::TypedParameter], result: TypedType) -> TypedType {
    parameters.iter().rev().fold(result, |result, parameter| {
        let parameter = match parameter {
            crate::TypedParameter::ImplicitUnit { type_ref }
            | crate::TypedParameter::Named { type_ref, .. } => type_ref.clone(),
        };
        TypedType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        }
    })
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
