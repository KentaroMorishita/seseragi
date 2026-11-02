use super::*;

pub(super) fn iterator_interface() -> ModuleInterface {
    let module = "std/iterator";
    let iterator = external_type(
        "Iterator",
        "std/prelude::Iterator",
        "std/prelude",
        "Iterator",
        vec![named("A")],
    );
    standard_interface(
        module,
        vec![
            canonical_type_export(
                module,
                "Iterator",
                "std/prelude::Iterator",
                1,
                "opaque-type",
            ),
            function_export(
                module,
                "unfold",
                ["S", "A"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("S")],
                        named_with(
                            "Maybe",
                            vec![InterfaceType::Tuple {
                                elements: vec![named("A"), named("S")],
                            }],
                        ),
                    ),
                    named("S"),
                ],
                iterator.clone(),
            ),
            function_export(
                module,
                "next",
                ["A"],
                Vec::new(),
                vec![iterator.clone()],
                named_with(
                    "Maybe",
                    vec![InterfaceType::Tuple {
                        elements: vec![named("A"), iterator],
                    }],
                ),
            ),
        ],
    )
}
