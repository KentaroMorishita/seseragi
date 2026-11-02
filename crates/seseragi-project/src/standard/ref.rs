use super::*;

pub(super) fn ref_interface() -> ModuleInterface {
    let module = "std/ref";
    let reference = |value: InterfaceType| named_with("Ref", vec![value]);
    standard_interface(
        module,
        vec![
            type_export(module, "Ref", 1, "opaque-type"),
            effect_function_export(
                module,
                "make",
                ["A"],
                Vec::new(),
                vec![named("A")],
                task(reference(named("A"))),
            ),
            effect_function_export(
                module,
                "get",
                ["A"],
                Vec::new(),
                vec![reference(named("A"))],
                task(named("A")),
            ),
            effect_function_export(
                module,
                "set",
                ["A"],
                Vec::new(),
                vec![named("A"), reference(named("A"))],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "update",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("A")),
                    reference(named("A")),
                ],
                task(named("Unit")),
            ),
            effect_function_export(
                module,
                "modify",
                ["A", "B"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("A")],
                        InterfaceType::Tuple {
                            elements: vec![named("B"), named("A")],
                        },
                    ),
                    reference(named("A")),
                ],
                task(named("B")),
            ),
        ],
    )
}
