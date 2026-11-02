pub fn native_benchmark_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    assert!(dictionaries.is_empty());
    match (name, arguments.as_slice()) {
        (
            "std/benchmark::benchmark" | "std/benchmark::suite" | "std/benchmark::inputSize",
            [_, _],
        ) => V::adt(
            match name {
                "std/benchmark::benchmark" => "std/benchmark::benchmark",
                "std/benchmark::suite" => "std/benchmark::suite",
                _ => "std/benchmark::inputSize",
            },
            V::Tuple(Shared::new(arguments)),
        ),
        ("std/benchmark::blackBox", [_]) => {
            std::hint::black_box(arguments.into_iter().next().unwrap())
        }
        ("std/benchmark::fail", [message]) => native_operation(
            "effect.fail",
            vec![V::adt(
                "std/benchmark::ExplicitBenchmarkFailure",
                message.clone(),
            )],
        ),
        _ => panic!("Rust ABI: unsupported std/benchmark operation"),
    }
}

struct NativeBenchmarkCase {
    name: String,
    input_size: Option<i64>,
    body: V,
}

fn native_benchmark_discover(
    mut modules: Vec<(String, V)>,
) -> Result<Vec<NativeBenchmarkCase>, String> {
    modules.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
    let mut cases = Vec::new();
    let mut module_names = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for (name, tree) in modules {
        native_test_validate_name("module", &name).map_err(|error| format!("Error: {error}"))?;
        if !module_names.insert(name.clone()) {
            return Err(format!("duplicate module {name}"));
        }
        let mut pending = vec![(tree, name, None)];
        while let Some((tree, parent, input_size)) = pending.pop() {
            let V::Adt(tree) = tree else {
                return Err("invalid Benchmark value".into());
            };
            let V::Tuple(parts) = &tree.get().1 else {
                return Err("invalid Benchmark value".into());
            };
            let [first, second] = parts.get().as_slice() else {
                return Err("invalid Benchmark value".into());
            };
            if tree.get().0 == "std/benchmark::inputSize" {
                let V::Int(size) = first else {
                    return Err("inputSize must be a positive Int".into());
                };
                if *size <= 0 || *size > 9_007_199_254_740_991 {
                    return Err("inputSize must be a positive Int".into());
                }
                pending.push((second.clone(), parent, Some(*size)));
                continue;
            }
            let kind = match tree.get().0 {
                "std/benchmark::benchmark" => "case",
                "std/benchmark::suite" => "suite",
                _ => return Err("invalid Benchmark value".into()),
            };
            let name = native_text(first);
            native_test_validate_name(kind, name).map_err(|error| format!("Error: {error}"))?;
            let name = format!("{parent}::{name}");
            if kind == "suite" {
                let V::Array(children) = second else {
                    return Err("invalid Benchmark children".into());
                };
                for child in children.get().iter().rev() {
                    pending.push((child.clone(), name.clone(), input_size));
                }
            } else {
                if !names.insert(name.clone()) {
                    return Err(format!("duplicate benchmark name {name}"));
                }
                cases.push(NativeBenchmarkCase {
                    name,
                    input_size,
                    body: second.clone(),
                });
            }
        }
    }
    Ok(cases)
}

