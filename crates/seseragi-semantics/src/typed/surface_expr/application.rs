use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, TypeRef};
use std::collections::BTreeMap;

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::functions::{
    infer_type_parameters, instantiated_application, instantiated_application_indexed,
    instantiated_application_result_type, substitute_type_parameters, InstantiatedApplication,
    TopLevelPureFunction,
};
use crate::typed::pure_issues::PureCallIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{application_argument_type_from_expr, typed_type_contains_hole};

mod candidates;

use candidates::{select_trait_method_candidate, type_trait_method_selection_error};

fn unconstrained_generic_signature(
    signature: &TopLevelPureFunction,
) -> Option<crate::TypedCallSignature> {
    (signature.constraints.is_empty()
        && signature.trait_identity.is_none()
        && !signature.type_parameters.is_empty())
    .then(|| crate::TypedCallSignature {
        type_parameters: signature.type_parameters.clone(),
        parameters: signature.parameters.clone(),
        result: signature.result.clone(),
        argument_types: Vec::new(),
        instantiated_type_arguments: Vec::new(),
    })
}

// First-class references keep the existing HKT-only projection. Ordinary
// monomorphic value uses already carry their checked callable type; the new
// instantiation descriptor belongs to applying a named generic call.
fn unconstrained_hkt_signature(
    signature: &TopLevelPureFunction,
) -> Option<crate::TypedCallSignature> {
    unconstrained_generic_signature(signature)
        .filter(|value| value.type_parameters.iter().any(|p| p.is_constructor()))
}

fn callable_reference(
    signature: &TopLevelPureFunction,
    callee_signature: Option<crate::TypedCallSignature>,
    type_ref: TypedType,
    origin: ByteSpan,
) -> TypedExpr {
    TypedExpr::Variable {
        name: signature.symbol.clone(),
        callable_signature: callee_signature,
        evidence: Vec::new(),
        type_ref,
        origin,
    }
}

pub(super) fn type_application(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_application_with(expression, context, type_surface_expression)
}

pub(super) fn type_trait_operator_contract_error(
    origin: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_trait_method_selection_error(
        &[],
        origin,
        PureCallIssue::TraitMethodNoMatch { callee: origin },
        context,
    )
}

pub(super) fn type_explicit_callable_value(
    signature: TopLevelPureFunction,
    arguments: &[TypeRef],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let callee_signature = unconstrained_hkt_signature(&signature);
    let (signature, issue) =
        instantiate_explicit_signature(signature, Some(arguments), span, context);
    let mut analysis = type_callable_value(&signature, span, context);
    if issue.is_some() {
        analysis.pure_call_issue = issue;
        analysis.semantic_type = SemanticTypeKey::Invalid;
    } else if callee_signature.is_some() && analysis.pure_call_issue.is_none() {
        let type_ref = application_argument_type_from_expr(&analysis.value);
        analysis.value = callable_reference(&signature, callee_signature, type_ref, span);
    }
    analysis
}

/// Types a named callable used without value arguments. Unconstrained
/// ordinary callables remain variables. Higher-kinded references retain their
/// checked scheme, while constrained callables capture dictionaries after
/// every remaining curried value parameter.
pub(super) fn type_callable_value(
    signature: &TopLevelPureFunction,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let mut application =
        instantiated_application(signature, context.expected(), 0, &[], &|value| {
            context.evidence_type(value)
        });
    refine_collection_parameters(&mut application, signature, context);
    for parameter in &mut application.parameters {
        *parameter = context.hydrate_callable_parameter(parameter.clone(), span);
    }
    application.result = context.hydrate_semantic_value(application.result);

    // A callable scheme is instantiated at a value use. Leaving its parameters
    // in an Array element or record field would make them free types in the
    // generated program. Let aliases supply their scheme as an expectation;
    // other uses need enough context to select one monotype.
    let value_type = instantiated_application_result_type(&application, 0);
    if matches!(value_type, TypedType::Function { .. })
        && contains_unresolved_type_parameter(
            &value_type,
            &signature.type_parameters,
            &application.resolved_type_parameters,
        )
    {
        let mut analysis = SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            callable_signature: None,
            name: signature.symbol.clone(),
            evidence: Vec::new(),
            type_ref: TypedType::Hole,
            origin: span,
        });
        analysis.pure_call_issue = Some(PureCallIssue::CallableTypeUnresolved { callee: span });
        analysis.semantic_type = SemanticTypeKey::Invalid;
        return analysis;
    }

    if let Some(issue) = context
        .expected()
        .and_then(|expected| context.recursive_call_issue(span, &[], &expected.type_ref))
    {
        let mut analysis = SurfaceExpressionAnalysis::valid(TypedExpr::Variable {
            callable_signature: None,
            name: signature.symbol.clone(),
            evidence: Vec::new(),
            type_ref: TypedType::Hole,
            origin: span,
        });
        analysis.pure_call_issue = Some(issue);
        analysis.semantic_type = SemanticTypeKey::Invalid;
        return analysis;
    }

    if signature.constraints.is_empty() && foreign_template(context, &signature.symbol).is_none() {
        // A saved reference needs the same checked function identity as an
        // immediate call. Keep quantified slots and nominal children instead
        // of forcing a later use to recover them from display types.
        let semantic_type = application
            .parameters
            .iter()
            .rev()
            .fold(application.result.clone(), |result, parameter| {
                SemanticValueType::function(parameter.clone(), result)
            })
            .key;
        return SurfaceExpressionAnalysis::valid_with_semantic_type(
            callable_reference(
                signature,
                unconstrained_hkt_signature(signature),
                instantiated_application_result_type(&application, 0),
                span,
            ),
            semantic_type,
        );
    }

    let mut analysis = type_known_application(
        signature.clone(),
        span,
        &[],
        span,
        context,
        type_surface_expression,
    );
    let unresolved_constraint = matches!(
        &analysis.value,
        TypedExpr::Call { evidence, .. } if evidence.is_empty()
    ) && analysis.pure_call_issue.is_none();
    if unresolved_constraint {
        let constraint = application
            .constraints
            .into_iter()
            .next()
            .expect("constrained callable must retain its first constraint");
        analysis.pure_call_issue = Some(PureCallIssue::MissingInstance {
            callee: span,
            constraint,
        });
        analysis.semantic_type = SemanticTypeKey::Invalid;
        if let TypedExpr::Call { type_ref, .. } = &mut analysis.value {
            *type_ref = TypedType::Hole;
        }
    }
    if analysis.pure_call_issue.is_none()
        && !signature.parameters.is_empty()
        && !closed_selected_method_value(&analysis.value, signature, context)
        && !available_selected_method_value(&analysis.value, signature, span, context)
    {
        analysis.value = abstract_callable_value(analysis.value, signature.parameters.len());
    }
    analysis
}

