pub(crate) fn canonical_instance_identity(trait_name: &str, type_identity: &str) -> String {
    canonical_instance_head_identity(trait_name, &[type_identity.to_owned()])
}

pub(crate) fn canonical_instance_head_identity(
    trait_identity: &str,
    arguments: &[String],
) -> String {
    format!("{trait_identity}<{}>", arguments.join(","))
}
