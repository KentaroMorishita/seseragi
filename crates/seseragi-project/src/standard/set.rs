use super::*;

pub(super) fn set_interface() -> ModuleInterface {
    let module = "std/set";
    let set = |element: &str| named_with("Set", vec![named(element)]);
    let mut exports = vec![
        type_export(module, "Set", 1, "opaque-type"),
        function_export(
            module,
            "empty",
            ["A"],
            vec![],
            vec![named("Unit")],
            set("A"),
        ),
        function_export(
            module,
            "singleton",
            ["A"],
            key_constraints("A"),
            vec![named("A")],
            set("A"),
        ),
        function_export(
            module,
            "fromIterable",
            ["C", "A"],
            {
                let mut constraints = vec![collection_constraint(
                    "Iterable",
                    vec![named("C"), named("A")],
                )];
                constraints.extend(key_constraints("A"));
                constraints
            },
            vec![named("C")],
            set("A"),
        ),
        function_export(
            module,
            "contains",
            ["A"],
            key_constraints("A"),
            vec![named("A"), set("A")],
            named("Bool"),
        ),
        function_export(
            module,
            "filter",
            ["A"],
            vec![],
            vec![function_type(vec![named("A")], named("Bool")), set("A")],
            set("A"),
        ),
        function_export(
            module,
            "map",
            ["A", "B"],
            key_constraints("B"),
            vec![function_type(vec![named("A")], named("B")), set("A")],
            set("B"),
        ),
        function_export(
            module,
            "isSubsetOf",
            ["A"],
            key_constraints("A"),
            vec![set("A"), set("A")],
            named("Bool"),
        ),
    ];
    for name in ["insert", "remove"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            key_constraints("A"),
            vec![named("A"), set("A")],
            set("A"),
        ));
    }
    for name in ["union", "intersection", "difference"] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            key_constraints("A"),
            vec![set("A"), set("A")],
            set("A"),
        ));
    }
    for (name, result) in [("toArray", "Array"), ("toList", "List")] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            vec![],
            vec![set("A")],
            named_with(result, vec![named("A")]),
        ));
    }
    for (name, result) in [("size", "Int"), ("isEmpty", "Bool")] {
        exports.push(function_export(
            module,
            name,
            ["A"],
            vec![],
            vec![set("A")],
            named(result),
        ));
    }
    standard_interface(module, exports)
}
