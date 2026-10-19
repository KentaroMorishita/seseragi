use super::*;
use seseragi_syntax::StandardTraitOperator;

fn expected(operator: &StandardTraitOperator, reverse_binders: bool) -> ContractType {
    let a = ContractType::Binder(u32::from(reverse_binders));
    let b = ContractType::Binder(u32::from(!reverse_binders));
    let applied = |argument| ContractType::Apply {
        constructor: Box::new(ContractType::Binder(2)),
        arguments: vec![argument],
    };
    let arrow = |parameter, result| ContractType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    };
    let first = match operator.method_name {
        "map" => arrow(a.clone(), b.clone()),
        "apply" => applied(arrow(a.clone(), b.clone())),
        "flatMap" => arrow(a.clone(), applied(b.clone())),
        _ => unreachable!("registered standard trait operator method"),
    };
    arrow(first, arrow(applied(a), applied(b)))
}

fn matches_operator_type(actual: &ContractType, operator: &StandardTraitOperator) -> bool {
    // These two method-only value quantifiers have no method dictionaries.
    // Prove either scoped bijection; never exchange the constructor binder,
    // merge repeated occurrences, or change the named method's own scheme.
    actual == &expected(operator, false) || actual == &expected(operator, true)
}

fn kinds_match(trait_parameters: &[TypeParameter], method_parameters: &[TypeParameter]) -> bool {
    matches!(trait_parameters, [parameter] if parameter.arity == 1)
        && method_parameters.len() == 2
        && method_parameters
            .iter()
            .all(|parameter| parameter.arity == 0)
}

pub(in crate::typed::instances::contracts) fn local_operator_method_matches(
    resolved: &ResolvedModule,
    trait_span: seseragi_syntax::ByteSpan,
    trait_parameters: &[TypeParameter],
    method: &SurfaceMethod,
    operator: &StandardTraitOperator,
) -> bool {
    if !kinds_match(trait_parameters, &method.type_parameters) {
        return false;
    }
    let Some(binders) = method_binders(resolved, method) else {
        return false;
    };
    let Some(symbols) = declaration_type_parameters(resolved, trait_span, trait_parameters) else {
        return false;
    };
    let substitutions = BTreeMap::from([(symbols[0], ContractType::Binder(2))]);
    contract_method(resolved, method, &binders, &substitutions)
        .and_then(|contract| aliases::normalize(resolved, contract))
        .is_some_and(|contract| {
            contract.constraints.is_empty() && matches_operator_type(&contract.type_ref, operator)
        })
}

pub(in crate::typed::instances::contracts) fn imported_operator_method_matches(
    resolved: &ResolvedModule,
    trait_parameters: &[TypeParameter],
    method: &InterfaceMethod,
    context: ImportedMethodContext<'_>,
    operator: &StandardTraitOperator,
) -> bool {
    if !kinds_match(trait_parameters, &method.scheme.type_parameters) {
        return false;
    }
    let substitutions =
        BTreeMap::from([(trait_parameters[0].name.clone(), ContractType::Binder(2))]);
    interface::contract_method_from_interface(
        method,
        &substitutions,
        context.bindings,
        context.trait_bindings,
        context.trait_name,
        context.trait_canonical,
    )
    .and_then(|contract| aliases::normalize(resolved, contract))
    .is_some_and(|contract| {
        contract.constraints.is_empty() && matches_operator_type(&contract.type_ref, operator)
    })
}
