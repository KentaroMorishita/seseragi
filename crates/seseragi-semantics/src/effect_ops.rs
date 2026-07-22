#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KnownEffectOperation {
    pub surface_name: &'static str,
    pub semantic_name: &'static str,
    pub requirement: Option<(&'static str, &'static str)>,
    pub failure_type: &'static str,
    pub success_type: &'static str,
    pub success_type_arguments: &'static [&'static str],
}

const KNOWN_EFFECT_OPERATIONS: &[KnownEffectOperation] = &[
    KnownEffectOperation {
        surface_name: "readLine",
        semantic_name: "std/prelude::readLine",
        requirement: Some(("stdin", "Stdin")),
        failure_type: "StdinError",
        success_type: "Maybe",
        success_type_arguments: &["String"],
    },
    KnownEffectOperation {
        surface_name: "print",
        semantic_name: "std/prelude::print",
        requirement: Some(("console", "Console")),
        failure_type: "ConsoleError",
        success_type: "Unit",
        success_type_arguments: &[],
    },
    KnownEffectOperation {
        surface_name: "println",
        semantic_name: "std/prelude::println",
        requirement: Some(("console", "Console")),
        failure_type: "ConsoleError",
        success_type: "Unit",
        success_type_arguments: &[],
    },
    KnownEffectOperation {
        surface_name: "succeed",
        semantic_name: "std/effect::succeed",
        requirement: None,
        failure_type: "Never",
        success_type: "Unit",
        success_type_arguments: &[],
    },
    KnownEffectOperation {
        surface_name: "fail",
        semantic_name: "std/effect::fail",
        requirement: None,
        failure_type: "Never",
        success_type: "Never",
        success_type_arguments: &[],
    },
    KnownEffectOperation {
        surface_name: "mapError",
        semantic_name: "std/effect::mapError",
        requirement: None,
        failure_type: "Never",
        success_type: "Never",
        success_type_arguments: &[],
    },
    KnownEffectOperation {
        surface_name: "fromEither",
        semantic_name: "std/effect::fromEither",
        requirement: None,
        failure_type: "Never",
        success_type: "Never",
        success_type_arguments: &[],
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

pub(crate) fn known_effect_operations() -> impl Iterator<Item = KnownEffectOperation> {
    KNOWN_EFFECT_OPERATIONS.iter().copied()
}

pub fn known_effect_operation_by_semantic(semantic_name: &str) -> Option<KnownEffectOperation> {
    KNOWN_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.semantic_name == semantic_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_println_by_surface_name() {
        let operation = known_effect_operation_by_surface("println").unwrap();
        assert_eq!(operation.semantic_name, "std/prelude::println");
        assert_eq!(operation.requirement, Some(("console", "Console")));
        assert_eq!(operation.failure_type, "ConsoleError");
        assert_eq!(operation.success_type, "Unit");
        assert!(operation.success_type_arguments.is_empty());
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
    fn resolves_read_line_with_maybe_string_success_type() {
        let operation = known_effect_operation_by_surface("readLine").unwrap();

        assert_eq!(operation.semantic_name, "std/prelude::readLine");
        assert_eq!(operation.requirement, Some(("stdin", "Stdin")));
        assert_eq!(operation.failure_type, "StdinError");
        assert_eq!(operation.success_type, "Maybe");
        assert_eq!(operation.success_type_arguments, ["String"]);
    }

    #[test]
    fn resolves_succeed_without_environment_or_failure() {
        let operation = known_effect_operation_by_surface("succeed").unwrap();

        assert_eq!(operation.semantic_name, "std/effect::succeed");
        assert_eq!(operation.requirement, None);
        assert_eq!(operation.failure_type, "Never");
        assert_eq!(operation.success_type, "Unit");
    }

    #[test]
    fn resolves_fail_as_a_generic_failure_operation() {
        let operation = known_effect_operation_by_surface("fail").unwrap();

        assert_eq!(operation.semantic_name, "std/effect::fail");
        assert_eq!(operation.requirement, None);
        assert_eq!(operation.success_type, "Never");
    }

    #[test]
    fn resolves_map_error_as_a_generic_failure_transform() {
        let operation = known_effect_operation_by_surface("mapError").unwrap();

        assert_eq!(operation.semantic_name, "std/effect::mapError");
        assert_eq!(operation.requirement, None);
    }

    #[test]
    fn resolves_from_either_as_a_value_conversion() {
        let operation = known_effect_operation_by_surface("fromEither").unwrap();

        assert_eq!(operation.semantic_name, "std/effect::fromEither");
        assert_eq!(operation.requirement, None);
        assert_eq!(
            known_effect_operation_by_semantic("std/effect::fromEither"),
            Some(operation)
        );
    }

    #[test]
    fn leaves_unknown_surface_operation_unmapped() {
        assert!(known_effect_operation_by_surface("unregistered").is_none());
    }
}
