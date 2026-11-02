use super::*;

pub(super) fn http_multipart_interface() -> ModuleInterface {
    let module = "std/http/multipart";
    let multipart = |environment: InterfaceType, failure: InterfaceType| {
        named_with("Multipart", vec![environment, failure])
    };
    let body = |environment: InterfaceType, failure: InterfaceType| {
        external_type(
            "Body",
            "std/http::Body",
            "std/http",
            "Body",
            vec![environment, failure],
        )
    };
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let maybe_string = named_with("Maybe", vec![named("String")]);
    let build_error = named("MultipartBuildError");
    let build_result =
        |success: InterfaceType| named_with("Either", vec![build_error.clone(), success]);
    standard_interface(
        module,
        vec![
            type_export(module, "Multipart", 2, "opaque-type"),
            opaque_adt_type_export(module, "MultipartBuildError", []),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartFieldName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartFileName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "MultipartBuildError",
                "InvalidMultipartMimeType",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "empty",
                ["R", "E"],
                Vec::new(),
                vec![named("Unit")],
                multipart(named("R"), named("E")),
            ),
            function_export(
                module,
                "appendText",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    named("String"),
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "appendBytes",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    maybe_string.clone(),
                    maybe_string.clone(),
                    bytes,
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "appendBody",
                ["R", "E"],
                Vec::new(),
                vec![
                    named("String"),
                    maybe_string.clone(),
                    maybe_string,
                    body(named("R"), named("E")),
                    multipart(named("R"), named("E")),
                ],
                build_result(multipart(named("R"), named("E"))),
            ),
            function_export(
                module,
                "contentType",
                ["R", "E"],
                Vec::new(),
                vec![multipart(named("R"), named("E"))],
                named("String"),
            ),
            function_export(
                module,
                "body",
                ["R", "E"],
                Vec::new(),
                vec![multipart(named("R"), named("E"))],
                body(named("R"), named("E")),
            ),
        ],
    )
}
