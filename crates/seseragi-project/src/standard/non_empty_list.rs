use super::*;

pub(super) fn non_empty_list_interface() -> ModuleInterface {
    let module = "std/non-empty-list";
    let list = || named_with("List", vec![named("A")]);
    let non_empty = || named_with("NonEmptyList", vec![named("A")]);
    let maybe_non_empty = || named_with("Maybe", vec![non_empty()]);
    standard_interface(
        module,
        vec![
            type_export(module, "NonEmptyList", 1, "opaque-type"),
            function_export(
                module,
                "singleton",
                ["A"],
                Vec::new(),
                vec![named("A")],
                non_empty(),
            ),
            function_export(
                module,
                "cons",
                ["A"],
                Vec::new(),
                vec![named("A"), list()],
                non_empty(),
            ),
            function_export(
                module,
                "fromList",
                ["A"],
                Vec::new(),
                vec![list()],
                maybe_non_empty(),
            ),
            function_export(
                module,
                "toList",
                ["A"],
                Vec::new(),
                vec![non_empty()],
                list(),
            ),
            function_export(
                module,
                "head",
                ["A"],
                Vec::new(),
                vec![non_empty()],
                named("A"),
            ),
            function_export(module, "tail", ["A"], Vec::new(), vec![non_empty()], list()),
            function_export(
                module,
                "reduce1",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("A"), named("A")], named("A")),
                    non_empty(),
                ],
                named("A"),
            ),
        ],
    )
}
