use super::*;

pub(super) fn process_interface() -> ModuleInterface {
    let module = "std/process";
    let process = || named("Process");
    let process_error = || named("ProcessError");
    let signal = || named("ProcessSignal");
    let environment = || record([required("process", process())]);
    let path = || external_type("Path", "std/path::Path", "std/path", "Path", vec![]);
    let non_empty_signals = || {
        external_type(
            "NonEmptyList",
            "std/non-empty-list::NonEmptyList",
            "std/non-empty-list",
            "NonEmptyList",
            vec![signal()],
        )
    };
    let stream = || {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment(), process_error(), signal()],
        )
    };
    let effect = |success| effect(environment(), process_error(), success);
    let mut exports = vec![
        type_export(module, "Process", 0, "opaque-type"),
        opaque_adt_type_export(module, "ProcessSignal", []),
    ];
    for constructor in ["Interrupt", "Terminate", "Hangup", "Quit", "User1", "User2"] {
        exports.push(constructor_export(
            module,
            "ProcessSignal",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "ProcessError", []));
    for (constructor, payload) in [
        ("UnsupportedProcessSignal", Some(signal())),
        ("ReservedProcessSignal", Some(signal())),
        ("InvalidArgumentEncoding", Some(named("Int"))),
        ("InvalidEnvironmentName", Some(named("String"))),
        ("InvalidEnvironmentEncoding", Some(named("String"))),
        ("CurrentDirectoryUnavailable", None),
    ] {
        exports.push(constructor_export(
            module,
            "ProcessError",
            constructor,
            [],
            payload,
        ));
    }
    exports.extend([
        effect_function_export(
            module,
            "arguments",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(named_with("Array", vec![named("String")])),
        ),
        effect_function_export(
            module,
            "environment",
            [],
            Vec::new(),
            vec![named("String")],
            effect(named_with("Maybe", vec![named("String")])),
        ),
        effect_function_export(
            module,
            "currentDirectory",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(path()),
        ),
        function_export(
            module,
            "signals",
            [],
            Vec::new(),
            vec![non_empty_signals()],
            stream(),
        ),
    ]);
    standard_interface(module, exports)
}
