use super::*;

pub(super) fn deferred_interface() -> ModuleInterface {
    let module = "std/deferred";
    let deferred = |failure: InterfaceType, success: InterfaceType| {
        named_with("Deferred", vec![failure, success])
    };
    let handle = || deferred(named("E"), named("A"));
    standard_interface(
        module,
        vec![
            type_export(module, "Deferred", 2, "opaque-type"),
            effect_function_export(
                module,
                "make",
                ["E", "A"],
                Vec::new(),
                vec![named("Unit")],
                task(handle()),
            ),
            effect_function_export(
                module,
                "await",
                ["E", "A"],
                Vec::new(),
                vec![handle()],
                effect(record([]), named("E"), named("A")),
            ),
            effect_function_export(
                module,
                "poll",
                ["E", "A"],
                Vec::new(),
                vec![handle()],
                task(named_with(
                    "Maybe",
                    vec![named_with("Either", vec![named("E"), named("A")])],
                )),
            ),
            effect_function_export(
                module,
                "complete",
                ["E", "A"],
                Vec::new(),
                vec![named_with("Either", vec![named("E"), named("A")]), handle()],
                task(named("Bool")),
            ),
            effect_function_export(
                module,
                "succeed",
                ["E", "A"],
                Vec::new(),
                vec![named("A"), handle()],
                task(named("Bool")),
            ),
            effect_function_export(
                module,
                "fail",
                ["E", "A"],
                Vec::new(),
                vec![named("E"), handle()],
                task(named("Bool")),
            ),
        ],
    )
}
