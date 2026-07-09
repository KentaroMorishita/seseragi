#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeEffectOperation {
    pub(crate) core_name: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
    pub(crate) await_result: bool,
}

const RUNTIME_EFFECT_OPERATIONS: &[RuntimeEffectOperation] = &[
    RuntimeEffectOperation {
        core_name: "stdin.readLine",
        runtime_feature: "effect.stdin.readLine",
        local_name: "_ssrg_stdin_readLine",
        module: "@seseragi/runtime/stdin",
        export_name: "readLine",
        source_map_name: "readLine",
        await_result: true,
    },
    RuntimeEffectOperation {
        core_name: "console.print",
        runtime_feature: "effect.console.print",
        local_name: "_ssrg_console_print",
        module: "@seseragi/runtime/console",
        export_name: "print",
        source_map_name: "print",
        await_result: false,
    },
    RuntimeEffectOperation {
        core_name: "console.println",
        runtime_feature: "effect.console.println",
        local_name: "_ssrg_console_println",
        module: "@seseragi/runtime/console",
        export_name: "println",
        source_map_name: "println",
        await_result: false,
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

pub(crate) fn runtime_effect_operation_by_local_name(
    local_name: &str,
) -> Option<RuntimeEffectOperation> {
    RUNTIME_EFFECT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.local_name == local_name)
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
    fn resolves_runtime_operation_by_generated_local_name() {
        let operation = runtime_effect_operation_by_local_name("_ssrg_console_println").unwrap();

        assert_eq!(operation.source_map_name, "println");
    }

    #[test]
    fn resolves_async_stdin_read_line_runtime_abi() {
        let operation = runtime_effect_operation("stdin.readLine").unwrap();

        assert_eq!(operation.runtime_feature, "effect.stdin.readLine");
        assert!(operation.await_result);
    }
}
