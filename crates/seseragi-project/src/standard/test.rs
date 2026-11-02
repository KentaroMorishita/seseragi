use super::*;

pub(super) fn test_interface() -> ModuleInterface {
    let module = "std/test";
    let test = || named("Test");
    let test_failure = || named("TestFailure");
    let maybe_string = || named_with("Maybe", vec![named("String")]);
    let environment = || {
        record([
            required(
                "clock",
                external_type(
                    "Clock",
                    "std/clock::Clock",
                    "std/clock",
                    "Clock",
                    Vec::new(),
                ),
            ),
            required(
                "random",
                external_type(
                    "Random",
                    "std/random::Random",
                    "std/random",
                    "Random",
                    Vec::new(),
                ),
            ),
            required("console", prelude_type("Console")),
            required(
                "logger",
                external_type("Logger", "std/log::Logger", "std/log", "Logger", Vec::new()),
            ),
        ])
    };
    let assertion = || effect(record([]), test_failure(), named("Unit"));
    let debug_constraint = |parameter: &str| InterfaceConstraint {
        name: "Debug".to_owned(),
        trait_identity: Some("std/prelude::Debug".to_owned()),
        arguments: vec![named(parameter)],
    };
    let mut exports = vec![
        alias_type_export(module, "TestEnvironment", [], environment()),
        opaque_adt_type_export(module, "TestFailure", []),
        constructor_export(
            module,
            "TestFailure",
            "AssertionFailed",
            [],
            Some(record([
                required("message", named("String")),
                required("expected", maybe_string()),
                required("actual", maybe_string()),
            ])),
        ),
        constructor_export(module, "TestFailure", "ExpectedTypedFailure", [], None),
        constructor_export(
            module,
            "TestFailure",
            "TypedFailureDidNotMatch",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "TestFailure",
            "ExplicitTestFailure",
            [],
            Some(named("String")),
        ),
        type_export(module, "Test", 0, "opaque-type"),
        function_export(
            module,
            "test",
            [],
            Vec::new(),
            vec![
                named("String"),
                effect(environment(), test_failure(), named("Unit")),
            ],
            test(),
        ),
        function_export(
            module,
            "suite",
            [],
            Vec::new(),
            vec![named("String"), named_with("Array", vec![test()])],
            test(),
        ),
        function_export(
            module,
            "skip",
            [],
            Vec::new(),
            vec![named("String"), test()],
            test(),
        ),
        function_export(
            module,
            "timeout",
            [],
            Vec::new(),
            vec![
                external_type(
                    "Duration",
                    "std/time::Duration",
                    "std/time",
                    "Duration",
                    Vec::new(),
                ),
                test(),
            ],
            test(),
        ),
    ];
    for name in ["equal", "notEqual"] {
        exports.push(effect_function_export(
            module,
            name,
            ["A"],
            vec![
                InterfaceConstraint {
                    name: "Eq".to_owned(),
                    trait_identity: Some("std/prelude::Eq".to_owned()),
                    arguments: vec![named("A")],
                },
                debug_constraint("A"),
            ],
            vec![named("A"), named("A")],
            assertion(),
        ));
    }
    for name in ["isTrue", "isFalse"] {
        exports.push(effect_function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Bool")],
            assertion(),
        ));
    }
    exports.push(effect_function_export(
        module,
        "fail",
        [],
        Vec::new(),
        vec![named("String")],
        assertion(),
    ));
    exports.push(effect_function_export(
        module,
        "expectFailure",
        ["R", "E", "A"],
        vec![debug_constraint("E")],
        vec![
            function_type(vec![named("E")], named("Bool")),
            effect(named("R"), named("E"), named("A")),
        ],
        effect(named("R"), test_failure(), named("Unit")),
    ));
    standard_interface(module, exports)
}
