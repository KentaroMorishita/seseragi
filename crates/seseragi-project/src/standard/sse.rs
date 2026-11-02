use super::*;

pub(super) fn sse_interface() -> ModuleInterface {
    let module = "std/sse";
    let event = named("Event");
    let build_error = named("SseBuildError");
    let parse_error = named("SseParseError");
    let limit = named("DecodeLimit");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let request = external_type(
        "Request",
        "std/http::Request",
        "std/http",
        "Request",
        Vec::new(),
    );
    let http_event = external_type(
        "HttpEvent",
        "std/http::HttpEvent",
        "std/http",
        "HttpEvent",
        Vec::new(),
    );
    let header = external_type(
        "HttpHeader",
        "std/http/server::HttpHeader",
        "std/http/server",
        "HttpHeader",
        Vec::new(),
    );
    let response = external_type(
        "HttpServerResponse",
        "std/http/server::HttpServerResponse",
        "std/http/server",
        "HttpServerResponse",
        Vec::new(),
    );
    let stream = |environment: InterfaceType, failure: InterfaceType, value: InterfaceType| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment, failure, value],
        )
    };
    let build_result =
        |success: InterfaceType| named_with("Either", vec![build_error.clone(), success]);
    standard_interface(
        module,
        vec![
            type_export(module, "Event", 0, "opaque-type"),
            type_export(module, "DecodeLimit", 0, "opaque-type"),
            opaque_adt_type_export(module, "SseBuildError", []),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseEventName",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseEventId",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseRetryMillis",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseComment",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseBuildError",
                "InvalidSseDecodeLimit",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "SseParseError", []),
            constructor_export(
                module,
                "SseParseError",
                "SseUnexpectedStatus",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "SseParseError",
                "SseInvalidContentType",
                [],
                Some(named("String")),
            ),
            constructor_export(module, "SseParseError", "SseInvalidUtf8", [], None),
            constructor_export(
                module,
                "SseParseError",
                "SseEventTooLarge",
                [],
                Some(named("Int")),
            ),
            constructor_export(module, "SseParseError", "SseMalformedId", [], None),
            constructor_export(
                module,
                "SseParseError",
                "SseMalformedRetry",
                [],
                Some(named("String")),
            ),
            constructor_export(
                module,
                "SseParseError",
                "SseMalformedHttpEvents",
                [],
                Some(named("String")),
            ),
            function_export(
                module,
                "event",
                [],
                Vec::new(),
                vec![named("String")],
                event.clone(),
            ),
            function_export(
                module,
                "withEventName",
                [],
                Vec::new(),
                vec![named("String"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "withId",
                [],
                Vec::new(),
                vec![named("String"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "withRetryMillis",
                [],
                Vec::new(),
                vec![named("Int"), event.clone()],
                build_result(event.clone()),
            ),
            function_export(
                module,
                "eventData",
                [],
                Vec::new(),
                vec![event.clone()],
                named("String"),
            ),
            function_export(
                module,
                "eventName",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "eventId",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("String")]),
            ),
            function_export(
                module,
                "eventRetryMillis",
                [],
                Vec::new(),
                vec![event.clone()],
                named_with("Maybe", vec![named("Int")]),
            ),
            function_export(
                module,
                "encode",
                [],
                Vec::new(),
                vec![event.clone()],
                bytes.clone(),
            ),
            function_export(
                module,
                "keepAlive",
                [],
                Vec::new(),
                vec![named("String")],
                build_result(bytes),
            ),
            function_export(
                module,
                "decodeLimit",
                [],
                Vec::new(),
                vec![named("Int")],
                build_result(limit.clone()),
            ),
            function_export(
                module,
                "defaultDecodeLimit",
                [],
                Vec::new(),
                vec![named("Unit")],
                limit.clone(),
            ),
            function_export(
                module,
                "withLastEventId",
                [],
                Vec::new(),
                vec![named("String"), request.clone()],
                build_result(request),
            ),
            function_export(
                module,
                "events",
                ["R", "E"],
                Vec::new(),
                vec![limit, stream(named("R"), named("E"), http_event)],
                stream(
                    named("R"),
                    named_with("Either", vec![named("E"), parse_error]),
                    event.clone(),
                ),
            ),
            effect_function_export(
                module,
                "response",
                ["R"],
                Vec::new(),
                vec![
                    named_with("Array", vec![header]),
                    stream(named("R"), named("Never"), event),
                ],
                effect(named("R"), named("Never"), response),
            ),
        ],
    )
}
