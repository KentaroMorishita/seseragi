use super::*;

pub(super) fn float_literal(value: &str) -> Result<String> {
    let value = value
        .replace('_', "")
        .parse::<f64>()
        .map_err(|_| "Rust backend: invalid Float literal")?;
    if !value.is_finite() {
        return Err("Rust backend: non-finite Float literal".into());
    }
    Ok(format!("V::Float(f64::from_bits({}_u64))", value.to_bits()))
}

pub(super) fn float_operation_arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/float::nan"
        | "std/float::positiveInfinity"
        | "std/float::negativeInfinity"
        | "std/float::format"
        | "std/float::fromInt"
        | "std/float::isNaN"
        | "std/float::isFinite"
        | "std/float::isInfinite"
        | "std/float::isNegativeZero"
        | "std/float::abs"
        | "std/float::sign" => 1,
        "std/float::ieeeEq"
        | "std/float::totalCompare"
        | "std/float::minimumNumber"
        | "std/float::maximumNumber" => 2,
        "std/float::clampNumber" => 3,
        _ => return None,
    })
}

fn float_signature(identity: &str) -> Option<(&'static str, usize)> {
    Some(match identity {
        "std/float::Add" => ("Add", 3),
        "std/float::Sub" => ("Sub", 3),
        "std/float::Mul" => ("Mul", 3),
        "std/float::Div" => ("Div", 3),
        "std/float::Rem" => ("Rem", 3),
        "std/float::Zero" => ("Zero", 1),
        "std/float::One" => ("One", 1),
        "Show<std/prelude::Float>" => ("Show", 1),
        "Debug<std/prelude::Float>" => ("Debug", 1),
        _ => return None,
    })
}

fn is_float(ty: &CoreType) -> bool {
    matches!(ty, CoreType::Named { name, arguments } if name == "Float" && arguments.is_empty())
}

pub(super) fn valid_float_dispatch(
    selected: &CoreCallEvidence,
    identity: &str,
    method: &str,
) -> bool {
    if float_dictionary(selected).is_none() {
        return true;
    }
    let expected = match selected.constraint.name.as_str() {
        "Add" => "add",
        "Sub" => "sub",
        "Mul" => "mul",
        "Div" => "div",
        "Rem" => "rem",
        "Zero" => "zero",
        "One" => "one",
        "Show" => "show",
        "Debug" => "debug",
        _ => return false,
    };
    identity == format!("std/prelude::{}", selected.constraint.name) && method == expected
}

pub(super) fn float_dictionary(selected: &CoreCallEvidence) -> Option<Result<String>> {
    let CoreInstanceEvidence::Standard {
        identity,
        type_arguments,
        evidence_arguments,
    } = &selected.evidence
    else {
        return None;
    };
    let (name, arity) = float_signature(identity)?;
    Some(
        if selected.constraint.name == name
            && selected
                .constraint
                .trait_identity
                .as_deref()
                .is_none_or(|identity| identity == format!("std/prelude::{name}"))
            && selected.constraint.arguments.len() == arity
            && selected.constraint.arguments.iter().all(is_float)
            && type_arguments.is_empty()
            && evidence_arguments.is_empty()
        {
            Ok(format!("native_float_dictionary({})", quoted(identity)))
        } else {
            Err("Rust backend: invalid canonical Float dictionary evidence".into())
        },
    )
}

pub(super) fn float_binary(evidence: &[CoreCallEvidence], operator: &str) -> bool {
    let [selected] = evidence else { return false };
    let name = match operator {
        "+" => "Add",
        "-" => "Sub",
        "*" => "Mul",
        "/" => "Div",
        "%" => "Rem",
        _ => return false,
    };
    matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. }
        if identity == &format!("std/float::{name}"))
        && matches!(float_dictionary(selected), Some(Ok(_)))
}

pub(super) fn closed_float_partial(
    module: &CoreModule,
    callee: &str,
    arguments: usize,
    evidence: &[CoreCallEvidence],
    deferred: &[CoreType],
) -> bool {
    let Some(function) = module.functions.iter().find(|f| f.symbol == callee) else {
        return false;
    };
    !function.is_effect
        && arguments < function.parameters.len()
        && function.parameters.len() - arguments == deferred.len()
        && !evidence.is_empty()
        && evidence.len() == function.constraints.len()
        && evidence
            .iter()
            .all(|e| matches!(float_dictionary(e), Some(Ok(_))))
        && deferred.iter().all(is_float)
}
