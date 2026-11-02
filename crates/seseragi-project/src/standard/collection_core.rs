use super::*;

pub(super) fn collection_core_interface() -> ModuleInterface {
    let module = "std/collection";
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "SizeError", []),
            constructor_export(
                module,
                "SizeError",
                "NonPositiveSize",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "ReduceStep", ["A"]),
            constructor_export(module, "ReduceStep", "Next", ["A"], Some(named("A"))),
            constructor_export(module, "ReduceStep", "Done", ["A"], Some(named("A"))),
            function_export(
                module,
                "reduceUntil",
                ["C", "A", "B"],
                vec![collection_constraint(
                    "Iterable",
                    vec![named("C"), named("A")],
                )],
                vec![
                    named("B"),
                    function_type(
                        vec![named("B"), named("A")],
                        external_type(
                            "ReduceStep",
                            "std/collection::ReduceStep",
                            module,
                            "ReduceStep",
                            vec![named("B")],
                        ),
                    ),
                    named("C"),
                ],
                named("B"),
            ),
        ],
    )
}
