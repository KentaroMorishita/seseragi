use super::*;

pub(super) fn list_element(ty: &CoreType) -> Option<&CoreType> {
    match ty {
        CoreType::Named { name, arguments } if name == "List" && arguments.len() == 1 => {
            Some(&arguments[0])
        }
        _ => None,
    }
}

pub(super) fn list_operation_arity(name: &str) -> Option<usize> {
    Some(match name {
        "std/list::empty"
        | "std/list::singleton"
        | "std/list::length"
        | "std/list::isEmpty"
        | "std/list::head"
        | "std/list::tail"
        | "std/list::toArray"
        | "std/list::reverse"
        | "std/array::toList" => 1,
        "std/prelude:::" | "std/list::get" | "std/list::take" | "std/list::drop"
        | "std/list::append" | "std/list::filter" | "std/list::find" => 2,
        _ => return None,
    })
}

pub(super) fn standard_trait(selected: &CoreCallEvidence, name: &str, ty: &CoreType) -> bool {
    selected.constraint.name == name
        && selected.constraint.arguments == [ty.clone()]
        && selected
            .constraint
            .trait_identity
            .as_deref()
            .is_none_or(|identity| identity == format!("std/prelude::{name}"))
}

impl Backend<'_> {
    pub(super) fn list_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let method = match identity.as_str() {
            "std/list::Show" => "Show",
            "std/list::Debug" => "Debug",
            _ => return Ok(None),
        };
        let ([element], [child]) = (type_arguments.as_slice(), evidence_arguments.as_slice())
        else {
            return Err("Rust backend: invalid canonical List display evidence arity".into());
        };
        let ty = CoreType::Named {
            name: "List".into(),
            arguments: vec![element.clone()],
        };
        if !standard_trait(selected, method, &ty) || !standard_trait(child, method, element) {
            return Err("Rust backend: invalid canonical List display identity or child".into());
        }
        let child = self.evidence(std::slice::from_ref(child), scope)?;
        self.capabilities.insert("list".into());
        Ok(Some(format!(
            "native_list_display_dictionary(({child}).remove(0), {})",
            method == "Debug"
        )))
    }
}

pub(super) fn primitive_display(selected: &CoreCallEvidence) -> Option<Result<String>> {
    let CoreInstanceEvidence::Standard {
        identity,
        type_arguments,
        evidence_arguments,
    } = &selected.evidence
    else {
        return None;
    };
    let (method, primitive) = ["Show", "Debug"].into_iter().find_map(|method| {
        ["Unit", "Int", "Bool", "String"]
            .into_iter()
            .find(|primitive| identity == &format!("{method}<std/prelude::{primitive}>"))
            .map(|primitive| (method, primitive))
    })?;
    let ty = CoreType::Named {
        name: primitive.into(),
        arguments: vec![],
    };
    Some(
        if standard_trait(selected, method, &ty)
            && type_arguments.is_empty()
            && evidence_arguments.is_empty()
        {
            Ok(format!(
                "native_primitive_display_dictionary({})",
                quoted(identity)
            ))
        } else {
            Err("Rust backend: invalid primitive display evidence".into())
        },
    )
}

pub(super) fn valid_display_dispatch(
    selected: &CoreCallEvidence,
    identity: &str,
    method: &str,
) -> bool {
    let CoreInstanceEvidence::Standard {
        identity: dictionary,
        ..
    } = &selected.evidence
    else {
        return true;
    };
    if !matches!(
        dictionary.as_str(),
        "std/list::Show" | "std/list::Debug" | "std/tuple::Show" | "std/tuple::Debug"
    ) && primitive_display(selected).is_none()
    {
        return true;
    }
    let expected = selected.constraint.name.as_str();
    identity == format!("std/prelude::{expected}") && method == expected.to_ascii_lowercase()
}
