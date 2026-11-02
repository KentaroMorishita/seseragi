use super::*;

pub(super) fn maybe_interface() -> ModuleInterface {
    let module = "std/maybe";
    let maybe = named_with("Maybe", vec![named("A")]);
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "withDefault",
                ["A"],
                vec![],
                vec![named("A"), maybe.clone()],
                named("A"),
            ),
            function_export(
                module,
                "orElse",
                ["A"],
                vec![],
                vec![maybe.clone(), maybe.clone()],
                maybe,
            ),
            sum_traversal_export(module, true, false),
            sum_traversal_export(module, false, false),
        ],
    )
}
