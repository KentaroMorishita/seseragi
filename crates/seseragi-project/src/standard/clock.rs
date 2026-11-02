use super::*;

pub(super) fn clock_interface() -> ModuleInterface {
    let module = "std/clock";
    standard_interface(
        module,
        vec![
            type_export(module, "Clock", 0, "opaque-type"),
            function_export(
                module,
                "now",
                [],
                Vec::new(),
                vec![named("Unit")],
                effect(
                    record([required("clock", named("Clock"))]),
                    named("Never"),
                    external_type(
                        "Instant",
                        "std/time::Instant",
                        "std/time",
                        "Instant",
                        Vec::new(),
                    ),
                ),
            ),
            effect_function_export(
                module,
                "sleep",
                [],
                Vec::new(),
                vec![external_type(
                    "Duration",
                    "std/time::Duration",
                    "std/time",
                    "Duration",
                    Vec::new(),
                )],
                effect(
                    record([required("clock", named("Clock"))]),
                    named("Never"),
                    named("Unit"),
                ),
            ),
        ],
    )
}
