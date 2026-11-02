use super::*;

pub(super) fn either_interface() -> ModuleInterface {
    let module = "std/either";
    let either = |error, value| named_with("Either", vec![named(error), named(value)]);
    let callback = |from, to| function_type(vec![named(from)], named(to));
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "mapLeft",
                ["E", "F", "A"],
                vec![],
                vec![callback("E", "F"), either("E", "A")],
                either("F", "A"),
            ),
            function_export(
                module,
                "mapRight",
                ["E", "A", "B"],
                vec![],
                vec![callback("A", "B"), either("E", "A")],
                either("E", "B"),
            ),
            function_export(
                module,
                "bimap",
                ["E", "F", "A", "B"],
                vec![],
                vec![callback("E", "F"), callback("A", "B"), either("E", "A")],
                either("F", "B"),
            ),
            function_export(
                module,
                "fold",
                ["E", "A", "B"],
                vec![],
                vec![callback("E", "B"), callback("A", "B"), either("E", "A")],
                named("B"),
            ),
            function_export(
                module,
                "swap",
                ["E", "A"],
                vec![],
                vec![either("E", "A")],
                either("A", "E"),
            ),
            sum_traversal_export(module, true, true),
            sum_traversal_export(module, false, true),
        ],
    )
}
