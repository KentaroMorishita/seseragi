#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeSignalOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const MODULE: &str = "@seseragi/runtime/signal";

macro_rules! operation {
    ($name:literal, $feature:literal) => {
        RuntimeSignalOperation {
            canonical: concat!("std/signal::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_signal_", $name),
            module: MODULE,
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeSignalOperation] = &[
    operation!("make", "signal.make"),
    operation!("read", "signal.read"),
    operation!("set", "signal.set"),
    operation!("update", "signal.update"),
    operation!("planSet", "signal.plan-set"),
    operation!("planUpdate", "signal.plan-update"),
    operation!("transaction", "signal.transaction"),
    operation!("map", "signal.map"),
    operation!("combine", "signal.combine"),
    operation!("constant", "signal.constant"),
    operation!("switchMap", "signal.switch-map"),
    operation!("subscribe", "signal.subscribe"),
    operation!("unsubscribe", "signal.unsubscribe"),
];

pub(crate) fn runtime_signal_operation(canonical: &str) -> Option<RuntimeSignalOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_signal_operation_for_feature(
    feature: &str,
) -> Option<RuntimeSignalOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_signal_calls_by_canonical_language_identity() {
        let operation = runtime_signal_operation("std/signal::transaction").unwrap();

        assert_eq!(operation.runtime_feature, "signal.transaction");
        assert_eq!(operation.module, "@seseragi/runtime/signal");
        assert_eq!(
            runtime_signal_operation_for_feature(operation.runtime_feature),
            Some(operation)
        );
    }
}
