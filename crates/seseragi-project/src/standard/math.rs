use super::*;

pub(super) fn math_interface() -> ModuleInterface {
    let module = "std/math";
    let mut exports = ["pi", "e", "tau"]
        .into_iter()
        .map(|name| value_export(module, name, named("Float")))
        .collect::<Vec<_>>();
    for name in [
        "sin", "cos", "tan", "asin", "acos", "atan", "exp", "log", "log2", "log10", "sqrt", "cbrt",
        "sinh", "cosh", "tanh",
    ] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float")],
            named("Float"),
        ));
    }
    for name in ["atan2", "hypot"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Float"),
        ));
    }
    standard_interface(module, exports)
}
