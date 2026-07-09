#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnownEffectOperation {
    pub(crate) surface_name: &'static str,
    pub(crate) semantic_name: &'static str,
    pub(crate) requirement_field: &'static str,
    pub(crate) requirement_type: &'static str,
    pub(crate) failure_type: &'static str,
    pub(crate) success_type: &'static str,
}

const KNOWN_EFFECT_OPERATIONS: &[KnownEffectOperation] = &[
    KnownEffectOperation {
        surface_name: "print",
        semantic_name: "std/prelude::print",
        requirement_field: "console",
        requirement_type: "Console",
        failure_type: "ConsoleError",
        success_type: "Unit",
    },
    KnownEffectOperation {
        surface_name: "println",
        semantic_name: "std/prelude::println",
        requirement_field: "console",
        requirement_type: "Console",
        failure_type: "ConsoleError",
        success_type: "Unit",
    },
];

pub(crate) fn known_effect_operation_by_surface(
    surface_name: &str,
) -> Option<KnownEffectOperation> {
    KNOWN_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.surface_name == surface_name)
}

pub(crate) fn known_effect_operation_by_semantic(
    semantic_name: &str,
) -> Option<KnownEffectOperation> {
    KNOWN_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.semantic_name == semantic_name)
}

pub(crate) fn semantic_effect_operation_name(surface_name: &str) -> String {
    known_effect_operation_by_surface(surface_name)
        .map(|operation| operation.semantic_name.to_owned())
        .unwrap_or_else(|| surface_name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_println_by_surface_name() {
        let operation = known_effect_operation_by_surface("println").unwrap();
        assert_eq!(operation.semantic_name, "std/prelude::println");
        assert_eq!(operation.requirement_field, "console");
        assert_eq!(operation.requirement_type, "Console");
        assert_eq!(operation.failure_type, "ConsoleError");
        assert_eq!(operation.success_type, "Unit");
    }

    #[test]
    fn resolves_println_by_semantic_name() {
        let operation = known_effect_operation_by_semantic("std/prelude::println").unwrap();
        assert_eq!(operation.surface_name, "println");
    }

    #[test]
    fn resolves_print_by_surface_name() {
        let operation = known_effect_operation_by_surface("print").unwrap();
        assert_eq!(operation.semantic_name, "std/prelude::print");
        assert_eq!(operation.success_type, "Unit");
    }

    #[test]
    fn leaves_unknown_surface_operation_unmapped() {
        assert!(known_effect_operation_by_surface("readLine").is_none());
        assert_eq!(semantic_effect_operation_name("readLine"), "readLine");
    }
}
