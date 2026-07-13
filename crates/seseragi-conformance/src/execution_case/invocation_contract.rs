use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::InterfaceType;

use crate::execution::{Invocation, InvocationArgument};

use super::effect_contract::standard_types::reject_user_shadow;

/// Validates an execution invocation against the final typed value export.
/// Effect environment and failure contracts are checked by the separate
/// Effect entry validator after this call boundary has been accepted.
pub(crate) fn validate_final_interface_invocation(
    interface: &TypedModuleInterface,
    entry_export: &str,
    invocation: &Invocation,
) -> Result<(), String> {
    if interface.schema != 1 || interface.stage != "typed-interface" {
        return Err("execution entry requires a final schema 1 typed interface".to_owned());
    }
    let export = interface
        .exports
        .iter()
        .find(|export| export.namespace == "value" && export.name == entry_export)
        .ok_or_else(|| {
            format!("execution entry export {entry_export} is missing from typed interface")
        })?;
    let (parameters, result) = flatten_function_type(&export.scheme.type_ref);
    let arguments = invocation_arguments(invocation);
    if arguments.len() != parameters.len() {
        return Err(format!(
            "execution entry {entry_export} expects {} arguments, got {}",
            parameters.len(),
            arguments.len()
        ));
    }
    for (index, (argument, parameter)) in arguments.iter().zip(parameters).enumerate() {
        let runner_type = invocation_argument_spelling(argument);
        if named_type_is(parameter, runner_type) {
            reject_user_shadow(interface, runner_type, "parameter type").map_err(|error| {
                format!(
                    "execution entry {entry_export} argument {} requires standard {runner_type}: {error}",
                    index + 1
                )
            })?;
        }
        if !argument_matches_type(interface, argument, parameter) {
            return Err(format!(
                "execution entry {entry_export} argument {} expects {}, got {}",
                index + 1,
                interface_type_spelling(parameter),
                invocation_argument_spelling(argument)
            ));
        }
    }

    let returns_effect = is_effect_type(interface, result);
    match invocation {
        Invocation::PureJson { .. } if returns_effect => Err(format!(
            "execution entry {entry_export} returns Effect<R, E, A> and cannot use pure invocation"
        )),
        Invocation::Effect { .. } if !returns_effect => Err(format!(
            "execution entry {entry_export} must return Effect<R, E, A> for Effect invocation"
        )),
        _ => Ok(()),
    }
}

fn flatten_function_type(type_ref: &InterfaceType) -> (Vec<&InterfaceType>, &InterfaceType) {
    let mut parameters = Vec::new();
    let mut result = type_ref;
    while let InterfaceType::Function {
        parameter,
        result: next,
    } = result
    {
        parameters.push(parameter.as_ref());
        result = next.as_ref();
    }
    (parameters, result)
}

fn invocation_arguments(invocation: &Invocation) -> &[InvocationArgument] {
    match invocation {
        Invocation::Effect { arguments } | Invocation::PureJson { arguments } => arguments,
    }
}

fn argument_matches_type(
    interface: &TypedModuleInterface,
    argument: &InvocationArgument,
    type_ref: &InterfaceType,
) -> bool {
    let expected = match argument {
        InvocationArgument::Unit => "Unit",
        InvocationArgument::String(_) => "String",
    };
    reject_user_shadow(interface, expected, "parameter type").is_ok()
        && named_type_is(type_ref, expected)
}

fn named_type_is(type_ref: &InterfaceType, expected: &str) -> bool {
    matches!(
        type_ref,
        InterfaceType::Named { name, arguments }
            if name == expected && arguments.is_empty()
    )
}

fn is_effect_type(interface: &TypedModuleInterface, type_ref: &InterfaceType) -> bool {
    reject_user_shadow(interface, "Effect", "result type").is_ok()
        && matches!(
            type_ref,
            InterfaceType::Named { name, arguments }
                if name == "Effect" && arguments.len() == 3
        )
}

fn invocation_argument_spelling(argument: &InvocationArgument) -> &'static str {
    match argument {
        InvocationArgument::Unit => "Unit",
        InvocationArgument::String(_) => "String",
    }
}

fn interface_type_spelling(type_ref: &InterfaceType) -> String {
    match type_ref {
        InterfaceType::Named { name, arguments }
        | InterfaceType::ExternalNamed {
            name, arguments, ..
        } => type_application_spelling(name, arguments),
        InterfaceType::Apply {
            constructor,
            arguments,
        } => type_application_spelling(constructor, arguments),
        InterfaceType::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(interface_type_spelling)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        InterfaceType::Function { .. } => "<function>".to_owned(),
        InterfaceType::Record { .. } => "<record>".to_owned(),
        InterfaceType::TypeConstructor { name, .. } => name.clone(),
        InterfaceType::Hole => "_".to_owned(),
    }
}

fn type_application_spelling(name: &str, arguments: &[InterfaceType]) -> String {
    if arguments.is_empty() {
        name.to_owned()
    } else {
        format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(interface_type_spelling)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests;
