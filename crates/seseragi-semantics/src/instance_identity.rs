#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CanonicalInstanceHeadKey {
    trait_identity: String,
    argument_identities: Vec<String>,
}

pub(crate) fn canonical_instance_head_key(
    trait_identity: &str,
    argument_identities: &[String],
) -> CanonicalInstanceHeadKey {
    let trait_identity = if !trait_identity.contains("::")
        && crate::prelude::trait_by_name(trait_identity).is_some()
    {
        format!("std/prelude::{trait_identity}")
    } else {
        trait_identity.to_owned()
    };
    CanonicalInstanceHeadKey {
        trait_identity,
        argument_identities: argument_identities.to_vec(),
    }
}

pub(crate) fn canonical_instance_identity(trait_name: &str, type_identity: &str) -> String {
    canonical_instance_head_identity(trait_name, &[type_identity.to_owned()])
}

pub(crate) fn canonical_instance_head_identity(
    trait_identity: &str,
    arguments: &[String],
) -> String {
    format!("{trait_identity}<{}>", arguments.join(","))
}
