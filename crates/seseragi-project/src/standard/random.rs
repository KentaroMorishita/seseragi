use super::*;

pub(super) fn random_interface() -> ModuleInterface {
    let module = "std/random";
    let environment = || record([required("random", named("Random"))]);
    let infallible = |success| effect(environment(), named("Never"), success);
    let fallible = |success| effect(environment(), named("RandomRangeError"), success);
    let mut exports = vec![
        type_export(module, "Random", 0, "opaque-type"),
        opaque_adt_type_export(module, "RandomRangeError", []),
        constructor_export(
            module,
            "RandomRangeError",
            "EmptyRandomIntRange",
            [],
            Some(record([
                required("lower", named("Int")),
                required("upperExclusive", named("Int")),
            ])),
        ),
        constructor_export(
            module,
            "RandomRangeError",
            "InvalidProbability",
            [],
            Some(named("Float")),
        ),
        opaque_adt_type_export(module, "RandomConfigError", []),
        constructor_export(
            module,
            "RandomConfigError",
            "NonPositiveRandomSize",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "RandomConfigError",
            "RandomSizeTooLarge",
            [],
            Some(named("Int")),
        ),
        type_export(module, "RandomSize", 0, "opaque-type"),
        function_export(
            module,
            "randomSize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("RandomConfigError"), named("RandomSize")],
            ),
        ),
        function_export(
            module,
            "algorithmId",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("String")),
        ),
        function_export(
            module,
            "nextBool",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Bool")),
        ),
        function_export(
            module,
            "nextInt",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Int")),
        ),
        function_export(
            module,
            "intBetween",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            fallible(named("Int")),
        ),
        function_export(
            module,
            "unitFloat",
            [],
            Vec::new(),
            vec![named("Unit")],
            infallible(named("Float")),
        ),
        function_export(
            module,
            "chance",
            [],
            Vec::new(),
            vec![named("Float")],
            fallible(named("Bool")),
        ),
        function_export(
            module,
            "randomBytes",
            [],
            Vec::new(),
            vec![named("RandomSize")],
            infallible(external_type(
                "Bytes",
                "std/bytes::Bytes",
                "std/bytes",
                "Bytes",
                Vec::new(),
            )),
        ),
        function_export(
            module,
            "choose",
            ["A"],
            Vec::new(),
            vec![external_type(
                "NonEmptyList",
                "std/non-empty-list::NonEmptyList",
                "std/non-empty-list",
                "NonEmptyList",
                vec![named("A")],
            )],
            infallible(named("A")),
        ),
        function_export(
            module,
            "shuffle",
            ["A"],
            Vec::new(),
            vec![named_with("Array", vec![named("A")])],
            infallible(named_with("Array", vec![named("A")])),
        ),
    ];
    standard_interface(module, std::mem::take(&mut exports))
}
