use super::*;

pub(super) fn logger_interface() -> ModuleInterface {
    let module = "std/log";
    let mut exports = vec![
        type_export(module, "Logger", 0, "opaque-type"),
        opaque_adt_type_export(module, "LogLevel", []),
    ];
    for constructor in ["LogTrace", "LogDebug", "LogInfo", "LogWarn", "LogFailure"] {
        exports.push(constructor_export(
            module,
            "LogLevel",
            constructor,
            [],
            None,
        ));
    }
    exports.push(opaque_adt_type_export(module, "LogValue", []));
    for (constructor, payload) in [
        ("LogString", named("String")),
        ("LogInt", named("Int")),
        ("LogFloat", named("Float")),
        ("LogBool", named("Bool")),
    ] {
        exports.push(constructor_export(
            module,
            "LogValue",
            constructor,
            [],
            Some(payload),
        ));
    }
    exports.extend([
        public_record_type_export(
            module,
            "LogEvent",
            [
                required("level", named("LogLevel")),
                required("message", named("String")),
                required(
                    "fields",
                    named_with(
                        "List",
                        vec![InterfaceType::Tuple {
                            elements: vec![named("String"), named("LogValue")],
                        }],
                    ),
                ),
            ],
        ),
        type_export(module, "LogError", 0, "opaque-type"),
        effect_function_export(
            module,
            "log",
            [],
            Vec::new(),
            vec![named("LogEvent")],
            effect(
                record([required("logger", named("Logger"))]),
                named("LogError"),
                named("Unit"),
            ),
        ),
    ]);
    standard_interface(module, exports)
}
