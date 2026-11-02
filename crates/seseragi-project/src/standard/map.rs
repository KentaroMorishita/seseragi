use super::*;

pub(super) fn map_interface() -> ModuleInterface {
    let module = "std/map";
    let map = |key: &str, value: &str| named_with("Map", vec![named(key), named(value)]);
    let pair = || InterfaceType::Tuple {
        elements: vec![named("K"), named("V")],
    };
    let mut exports = vec![
        type_export(module, "Map", 2, "opaque-type"),
        function_export(
            module,
            "empty",
            ["K", "V"],
            vec![],
            vec![named("Unit")],
            map("K", "V"),
        ),
        function_export(
            module,
            "singleton",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), named("V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "fromEntries",
            ["C", "K", "V"],
            {
                let mut constraints =
                    vec![collection_constraint("Iterable", vec![named("C"), pair()])];
                constraints.extend(key_constraints("K"));
                constraints
            },
            vec![named("C")],
            map("K", "V"),
        ),
        function_export(
            module,
            "get",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            named_with("Maybe", vec![named("V")]),
        ),
        function_export(
            module,
            "containsKey",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            named("Bool"),
        ),
        function_export(
            module,
            "insert",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), named("V"), map("K", "V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "upsert",
            ["K", "V"],
            key_constraints("K"),
            vec![
                named("K"),
                function_type(vec![named_with("Maybe", vec![named("V")])], named("V")),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "remove",
            ["K", "V"],
            key_constraints("K"),
            vec![named("K"), map("K", "V")],
            map("K", "V"),
        ),
        function_export(
            module,
            "filter",
            ["K", "V"],
            vec![],
            vec![
                function_type(vec![named("K"), named("V")], named("Bool")),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "mapValues",
            ["K", "A", "B"],
            vec![],
            vec![function_type(vec![named("A")], named("B")), map("K", "A")],
            map("K", "B"),
        ),
        function_export(
            module,
            "mapKeysWith",
            ["K1", "K2", "V"],
            key_constraints("K2"),
            vec![
                function_type(vec![named("V"), named("V")], named("V")),
                function_type(vec![named("K1")], named("K2")),
                map("K1", "V"),
            ],
            map("K2", "V"),
        ),
        function_export(
            module,
            "mergeWith",
            ["K", "V"],
            key_constraints("K"),
            vec![
                function_type(vec![named("V"), named("V")], named("V")),
                map("K", "V"),
                map("K", "V"),
            ],
            map("K", "V"),
        ),
        function_export(
            module,
            "keys",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![named("K")]),
        ),
        function_export(
            module,
            "values",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![named("V")]),
        ),
        function_export(
            module,
            "entries",
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named_with("Array", vec![pair()]),
        ),
    ];
    for (name, result) in [("size", "Int"), ("isEmpty", "Bool")] {
        exports.push(function_export(
            module,
            name,
            ["K", "V"],
            vec![],
            vec![map("K", "V")],
            named(result),
        ));
    }
    standard_interface(module, exports)
}
