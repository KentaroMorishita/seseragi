use super::*;

pub(super) fn semaphore_interface() -> ModuleInterface {
    let module = "std/semaphore";
    standard_interface(
        module,
        vec![
            type_export(module, "Semaphore", 0, "opaque-type"),
            type_export(module, "Permit", 0, "opaque-type"),
            opaque_adt_type_export(module, "SemaphoreCreateError", []),
            constructor_export(
                module,
                "SemaphoreCreateError",
                "NonPositivePermits",
                [],
                Some(named("Int")),
            ),
            effect_function_export(
                module,
                "make",
                [],
                Vec::new(),
                vec![named("Int")],
                effect(
                    record([]),
                    named("SemaphoreCreateError"),
                    named("Semaphore"),
                ),
            ),
            effect_function_export(
                module,
                "acquire",
                [],
                Vec::new(),
                vec![named("Semaphore")],
                task(named("Permit")),
            ),
            effect_function_export(
                module,
                "release",
                [],
                Vec::new(),
                vec![named("Permit")],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "withPermit",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    named("Semaphore"),
                    effect(named("R"), named("E"), named("A")),
                ],
                effect(named("R"), named("E"), named("A")),
            ),
            effect_function_export(
                module,
                "available",
                [],
                Vec::new(),
                vec![named("Semaphore")],
                task(named("Int")),
            ),
        ],
    )
}
