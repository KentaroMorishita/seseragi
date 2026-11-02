use super::*;

pub(super) fn websocket_client_interface() -> ModuleInterface {
    let module = "std/websocket";
    let connection = named("WebSocketConnection");
    let event = named("WebSocketEvent");
    let close = named("WebSocketClose");
    let error = named("WebSocketError");
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let capacity = external_type(
        "BufferCapacity",
        "std/stream::BufferCapacity",
        "std/stream",
        "BufferCapacity",
        Vec::new(),
    );
    let stream = external_type(
        "Stream",
        "std/stream::Stream",
        "std/stream",
        "Stream",
        vec![record([]), error.clone(), event.clone()],
    );
    standard_interface(
        module,
        vec![
            type_export(module, "WebSocketClient", 0, "opaque-type"),
            type_export(module, "WebSocketConnection", 0, "opaque-type"),
            opaque_adt_type_export(module, "WebSocketEvent", []),
            type_export(module, "WebSocketClose", 0, "opaque-type"),
            opaque_adt_type_export(module, "WebSocketError", []),
            effect_function_export(
                module,
                "connect",
                [],
                Vec::new(),
                vec![record([
                    required("url", named("String")),
                    required("protocols", named_with("Array", vec![named("String")])),
                    required("receiveBuffer", capacity.clone()),
                ])],
                effect(
                    record([required("webSocketClient", named("WebSocketClient"))]),
                    error.clone(),
                    connection.clone(),
                ),
            ),
            function_export(
                module,
                "messages",
                [],
                Vec::new(),
                vec![connection.clone()],
                stream,
            ),
            effect_function_export(
                module,
                "sendText",
                [],
                Vec::new(),
                vec![named("String"), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            effect_function_export(
                module,
                "sendBytes",
                [],
                Vec::new(),
                vec![bytes.clone(), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            effect_function_export(
                module,
                "closeConnection",
                [],
                Vec::new(),
                vec![named("Int"), named("String"), connection.clone()],
                effect(record([]), error.clone(), named("Unit")),
            ),
            function_export(
                module,
                "selectedProtocol",
                [],
                Vec::new(),
                vec![connection],
                named("String"),
            ),
            function_export(
                module,
                "foldEvent",
                ["A"],
                Vec::new(),
                vec![
                    function_type(vec![named("String")], named("A")),
                    function_type(vec![bytes], named("A")),
                    function_type(vec![close.clone()], named("A")),
                    event,
                ],
                named("A"),
            ),
            function_export(
                module,
                "closeCode",
                [],
                Vec::new(),
                vec![close.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "closeReason",
                [],
                Vec::new(),
                vec![close.clone()],
                named("String"),
            ),
            function_export(
                module,
                "closeWasClean",
                [],
                Vec::new(),
                vec![close],
                named("Bool"),
            ),
            function_export(
                module,
                "errorMessage",
                [],
                Vec::new(),
                vec![error],
                named("String"),
            ),
        ],
    )
}
