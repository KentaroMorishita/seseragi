use super::*;

pub(super) fn char_interface() -> ModuleInterface {
    let module = "std/char";
    standard_interface(
        module,
        vec![
            function_export(
                module,
                "codePoint",
                [],
                Vec::new(),
                vec![named("Char")],
                named("Int"),
            ),
            function_export(
                module,
                "fromCodePoint",
                [],
                Vec::new(),
                vec![named("Int")],
                named_with("Maybe", vec![named("Char")]),
            ),
            function_export(
                module,
                "toString",
                [],
                Vec::new(),
                vec![named("Char")],
                named("String"),
            ),
        ],
    )
}
