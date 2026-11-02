use super::*;

pub(super) fn grapheme_interface() -> ModuleInterface {
    let module = "std/text/grapheme";
    let error = external_type(
        "GraphemeSliceError",
        "std/text/grapheme::GraphemeSliceError",
        module,
        "GraphemeSliceError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "GraphemeSliceError", []),
            constructor_export(
                module,
                "GraphemeSliceError",
                "InvalidGraphemeRange",
                [],
                Some(text_range_payload()),
            ),
            function_export(
                module,
                "length",
                [],
                Vec::new(),
                vec![named("String")],
                named("Int"),
            ),
            function_export(
                module,
                "clusters",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Array", vec![named("String")]),
            ),
            function_export(
                module,
                "byteBoundaries",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Array", vec![named("Int")]),
            ),
            function_export(
                module,
                "at",
                [],
                Vec::new(),
                vec![named("Int"), named("String")],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "slice",
                [],
                Vec::new(),
                vec![named("Int"), named("Int"), named("String")],
                named_with("Either", vec![error, named("String")]),
            ),
        ],
    )
}