#[derive(Debug, PartialEq)]
struct NativeBenchmarkFailure {
    kind: &'static str,
    detail: String,
}
impl NativeBenchmarkFailure {
    fn new(kind: &'static str, detail: &str) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

struct NativeBenchmarkOptions {
    warmup: u64,
    samples: usize,
    minimum_sample_ms: u64,
    timeout: u64,
    cleanup: u64,
    seed: i64,
    filter: Option<String>,
    exact: Option<String>,
}
impl NativeBenchmarkOptions {
    fn arguments() -> Self {
        let mut options = Self {
            warmup: 10,
            samples: 50,
            minimum_sample_ms: 100,
            timeout: 30_000,
            cleanup: 5_000,
            seed: 0,
            filter: None,
            exact: None,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let value = args.next().expect("benchmark runner option value");
            match flag.as_str() {
                "--warmup" => options.warmup = value.parse().expect("benchmark warmup"),
                "--samples" => options.samples = value.parse().expect("benchmark samples"),
                "--minimum-sample-ms" => {
                    options.minimum_sample_ms = value.parse().expect("benchmark minimum sample")
                }
                "--timeout-ms" => options.timeout = value.parse().expect("benchmark timeout"),
                "--cleanup-grace-ms" => {
                    options.cleanup = value.parse().expect("benchmark cleanup grace")
                }
                "--seed" => options.seed = value.parse().expect("benchmark seed"),
                "--filter" => options.filter = Some(value),
                "--exact" => options.exact = Some(value),
                _ => panic!("unknown native benchmark runner option"),
            }
        }
        options
    }
}

fn native_benchmark_seed(name: &str, seed: i64) -> u64 {
    let hash = name.bytes().fold(0xcbf29ce484222325u64, |value, byte| {
        (value ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    });
    seed as u64 ^ hash.wrapping_add(0x9e3779b97f4a7c15)
}

#[derive(Default)]
struct NativeBenchmarkHost {
    cancelled: Cell<bool>,
    handlers: Vec<native_process_signal_host::Handler>,
}
impl NativeBenchmarkHost {
    fn cancelled(&self) -> bool {
        if !self.handlers.is_empty()
            && native_process_signal_host::drain()
                .iter()
                .any(|signal| *signal < 2)
        {
            self.cancelled.set(true);
        }
        self.cancelled.get()
    }
}

fn native_benchmark_invoke(
    body: &V,
    environment: &Rc<RefCell<NativeTestEnvironment>>,
    options: &NativeBenchmarkOptions,
    host: &NativeBenchmarkHost,
) -> Result<(), NativeBenchmarkFailure> {
    let mut scheduler = NativeScheduler::default();
    environment.borrow_mut().cleanup = None;
    let context = NativeContext {
        test: Some(environment.clone()),
        ..NativeContext::default()
    };
    let root = scheduler.spawn(context, body.clone()).fiber().task;
    environment.borrow_mut().root = root;
    let deadline = std::time::Instant::now() + native_test_timer_delay(options.timeout);
    let mut timed_out = false;
    let mut cancelling = false;
    loop {
        let now = std::time::Instant::now();
        if !cancelling && host.cancelled() {
            cancelling = true;
            environment.borrow_mut().cleanup = Some((
                now + native_test_cleanup_delay(options.cleanup, true),
                "after cancellation",
            ));
            scheduler.cancel(root);
        }
        if environment.borrow().cleanup.is_none() && now >= deadline {
            timed_out = true;
            environment.borrow_mut().cleanup = Some((
                now + native_test_cleanup_delay(options.cleanup, true),
                "after timeout",
            ));
            scheduler.cancel(root);
        }
        if environment
            .borrow()
            .cleanup
            .is_some_and(|(deadline, _)| now >= deadline)
        {
            scheduler.discard_test(environment);
            return Err(NativeBenchmarkFailure::new(
                "resource-leak",
                "benchmark resource cleanup did not complete",
            ));
        }
        if let Some((id, _context, outcome)) = scheduler.tick(options.cleanup) {
            if id == root {
                return match outcome {
                    Err(NativeExit::Defect(errors)) => Err(NativeBenchmarkFailure::new(
                        "defect",
                        errors
                            .first()
                            .map(|error| error.value.as_str())
                            .unwrap_or("native host panic"),
                    )),
                    _ if timed_out => Err(NativeBenchmarkFailure::new(
                        "timeout",
                        "benchmark body timed out",
                    )),
                    Ok(_) => Ok(()),
                    Err(NativeExit::Failure(error)) => {
                        assert!(error
                            .value
                            .tagged("std/benchmark::ExplicitBenchmarkFailure"));
                        Err(NativeBenchmarkFailure::new(
                            "typed-failure",
                            &format!(
                                "{{\"tag\":\"ExplicitBenchmarkFailure\",\"value\":{}}}",
                                native_json_string(native_text(&error.value.payload()))
                            ),
                        ))
                    }
                    Err(NativeExit::Cancelled) => Err(NativeBenchmarkFailure::new(
                        "cancelled",
                        "effect execution cancelled",
                    )),
                };
            }
        }
        if scheduler.ready.is_empty() {
            let deadline = environment
                .borrow()
                .cleanup
                .map(|(deadline, _)| deadline)
                .unwrap_or(deadline);
            scheduler.park_until_ready(Some(
                deadline.saturating_duration_since(std::time::Instant::now()),
            ));
        }
    }
}

fn native_benchmark_measure(
    mut invoke: impl FnMut() -> Result<(), NativeBenchmarkFailure>,
    options: &NativeBenchmarkOptions,
    mut now_ns: impl FnMut() -> f64,
) -> Result<(u64, Vec<f64>), NativeBenchmarkFailure> {
    for _ in 0..options.warmup {
        invoke()?;
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(options.timeout);
    let minimum = options.minimum_sample_ms as f64 * 1_000_000.0;
    let mut iterations = 1u64;
    let mut samples = Vec::new();
    let mut calibrated = false;
    while samples.len() < options.samples {
        let start = now_ns();
        for iteration in 0..iterations {
            if iteration % 256 == 0 && std::time::Instant::now() > deadline {
                return Err(NativeBenchmarkFailure::new(
                    "timeout",
                    "benchmark measurement timed out",
                ));
            }
            invoke()?;
        }
        let elapsed = now_ns() - start;
        if !elapsed.is_finite() || elapsed < 0.0 {
            return Err(NativeBenchmarkFailure::new(
                "defect",
                "invalid monotonic measurement clock",
            ));
        }
        if elapsed < minimum {
            let next = if elapsed == 0.0 {
                iterations as f64 * 10.0
            } else {
                (iterations as f64 + 1.0).max((iterations as f64 * minimum / elapsed * 1.1).ceil())
            };
            if !next.is_finite() || next > 1_000_000_000.0 {
                return Err(NativeBenchmarkFailure::new(
                    "defect",
                    "measurement clock could not calibrate a bounded batch",
                ));
            }
            iterations = next as u64;
            samples.clear();
            calibrated = false;
        } else if !calibrated {
            calibrated = true;
        } else {
            samples.push(elapsed / iterations as f64);
        }
    }
    Ok((iterations, samples))
}

fn native_benchmark_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[middle]
    } else {
        sorted[middle - 1] + (sorted[middle] - sorted[middle - 1]) / 2.0
    }
}

fn native_benchmark_run(
    modules: Vec<(String, V)>,
    options: &NativeBenchmarkOptions,
    host: &NativeBenchmarkHost,
) -> Result<String, String> {
    if options.warmup > 9_007_199_254_740_991
        || options.samples < 3
        || options.samples as u64 > 9_007_199_254_740_991
        || options.minimum_sample_ms == 0
        || options.minimum_sample_ms > 9_007_199_254_740_991
        || options.timeout == 0
        || options.timeout > 9_007_199_254_740_991
        || options.cleanup > 9_007_199_254_740_991
    {
        return Err("invalid benchmark configuration".into());
    }
    if options.filter.is_some() && options.exact.is_some() {
        return Err("filter and exact are mutually exclusive".into());
    }
    let cases = native_benchmark_discover(modules)?
        .into_iter()
        .filter(|case| {
            if let Some(exact) = &options.exact {
                &case.name == exact
            } else {
                options
                    .filter
                    .as_ref()
                    .is_none_or(|filter| case.name.contains(filter))
            }
        })
        .collect::<Vec<_>>();
    if cases.is_empty() {
        return Err("benchmark discovery selected zero cases".into());
    }
    let mut results = Vec::new();
    for case in cases {
        let environment = Rc::new(RefCell::new(NativeTestEnvironment {
            random: native_benchmark_seed(&case.name, options.seed),
            ..NativeTestEnvironment::default()
        }));
        let origin = std::time::Instant::now();
        let cancelled = host.cancelled();
        let measured = if cancelled {
            Err(NativeBenchmarkFailure::new(
                "cancelled",
                "benchmark run cancelled",
            ))
        } else {
            native_benchmark_measure(
                || native_benchmark_invoke(&case.body, &environment, options, host),
                options,
                || origin.elapsed().as_nanos() as f64,
            )
        };
        let mut result = format!(
            "{{\"name\":{},\"inputSize\":{}",
            native_json_string(&case.name),
            case.input_size
                .map(|size| size.to_string())
                .unwrap_or_else(|| "null".into())
        );
        match measured {
            Ok((iterations, samples)) => {
                let median = native_benchmark_median(&samples);
                let mad = native_benchmark_median(
                    &samples
                        .iter()
                        .map(|sample| (sample - median).abs())
                        .collect::<Vec<_>>(),
                );
                result.push_str(&format!(",\"status\":\"passed\",\"iterations\":{iterations},\"samples\":[{}],\"summary\":{{\"median\":{},\"medianAbsoluteDeviation\":{},\"minimum\":{},\"maximum\":{},\"sampleCount\":{}}}", samples.iter().map(|value| native_number_text(*value)).collect::<Vec<_>>().join(","), native_number_text(median), native_number_text(mad), native_number_text(samples.iter().copied().fold(f64::INFINITY, f64::min)), native_number_text(samples.iter().copied().fold(f64::NEG_INFINITY, f64::max)), samples.len()));
            }
            Err(failure) => result.push_str(&format!(
                ",\"status\":\"failed\",\"failure\":{{\"kind\":{},\"detail\":{}}}",
                native_json_string(failure.kind),
                native_json_string(&failure.detail)
            )),
        }
        let environment = environment.borrow();
        result.push_str(&format!(
            ",\"stdout\":{},\"stderr\":{}}}",
            native_json_string(&environment.stdout),
            native_json_string(&environment.stderr)
        ));
        results.push(result);
    }
    Ok(format!("[{}]", results.join(",")))
}

pub fn run_native_benchmarks_entry(initialize: impl FnOnce() -> Vec<(String, V)>) -> i32 {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    enable_cycle_collection();
    let outcome = catch_native(|| {
        let host = NativeBenchmarkHost {
            cancelled: Cell::new(false),
            handlers: [0, 1]
                .into_iter()
                .map(native_process_signal_host::Handler::install)
                .collect::<Result<Vec<_>, _>>()?,
        };
        let code =
            match native_benchmark_run(initialize(), &NativeBenchmarkOptions::arguments(), &host) {
                Ok(report) => {
                    println!("{report}");
                    0
                }
                Err(error) => {
                    eprintln!("benchmark: Error: {error}");
                    2
                }
            };
        Ok(V::Int(code))
    });
    let code = match &outcome {
        Ok(V::Int(code)) => *code as i32,
        Err(NativeExit::Defect(errors)) => {
            for error in errors {
                eprintln!("benchmark discovery: {}", error.value);
            }
            2
        }
        _ => 2,
    };
    drop(outcome);
    clear_globals();
    clear_dictionaries();
    let collection = collect_cycles().expect("native benchmarks released their Ref borrows");
    if std::env::var_os("SESERAGI_NATIVE_METRICS").is_some() {
        eprintln!(
            "native managed-live={} collected={} scanned={}",
            LIVE.with(Cell::get),
            collection.reclaimed,
            collection.scanned
        );
    }
    std::panic::set_hook(previous_hook);
    code
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    fn options() -> NativeBenchmarkOptions {
        NativeBenchmarkOptions {
            warmup: 2,
            samples: 3,
            minimum_sample_ms: 1,
            timeout: 1_000,
            cleanup: 10,
            seed: 42,
            filter: None,
            exact: None,
        }
    }
    fn operation(name: &str, arguments: Vec<V>) -> V {
        native_benchmark_operation(name, vec![], arguments)
    }
    fn case(name: &str, body: V) -> V {
        operation("std/benchmark::benchmark", vec![V::text(name), body])
    }
    fn size(size: i64, tree: V) -> V {
        operation("std/benchmark::inputSize", vec![V::Int(size), tree])
    }
    fn finish() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn values_are_cold_black_box_preserves_identity_and_failure_replays() {
        enable_cycle_collection();
        let counter = V::reference(V::Int(0));
        let same = operation("std/benchmark::blackBox", vec![counter.clone()]);
        same.write_reference(V::Int(42));
        assert_eq!(counter.read_reference().render(), "42");
        let plan = V::leaf(vec![counter.clone()], |c, _| {
            c[0].write_reference(V::Int(99));
            Ok(V::Unit)
        });
        let tree = size(0, case("", plan));
        assert_eq!(counter.read_reference().render(), "42");
        for message in ["", "雪\n\""] {
            let failure = operation("std/benchmark::fail", vec![V::text(message)]);
            for _ in 0..2 {
                let Err(NativeExit::Failure(error)) =
                    NativeContext::default().root(failure.clone())
                else {
                    panic!("expected typed failure");
                };
                assert!(error
                    .value
                    .tagged("std/benchmark::ExplicitBenchmarkFailure"));
                assert_eq!(native_text(&error.value.payload()), message);
            }
        }
        drop((tree, counter, same));
        let cycle = V::reference(V::Unit);
        let boxed = operation("std/benchmark::blackBox", vec![cycle.clone()]);
        cycle.write_reference(boxed);
        drop(cycle);
        finish();
    }

    #[test]
    fn discovery_sorts_utf16_preserves_order_and_innermost_size_without_running_bodies() {
        enable_cycle_collection();
        let body = V::leaf(vec![], |_, _| panic!("discovery cannot execute a body"));
        let tree = size(
            10,
            operation(
                "std/benchmark::suite",
                vec![
                    V::text("suite"),
                    V::Array(Shared::new(vec![
                        size(20, case("first", body.clone())),
                        case("second", body.clone()),
                    ])),
                ],
            ),
        );
        let cases = native_benchmark_discover(vec![
            ("\u{e000}".into(), case("last", body.clone())),
            ("😀".into(), tree),
        ])
        .unwrap();
        assert_eq!(
            cases
                .iter()
                .map(|case| (case.name.as_str(), case.input_size))
                .collect::<Vec<_>>(),
            [
                ("😀::suite::first", Some(20)),
                ("😀::suite::second", Some(10)),
                ("\u{e000}::last", None)
            ]
        );
        drop(cases);
        for number in [i64::MIN, -1, 0, 9_007_199_254_740_992, i64::MAX] {
            assert!(
                matches!(native_benchmark_discover(vec![("main".into(), size(number, case("a", body.clone())))]), Err(error) if error == "inputSize must be a positive Int")
            );
        }
        assert!(
            matches!(native_benchmark_discover(vec![("main".into(), case("a", body.clone())), ("main".into(), case("b", body.clone()))]), Err(error) if error == "duplicate module main")
        );
        drop(body);
        finish();
    }

    #[test]
    fn warmup_calibration_and_summary_match_the_ts_fixed_clock() {
        let time = Cell::new(0.0);
        let calls = Cell::new(0);
        let measured = native_benchmark_measure(
            || {
                calls.set(calls.get() + 1);
                time.set(time.get() + 1_000_000.0);
                Ok(())
            },
            &options(),
            || time.get(),
        )
        .unwrap();
        assert_eq!(calls.get(), 6);
        assert_eq!(measured, (1, vec![1_000_000.0; 3]));
        assert_eq!(native_benchmark_median(&[4.0, 1.0, 3.0, 2.0]), 2.5);
        assert_eq!(native_benchmark_median(&[1e308, 1e308]), 1e308);
    }

    #[test]
    fn recalibration_discards_short_samples_and_keeps_a_fixed_iteration_count() {
        let mut config = options();
        config.warmup = 0;
        let mut timings = [
            0.0, 1e6, 0.0, 1e6, 0.0, 1e5, 0.0, 11e6, 0.0, 11e6, 0.0, 11e6, 0.0, 11e6,
        ]
        .into_iter();
        let calls = Cell::new(0);
        let measured = native_benchmark_measure(
            || {
                calls.set(calls.get() + 1);
                Ok(())
            },
            &config,
            || timings.next().unwrap(),
        )
        .unwrap();
        assert_eq!(measured, (11, vec![1e6; 3]));
        assert_eq!(calls.get(), 47);
        assert!(timings.next().is_none());
    }

    #[test]
    fn invalid_or_unbounded_clocks_keep_ts_defect_details_and_omit_measurements() {
        let mut config = options();
        config.warmup = 0;
        for elapsed in [-1.0, f64::NAN, f64::INFINITY] {
            let mut times = [0.0, elapsed].into_iter();
            assert_eq!(
                native_benchmark_measure(|| Ok(()), &config, || times.next().unwrap()),
                Err(NativeBenchmarkFailure::new(
                    "defect",
                    "invalid monotonic measurement clock"
                ))
            );
        }
        config.minimum_sample_ms = 9_007_199_254_740_991;
        let mut times = [0.0, 1.0].into_iter();
        assert_eq!(
            native_benchmark_measure(|| Ok(()), &config, || times.next().unwrap()),
            Err(NativeBenchmarkFailure::new(
                "defect",
                "measurement clock could not calibrate a bounded batch"
            ))
        );
        assert_eq!(
            native_benchmark_measure(
                || Err(NativeBenchmarkFailure::new("typed-failure", "failed")),
                &options(),
                || panic!("warmup failure precedes measurement")
            ),
            Err(NativeBenchmarkFailure::new("typed-failure", "failed"))
        );
    }

    #[test]
    fn invocation_classifies_typed_failures_defects_and_cancellation_and_releases_roots() {
        enable_cycle_collection();
        let environment = Rc::new(RefCell::new(NativeTestEnvironment::default()));
        let host = NativeBenchmarkHost::default();
        let typed = operation("std/benchmark::fail", vec![V::text("雪\n\"")]);
        for _ in 0..2 {
            assert_eq!(
                native_benchmark_invoke(&typed, &environment, &options(), &host),
                Err(NativeBenchmarkFailure::new(
                    "typed-failure",
                    "{\"tag\":\"ExplicitBenchmarkFailure\",\"value\":\"雪\\n\\\"\"}"
                ))
            );
        }
        let defect = V::leaf(vec![], |_, _| {
            Err(NativeExit::defects(vec!["broken".into()]))
        });
        assert_eq!(
            native_benchmark_invoke(&defect, &environment, &options(), &host),
            Err(NativeBenchmarkFailure::new("defect", "broken"))
        );
        let cancelled = V::leaf(vec![], |_, _| Err(NativeExit::Cancelled));
        assert_eq!(
            native_benchmark_invoke(&cancelled, &environment, &options(), &host),
            Err(NativeBenchmarkFailure::new(
                "cancelled",
                "effect execution cancelled"
            ))
        );
        drop((typed, defect, cancelled, environment, host));
        finish();
    }

    #[test]
    fn cancelled_runs_report_each_case_without_invoking_bodies() {
        enable_cycle_collection();
        let body = V::leaf(vec![], |_, _| {
            panic!("cancelled benchmark cannot invoke its body")
        });
        let tree = operation(
            "std/benchmark::suite",
            vec![
                V::text("suite"),
                V::Array(Shared::new(vec![
                    case("first", body.clone()),
                    case("second", body),
                ])),
            ],
        );
        let host = NativeBenchmarkHost {
            cancelled: Cell::new(true),
            ..NativeBenchmarkHost::default()
        };
        let result = native_benchmark_run(vec![("main".into(), tree)], &options(), &host).unwrap();
        assert_eq!(result, "[{\"name\":\"main::suite::first\",\"inputSize\":null,\"status\":\"failed\",\"failure\":{\"kind\":\"cancelled\",\"detail\":\"benchmark run cancelled\"},\"stdout\":\"\",\"stderr\":\"\"},{\"name\":\"main::suite::second\",\"inputSize\":null,\"status\":\"failed\",\"failure\":{\"kind\":\"cancelled\",\"detail\":\"benchmark run cancelled\"},\"stdout\":\"\",\"stderr\":\"\"}]");
        drop(host);
        finish();
    }
}
