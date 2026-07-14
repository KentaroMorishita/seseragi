use crate::{ResolvedModule, SymbolId};
use seseragi_syntax::{SurfaceMethod, TypeRef};
use std::collections::BTreeMap;

mod model;
mod resolution;

use model::{ContractMethod, ContractType};
use resolution::{contract_constraint, contract_type, declaration_type_parameters, method_binders};

pub(super) fn method_contract_matches(
    resolved: &ResolvedModule,
    trait_span: seseragi_syntax::ByteSpan,
    trait_parameters: &[String],
    instance_arguments: &[TypeRef],
    expected: &SurfaceMethod,
    actual: &SurfaceMethod,
) -> Option<bool> {
    if trait_parameters.len() != instance_arguments.len()
        || expected.type_parameters.len() != actual.type_parameters.len()
    {
        return Some(false);
    }

    let expected_binders = method_binders(resolved, expected)?;
    let actual_binders = method_binders(resolved, actual)?;
    let trait_symbols = declaration_type_parameters(resolved, trait_span, trait_parameters)?;
    let mut substitutions = BTreeMap::new();
    for (symbol, argument) in trait_symbols.into_iter().zip(instance_arguments) {
        substitutions.insert(
            symbol,
            contract_type(resolved, argument, &BTreeMap::new(), &BTreeMap::new())?,
        );
    }

    let expected = contract_method(resolved, expected, &expected_binders, &substitutions)?;
    let actual = contract_method(resolved, actual, &actual_binders, &BTreeMap::new())?;
    Some(expected == actual)
}

fn contract_method(
    resolved: &ResolvedModule,
    method: &SurfaceMethod,
    binders: &BTreeMap<SymbolId, u32>,
    substitutions: &BTreeMap<SymbolId, ContractType>,
) -> Option<ContractMethod> {
    let result = contract_type(resolved, &method.return_type, binders, substitutions)?;
    let type_ref = method
        .parameters
        .iter()
        .rev()
        .try_fold(result, |result, parameter| {
            Some(ContractType::Function {
                parameter: Box::new(contract_type(
                    resolved,
                    &parameter.type_ref,
                    binders,
                    substitutions,
                )?),
                result: Box::new(result),
            })
        })?;
    let mut constraints = method
        .constraints
        .iter()
        .map(|constraint| contract_constraint(resolved, constraint, binders, substitutions))
        .collect::<Option<Vec<_>>>()?;
    constraints.sort();
    Some(ContractMethod {
        type_ref,
        constraints,
    })
}
