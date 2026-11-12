use super::*;

pub(super) fn stream_interface() -> ModuleInterface {
    let module = "std/stream";
    let stream = |environment: InterfaceType, failure: InterfaceType, value: InterfaceType| {
        named_with("Stream", vec![environment, failure, value])
    };
    let source = |value: InterfaceType| stream(named("R"), named("E"), value);
    let buffer_capacity = named("BufferCapacity");
    standard_interface(
        module,
        vec![
            type_export(module, "Stream", 3, "opaque-type"),
            type_export(module, "BufferCapacity", 0, "opaque-type"),
            opaque_adt_type_export(module, "BufferCapacityError", []),
            constructor_export(
                module,
                "BufferCapacityError",
                "NonPositiveBufferCapacity",
                [],
                Some(named("Int")),
            ),
            function_export(
                module,
                "empty",
                ["R", "E", "A"],
                Vec::new(),
                Vec::new(),
                source(named("A")),
            ),
            function_export(
                module,
                "singleton",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("A")],
                source(named("A")),
            ),
            function_export(
                module,
                "fromArray",
                ["R", "E", "A"],
                Vec::new(),
                vec![named_with("Array", vec![named("A")])],
                source(named("A")),
            ),
            function_export(
                module,
                "fromIterable",
                ["C", "R", "E", "A"],
                vec![InterfaceConstraint {
                    name: "Iterable".to_owned(),
                    trait_identity: Some("std/prelude::Iterable".to_owned()),
                    arguments: vec![named("C"), named("A")],
                }],
                vec![named("C")],
                source(named("A")),
            ),
            function_export(
                module,
                "fromEffect",
                ["R", "E", "A"],
                Vec::new(),
                vec![effect(named("R"), named("E"), named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "unfold",
                ["S", "R", "E", "A"],
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
                source(named("A")),
            ),
            function_export(
                module,
                "map",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("B")),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "filter",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named("Bool")),
                    source(named("A")),
                ],
                source(named("A")),
            ),
            function_export(
                module,
                "filterMap",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], named_with("Maybe", vec![named("B")])),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "mapError",
                ["R", "E", "F", "A"],
                Vec::new(),
                vec![
                    function_type(vec![named("E")], named("F")),
                    source(named("A")),
                ],
                stream(named("R"), named("F"), named("A")),
            ),
            function_export(
                module,
                "flatMap",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    function_type(vec![named("A")], source(named("B"))),
                    source(named("A")),
                ],
                source(named("B")),
            ),
            function_export(
                module,
                "take",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("Int"), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "drop",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("Int"), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "chunk",
                ["R", "E", "A"],
                Vec::new(),
                vec![named("Int"), source(named("A"))],
                named_with(
                    "Either",
                    vec![
                        external_type(
                            "SizeError",
                            "std/collection::SizeError",
                            "std/collection",
                            "SizeError",
                            Vec::new(),
                        ),
                        source(named_with("Array", vec![named("A")])),
                    ],
                ),
            ),
            function_export(
                module,
                "concat",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A")), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "zip",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![source(named("B")), source(named("A"))],
                source(InterfaceType::Tuple {
                    elements: vec![named("A"), named("B")],
                }),
            ),
            function_export(
                module,
                "merge",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A")), source(named("A"))],
                source(named("A")),
            ),
            function_export(
                module,
                "bufferCapacity",
                [],
                Vec::new(),
                vec![named("Int")],
                named_with(
                    "Either",
                    vec![named("BufferCapacityError"), buffer_capacity.clone()],
                ),
            ),
            function_export(
                module,
                "buffer",
                ["R", "E", "A"],
                Vec::new(),
                vec![buffer_capacity, source(named("A"))],
                source(named("A")),
            ),
            effect_function_export(
                module,
                "runCollect",
                ["R", "E", "A"],
                Vec::new(),
                vec![source(named("A"))],
                effect(
                    named("R"),
                    named("E"),
                    named_with("Array", vec![named("A")]),
                ),
            ),
            effect_function_export(
                module,
                "runFold",
                ["R", "E", "A", "B"],
                Vec::new(),
                vec![
                    named("B"),
                    function_type(vec![named("B"), named("A")], named("B")),
                    source(named("A")),
                ],
                effect(named("R"), named("E"), named("B")),
            ),
            effect_function_export(
                module,
                "runForEach",
                ["R", "E", "A"],
                Vec::new(),
                vec![
                    function_type(
                        vec![named("A")],
                        effect(named("R"), named("E"), named("Unit")),
                    ),
                    source(named("A")),
                ],
                effect(named("R"), named("E"), named("Unit")),
            ),
        ],
    )
}
