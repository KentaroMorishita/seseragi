use super::*;

pub(super) fn http_server_interface() -> ModuleInterface {
    let module = "std/http/server";
    let request = named("HttpServerRequest");
    let response = named("HttpServerResponse");
    let handler_request = external_type(
        "HttpServerRequest",
        "std/http/server::HttpServerRequest",
        module,
        "HttpServerRequest",
        Vec::new(),
    );
    let handler_response = external_type(
        "HttpServerResponse",
        "std/http/server::HttpServerResponse",
        module,
        "HttpServerResponse",
        Vec::new(),
    );
    let header = named("HttpHeader");
    let headers = named_with("Array", vec![header.clone()]);
    let handler = |environment: InterfaceType, failure: InterfaceType| {
        function_type(
            vec![handler_request.clone()],
            effect(environment, failure, handler_response.clone()),
        )
    };
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let exports = vec![
        type_export(module, "HttpServer", 0, "opaque-type"),
        type_export(module, "HttpServerRequest", 0, "opaque-type"),
        type_export(module, "HttpServerResponse", 0, "opaque-type"),
        type_export(module, "HttpServerHandle", 0, "opaque-type"),
        type_export(module, "HttpServerError", 0, "opaque-type"),
        alias_type_export(
            module,
            "HttpHeader",
            [],
            record([
                required("name", named("String")),
                required("value", named("String")),
            ]),
        ),
        alias_type_export(
            module,
            "Handler",
            ["R", "E"],
            handler(named("R"), named("E")),
        ),
        function_export(
            module,
            "requestMethod",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestUrl",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestPath",
            [],
            Vec::new(),
            vec![request.clone()],
            named("String"),
        ),
        function_export(
            module,
            "requestQuery",
            [],
            Vec::new(),
            vec![request.clone()],
            named_with("Maybe", vec![named("String")]),
        ),
        function_export(
            module,
            "requestHeaders",
            [],
            Vec::new(),
            vec![request.clone()],
            headers.clone(),
        ),
        function_export(
            module,
            "requestHeaderValues",
            [],
            Vec::new(),
            vec![named("String"), request.clone()],
            named_with("Array", vec![named("String")]),
        ),
        function_export(
            module,
            "requestBody",
            [],
            Vec::new(),
            vec![request.clone()],
            bytes.clone(),
        ),
        function_export(
            module,
            "header",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            header,
        ),
        function_export(
            module,
            "emptyResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone()],
            response.clone(),
        ),
        function_export(
            module,
            "bytesResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone(), bytes.clone()],
            response.clone(),
        ),
        effect_function_export(
            module,
            "streamResponse",
            ["R"],
            Vec::new(),
            vec![
                named("Int"),
                headers.clone(),
                external_type(
                    "Stream",
                    "std/stream::Stream",
                    "std/stream",
                    "Stream",
                    vec![named("R"), named("Never"), bytes],
                ),
            ],
            effect(named("R"), named("Never"), response.clone()),
        ),
        function_export(
            module,
            "textResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers.clone(), named("String")],
            response.clone(),
        ),
        function_export(
            module,
            "jsonResponse",
            [],
            Vec::new(),
            vec![named("Int"), headers, named("String")],
            response.clone(),
        ),
        function_export(
            module,
            "pureHandler",
            [],
            Vec::new(),
            vec![function_type(vec![request.clone()], response.clone())],
            handler(record([]), named("Never")),
        ),
        function_export(
            module,
            "recoverHandler",
            ["R", "E"],
            Vec::new(),
            vec![
                function_type(vec![named("E")], response.clone()),
                handler(named("R"), named("E")),
            ],
            handler(named("R"), named("Never")),
        ),
        function_export(
            module,
            "errorMessage",
            [],
            Vec::new(),
            vec![named("HttpServerError")],
            named("String"),
        ),
        function_export(
            module,
            "listen",
            ["R"],
            Vec::new(),
            vec![record([
                optional("hostname", named("String")),
                required("port", named("Int")),
                required("handler", handler(named("R"), named("Never"))),
            ])],
            effect(
                requirement_merge(vec![
                    named("R"),
                    record([required("httpServer", named("HttpServer"))]),
                ]),
                named("HttpServerError"),
                named("HttpServerHandle"),
            ),
        ),
        function_export(
            module,
            "serveOnce",
            ["R"],
            Vec::new(),
            vec![record([
                optional("hostname", named("String")),
                required("port", named("Int")),
                required("handler", handler(named("R"), named("Never"))),
            ])],
            effect(
                requirement_merge(vec![
                    named("R"),
                    record([required("httpServer", named("HttpServer"))]),
                ]),
                named("HttpServerError"),
                named("Unit"),
            ),
        ),
        function_export(
            module,
            "close",
            [],
            Vec::new(),
            vec![named("HttpServerHandle")],
            effect(
                record([required("httpServer", named("HttpServer"))]),
                named("Never"),
                named("Unit"),
            ),
        ),
    ];
    standard_interface(module, exports)
}
