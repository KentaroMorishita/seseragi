use super::*;

pub(super) fn benchmark_interface() -> ModuleInterface {
    let module = "std/benchmark";
    let benchmark = || named("Benchmark");
    let failure = || named("BenchmarkFailure");
    let environment = || {
        record([
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
    standard_interface(
        module,
        vec![
            alias_type_export(module, "BenchmarkEnvironment", [], environment()),
            opaque_adt_type_export(module, "BenchmarkFailure", []),
            constructor_export(
                module,
                "BenchmarkFailure",
                "ExplicitBenchmarkFailure",
                [],
                Some(named("String")),
            ),
            type_export(module, "Benchmark", 0, "opaque-type"),
            function_export(
                module,
                "benchmark",
                [],
                Vec::new(),
                vec![
                    named("String"),
                    effect(environment(), failure(), named("Unit")),
                ],
                benchmark(),
            ),
            function_export(
                module,
                "suite",
                [],
                Vec::new(),
                vec![named("String"), named_with("Array", vec![benchmark()])],
                benchmark(),
            ),
            function_export(
                module,
                "inputSize",
                [],
                Vec::new(),
                vec![named("Int"), benchmark()],
                benchmark(),
            ),
            function_export(
                module,
                "blackBox",
                ["A"],
                Vec::new(),
                vec![named("A")],
                named("A"),
            ),
            effect_function_export(
                module,
                "fail",
                [],
                Vec::new(),
                vec![named("String")],
                effect(record([]), failure(), named("Unit")),
            ),
        ],
    )
}
