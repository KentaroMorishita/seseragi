use super::*;

pub(super) fn console_interface() -> ModuleInterface {
    let module = "std/console";
    let console = || prelude_type("Console");
    let console_error = || prelude_type("ConsoleError");
    let environment = || record([required("console", console())]);
    let output = || effect(environment(), console_error(), named("Unit"));
    let mut exports = vec![
        canonical_type_export(module, "Console", "std/prelude::Console", 0, "opaque-type"),
        canonical_type_export(
            module,
            "ConsoleError",
            "std/prelude::ConsoleError",
            0,
            "opaque-type",
        ),
    ];
    for name in ["print", "println", "error", "errorLine"] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            output(),
        ));
    }
    exports.push(effect_function_export(
        module,
        "printValue",
        ["A"],
        vec![InterfaceConstraint {
            name: "Show".to_owned(),
            trait_identity: Some("std/prelude::Show".to_owned()),
            arguments: vec![named("A")],
        }],
        vec![named("A")],
        output(),
    ));
    exports.push(effect_function_export(
        module,
        "flush",
        [],
        Vec::new(),
        vec![named("Unit")],
        output(),
    ));
    standard_interface(module, exports)
}
