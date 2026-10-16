use super::CoreType;

pub(super) fn primitive(ty: &CoreType, primitive: &str) -> bool {
    match ty {
        CoreType::Named { name, arguments } => name == primitive && arguments.is_empty(),
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => canonical == &format!("std/prelude::{primitive}") && arguments.is_empty(),
        _ => false,
    }
}
