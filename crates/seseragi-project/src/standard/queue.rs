use super::*;

pub(super) fn queue_interface() -> ModuleInterface {
    let module = "std/queue";
    let queue = |value: InterfaceType| named_with("Queue", vec![value]);
    standard_interface(
        module,
        vec![
            type_export(module, "Queue", 1, "opaque-type"),
            opaque_adt_type_export(module, "QueueCreateError", []),
            constructor_export(
                module,
                "QueueCreateError",
                "NonPositiveCapacity",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "QueueClosed", []),
            constructor_export(module, "QueueClosed", "QueueClosed", [], None),
            effect_function_export(
                module,
                "bounded",
                ["A"],
                Vec::new(),
                vec![named("Int")],
                effect(record([]), named("QueueCreateError"), queue(named("A"))),
            ),
            effect_function_export(
                module,
                "unbounded",
                ["A"],
                Vec::new(),
                vec![named("Unit")],
                task(queue(named("A"))),
            ),
            effect_function_export(
                module,
                "offer",
                ["A"],
                Vec::new(),
                vec![named("A"), queue(named("A"))],
                effect(record([]), named("QueueClosed"), named("Unit")),
            ),
            effect_function_export(
                module,
                "take",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                effect(record([]), named("QueueClosed"), named("A")),
            ),
            effect_function_export(
                module,
                "tryOffer",
                ["A"],
                Vec::new(),
                vec![named("A"), queue(named("A"))],
                task(named_with(
                    "Either",
                    vec![named("QueueClosed"), named("Bool")],
                )),
            ),
            effect_function_export(
                module,
                "tryTake",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named_with(
                    "Either",
                    vec![named("QueueClosed"), named_with("Maybe", vec![named("A")])],
                )),
            ),
            effect_function_export(
                module,
                "size",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named("Int")),
            ),
            effect_function_export(
                module,
                "close",
                ["A"],
                Vec::new(),
                vec![queue(named("A"))],
                task(named("Unit")),
            ),
        ],
    )
}