/// A source body can capture a dictionary it already received. A newly
/// generalized let scheme cannot: its dictionaries are introduced at a later
/// stage. Keep this proof separate from the closed monomorphic case above.
fn available_selected_method_value(
    value: &TypedExpr,
    signature: &TopLevelPureFunction,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> bool {
    let TypedExpr::Call {
        arguments,
        evidence,
        trait_dispatch: Some(dispatch),
        deferred_evidence_parameters,
        deferred_evidence_type_constructor_parameters,
        evidence_argument_index,
        type_ref,
        ..
    } = value
    else {
        return false;
    };
    let Some(selected) = evidence.first() else {
        return false;
    };
    let context = context.with_lexical_scope(span);
    if (context.deferred_let_evidence && !context.deferred_method_value)
        || !arguments.is_empty()
        || (dispatch.method_contract.is_some()
            && !context.resolution.declared_method_value_scheme(signature))
        || !deferred_evidence_parameters.is_empty()
        || !deferred_evidence_type_constructor_parameters.is_empty()
        || evidence_argument_index.is_some()
        || signature.trait_identity.as_deref() != Some(dispatch.trait_identity.as_str())
        || signature.trait_method.as_deref() != Some(dispatch.method.as_str())
        || signature.constraint_identities.first() != Some(&Some(dispatch.trait_identity.clone()))
        || signature.constraints.len() != evidence.len()
    {
        return false;
    }
    // Parameter dictionaries have no instance method body in this module.
    // Certify only the canonical signatures also validated by the native ABI.
    // A selected local/imported provider retains its checked method body.
    if matches!(
        selected.evidence,
        crate::TypedInstanceEvidence::Parameter { .. }
    ) && !matches!(
        (dispatch.trait_identity.as_str(), dispatch.method.as_str()),
        ("std/prelude::Eq", "eq")
            | ("std/prelude::Add", "add")
            | ("std/prelude::Sub", "sub")
            | ("std/prelude::Mul", "mul")
            | ("std/prelude::Div", "div")
            | ("std/prelude::Rem", "rem")
            | ("std/prelude::Pow", "pow")
    ) && !canonical_ord_method_scheme(signature)
        && !context.resolution.declared_method_value_scheme(signature)
    {
        return false;
    }
    (available_method_type(type_ref, &context)
        || (context.deferred_method_value && method_value_scheme(signature, &context)))
        && evidence
            .iter()
            .zip(&signature.constraint_identities)
            .all(|(selected, identity)| {
                identity
                    .as_ref()
                    .is_some_and(|identity| available_method_evidence(selected, identity, &context))
            })
}

pub(super) fn method_value_scheme(
    signature: &TopLevelPureFunction,
    context: &PureExpressionContext<'_>,
) -> bool {
    standard_method_value_scheme(signature)
        || context.resolution.declared_method_value_scheme(signature)
}

/// Bounded dictionary-only alias schemes. Their source signature is the
/// canonical Prelude contract also checked by the native method-value ABI.
/// User traits and method-owned constraints require their own signature proof.
pub(super) fn standard_method_value_scheme(signature: &TopLevelPureFunction) -> bool {
    if canonical_ord_method_scheme(signature) {
        return true;
    }
    let Some(identity) = &signature.trait_identity else {
        return false;
    };
    signature.constraints.len() == 1
        && signature.constraint_identities.as_slice() == [Some(identity.clone())]
        && signature.type_parameters.iter().all(|p| p.arity == 0)
        && matches!(
            (identity.as_str(), signature.trait_method.as_deref()),
            ("std/prelude::Eq", Some("eq"))
                | ("std/prelude::Add", Some("add"))
                | ("std/prelude::Sub", Some("sub"))
                | ("std/prelude::Mul", Some("mul"))
                | ("std/prelude::Div", Some("div"))
                | ("std/prelude::Rem", Some("rem"))
                | ("std/prelude::Pow", Some("pow"))
        )
}

/// Ord has a value-kind primary parameter and an Ordering result. Its trusted
/// Prelude signature, rather than a same-named source declaration or the number
/// of arrows in an implementation, certifies the method value's full type.
fn canonical_ord_value_type(signature: &TopLevelPureFunction) -> Option<TypedType> {
    if signature.constraint_identities.as_slice() != [Some("std/prelude::Ord".into())]
        || signature.constraints.len() != 1
        || signature.type_parameters.iter().any(|p| p.arity != 0)
    {
        return None;
    }
    let [argument] = signature.constraints[0].arguments.as_slice() else {
        return None;
    };
    if typed_type_contains_hole(argument) {
        return None;
    }
    let method = crate::prelude::trait_method_by_canonical("std/prelude::Ord::compare")?;
    let declared = crate::prelude::trait_method_signature(method);
    let [parameter] = declared.type_parameters.as_slice() else {
        return None;
    };
    let substitutions = BTreeMap::from([(parameter.name.clone(), argument.clone())]);
    let value = declared
        .parameters
        .into_iter()
        .rev()
        .fold(declared.result, |result, parameter| TypedType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        });
    Some(substitute_type_parameters(&value, &substitutions))
}

fn canonical_ord_method_scheme(signature: &TopLevelPureFunction) -> bool {
    signature.trait_identity.as_deref() == Some("std/prelude::Ord")
        && signature.trait_method.as_deref() == Some("compare")
        && canonical_ord_value_type(signature).is_some_and(|expected| {
            signature
                .parameters
                .iter()
                .rev()
                .fold(signature.result.clone(), |result, parameter| {
                    TypedType::Function {
                        parameter: Box::new(parameter.clone()),
                        result: Box::new(result),
                    }
                })
                == expected
        })
}

pub(super) fn method_factory_alias(
    signature: &TopLevelPureFunction,
    context: &PureExpressionContext<'_>,
) -> bool {
    signature.trait_identity.is_none()
        && signature.parameters.is_empty()
        && !signature.constraints.is_empty()
        && (signature.type_parameters.iter().all(|p| p.arity == 0)
            || matches!(signature.constraint_identities.first(), Some(Some(identity))
                if context.resolution.declared_method_factory_trait(identity)))
        && matches!(signature.constraint_identities.first(), Some(Some(identity)) if (signature.constraints.len() == 1 && matches!(identity.as_str(),
            "std/prelude::Eq" | "std/prelude::Add" | "std/prelude::Sub"
            | "std/prelude::Mul" | "std/prelude::Div" | "std/prelude::Rem" | "std/prelude::Pow"))
            || context.resolution.declared_method_factory_trait(identity)
            || (identity == "std/prelude::Ord"
                && canonical_ord_value_type(signature).as_ref() == Some(&signature.result)))
}

fn available_method_evidence(
    selected: &crate::TypedCallEvidence,
    trait_identity: &str,
    context: &PureExpressionContext<'_>,
) -> bool {
    use crate::TypedInstanceEvidence;
    match &selected.evidence {
        TypedInstanceEvidence::Parameter { .. } => {
            context.available_method_parameter(selected, trait_identity)
        }
        TypedInstanceEvidence::Local {
            type_arguments,
            evidence_arguments,
            ..
        }
        | TypedInstanceEvidence::Imported {
            type_arguments,
            evidence_arguments,
            ..
        } => {
            selected
                .constraint
                .arguments
                .iter()
                .chain(type_arguments)
                .all(|ty| available_method_type(ty, context))
                && evidence_arguments.iter().all(|child| {
                    context
                        .trait_identity(&child.constraint.name)
                        .is_some_and(|identity| {
                            available_method_evidence(child, &identity, context)
                        })
                })
        }
        // Do not recapture a primitive dictionary's erased spelling as a
        // lexical binder. Conditional standard dictionaries need a separate
        // proof before their open type arguments enter this path.
        TypedInstanceEvidence::Standard { .. } => closed_method_evidence(selected, context),
    }
}

fn available_method_type(ty: &TypedType, context: &PureExpressionContext<'_>) -> bool {
    if closed_method_type(ty, context) {
        return true;
    }
    match ty {
        TypedType::Function { parameter, result } => {
            available_method_type(parameter, context) && available_method_type(result, context)
        }
        TypedType::Tuple { elements } => {
            elements.iter().all(|ty| available_method_type(ty, context))
        }
        TypedType::Record { closed, fields } => {
            *closed
                && fields
                    .iter()
                    .all(|f| available_method_type(&f.type_ref, context))
        }
        TypedType::Named { .. } | TypedType::ExternalNamed { .. } => {
            fn known(key: &SemanticTypeKey, context: &PureExpressionContext<'_>) -> bool {
                match key {
                    SemanticTypeKey::TypeParameter(id) => context
                        .lexical_type_parameters
                        .values()
                        .any(|available| available == id),
                    SemanticTypeKey::Adt { arguments, .. }
                    | SemanticTypeKey::Struct { arguments, .. }
                    | SemanticTypeKey::ExternalNominal { arguments, .. } => {
                        arguments.iter().all(|a| known(&a.key, context))
                    }
                    SemanticTypeKey::NamedGeneric { name, arguments } => {
                        crate::prelude::type_constructor_arity(name).is_some()
                            && arguments.iter().all(|a| known(&a.key, context))
                    }
                    SemanticTypeKey::Application {
                        constructor,
                        arguments,
                    } => {
                        known(&constructor.key, context)
                            && arguments.iter().all(|a| known(&a.key, context))
                    }
                    SemanticTypeKey::Tuple(elements) => elements.iter().all(|k| known(k, context)),
                    // This key does not retain whether its row is closed.
                    SemanticTypeKey::Record(_) => false,
                    _ => false,
                }
            }
            known(&context.semantic_value_from_typed_type(ty).key, context)
        }
        TypedType::Hole | TypedType::RequirementMerge { .. } => false,
    }
}

