#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeEffectOperation {
    pub(crate) core_name: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_EFFECT_OPERATIONS: &[RuntimeEffectOperation] = &[
    RuntimeEffectOperation {
        core_name: "stdin.readLine",
        runtime_feature: "effect.stdin.readLine",
        local_name: "_ssrg_stdin_readLine",
        module: "@seseragi/runtime/stdin",
        export_name: "readLine",
        source_map_name: "readLine",
    },
    RuntimeEffectOperation {
        core_name: "console.print",
        runtime_feature: "effect.console.print",
        local_name: "_ssrg_console_print",
        module: "@seseragi/runtime/console",
        export_name: "print",
        source_map_name: "print",
    },
    RuntimeEffectOperation {
        core_name: "console.println",
        runtime_feature: "effect.console.println",
        local_name: "_ssrg_console_println",
        module: "@seseragi/runtime/console",
        export_name: "println",
        source_map_name: "println",
    },
    RuntimeEffectOperation {
        core_name: "effect.succeed",
        runtime_feature: "effect.core.succeed",
        local_name: "_ssrg_effect_succeed",
        module: "@seseragi/runtime/effect",
        export_name: "succeed",
        source_map_name: "succeed",
    },
    RuntimeEffectOperation {
        core_name: "effect.fail",
        runtime_feature: "effect.core.fail",
        local_name: "_ssrg_effect_fail",
        module: "@seseragi/runtime/effect",
        export_name: "fail",
        source_map_name: "fail",
    },
    RuntimeEffectOperation {
        core_name: "effect.mapError",
        runtime_feature: "effect.core.mapError",
        local_name: "_ssrg_effect_mapError",
        module: "@seseragi/runtime/effect",
        export_name: "mapError",
        source_map_name: "mapError",
    },
    RuntimeEffectOperation {
        core_name: "effect.flatMap",
        runtime_feature: "effect.core.flatMap",
        local_name: "_ssrg_effect_flatMap",
        module: "@seseragi/runtime/effect",
        export_name: "flatMap",
        source_map_name: "flatMap",
    },
];

pub(crate) fn runtime_effect_operation(core_name: &str) -> Option<RuntimeEffectOperation> {
    RUNTIME_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.core_name == core_name)
}

pub(crate) fn runtime_effect_operation_for_feature(
    feature: &str,
) -> Option<RuntimeEffectOperation> {
    RUNTIME_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_console_println_runtime_abi() {
        let operation = runtime_effect_operation("console.println").unwrap();

        assert_eq!(operation.runtime_feature, "effect.console.println");
        assert_eq!(operation.module, "@seseragi/runtime/console");
        assert_eq!(operation.export_name, "println");
    }

    #[test]
    fn rejects_unknown_core_effect_operation() {
        assert!(runtime_effect_operation("stdin.readChunk").is_none());
    }

    #[test]
    fn resolves_console_print_runtime_abi() {
        let operation = runtime_effect_operation("console.print").unwrap();

        assert_eq!(operation.runtime_feature, "effect.console.print");
        assert_eq!(operation.export_name, "print");
    }

    #[test]
    fn resolves_cold_stdin_read_line_runtime_abi() {
        let operation = runtime_effect_operation("stdin.readLine").unwrap();

        assert_eq!(operation.runtime_feature, "effect.stdin.readLine");
        assert_eq!(operation.module, "@seseragi/runtime/stdin");
    }

    #[test]
    fn resolves_effect_composition_runtime_abi() {
        let operation = runtime_effect_operation("effect.flatMap").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.flatMap");
        assert_eq!(operation.export_name, "flatMap");
    }

    #[test]
    fn resolves_typed_failure_runtime_abi() {
        let operation = runtime_effect_operation("effect.fail").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.fail");
        assert_eq!(operation.export_name, "fail");
    }

    #[test]
    fn resolves_failure_mapping_runtime_abi() {
        let operation = runtime_effect_operation("effect.mapError").unwrap();

        assert_eq!(operation.runtime_feature, "effect.core.mapError");
        assert_eq!(operation.export_name, "mapError");
    }
}
