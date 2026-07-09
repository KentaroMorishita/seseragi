#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeEffectOperation {
    pub(crate) core_name: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_EFFECT_OPERATIONS: &[RuntimeEffectOperation] = &[RuntimeEffectOperation {
    core_name: "console.println",
    runtime_feature: "effect.console.println",
    local_name: "_ssrg_console_println",
    module: "@seseragi/runtime/console",
    export_name: "println",
    source_map_name: "println",
}];

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
        assert!(runtime_effect_operation("stdin.readLine").is_none());
    }

    #[test]
    fn resolves_runtime_operation_by_generated_local_name() {
        let operation = runtime_effect_operation_by_local_name("_ssrg_console_println").unwrap();

        assert_eq!(operation.source_map_name, "println");
    }
}