/// A closed method already has its primary dictionary. Preserve that method
/// value: eta-expanding all trait parameters would postpone a source method
/// body whose first stage returns a function. Generalized callables and method
/// constraints still need their separate dictionary-introduction stages.
fn closed_selected_method_value(
    value: &TypedExpr,
    signature: &TopLevelPureFunction,
    context: &PureExpressionContext<'_>,
) -> bool {
    let TypedExpr::Call {
        arguments,
        evidence,
        trait_dispatch: Some(dispatch),
        deferred_evidence_parameters,
        deferred_evidence_type_constructor_parameters,
        evidence_argument_index,
        type_ref,
        ..
    } = value
    else {
        return false;
    };
    arguments.is_empty()
        && !evidence.is_empty()
        && (dispatch.method_contract.is_none()
            || context.resolution.declared_method_value_scheme(signature))
        && deferred_evidence_parameters.is_empty()
        && deferred_evidence_type_constructor_parameters.is_empty()
        && evidence_argument_index.is_none()
        && signature.trait_identity.as_deref() == Some(dispatch.trait_identity.as_str())
        && signature.trait_method.as_deref() == Some(dispatch.method.as_str())
        && signature.constraint_identities.len() == evidence.len()
        && signature.constraint_identities[0].as_deref() == Some(dispatch.trait_identity.as_str())
        // A residual declaration binder is not a closed type, even if an
        // inferred substitution kept its spelling. Do not identify it with a
        // builtin of the same name after erasure; retain scheme abstraction.
        && !contains_unresolved_type_parameter(type_ref, &signature.type_parameters, &Default::default())
        && closed_method_type(type_ref, context)
        && evidence.iter().all(|selected| closed_method_evidence(selected, context))
}

fn closed_method_evidence(
    selected: &crate::TypedCallEvidence,
    context: &PureExpressionContext<'_>,
) -> bool {
    use crate::TypedInstanceEvidence;
    let (type_arguments, children) = match &selected.evidence {
        TypedInstanceEvidence::Local {
            type_arguments,
            evidence_arguments,
            ..
        }
        | TypedInstanceEvidence::Imported {
            type_arguments,
            evidence_arguments,
            ..
        }
        | TypedInstanceEvidence::Standard {
            type_arguments,
            evidence_arguments,
            ..
        } => (type_arguments, evidence_arguments),
        TypedInstanceEvidence::Parameter { .. } => return false,
    };
    selected
        .constraint
        .arguments
        .iter()
        .chain(type_arguments)
        .all(|ty| closed_method_type(ty, context))
        && children
            .iter()
            .all(|child| closed_method_evidence(child, context))
}

fn closed_method_type(ty: &TypedType, context: &PureExpressionContext<'_>) -> bool {
    match ty {
        TypedType::Function { parameter, result } => {
            closed_method_type(parameter, context) && closed_method_type(result, context)
        }
        TypedType::Tuple { elements } => elements.iter().all(|ty| closed_method_type(ty, context)),
        TypedType::Record { closed, fields } => {
            *closed
                && fields
                    .iter()
                    .all(|field| closed_method_type(&field.type_ref, context))
        }
        TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
            // Use the scoped semantic key so a binder named Int or Array is
            // never certified merely because it has a standard spelling.
            let value = context.semantic_value_from_typed_type(ty);
            let owner_is_closed = match &value.key {
                SemanticTypeKey::Adt { .. }
                | SemanticTypeKey::Struct { .. }
                | SemanticTypeKey::ExternalNominal { .. } => true,
                SemanticTypeKey::NamedGeneric { name, .. } => {
                    crate::prelude::type_constructor_arity(name).is_some()
                }
                SemanticTypeKey::Other => matches!(ty, TypedType::Named { name, .. }
                    if crate::prelude::type_constructor_arity(name).is_some()),
                _ => false,
            };
            owner_is_closed && arguments.iter().all(|ty| closed_method_type(ty, context))
        }
        TypedType::Hole | TypedType::RequirementMerge { .. } => false,
    }
}

/// Keep a constrained first-class callable's declared arity explicit. This
/// lets a generalized alias abstract its dictionaries after its value
/// parameters, including when the callable's result is itself a function.
fn abstract_callable_value(mut value: TypedExpr, arity: usize) -> TypedExpr {
    let TypedExpr::Call {
        arguments,
        deferred_evidence_parameters,
        deferred_evidence_type_constructor_parameters,
        evidence_argument_index,
        type_ref,
        origin,
        ..
    } = &mut value
    else {
        return value;
    };
    let origin = *origin;
    let mut parameters = Vec::new();
    for index in 0..arity {
        let TypedType::Function { parameter, result } = type_ref.clone() else {
            return value;
        };
        let name = format!("__ssrg$partial${index}");
        arguments.push(TypedExpr::Variable {
            callable_signature: None,
            name: name.clone(),
            evidence: Vec::new(),
            type_ref: (*parameter).clone(),
            origin,
        });
        parameters.push(crate::TypedParameter::Named {
            name,
            type_ref: *parameter,
            origin,
        });
        *type_ref = *result;
    }
    deferred_evidence_parameters.clear();
    deferred_evidence_type_constructor_parameters.clear();
    *evidence_argument_index = None;
    let mut body_type = type_ref.clone();
    for parameter in parameters.into_iter().rev() {
        let crate::TypedParameter::Named { type_ref, .. } = &parameter else {
            unreachable!()
        };
        body_type = TypedType::Function {
            parameter: Box::new(type_ref.clone()),
            result: Box::new(body_type),
        };
        value = TypedExpr::Lambda {
            parameter,
            body: Box::new(value),
            type_ref: body_type.clone(),
            origin,
        };
    }
    value
}

