use super::*;

pub(super) fn character_literal(value: &str) -> Result<String> {
    let mut scalars = value.chars();
    let Some(scalar) = scalars.next() else {
        return Err("Rust backend: Char must contain one Unicode scalar".into());
    };
    if scalars.next().is_some() {
        return Err("Rust backend: Char must contain one Unicode scalar".into());
    }
    Ok(format!("V::Char({scalar:?})"))
}

pub(super) fn character_reference(name: &str) -> Option<String> {
    if let Some(function) = character_operation(name) {
        return Some(format!("V::fun(vec![], |_, value| {function}(value))"));
    }
    match name {
        "std/prelude::Just" => {
            Some("V::fun(vec![], |_, value| V::adt(\"std/prelude::Just\", value))".into())
        }
        "std/prelude::Nothing"
        | "std/prelude::Less"
        | "std/prelude::Equal"
        | "std/prelude::Greater" => Some(format!("V::adt({}, V::Unit)", quoted(name))),
        _ => None,
    }
}

pub(super) fn character_operation(name: &str) -> Option<&'static str> {
    Some(match name {
        "std/char::codePoint" => "native_char_code_point",
        "std/char::fromCodePoint" => "native_char_from_code_point",
        "std/char::toString" => "native_char_to_string",
        _ => return None,
    })
}

fn standard_constraint(evidence: &CoreCallEvidence, name: &str) -> bool {
    evidence.constraint.name == name
        && evidence
            .constraint
            .trait_identity
            .as_deref()
            .is_none_or(|identity| identity == format!("std/prelude::{name}"))
        && evidence.constraint.arguments.len() == 1
}

fn character_evidence(evidence: &CoreCallEvidence, name: &str) -> bool {
    standard_constraint(evidence, name)
        && matches!(&evidence.constraint.arguments[0],
            CoreType::Named { name, arguments } if name == "Char" && arguments.is_empty())
        && matches!(&evidence.evidence,
            CoreInstanceEvidence::Standard { type_arguments, evidence_arguments, .. }
                if type_arguments.is_empty() && evidence_arguments.is_empty())
}

pub(super) fn character_dictionary(evidence: &CoreCallEvidence) -> Option<Result<String>> {
    let CoreInstanceEvidence::Standard { identity, .. } = &evidence.evidence else {
        return None;
    };
    let name = match identity.as_str() {
        "std/char::Eq" => "Eq",
        "std/char::Ord" => "Ord",
        "Show<std/prelude::Char>" => "Show",
        "Debug<std/prelude::Char>" => "Debug",
        _ => return None,
    };
    Some(if character_evidence(evidence, name) {
        Ok(format!("native_char_dictionary({})", quoted(identity)))
    } else {
        Err("Rust backend: invalid canonical Char dictionary evidence".into())
    })
}

fn comparison_trait(operator: &str) -> Option<&'static str> {
    match operator {
        "==" | "!=" => Some("Eq"),
        "<" | "<=" | ">" | ">=" => Some("Ord"),
        _ => None,
    }
}

pub(super) fn character_comparison(evidence: &[CoreCallEvidence], operator: &str) -> bool {
    let ([selected], Some(name)) = (evidence, comparison_trait(operator)) else {
        return false;
    };
    character_evidence(selected, name)
        && matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. }
            if identity == &format!("std/char::{name}"))
}

pub(super) fn dictionary_comparison(evidence: &[CoreCallEvidence], operator: &str) -> bool {
    let ([selected], Some(name)) = (evidence, comparison_trait(operator)) else {
        return false;
    };
    // A generic or user-defined instance must use its checked method. Never
    // infer the argument representation from a type's source-level spelling.
    standard_constraint(selected, name)
        && matches!(
            selected.evidence,
            CoreInstanceEvidence::Parameter { .. } | CoreInstanceEvidence::Local { .. }
        )
}

pub(super) fn closed_character_partial(
    module: &CoreModule,
    callee: &str,
    arguments: usize,
    evidence: &[CoreCallEvidence],
    deferred: &[CoreType],
) -> bool {
    let Some(function) = module
        .functions
        .iter()
        .find(|function| function.symbol == callee)
    else {
        return false;
    };
    // The native ABI binds dictionaries in the factory before source arguments.
    // TS represents the remaining source arrows explicitly to pass dictionaries
    // after saturation. This narrow case has already selected every closed Char
    // dictionary; neither a future argument nor a returned function chooses one.
    !function.is_effect
        && arguments < function.parameters.len()
        && function.parameters.len() - arguments == deferred.len()
        && !evidence.is_empty()
        && evidence.len() == function.constraints.len()
        && evidence
            .iter()
            .all(|selected| matches!(character_dictionary(selected), Some(Ok(_))))
        && deferred.iter().all(|ty| {
            matches!(ty, CoreType::Named { name, arguments } if name == "Char" && arguments.is_empty())
        })
}
