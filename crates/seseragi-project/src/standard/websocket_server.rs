use super::*;

pub(super) fn websocket_server_interface() -> ModuleInterface {
    let module = "std/websocket/server";
    let connection = external_type(
        "WebSocketConnection",
        "std/websocket::WebSocketConnection",
        "std/websocket",
        "WebSocketConnection",
        Vec::new(),
    );
    let error = external_type(
        "WebSocketError",
        "std/websocket::WebSocketError",
        "std/websocket",
        "WebSocketError",
        Vec::new(),
    );
    let capacity = external_type(
        "BufferCapacity",
        "std/stream::BufferCapacity",
        "std/stream",
        "BufferCapacity",
        Vec::new(),
    );
    let handler = |environment: InterfaceType| {
        function_type(
            vec![connection.clone()],
            effect(environment, named("Never"), named("Unit")),
        )
    };
    standard_interface(
        module,
        vec![
            type_export(module, "WebSocketServer", 0, "opaque-type"),
            type_export(module, "WebSocketServerHandle", 0, "opaque-type"),
            alias_type_export(module, "Handler", ["R"], handler(named("R"))),
            effect_function_export(
                module,
                "listen",
                ["R"],
                Vec::new(),
                vec![record([
                    optional("hostname", named("String")),
                    required("port", named("Int")),
                    required("path", named("String")),
                    required("protocols", named_with("Array", vec![named("String")])),
                    required("receiveBuffer", capacity),
                    required("handler", handler(named("R"))),
                ])],
                effect(
                    requirement_merge(vec![
                        named("R"),
                        record([required("webSocketServer", named("WebSocketServer"))]),
                    ]),
                    error,
                    named("WebSocketServerHandle"),
                ),
            ),
            effect_function_export(
                module,
                "closeServer",
                [],
                Vec::new(),
                vec![named("WebSocketServerHandle")],
                effect(
                    record([required("webSocketServer", named("WebSocketServer"))]),
                    named("Never"),
                    named("Unit"),
                ),
            ),
        ],
    )
}
