use super::*;

pub(super) fn path_interface() -> ModuleInterface {
    let module = "std/path";
    let path = named("Path");
    let path_result = named_with("Either", vec![named("PathError"), path.clone()]);
    standard_interface(
        module,
        vec![
            type_export(module, "Path", 0, "opaque-type"),
            opaque_adt_type_export(module, "PathError", []),
            constructor_export(module, "PathError", "EmptyPath", [], None),
            constructor_export(
                module,
                "PathError",
                "PathContainsNul",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "PathError",
                "PathContainsBackslash",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(module, "PathError", "InvalidDriveRoot", [], None),
            constructor_export(module, "PathError", "InvalidUncRoot", [], None),
            constructor_export(
                module,
                "PathError",
                "InvalidPathSegment",
                [],
                Some(named("String")),
            ),
            constructor_export(module, "PathError", "AbsoluteChildPath", [], None),
            function_export(
                module,
                "parse",
                [],
                Vec::new(),
                vec![named("String")],
                path_result.clone(),
            ),
            function_export(
                module,
                "render",
                [],
                Vec::new(),
                vec![path.clone()],
                named("String"),
            ),
            function_export(
                module,
                "current",
                [],
                Vec::new(),
                vec![named("Unit")],
                path.clone(),
            ),
            function_export(
                module,
                "isAbsolute",
                [],
                Vec::new(),
                vec![path.clone()],
                named("Bool"),
            ),
            function_export(
                module,
                "normalize",
                [],
                Vec::new(),
                vec![path.clone()],
                path.clone(),
            ),
            function_export(
                module,
                "join",
                [],
                Vec::new(),
                vec![path.clone(), path.clone()],
                path_result.clone(),
            ),
            function_export(
                module,
                "child",
                [],
                Vec::new(),
                vec![named("String"), path.clone()],
                path_result,
            ),
            function_export(
                module,
                "parent",
                [],
                Vec::new(),
                vec![path.clone()],
                named_with("Maybe", vec![path.clone()]),
            ),
            function_export(
                module,
                "fileName",
                [],
                Vec::new(),
                vec![path.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "extension",
                [],
                Vec::new(),
                vec![path],
                named_with("Maybe", vec![named("String")]),
            ),
        ],
    )
}
