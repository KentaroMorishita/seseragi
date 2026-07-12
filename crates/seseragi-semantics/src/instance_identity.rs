pub(crate) fn canonical_instance_identity(trait_name: &str, type_identity: &str) -> String {
    format!("{trait_name}<{type_identity}>")
}