pub(crate) fn type_application_with(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
    mut type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let (callee, argument_nodes) = flatten_application(expression);
    let explicit_type_arguments = explicit_type_arguments(callee);
    if let SurfaceExpr::Member {
        receiver,
        field,
        field_span,
        ..
    } = callee
    {
        let receiver_analysis = type_argument(receiver, &context.without_expected());
        if let Some(signature) = context.inherent_method(&receiver_analysis.semantic_type, field) {
            let mut method_arguments = Vec::with_capacity(argument_nodes.len() + 1);
            method_arguments.push(receiver.as_ref());
            method_arguments.extend(argument_nodes);
            return type_known_application_with_explicit(
                signature,
                explicit_type_arguments,
                *field_span,
                &method_arguments,
                expression.span(),
                context,
                &mut type_argument,
            );
        }
        if context.target(callee.span()).is_none() {
            let field_value = type_argument(callee, &context.without_expected());
            if matches!(
                application_argument_type_from_expr(&field_value.value),
                TypedType::Function { .. }
            ) {
                return type_function_expression_application(
                    callee,
                    field_value,
                    &argument_nodes,
                    expression.span(),
                    explicit_type_arguments,
                    context,
                    &mut type_argument,
                );
            }
        }
    }
    let callee_span = callee.span();
    if context.unproved_trait_operator(callee_span) {
        if argument_nodes.len() >= 2 {
            if let Some((operator, signature)) = context.trait_operator_method(callee_span) {
                // Check the real member's signature and dictionary stage in ABI
                // order, then retain source order for shared Core staging.
                let mut method_arguments = argument_nodes.clone();
                for (index, source) in operator.method_operand_sources.iter().enumerate() {
                    method_arguments[index] = argument_nodes[*source];
                }
                let mut analysis = type_known_application_with_explicit(
                    signature,
                    explicit_type_arguments,
                    callee_span,
                    &method_arguments,
                    expression.span(),
                    context,
                    &mut type_argument,
                );
                if !analysis.has_issues() {
                    if let TypedExpr::Call {
                        callee,
                        arguments,
                        trait_dispatch: Some(_),
                        ..
                    } = &mut analysis.value
                    {
                        *callee = operator.spelling.to_owned();
                        if operator.method_operand_sources == [1, 0] {
                            arguments.swap(0, 1);
                        }
                    }
                }
                return analysis;
            }
        }
        return type_trait_method_selection_error(
            &argument_nodes,
            expression.span(),
            PureCallIssue::TraitMethodNoMatch {
                callee: callee_span,
            },
            context,
        );
    }
    let mut signature = if let Some(target) = context
        .target(callee_span)
        .or_else(|| context.operator_target(callee_span))
    {
        let Some(signature) = context.callable_value(target) else {
            return type_unknown_application(callee, &argument_nodes, expression.span(), context);
        };
        signature
    } else if matches!(callee, SurfaceExpr::Name { .. }) {
        match select_trait_method_candidate(callee_span, &argument_nodes, context) {
            Some(Ok(signature)) => signature,
            Some(Err(issue)) => {
                return type_trait_method_selection_error(
                    &argument_nodes,
                    expression.span(),
                    issue,
                    context,
                );
            }
            None => {
                return type_unknown_application(
                    callee,
                    &argument_nodes,
                    expression.span(),
                    context,
                );
            }
        }
    } else {
        return type_unknown_application(callee, &argument_nodes, expression.span(), context);
    };

    if signature.symbol == "std/prelude:::" && explicit_type_arguments.is_none() {
        if let [head, tail] = argument_nodes.as_slice() {
            return super::cons::type_cons_with(
                head,
                tail,
                expression.span(),
                context,
                type_argument,
            );
        }
    }

    if argument_nodes.len() >= 2
        && seseragi_syntax::standard_trait_operator(&signature.symbol)
            .is_some_and(|operator| operator.method_operand_sources == [1, 0])
    {
        // Retain the method's established contextual inference order. This is
        // static checking only: Core receives source order and stages evaluation.
        signature.parameters.swap(0, 1);
        signature.semantic_parameters.swap(0, 1);
        let mut method_arguments = argument_nodes.clone();
        method_arguments.swap(0, 1);
        let mut analysis = type_known_application_with_explicit(
            signature,
            explicit_type_arguments,
            callee_span,
            &method_arguments,
            expression.span(),
            context,
            type_argument,
        );
        if let TypedExpr::Call { arguments, .. } = &mut analysis.value {
            arguments.swap(0, 1);
        }
        if let Some(PureCallIssue::ArgumentType { index, .. }) = &mut analysis.pure_call_issue {
            if *index < 2 {
                *index = 1 - *index;
            }
        }
        return analysis;
    }

    type_known_application_with_explicit(
        signature,
        explicit_type_arguments,
        callee_span,
        &argument_nodes,
        expression.span(),
        context,
        type_argument,
    )
}

