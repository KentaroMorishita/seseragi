pub(crate) fn is_contextual_declaration_start(raw: &str) -> bool {
    matches!(
        raw,
        "import"
            | "newtype"
            | "alias"
            | "type"
            | "struct"
            | "opaque"
            | "operator"
            | "instance"
            | "trait"
    )
}
