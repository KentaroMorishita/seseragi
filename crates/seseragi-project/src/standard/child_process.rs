use super::*;

pub(super) fn child_process_interface() -> ModuleInterface {
    let module = "std/child-process";
    let path = || external_type("Path", "std/path::Path", "std/path", "Path", Vec::new());
    let bytes = || {
        external_type(
            "Bytes",
            "std/bytes::Bytes",
            "std/bytes",
            "Bytes",
            Vec::new(),
        )
    };
    let duration = || {
        external_type(
            "Duration",
            "std/time::Duration",
            "std/time",
            "Duration",
            Vec::new(),
        )
    };
    let buffer_capacity = || {
        external_type(
            "BufferCapacity",
            "std/stream::BufferCapacity",
            "std/stream",
            "BufferCapacity",
            Vec::new(),
        )
    };
    let stream = |environment, failure, value| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment, failure, value],
        )
    };
    let process_signal = || {
        external_type(
            "ProcessSignal",
            "std/process::ProcessSignal",
            "std/process",
            "ProcessSignal",
            Vec::new(),
        )
    };
    let either = |left, right| named_with("Either", vec![left, right]);
    let environment = || record([required("childProcesses", named("ChildProcesses"))]);
    let mut exports = vec![
        type_export(module, "ChildProcesses", 0, "opaque-type"),
        opaque_adt_type_export(module, "Executable", []),
        constructor_export(
            module,
            "Executable",
            "SearchPath",
            [],
            Some(named("String")),
        ),
        constructor_export(module, "Executable", "ExecutablePath", [], Some(path())),
        type_export(module, "Command", 0, "opaque-type"),
        type_export(module, "CaptureLimit", 0, "opaque-type"),
        opaque_adt_type_export(module, "ChildProcessConfigError", []),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EmptyExecutableName",
            [],
            None,
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "ExecutableNameContainsSeparator",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "ArgumentContainsNul",
            [],
            Some(record([
                required("index", named("Int")),
                required("offset", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EnvironmentNameContainsNul",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "EnvironmentValueContainsNul",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessConfigError",
            "InvalidCaptureLimit",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "ChildOutputChannel", []),
        constructor_export(module, "ChildOutputChannel", "ChildStdout", [], None),
        constructor_export(module, "ChildOutputChannel", "ChildStderr", [], None),
        opaque_adt_type_export(module, "ChildProcessError", []),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildSpawnFailed",
            [],
            Some(record([
                required("executable", named("Executable")),
                required("detail", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildInputAfterClose",
            [],
            None,
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildOutputReadFailed",
            [],
            Some(record([
                required("channel", named("ChildOutputChannel")),
                required("detail", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "UnsupportedChildSignal",
            [],
            Some(process_signal()),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildInputFailed",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildOutputLimitExceeded",
            [],
            Some(record([
                required("channel", named("ChildOutputChannel")),
                required("limitBytes", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildWaitFailed",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "ChildProcessError",
            "ChildTerminationFailed",
            [],
            Some(named("String")),
        ),
        opaque_adt_type_export(module, "ChildExitStatus", []),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildExited",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildSignaled",
            [],
            Some(process_signal()),
        ),
        constructor_export(
            module,
            "ChildExitStatus",
            "ChildHostTerminated",
            [],
            Some(named("String")),
        ),
        opaque_adt_type_export(module, "ChildInput", []),
        constructor_export(module, "ChildInput", "WriteChildStdin", [], Some(bytes())),
        constructor_export(module, "ChildInput", "CloseChildStdin", [], None),
        constructor_export(
            module,
            "ChildInput",
            "SignalChild",
            [],
            Some(process_signal()),
        ),
        constructor_export(module, "ChildInput", "KillChild", [], None),
        opaque_adt_type_export(module, "ChildEvent", []),
        constructor_export(module, "ChildEvent", "ChildStdoutChunk", [], Some(bytes())),
        constructor_export(module, "ChildEvent", "ChildStderrChunk", [], Some(bytes())),
        constructor_export(
            module,
            "ChildEvent",
            "ChildExitedWith",
            [],
            Some(named("ChildExitStatus")),
        ),
        public_record_type_export(
            module,
            "CapturedProcess",
            [
                required("status", named("ChildExitStatus")),
                required("stdout", bytes()),
                required("stderr", bytes()),
            ],
        ),
        function_export(
            module,
            "command",
            [],
            Vec::new(),
            vec![named("Executable")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "addArgument",
            [],
            Vec::new(),
            vec![named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "addArguments",
            [],
            Vec::new(),
            vec![named_with("Array", vec![named("String")]), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "inDirectory",
            [],
            Vec::new(),
            vec![path(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "setEnvironment",
            [],
            Vec::new(),
            vec![named("String"), named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "unsetEnvironment",
            [],
            Vec::new(),
            vec![named("String"), named("Command")],
            either(named("ChildProcessConfigError"), named("Command")),
        ),
        function_export(
            module,
            "clearEnvironment",
            [],
            Vec::new(),
            vec![named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "terminationGrace",
            [],
            Vec::new(),
            vec![duration(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "outputBuffer",
            [],
            Vec::new(),
            vec![buffer_capacity(), named("Command")],
            named("Command"),
        ),
        function_export(
            module,
            "captureLimit",
            [],
            Vec::new(),
            vec![named("Int")],
            either(named("ChildProcessConfigError"), named("CaptureLimit")),
        ),
        function_export(
            module,
            "defaultCaptureLimit",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("CaptureLimit"),
        ),
        function_export(
            module,
            "runStreaming",
            ["R", "E"],
            Vec::new(),
            vec![
                stream(named("R"), named("E"), named("ChildInput")),
                named("Command"),
            ],
            stream(
                requirement_merge(vec![named("R"), environment()]),
                either(named("E"), named("ChildProcessError")),
                named("ChildEvent"),
            ),
        ),
        effect_function_export(
            module,
            "runCaptured",
            [],
            Vec::new(),
            vec![named("CaptureLimit"), bytes(), named("Command")],
            effect(
                environment(),
                named("ChildProcessError"),
                named("CapturedProcess"),
            ),
        ),
        effect_function_export(
            module,
            "runInherited",
            [],
            Vec::new(),
            vec![named("Command")],
            effect(
                environment(),
                named("ChildProcessError"),
                named("ChildExitStatus"),
            ),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}