pub(super) fn type_inherent_method_member(
    receiver: &SurfaceExpr,
    field: &str,
    field_span: ByteSpan,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> Option<SurfaceExpressionAnalysis> {
    let receiver_analysis = type_surface_expression(receiver, &context.without_expected());
    let signature = context.inherent_method(&receiver_analysis.semantic_type, field)?;
    Some(type_known_application(
        signature,
        field_span,
        &[receiver],
        span,
        context,
        type_surface_expression,
    ))
}

pub(crate) fn type_known_application(
    signature: crate::typed::functions::TopLevelPureFunction,
    callee_span: ByteSpan,
    argument_nodes: &[&SurfaceExpr],
    expression_span: ByteSpan,
    context: &PureExpressionContext<'_>,
    type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    type_known_application_with_explicit(
        signature,
        None,
        callee_span,
        argument_nodes,
        expression_span,
        context,
        type_argument,
    )
}

pub(crate) fn type_known_application_with_explicit(
    signature: crate::typed::functions::TopLevelPureFunction,
    explicit_type_arguments: Option<&[TypeRef]>,
    callee_span: ByteSpan,
    argument_nodes: &[&SurfaceExpr],
    expression_span: ByteSpan,
    context: &PureExpressionContext<'_>,
    mut type_argument: impl FnMut(&SurfaceExpr, &PureExpressionContext<'_>) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let callee_signature = unconstrained_generic_signature(&signature);
    let explicit_instantiation = explicit_type_arguments.map(|arguments| {
        arguments
            .iter()
            .map(|argument| context.evidence_type(&context.semantic_value_from_type_ref(argument)))
            .collect::<Vec<_>>()
    });
    let (mut signature, explicit_issue) =
        instantiate_explicit_signature(signature, explicit_type_arguments, callee_span, context);
    let method_factory_alias = method_factory_alias(&signature, context);
    let declared_arity = signature.parameters.len();
    // Infer across the whole source application, including function-valued results.
    // Dictionary application still belongs immediately after the declared parameters.
    while let TypedType::Function { parameter, result } = signature.result.clone() {
        let (parameter_key, result_key) = match &signature.semantic_result {
            SemanticTypeKey::Function { parameter, result } => {
                (parameter.key.clone(), result.key.clone())
            }
            _ => (
                context.semantic_value_from_typed_type(&parameter).key,
                context.semantic_value_from_typed_type(&result).key,
            ),
        };
        signature.semantic_parameters.push(parameter_key);
        signature.parameters.push(*parameter);
        signature.result = *result;
        signature.semantic_result = result_key;
    }
    let expected_application = context.expected();
    let mut analyses = (0..argument_nodes.len())
        .map(|_| None)
        .collect::<Vec<Option<SurfaceExpressionAnalysis>>>();
    let mut semantic_arguments = (0..argument_nodes.len())
        .map(|_| None)
        .collect::<Vec<Option<SemanticValueType>>>();
    let inferred_arguments = super::let_inference::infer_call_arguments(
        &signature,
        argument_nodes,
        expected_application,
        context,
    );
    let argument_order = argument_nodes
        .iter()
        .enumerate()
        .filter(|(_, argument)| !requires_callable_context(argument, context))
        .chain(
            argument_nodes
                .iter()
                .enumerate()
                .filter(|(_, argument)| requires_callable_context(argument, context)),
        )
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in argument_order {
        let argument = argument_nodes[index];
        let indexed_arguments = semantic_arguments
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| argument.clone().map(|argument| (index, argument)))
            .collect::<Vec<_>>();
        let mut partial_application = instantiated_application_indexed(
            &signature,
            expected_application,
            argument_nodes.len(),
            &indexed_arguments,
            &|value| context.evidence_type(value),
        );
        refine_collection_parameters(&mut partial_application, &signature, context);
        for parameter in &mut partial_application.parameters {
            *parameter = context.hydrate_callable_parameter(parameter.clone(), callee_span);
        }
        let expected = partial_application
            .parameters
            .get(index)
            .cloned()
            .filter(|expected| {
                !is_unresolved_parameter_expectation(
                    &expected.type_ref,
                    &signature.type_parameters,
                    &partial_application.resolved_type_parameters,
                )
            })
            .map(|mut expected| {
                expected.type_ref = mask_unresolved_type_parameters(
                    &expected.type_ref,
                    &signature.type_parameters,
                    &partial_application.resolved_type_parameters,
                );
                expected
            });
        let expected = inferred_arguments
            .as_ref()
            .and_then(|arguments| arguments.get(index))
            .and_then(|argument| argument.clone())
            .or(expected);
        let argument_context = context.with_expected(expected);
        let analysis = type_argument(argument, &argument_context);
        semantic_arguments[index] = Some(SemanticValueType {
            type_ref: application_argument_type_from_expr(&analysis.value),
            key: analysis.semantic_type.clone(),
        });
        analyses[index] = Some(analysis);
    }
    let indexed_arguments = semantic_arguments
        .iter()
        .enumerate()
        .filter_map(|(index, argument)| argument.clone().map(|argument| (index, argument)))
        .collect::<Vec<_>>();
    let mut application = instantiated_application_indexed(
        &signature,
        expected_application,
        argument_nodes.len(),
        &indexed_arguments,
        &|value| context.evidence_type(value),
    );
    refine_collection_parameters(&mut application, &signature, context);
    for parameter in &mut application.parameters {
        *parameter = context.hydrate_callable_parameter(parameter.clone(), callee_span);
    }
    application.result = context.hydrate_semantic_value(application.result);
    let child_analyses = analyses
        .into_iter()
        .map(|analysis| analysis.expect("every application argument is typed"))
        .collect::<Vec<_>>();
    let arguments = child_analyses
        .iter()
        .map(|analysis| analysis.value.clone())
        .collect::<Vec<_>>();
    let semantic_arguments = semantic_arguments
        .into_iter()
        .map(|argument| argument.expect("every application argument has a semantic type"))
        .collect::<Vec<_>>();
    let mut issue = explicit_issue.or_else(|| {
        call_issue(
            callee_span,
            signature.parameters.len(),
            &application.parameters,
            argument_nodes,
            &arguments,
            &semantic_arguments,
        )
    });
    let computed_argument_issue = child_analyses
        .iter()
        .filter(|child| child.prioritize_computed_argument_issue)
        .filter_map(|child| child.pure_call_issue.as_ref())
        .find(|issue| matches!(issue, PureCallIssue::ArgumentType { .. }))
        .cloned();
    let prioritize_computed_argument_issue = issue.is_none() && computed_argument_issue.is_some();
    if prioritize_computed_argument_issue {
        // Preserve a computed callee's precise argument error through an enclosing
        // constrained call, leaving ordinary-call diagnostic priority intact.
        issue = computed_argument_issue;
    }
    if issue.is_none() && !context.recursive_groups.is_empty() {
        // Infer the callback expectation from the other arguments. A recursive
        // member's own scheme must not erase a more specific expectation.
        let other_arguments = indexed_arguments
            .iter()
            .filter(|(index, _)| {
                let mut value = argument_nodes[*index];
                while let SurfaceExpr::Grouped { value: inner, .. } = value {
                    value = inner;
                }
                !context.is_recursive_member(value.span())
            })
            .cloned()
            .collect::<Vec<_>>();
        let callback_application = instantiated_application_indexed(
            &signature,
            expected_application,
            argument_nodes.len(),
            &other_arguments,
            &|value| context.evidence_type(value),
        );
        for (argument, parameter) in argument_nodes.iter().zip(&callback_application.parameters) {
            let mut value = *argument;
            while let SurfaceExpr::Grouped { value: inner, .. } = value {
                value = inner;
            }
            if let Some(found) =
                context.recursive_call_issue(value.span(), &[], &parameter.type_ref)
            {
                issue = Some(found);
                break;
            }
        }
    }
    if issue.is_none() {
        issue = context.recursive_call_issue(
            callee_span,
            &semantic_arguments
                .iter()
                .map(|v| v.type_ref.clone())
                .collect::<Vec<_>>(),
            &instantiated_application_result_type(&application, arguments.len()),
        );
    }
    // An invalid argument cannot supply reliable type-parameter evidence.
    // Preserve its primary diagnostic instead of cascading into an unresolved
    // callable-result diagnostic during recovery.
    if issue.is_none() && child_analyses.iter().all(|child| !child.has_issues()) {
        let callable_result = instantiated_application_result_type(&application, arguments.len());
        if matches!(callable_result, TypedType::Function { .. })
            && contains_unresolved_type_parameter(
                &callable_result,
                &signature.type_parameters,
                &application.resolved_type_parameters,
            )
        {
            issue = Some(PureCallIssue::CallableTypeUnresolved {
                callee: expression_span,
            });
        }
    }
    // Inferred primitive keys must reach dictionary matching without being
    // recaptured by a same-name lexical nominal declaration.
    for (constraint, inferred) in application
        .constraints
        .iter_mut()
        .zip(&application.evidence_constraints)
    {
        constraint.arguments = inferred
            .arguments
            .iter()
            .map(|argument| {
                crate::typed::call_evidence::preserve_evidence_identity(
                    argument,
                    context.resolution,
                )
            })
            .collect();
    }
    let saturated = arguments.len() >= declared_arity;
    let concrete_partial_constraints = !saturated
        && signature.trait_identity.is_none()
        && !application.constraints.is_empty()
        && application.constraints.iter().all(|constraint| {
            constraint.arguments.iter().all(|argument| {
                !contains_unresolved_type_parameter(
                    argument,
                    &signature.type_parameters,
                    &application.resolved_type_parameters,
                )
            })
        });
    let scoped_partial_evidence = (!saturated
        && signature.trait_identity.is_none()
        && !application.constraints.is_empty()
        && !concrete_partial_constraints)
        .then(|| {
            context
                .select_call_evidence(&application.constraints, &application.constraint_identities)
                .ok()
                .filter(|evidence| {
                    evidence.iter().all(|item| {
                        matches!(
                            item.evidence,
                            crate::TypedInstanceEvidence::Parameter { .. }
                        )
                    })
                })
        })
        .flatten();
    let requires_evidence = saturated
        || signature.trait_identity.is_some()
        || concrete_partial_constraints
        || scoped_partial_evidence.is_some();
    let mut evidence = if issue.is_none() && requires_evidence {
        match scoped_partial_evidence.map(Ok).unwrap_or_else(|| {
            if signature.trait_method.is_some() {
                context.select_trait_call_evidence(
                    &application.constraints,
                    &application.constraint_identities,
                )
            } else {
                context.select_call_evidence(
                    &application.constraints,
                    &application.constraint_identities,
                )
            }
        }) {
            Ok(evidence) => evidence,
            Err(constraint) => {
                issue = Some(PureCallIssue::MissingInstance {
                    callee: callee_span,
                    constraint,
                });
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    if issue.is_none() {
        if let Some(template) = foreign_template(context, &signature.symbol) {
            let mut bindings = BTreeMap::new();
            for (formal, actual) in template.parameters.iter().zip(&application.parameters) {
                infer_type_parameters(
                    formal,
                    &crate::typed::functions::evidence_type(actual),
                    &template.type_parameters,
                    &mut bindings,
                );
            }
            infer_type_parameters(
                &template.result,
                &crate::typed::functions::evidence_type(&application.result),
                &template.type_parameters,
                &mut bindings,
            );
            for parameter in &template.type_parameters {
                let value = bindings.get(&parameter.name);
                if parameter.arity != 0
                    || !value.is_some_and(|value| concrete_foreign_codec(value, context))
                {
                    issue = Some(PureCallIssue::ForeignGenericCodecRequired {
                        callee: callee_span,
                    });
                    break;
                }
                // Preserve canonical primitive type arguments where a caller
                // nominal or generic binder would recapture their display name.
                // The runtime codec continues to use the same scalar string ABI.
                let value = crate::typed::call_evidence::preserve_evidence_identity(
                    value.unwrap(),
                    context.resolution,
                );
                evidence.push(crate::TypedCallEvidence {
                    trait_identity: None,
                    constraint: crate::TypedConstraint {
                        name: "ForeignCodec".into(),
                        arguments: vec![value.clone()],
                    },
                    evidence: crate::TypedInstanceEvidence::Standard {
                        identity: "@seseragi/foreign-codec".into(),
                        type_arguments: vec![value],
                        evidence_arguments: Vec::new(),
                    },
                });
            }
        }
    }
    let type_ref = if issue.is_none() {
        instantiated_application_result_type(&application, arguments.len())
    } else {
        TypedType::Hole
    };
    let semantic_type = if issue.is_some() {
        SemanticTypeKey::Invalid
    } else if arguments.len() >= signature.parameters.len() {
        application.result.key.clone()
    } else {
        SemanticTypeKey::Other
    };
    let deferred_evidence_parameters =
        if !evidence.is_empty() && !saturated && signature.trait_identity.is_none() {
            application
                .parameters
                .iter()
                .take(declared_arity)
                .skip(arguments.len())
                .map(|parameter| parameter.type_ref.clone())
                .collect()
        } else {
            Vec::new()
        };
    let deferred_evidence_type_constructor_parameters = if deferred_evidence_parameters.is_empty() {
        Vec::new()
    } else {
        signature
            .type_parameters
            .iter()
            .filter(|parameter| parameter.is_constructor())
            .map(|parameter| parameter.name.clone())
            .collect()
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Call {
            callee: signature.symbol.clone(),
            callee_signature: if issue.is_none() {
                // The source checker has instantiated every method stage.
                // Preserve that complete returned type at a dictionary-only
                // alias use; host inference cannot recover a function-valued
                // result from an erased dictionary argument.
                let factory_signature = method_factory_alias
                    .then(|| {
                        let result = application
                            .parameters
                            .iter()
                            .take(arguments.len())
                            .rev()
                            .fold(type_ref.clone(), |result, parameter| TypedType::Function {
                                parameter: Box::new(context.evidence_type(parameter)),
                                result: Box::new(result),
                            });
                        (!typed_type_contains_hole(&result)).then_some(crate::TypedCallSignature {
                            type_parameters: Vec::new(),
                            parameters: Vec::new(),
                            result,
                            argument_types: Vec::new(),
                            instantiated_type_arguments: Vec::new(),
                        })
                    })
                    .flatten();
                factory_signature.or(callee_signature).map(|mut signature| {
                    let selected = explicit_instantiation.clone().or_else(|| {
                        signature
                            .type_parameters
                            .iter()
                            .map(|parameter| {
                                application
                                    .instantiated_type_arguments
                                    .get(&parameter.name)
                                    .cloned()
                            })
                            .collect::<Option<Vec<_>>>()
                    });
                    if let Some(selected) = selected.filter(|selected| {
                        selected.len() == signature.type_parameters.len()
                            && selected
                                .iter()
                                .all(|value| !typed_type_contains_hole(value))
                    }) {
                        signature.instantiated_type_arguments = selected;
                    }
                    signature.argument_types = arguments
                        .iter()
                        .map(application_argument_type_from_expr)
                        .collect();
                    signature
                })
            } else {
                None
            },
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index: (argument_nodes.len() > declared_arity)
                .then_some(declared_arity),
            trait_dispatch: signature
                .trait_identity
                .clone()
                .zip(signature.trait_method.clone())
                .map(|(trait_identity, method)| crate::TypedTraitDispatch {
                    trait_identity,
                    method,
                    method_contract: (application.constraints.len() > 1).then(|| {
                        crate::TypedTraitMethodContract {
                            parameters: application
                                .parameters
                                .iter()
                                .take(declared_arity)
                                .map(|parameter| context.evidence_type(parameter))
                                .collect(),
                            constraints: application.constraints.clone(),
                            constraint_identities: application.constraint_identities.clone(),
                        }
                    }),
                }),
            type_ref,
            origin: expression_span,
        },
        semantic_type,
    );
    result.pure_call_issue = issue;
    result.prioritize_computed_argument_issue = prioritize_computed_argument_issue;
    for child in child_analyses {
        result.merge_issues_from(child);
    }
    if !result.has_issues() {
        result.value = stage_available_method_application(
            result.value,
            &signature,
            instantiated_application_result_type(&application, 0),
            callee_span,
            context,
        );
    }
    result
}

/// A primary Parameter describes a method's complete value type, not its
/// implementation's source stages. Specialize the available dictionaries once
/// before applying source arguments, using the same checked value entry as a
/// saved method. Keep ordinary and unproved dictionary ABIs separate.
fn stage_available_method_application(
    call: TypedExpr,
    signature: &TopLevelPureFunction,
    method_type: TypedType,
    callee_span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> TypedExpr {
    let TypedExpr::Call {
        arguments,
        evidence,
        trait_dispatch: Some(dispatch),
        ..
    } = &call
    else {
        return call;
    };
    if arguments.is_empty()
        || dispatch.method_contract.is_none()
        // Operator resolution still has to restore source operand order
        // before the shared Core operator staging selects the method value.
        || context.operator_target(callee_span).is_some()
        || seseragi_syntax::standard_trait_operator(&signature.symbol).is_some()
        || !matches!(
            evidence.first().map(|selected| &selected.evidence),
            Some(crate::TypedInstanceEvidence::Parameter { .. })
        )
        || !context.resolution.declared_method_value_scheme(signature)
    {
        return call;
    }
    let mut value = call.clone();
    let TypedExpr::Call {
        arguments,
        callee_signature,
        deferred_evidence_parameters,
        deferred_evidence_type_constructor_parameters,
        evidence_argument_index,
        type_ref,
        origin,
        ..
    } = &mut value
    else {
        unreachable!()
    };
    arguments.clear();
    *callee_signature = None;
    deferred_evidence_parameters.clear();
    deferred_evidence_type_constructor_parameters.clear();
    *evidence_argument_index = None;
    *type_ref = method_type.clone();
    *origin = callee_span;
    if !available_selected_method_value(&value, signature, callee_span, context) {
        return call;
    }
    let TypedExpr::Call {
        arguments,
        type_ref,
        origin,
        ..
    } = call
    else {
        unreachable!()
    };
    let name = format!("__ssrg$method$call${}", callee_span.start);
    TypedExpr::Block {
        statements: vec![crate::TypedBlockStatement::Let {
            constraints: Vec::new(),
            constraint_identities: Vec::new(),
            type_parameters: Vec::new(),
            pattern: crate::TypedPattern::Binding {
                symbol: crate::SymbolId(u32::MAX),
                name: name.clone(),
                type_ref: method_type,
                origin: callee_span,
            },
            value,
            origin: callee_span,
        }],
        result: Box::new(TypedExpr::Call {
            callee: name,
            arguments,
            callee_signature: None,
            evidence: Vec::new(),
            deferred_evidence_parameters: Vec::new(),
            deferred_evidence_type_constructor_parameters: Vec::new(),
            evidence_argument_index: None,
            trait_dispatch: None,
            type_ref: type_ref.clone(),
            origin,
        }),
        type_ref,
        origin,
    }
}

pub(crate) fn explicit_type_arguments(expression: &SurfaceExpr) -> Option<&[TypeRef]> {
    match expression {
        SurfaceExpr::Name { type_arguments, .. } | SurfaceExpr::Member { type_arguments, .. } => {
            type_arguments.as_deref()
        }
        _ => None,
    }
}

/// Iterable and Reducible determine their element from the collection. Reuse
/// the same scoped/imported evidence selection as `for`, so
/// constructors infer element/key/value types without a collection-specific
/// array shortcut or a Map/Set-only inference rule.
fn refine_collection_parameters(
    application: &mut InstantiatedApplication,
    signature: &TopLevelPureFunction,
    context: &PureExpressionContext<'_>,
) {
    let mut substitutions = BTreeMap::new();
    // Operator selection consumes the identity-preserving evidence arguments,
    // rather than the display types substituted for source annotations.
    for constraint in &application.evidence_constraints {
        if let [left, right, output] = constraint.arguments.as_slice() {
            let selected = context.select_binary_operator_evidence(
                &constraint.name,
                left.clone(),
                right.clone(),
            );
            if let Ok((actual, _)) = selected {
                infer_type_parameters(
                    output,
                    &context.evidence_type(&actual),
                    &signature.type_parameters,
                    &mut substitutions,
                );
            }
        }
    }
    for (constraint, identity) in application
        .constraints
        .iter()
        .zip(&application.constraint_identities)
    {
        // An absent identity denotes the standard-operation fallback, exactly
        // as in select_function_call_evidence; named user traits stay distinct.
        if !matches!(constraint.name.as_str(), "Iterable" | "Reducible")
            || identity
                .as_deref()
                .is_some_and(|identity| identity != format!("std/prelude::{}", constraint.name))
        {
            continue;
        }
        let [collection, element] = constraint.arguments.as_slice() else {
            continue;
        };
        let Some(actual) =
            context.infer_standard_collection_element(&constraint.name, collection.clone())
        else {
            continue;
        };
        infer_type_parameters(
            element,
            &actual,
            &signature.type_parameters,
            &mut substitutions,
        );
    }
    if substitutions.is_empty() {
        return;
    }
    // Collection evidence refines the source scheme, while expected-result
    // inference may already have attached an ADT key to a still-generic type.
    // Substitute the source type once and recover its matching scoped key;
    // rebuilding a type from that stale key can apply the ADT arguments twice.
    // An unchanged value already has its checked key, including primitives
    // whose display names now denote a different local nominal declaration.
    for parameter in &mut application.parameters {
        let refined = substitute_type_parameters(&parameter.type_ref, &substitutions);
        if refined != parameter.type_ref {
            *parameter = context.semantic_value_from_typed_type(&refined);
        }
    }
    let refined = substitute_type_parameters(&application.result.type_ref, &substitutions);
    if refined != application.result.type_ref {
        application.result = context.semantic_value_from_typed_type(&refined);
    }
    for constraint in application
        .constraints
        .iter_mut()
        .chain(&mut application.evidence_constraints)
    {
        for argument in &mut constraint.arguments {
            *argument = substitute_type_parameters(argument, &substitutions);
        }
    }
    application
        .resolved_type_parameters
        .extend(substitutions.into_keys());
}

fn instantiate_explicit_signature(
    mut signature: TopLevelPureFunction,
    arguments: Option<&[TypeRef]>,
    callee: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> (TopLevelPureFunction, Option<PureCallIssue>) {
    let Some(arguments) = arguments else {
        return (signature, None);
    };
    if arguments.len() != signature.type_parameters.len() {
        return (
            signature.clone(),
            Some(PureCallIssue::TypeArgumentArity {
                callee,
                expected: signature.type_parameters.len(),
                actual: arguments.len(),
            }),
        );
    }
    let substitutions = signature
        .type_parameters
        .iter()
        .zip(arguments)
        .map(|(parameter, argument)| {
            (
                parameter.name.clone(),
                context.semantic_value_from_type_ref(argument).type_ref,
            )
        })
        .collect::<BTreeMap<_, _>>();
    signature.parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            crate::typed::functions::substitute_type_parameters(parameter, &substitutions)
        })
        .collect();
    signature.result =
        crate::typed::functions::substitute_type_parameters(&signature.result, &substitutions);
    signature.constraints = signature
        .constraints
        .iter()
        .map(|constraint| crate::TypedConstraint {
            name: constraint.name.clone(),
            arguments: constraint
                .arguments
                .iter()
                .map(|argument| {
                    crate::typed::functions::substitute_type_parameters(argument, &substitutions)
                })
                .collect(),
        })
        .collect();
    signature.semantic_parameters = signature
        .parameters
        .iter()
        .map(|parameter| context.semantic_value_from_typed_type(parameter).key)
        .collect();
    signature.semantic_result = context
        .semantic_value_from_typed_type(&signature.result)
        .key;
    signature.type_parameters.clear();
    (signature, None)
}

// Like lambdas, operator sections need operand types from the other arguments
// before selecting their dictionary. This changes typing order, not execution order.
fn requires_callable_context(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> bool {
    if context
        .target(expression.span())
        .and_then(|target| context.callable_value(target))
        .is_some_and(|signature| !signature.type_parameters.is_empty())
    {
        return true;
    }
    match expression {
        SurfaceExpr::Lambda { .. } => true,
        SurfaceExpr::Name { name, .. } => {
            context.is_recursive_member(expression.span())
                || name == ":"
                || seseragi_syntax::standard_operator(name).is_some()
                || seseragi_syntax::standard_trait_operator(name).is_some()
        }
        SurfaceExpr::Grouped { value, .. } => requires_callable_context(value, context),
        _ => false,
    }
}

// A computed callee is already a checked monomorphic function value. Capture it
// before typing the ordinary curried application, so its concrete result reaches
// enclosing trait selection and its source evaluation happens exactly once.
fn type_function_expression_application(
    callee: &SurfaceExpr,
    callee_analysis: SurfaceExpressionAnalysis,
    arguments: &[&SurfaceExpr],
    span: ByteSpan,
    explicit_type_arguments: Option<&[TypeRef]>,
    context: &PureExpressionContext<'_>,
    type_argument: &mut impl FnMut(
        &SurfaceExpr,
        &PureExpressionContext<'_>,
    ) -> SurfaceExpressionAnalysis,
) -> SurfaceExpressionAnalysis {
    let mut result_type = application_argument_type_from_expr(&callee_analysis.value);
    let mut parameters = Vec::new();
    while let TypedType::Function { parameter, result } = result_type {
        parameters.push(*parameter);
        result_type = *result;
    }
    debug_assert!(!parameters.is_empty());
    let (name, callee_span) = if let SurfaceExpr::Member { field_span, .. } = callee {
        (format!("__ssrg$field${}", callee.span().start), *field_span)
    } else {
        (
            format!("__ssrg$callee${}", callee.span().start),
            callee.span(),
        )
    };
    let signature = TopLevelPureFunction {
        symbol: name.clone(),
        trait_identity: None,
        trait_method: None,
        type_parameters: Vec::new(),
        constraints: Vec::new(),
        constraint_identities: Vec::new(),
        semantic_parameters: parameters
            .iter()
            .map(|parameter| context.semantic_value_from_typed_type(parameter).key)
            .collect(),
        parameters,
        semantic_result: context.semantic_value_from_typed_type(&result_type).key,
        result: result_type,
    };
    let mut call = type_known_application_with_explicit(
        signature,
        explicit_type_arguments,
        callee_span,
        arguments,
        span,
        context,
        type_argument,
    );
    let type_ref = application_argument_type_from_expr(&call.value);
    let callee_type = application_argument_type_from_expr(&callee_analysis.value);
    call.value = TypedExpr::Block {
        statements: vec![crate::TypedBlockStatement::Let {
            constraints: Vec::new(),
            constraint_identities: Vec::new(),
            type_parameters: Vec::new(),
            pattern: crate::TypedPattern::Binding {
                symbol: crate::SymbolId(u32::MAX),
                name,
                type_ref: callee_type,
                origin: callee.span(),
            },
            value: callee_analysis.value.clone(),
            origin: callee.span(),
        }],
        result: Box::new(call.value),
        type_ref,
        origin: span,
    };
    call.prioritize_computed_argument_issue = matches!(
        call.pure_call_issue,
        Some(PureCallIssue::ArgumentType { .. })
    );
    call.merge_issues_from(callee_analysis);
    call
}

fn type_unknown_application(
    callee: &SurfaceExpr,
    arguments: &[&SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let child_context = context.without_expected();
    let mut callee_analysis = type_surface_expression(callee, &child_context);
    if matches!(
        application_argument_type_from_expr(&callee_analysis.value),
        TypedType::Function { .. }
    ) {
        return type_function_expression_application(
            callee,
            callee_analysis,
            arguments,
            span,
            None,
            context,
            &mut type_surface_expression,
        );
    }
    for argument in arguments {
        let child = type_surface_expression(argument, &child_context);
        callee_analysis.merge_issues_from(child);
    }
    callee_analysis.value = TypedExpr::Variable {
        callable_signature: None,
        name: String::new(),
        evidence: Vec::new(),
        type_ref: TypedType::Hole,
        origin: span,
    };
    callee_analysis.semantic_type = SemanticTypeKey::Invalid;
    callee_analysis
}

fn call_issue(
    callee: ByteSpan,
    parameter_count: usize,
    parameters: &[SemanticValueType],
    argument_nodes: &[&SurfaceExpr],
    arguments: &[TypedExpr],
    semantic_arguments: &[SemanticValueType],
) -> Option<PureCallIssue> {
    if arguments.len() > parameter_count {
        return Some(PureCallIssue::Arity {
            callee,
            expected: parameter_count,
            actual: arguments.len(),
        });
    }
    arguments
        .iter()
        .zip(semantic_arguments)
        .zip(parameters)
        .zip(argument_nodes)
        .enumerate()
        .find_map(
            |(index, (((argument, actual_semantic), expected), source))| {
                let actual = application_argument_type_from_expr(argument);
                (!typed_type_contains_hole(&actual)
                    && !semantic_values_are_compatible(expected, actual_semantic))
                .then(|| PureCallIssue::ArgumentType {
                    argument: source.span(),
                    index,
                    expected: expected.type_ref.clone(),
                    actual,
                })
            },
        )
}

fn flatten_application(expression: &SurfaceExpr) -> (&SurfaceExpr, Vec<&SurfaceExpr>) {
    let mut callee = expression;
    let mut arguments = Vec::new();
    loop {
        match callee {
            SurfaceExpr::Application {
                function, argument, ..
            } => {
                arguments.push(argument.as_ref());
                callee = function.as_ref();
            }
            SurfaceExpr::Grouped { value, .. } => callee = value.as_ref(),
            _ => break,
        }
    }
    arguments.reverse();
    (callee, arguments)
}

fn is_unresolved_parameter_expectation(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> bool {
    match type_ref {
        TypedType::Named { name, .. } => {
            parameters.iter().any(|parameter| parameter.name == *name) && !resolved.contains(name)
        }
        TypedType::Hole => true,
        TypedType::ExternalNamed { .. }
        | TypedType::Record { .. }
        | TypedType::Tuple { .. }
        | TypedType::Function { .. }
        | TypedType::RequirementMerge { .. } => false,
    }
}

fn mask_unresolved_type_parameters(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> TypedType {
    match type_ref {
        TypedType::Named { name, .. }
            if parameters.iter().any(|parameter| parameter.name == *name)
                && !resolved.contains(name) =>
        {
            TypedType::Hole
        }
        TypedType::Named { name, arguments } => TypedType::Named {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| mask_unresolved_type_parameters(argument, parameters, resolved))
                .collect(),
        },
        TypedType::ExternalNamed {
            canonical,
            name,
            arguments,
        } => TypedType::ExternalNamed {
            canonical: canonical.clone(),
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| mask_unresolved_type_parameters(argument, parameters, resolved))
                .collect(),
        },
        TypedType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| crate::TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: mask_unresolved_type_parameters(
                        &field.type_ref,
                        parameters,
                        resolved,
                    ),
                })
                .collect(),
        },
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements
                .iter()
                .map(|element| mask_unresolved_type_parameters(element, parameters, resolved))
                .collect(),
        },
        TypedType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(mask_unresolved_type_parameters(
                parameter, parameters, resolved,
            )),
            result: Box::new(mask_unresolved_type_parameters(
                result, parameters, resolved,
            )),
        },
        TypedType::RequirementMerge { operands } => {
            crate::typed::type_ref::normalize_requirement_merge(
                operands
                    .iter()
                    .map(|operand| mask_unresolved_type_parameters(operand, parameters, resolved))
                    .collect(),
            )
        }
        TypedType::Hole => TypedType::Hole,
    }
}

fn contains_unresolved_type_parameter(
    type_ref: &TypedType,
    parameters: &[seseragi_syntax::TypeParameter],
    resolved: &std::collections::BTreeSet<String>,
) -> bool {
    match type_ref {
        TypedType::Named { name, arguments } => {
            (parameters.iter().any(|parameter| parameter.name == *name) && !resolved.contains(name))
                || arguments.iter().any(|argument| {
                    contains_unresolved_type_parameter(argument, parameters, resolved)
                })
        }
        TypedType::ExternalNamed { arguments, .. } => arguments
            .iter()
            .any(|argument| contains_unresolved_type_parameter(argument, parameters, resolved)),
        TypedType::Record { fields, .. } => fields
            .iter()
            .any(|field| contains_unresolved_type_parameter(&field.type_ref, parameters, resolved)),
        TypedType::Tuple { elements } => elements
            .iter()
            .any(|element| contains_unresolved_type_parameter(element, parameters, resolved)),
        TypedType::Function { parameter, result } => {
            contains_unresolved_type_parameter(parameter, parameters, resolved)
                || contains_unresolved_type_parameter(result, parameters, resolved)
        }
        TypedType::RequirementMerge { operands } => operands
            .iter()
            .any(|operand| contains_unresolved_type_parameter(operand, parameters, resolved)),
        TypedType::Hole => true,
    }
}

// Resolve the original declaration scheme, including imported/reexported generic
// foreign callables. An explicit instantiation may have cleared its local binder.
fn foreign_template<'a>(
    context: &'a PureExpressionContext<'_>,
    symbol: &str,
) -> Option<&'a TopLevelPureFunction> {
    let resolved = context.resolution.resolved();
    fn contains(members: &[seseragi_syntax::SurfaceForeignMember], origin: ByteSpan) -> bool {
        members.iter().any(|member| match member {
            seseragi_syntax::SurfaceForeignMember::Function {
                name_span,
                type_parameters,
                ..
            } => *name_span == origin && !type_parameters.is_empty(),
            seseragi_syntax::SurfaceForeignMember::Namespace { members, .. } => {
                contains(members, origin)
            }
            _ => false,
        })
    }
    let target = resolved
        .symbols
        .iter()
        .find(|target| target.canonical.as_deref() == Some(symbol))?;
    let foreign = resolved
        .foreign_modules
        .iter()
        .any(|module| contains(&module.members, target.origin))
        || resolved.imports.iter().any(|import| {
            import.export.symbol == symbol
                && import.export.declaration_kind.as_deref() == Some("generic-foreign-function")
        });
    foreign
        .then(|| context.resolution.callable(target.id))
        .flatten()
}
fn concrete_foreign_codec(value: &TypedType, context: &PureExpressionContext<'_>) -> bool {
    if let TypedType::ExternalNamed {
        name,
        canonical,
        arguments,
    } = value
    {
        return arguments.is_empty()
            && matches!(
                name.as_str(),
                "Unit" | "Bool" | "Char" | "String" | "Int" | "Float"
            )
            && canonical == &format!("std/prelude::{name}");
    }
    let TypedType::Named { name, arguments } = value else {
        return false;
    };
    arguments.is_empty()
        && matches!(
            name.as_str(),
            "Unit" | "Bool" | "Char" | "String" | "Int" | "Float"
        )
        && !context.lexical_type_parameters.contains_key(name)
        && matches!(
            context.semantic_value_from_typed_type(value).key,
            SemanticTypeKey::Other
        )
}
