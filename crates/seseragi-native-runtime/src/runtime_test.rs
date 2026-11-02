pub fn native_test_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    let failure = |value| native_operation("effect.fail", vec![value]);
    let success = || native_operation("effect.succeed", vec![V::Unit]);
    match (name, arguments.as_slice()) {
        ("std/test::test", [_, _])
        | ("std/test::suite", [_, _])
        | ("std/test::skip", [_, _])
        | ("std/test::timeout", [_, _]) => {
            assert!(dictionaries.is_empty());
            let tag = match name {
                "std/test::test" => "std/test::test",
                "std/test::suite" => "std/test::suite",
                "std/test::skip" => "std/test::skip",
                _ => "std/test::timeout",
            };
            V::adt(tag, V::Tuple(Shared::new(arguments)))
        }
        ("std/test::equal" | "std/test::notEqual", [expected, actual]) => {
            let [equality, debug] = dictionaries.as_slice() else {
                panic!("Rust ABI: test Eq/Debug arity")
            };
            let V::Bool(equal) = equality
                .field("eq")
                .apply(expected.clone())
                .apply(actual.clone())
            else {
                panic!("Rust ABI: test Eq result")
            };
            if equal == (name == "std/test::equal") {
                return success();
            }
            let expected = if name == "std/test::equal" {
                Some(debug.field("debug").apply(expected.clone()))
            } else {
                None
            };
            let actual = debug.field("debug").apply(actual.clone());
            failure(native_test_assertion(
                if name == "std/test::equal" {
                    "values are not equal"
                } else {
                    "values are equal"
                },
                expected,
                Some(actual),
            ))
        }
        ("std/test::isTrue" | "std/test::isFalse", [V::Bool(actual)]) => {
            assert!(dictionaries.is_empty());
            let expected = name == "std/test::isTrue";
            if *actual == expected {
                return success();
            }
            failure(native_test_assertion(
                if expected {
                    "expected true"
                } else {
                    "expected false"
                },
                Some(V::text(if expected { "true" } else { "false" })),
                Some(V::text(if *actual { "true" } else { "false" })),
            ))
        }
        ("std/test::fail", [message]) => {
            assert!(dictionaries.is_empty());
            failure(V::adt("std/test::ExplicitTestFailure", message.clone()))
        }
        ("std/test::expectFailure", [predicate, source]) => {
            let [debug] = dictionaries.as_slice() else {
                panic!("Rust ABI: test Debug arity")
            };
            V::bind(
                V::effect(Instruction::Attempt, vec![source.clone()]),
                V::fun(vec![predicate.clone(), debug.clone()], |c, result| {
                    if result.tagged("std/prelude::Right") {
                        return native_operation(
                            "effect.fail",
                            vec![V::adt("std/test::ExpectedTypedFailure", V::Unit)],
                        );
                    }
                    assert!(result.tagged("std/prelude::Left"));
                    let error = result.payload();
                    let V::Bool(matches) = c[0].apply(error.clone()) else {
                        panic!("Rust ABI: test predicate result")
                    };
                    if matches {
                        native_operation("effect.succeed", vec![V::Unit])
                    } else {
                        native_operation(
                            "effect.fail",
                            vec![V::adt(
                                "std/test::TypedFailureDidNotMatch",
                                c[1].field("debug").apply(error),
                            )],
                        )
                    }
                }),
            )
        }
        ("std/test::ExpectedTypedFailure", []) => V::adt("std/test::ExpectedTypedFailure", V::Unit),
        (
            "std/test::AssertionFailed"
            | "std/test::ExplicitTestFailure"
            | "std/test::TypedFailureDidNotMatch",
            [payload],
        ) => {
            assert!(dictionaries.is_empty());
            V::adt(
                match name {
                    "std/test::AssertionFailed" => "std/test::AssertionFailed",
                    "std/test::ExplicitTestFailure" => "std/test::ExplicitTestFailure",
                    _ => "std/test::TypedFailureDidNotMatch",
                },
                payload.clone(),
            )
        }
        _ => panic!("Rust ABI: unsupported std/test operation"),
    }
}

fn native_test_assertion(message: &str, expected: Option<V>, actual: Option<V>) -> V {
    let maybe = |value: Option<V>| match value {
        Some(value) => V::adt("std/prelude::Just", value),
        None => V::adt("std/prelude::Nothing", V::Unit),
    };
    V::adt(
        "std/test::AssertionFailed",
        V::Record(Shared::new(BTreeMap::from([
            ("message".into(), V::text(message)),
            ("expected".into(), maybe(expected)),
            ("actual".into(), maybe(actual)),
        ]))),
    )
}

fn native_test_render_failure(failure: &V) -> String {
    if failure.tagged("std/test::ExpectedTypedFailure") {
        return "expected a typed failure".into();
    }
    if failure.tagged("std/test::ExplicitTestFailure") {
        return native_text(&failure.payload()).to_owned();
    }
    if failure.tagged("std/test::TypedFailureDidNotMatch") {
        return format!(
            "typed failure did not match: {}",
            native_text(&failure.payload())
        );
    }
    assert!(
        failure.tagged("std/test::AssertionFailed"),
        "Rust ABI: TestFailure owner"
    );
    let fields = failure.payload();
    let mut output = native_text(&fields.field("message")).to_owned();
    for field in ["expected", "actual"] {
        let value = fields.field(field);
        if value.tagged("std/prelude::Just") {
            output.push_str(&format!("; {field} {}", native_text(&value.payload())));
        }
    }
    output
}

#[derive(Default)]
struct NativeTestEnvironment {
    root: usize,
    instant: u128,
    random: u64,
    stdout: String,
    stderr: String,
    cleanup: Option<(std::time::Instant, &'static str)>,
}
impl NativeTestEnvironment {
    fn new(seed: i64, index: usize) -> Self {
        // CLI seeds pass through JavaScript numbers in the TS runner.
        Self {
            random: (seed as f64 as i128 as u64) ^ (index as u64).wrapping_add(0x9e3779b97f4a7c15),
            ..Self::default()
        }
    }
    fn next(&mut self) -> u64 {
        let mut state = u128::from(self.random);
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        self.random = state as u64;
        self.random
    }
    fn unit_float(&mut self) -> f64 {
        (self.next() >> 11) as f64 / 9_007_199_254_740_992.0
    }
}

struct NativeTestCase {
    index: usize,
    name: String,
    body: V,
    skip: Option<String>,
    timeout: Option<u64>,
}

fn native_test_random(
    name: &str,
    arguments: &[V],
    random: &mut NativeTestEnvironment,
) -> NativeOutcome {
    match (name, arguments) {
        ("std/random::algorithmId", [V::Unit]) => Ok(V::text("seseragi-test-xorshift64")),
        ("std/random::nextBool", [V::Unit]) => Ok(V::Bool(random.next() & 1 == 1)),
        ("std/random::nextInt", [V::Unit]) => Ok(V::Int(random.next() as i32 as i64)),
        ("std/random::unitFloat", [V::Unit]) => Ok(V::Float(random.unit_float())),
        ("std/random::intBetween", [V::Int(lower), V::Int(upper)]) => {
            checked_int_value(*lower);
            checked_int_value(*upper);
            if upper <= lower {
                return Err(NativeExit::failure(V::adt(
                    "std/random::EmptyRandomIntRange",
                    V::Record(Shared::new(BTreeMap::from([
                        ("lower".into(), V::Int(*lower)),
                        ("upperExclusive".into(), V::Int(*upper)),
                    ]))),
                )));
            }
            let width = (*upper as f64 - *lower as f64) as u64;
            Ok(V::Int(
                (*lower as f64 + (random.next() % width) as f64) as i64,
            ))
        }
        ("std/random::chance", [V::Float(probability)]) => {
            if !(0.0..=1.0).contains(probability) {
                return Err(NativeExit::failure(V::adt(
                    "std/random::InvalidProbability",
                    V::Float(*probability),
                )));
            }
            Ok(V::Bool(random.unit_float() < *probability))
        }
        ("std/random::randomBytes", [size]) => {
            assert!(size.tagged("std/random::RandomSize"));
            let V::Int(size) = size.payload() else {
                panic!("Rust ABI: RandomSize")
            };
            Ok(V::Bytes(Shared::new(
                (0..size).map(|_| random.next() as u8).collect(),
            )))
        }
        ("std/random::choose", [values]) => {
            let values = native_non_empty_values(values);
            Ok(values[(random.next() % values.len() as u64) as usize].clone())
        }
        ("std/random::shuffle", [V::Array(values)]) => {
            let mut values = values.get().clone();
            for index in (1..values.len()).rev() {
                values.swap(index, (random.next() % (index + 1) as u64) as usize);
            }
            Ok(V::Array(Shared::new(values)))
        }
        _ => panic!("Rust ABI: unadmitted test Random operation"),
    }
}

fn native_test_validate_name(kind: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.contains("::") || value.contains(['\r', '\n']) {
        return Err(format!(
            "{kind} name is not canonical: {}",
            native_json_string(value)
        ));
    }
    Ok(())
}
fn native_test_discover(modules: Vec<(String, V)>) -> Result<Vec<NativeTestCase>, String> {
    let mut cases = Vec::new();
    for (name, tree) in modules {
        native_test_validate_name("module", &name)?;
        let mut pending = vec![(tree, name, None, None)];
        while let Some((tree, parent, skip, timeout)) = pending.pop() {
            let V::Adt(tree) = tree else {
                panic!("Rust ABI: Test tree")
            };
            let V::Tuple(parts) = &tree.get().1 else {
                panic!("Rust ABI: Test parts")
            };
            let [first, second] = parts.get().as_slice() else {
                panic!("Rust ABI: Test arity")
            };
            match tree.get().0 {
                "std/test::skip" => {
                    let reason = native_text(first);
                    native_test_validate_name("skip reason", reason)?;
                    pending.push((second.clone(), parent, Some(reason.to_owned()), timeout));
                }
                "std/test::timeout" => {
                    let duration = native_duration_nanoseconds(first)
                        .expect("canonical test Duration")
                        / 1_000_000;
                    pending.push((second.clone(), parent, skip, Some(duration)));
                }
                tag @ ("std/test::test" | "std/test::suite") => {
                    let kind = if tag == "std/test::test" {
                        "case"
                    } else {
                        "suite"
                    };
                    let name = native_text(first);
                    native_test_validate_name(kind, name)?;
                    let name = format!("{parent}::{name}");
                    if kind == "suite" {
                        let V::Array(children) = second else {
                            panic!("Rust ABI: suite children")
                        };
                        for child in children.get().iter().rev() {
                            pending.push((child.clone(), name.clone(), skip.clone(), timeout));
                        }
                    } else {
                        cases.push(NativeTestCase {
                            index: cases.len(),
                            name,
                            body: second.clone(),
                            skip,
                            timeout,
                        });
                    }
                }
                _ => panic!("Rust ABI: Test owner"),
            }
        }
    }
    let mut names = std::collections::BTreeSet::new();
    for case in &cases {
        if !names.insert(&case.name) {
            return Err(format!("duplicate test name {}", case.name));
        }
    }
    Ok(cases)
}

struct NativeTestOptions {
    filter: Option<String>,
    exact: Option<String>,
    jobs: usize,
    timeout: u64,
    cleanup: u64,
    seed: i64,
}
impl NativeTestOptions {
    fn arguments() -> Self {
        let mut options = Self {
            filter: None,
            exact: None,
            jobs: 1,
            timeout: 30_000,
            cleanup: 5_000,
            seed: 0,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let value = args.next().expect("test runner option value");
            match flag.as_str() {
                "--filter" => options.filter = Some(value),
                "--exact" => options.exact = Some(value),
                "--jobs" => options.jobs = value.parse().expect("test jobs"),
                "--timeout-ms" => options.timeout = value.parse().expect("test timeout"),
                "--cleanup-grace-ms" => {
                    options.cleanup = value.parse().expect("test cleanup grace")
                }
                "--seed" => options.seed = value.parse().expect("test seed"),
                _ => panic!("unknown native test runner option"),
            }
        }
        assert!(options.jobs > 0);
        options
    }
}

#[derive(Default)]
struct NativeTestResult {
    detail: Option<String>,
    stdout: String,
    stderr: String,
}
struct NativeTestExecution {
    position: usize,
    environment: Rc<RefCell<NativeTestEnvironment>>,
    deadline: std::time::Instant,
    timeout: u64,
    timed_out: bool,
}

fn native_test_timer_delay(milliseconds: u64) -> std::time::Duration {
    // Host timers clamp zero, sub-millisecond, and overflowing delays to one millisecond.
    std::time::Duration::from_millis(if milliseconds == 0 || milliseconds > i32::MAX as u64 {
        1
    } else {
        milliseconds
    })
}

fn native_test_cleanup_delay(milliseconds: u64, cancelling: bool) -> std::time::Duration {
    // Zero cancellation grace returns before asynchronous cleanup settles.
    if cancelling && milliseconds == 0 {
        std::time::Duration::ZERO
    } else {
        native_test_timer_delay(milliseconds)
    }
}

fn native_test_run(modules: Vec<(String, V)>, options: &NativeTestOptions) -> i32 {
    let cases = match native_test_discover(modules) {
        Ok(cases) => cases
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
            .collect::<Vec<_>>(),
        Err(error) => {
            eprintln!("seseragi: {error}");
            return 2;
        }
    };
    if cases.is_empty() {
        eprintln!("seseragi: test selection matched zero cases");
        return 2;
    }
    let mut results = (0..cases.len())
        .map(|_| NativeTestResult::default())
        .collect::<Vec<_>>();
    let mut scheduler = NativeScheduler::default();
    let mut active = BTreeMap::<usize, NativeTestExecution>::new();
    let mut next = 0;
    while next < cases.len() || !active.is_empty() {
        while next < cases.len() && active.len() < options.jobs {
            let position = next;
            next += 1;
            let case = &cases[position];
            if case.skip.is_some() {
                continue;
            }
            let environment = Rc::new(RefCell::new(NativeTestEnvironment::new(
                options.seed,
                case.index,
            )));
            let context = NativeContext {
                test: Some(environment.clone()),
                ..NativeContext::default()
            };
            let root = scheduler.spawn(context, case.body.clone()).fiber().task;
            environment.borrow_mut().root = root;
            let timeout = case.timeout.unwrap_or(options.timeout);
            active.insert(
                root,
                NativeTestExecution {
                    position,
                    environment,
                    deadline: std::time::Instant::now() + native_test_timer_delay(timeout),
                    timeout,
                    timed_out: false,
                },
            );
        }
        let now = std::time::Instant::now();
        let mut expired = Vec::new();
        for (root, execution) in &mut active {
            let mut environment = execution.environment.borrow_mut();
            if environment.cleanup.is_none() && now >= execution.deadline {
                execution.timed_out = true;
                environment.cleanup = Some((
                    now + native_test_cleanup_delay(options.cleanup, true),
                    "after timeout",
                ));
                scheduler.cancel(*root);
            }
            if environment
                .cleanup
                .is_some_and(|(deadline, _)| now >= deadline)
            {
                expired.push(*root);
            }
        }
        for root in expired {
            let execution = active.remove(&root).unwrap();
            let mut environment = execution.environment.borrow_mut();
            let reason = environment.cleanup.unwrap().1;
            results[execution.position] = NativeTestResult {
                detail: Some(format!("resource leak {reason}")),
                stdout: std::mem::take(&mut environment.stdout),
                stderr: std::mem::take(&mut environment.stderr),
            };
            drop(environment);
            scheduler.discard_test(&execution.environment);
        }
        if let Some((root, _context, outcome)) = scheduler.tick(options.cleanup) {
            if let Some(execution) = active.remove(&root) {
                let mut environment = execution.environment.borrow_mut();
                let detail = if execution.timed_out {
                    Some(format!("timed out after {} ms", execution.timeout))
                } else {
                    match outcome {
                        Ok(_) => None,
                        Err(NativeExit::Failure(error)) => {
                            Some(native_test_render_failure(&error.value))
                        }
                        Err(NativeExit::Cancelled) => Some("cancelled".into()),
                        Err(NativeExit::Defect(errors)) => Some(format!(
                            "defect: {}",
                            errors
                                .first()
                                .map(|error| error.value.as_str())
                                .unwrap_or("native host panic")
                        )),
                    }
                };
                results[execution.position] = NativeTestResult {
                    detail,
                    stdout: std::mem::take(&mut environment.stdout),
                    stderr: std::mem::take(&mut environment.stderr),
                };
            }
        }
        if scheduler.ready.is_empty() && !active.is_empty() {
            let deadline = active
                .values()
                .map(|execution| {
                    execution
                        .environment
                        .borrow()
                        .cleanup
                        .map(|(deadline, _)| deadline)
                        .unwrap_or(execution.deadline)
                })
                .min()
                .unwrap();
            scheduler.park_until_ready(Some(
                deadline.saturating_duration_since(std::time::Instant::now()),
            ));
        }
    }
    let (mut passed, mut failed, mut skipped) = (0, 0, 0);
    for (case, result) in cases.iter().zip(results) {
        if let Some(reason) = &case.skip {
            skipped += 1;
            println!("SKIP {} -- {reason}", case.name);
        } else if let Some(detail) = result.detail {
            failed += 1;
            println!("FAIL {}", case.name);
            eprintln!("{}: {detail}", case.name);
        } else {
            passed += 1;
            println!("PASS {}", case.name);
        }
        if !result.stdout.is_empty() {
            eprint!("{} stdout:\n{}", case.name, result.stdout);
        }
        if !result.stderr.is_empty() {
            eprint!("{} stderr:\n{}", case.name, result.stderr);
        }
    }
    println!("{passed} passed; {failed} failed; {skipped} skipped");
    i32::from(failed > 0)
}

pub fn run_native_tests_entry(initialize: impl FnOnce() -> Vec<(String, V)>) -> i32 {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    enable_cycle_collection();
    let outcome = catch_native(|| {
        Ok(V::Int(
            native_test_run(initialize(), &NativeTestOptions::arguments()) as i64,
        ))
    });
    let code = match &outcome {
        Ok(V::Int(code)) => *code as i32,
        Err(NativeExit::Defect(errors)) => {
            for error in errors {
                eprintln!("seseragi: {}", error.value);
            }
            2
        }
        _ => 2,
    };
    drop(outcome);
    clear_globals();
    clear_dictionaries();
    let collection = collect_cycles().expect("native tests released their Ref borrows");
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
mod native_test_tests {
    use super::*;

    fn operation(name: &str, arguments: Vec<V>) -> V {
        native_test_operation(name, vec![], arguments)
    }
    fn succeed(value: V) -> V {
        native_operation("effect.succeed", vec![value])
    }
    fn increment(counter: &V) {
        counter.write_reference(binary("+", counter.read_reference(), V::Int(1)));
    }
    fn finish() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    fn debug(counter: &V) -> V {
        V::dict(vec![(
            "debug",
            V::fun(vec![counter.clone()], |c, value| {
                increment(&c[0]);
                V::text(&format!("debug:{}", value.render()))
            }),
        )])
    }
    fn typed(plan: V) -> V {
        match NativeContext::default().root(plan) {
            Err(NativeExit::Failure(error)) => error.value,
            _ => panic!("expected typed test failure"),
        }
    }

    #[test]
    fn assertions_use_selected_dictionaries_eagerly_once_and_replay_cold_failures() {
        enable_cycle_collection();
        let equal_calls = V::reference(V::Int(0));
        let debug_calls = V::reference(V::Int(0));
        let equality = V::dict(vec![(
            "eq",
            V::fun(vec![equal_calls.clone()], |c, expected| {
                V::fun(vec![c[0].clone(), expected], |c, actual| {
                    increment(&c[0]);
                    V::Bool(c[1].render() == actual.render())
                })
            }),
        )]);
        for (name, left, right, expected, debug_count) in [
            ("std/test::equal", 1, 1, None, "0"),
            (
                "std/test::equal",
                1,
                2,
                Some("values are not equal; expected debug:1; actual debug:2"),
                "2",
            ),
            ("std/test::notEqual", 1, 2, None, "2"),
            (
                "std/test::notEqual",
                1,
                1,
                Some("values are equal; actual debug:1"),
                "3",
            ),
        ] {
            let plan = native_test_operation(
                name,
                vec![equality.clone(), debug(&debug_calls)],
                vec![V::Int(left), V::Int(right)],
            );
            for _ in 0..2 {
                let outcome = NativeContext::default().root(plan.clone());
                if let Some(expected) = expected {
                    let Err(NativeExit::Failure(error)) = outcome else {
                        panic!("assertion lost typed failure")
                    };
                    assert_eq!(native_test_render_failure(&error.value), expected);
                } else {
                    assert!(matches!(outcome, Ok(V::Unit)));
                }
            }
            assert_eq!(debug_calls.read_reference().render(), debug_count);
        }
        assert_eq!(equal_calls.read_reference().render(), "4");
        drop((equality, equal_calls, debug_calls));
        finish();
    }

    #[test]
    fn expect_failure_is_cold_repeats_and_only_handles_typed_failures() {
        enable_cycle_collection();
        let source_calls = V::reference(V::Int(0));
        let predicate_calls = V::reference(V::Int(0));
        let debug_calls = V::reference(V::Int(0));
        let source = V::leaf(vec![source_calls.clone()], |c, _| {
            increment(&c[0]);
            Err(NativeExit::failure(V::text("typed")))
        });
        let predicate = V::fun(vec![predicate_calls.clone()], |c, error| {
            increment(&c[0]);
            V::Bool(native_text(&error) == "typed")
        });
        let plan = native_test_operation(
            "std/test::expectFailure",
            vec![debug(&debug_calls)],
            vec![predicate.clone(), source],
        );
        assert_eq!(source_calls.read_reference().render(), "0");
        for _ in 0..2 {
            assert!(matches!(
                NativeContext::default().root(plan.clone()),
                Ok(V::Unit)
            ));
        }
        assert_eq!(source_calls.read_reference().render(), "2");
        assert_eq!(predicate_calls.read_reference().render(), "2");
        assert_eq!(debug_calls.read_reference().render(), "0");
        let success = native_test_operation(
            "std/test::expectFailure",
            vec![debug(&debug_calls)],
            vec![predicate.clone(), succeed(V::Unit)],
        );
        assert!(typed(success).tagged("std/test::ExpectedTypedFailure"));
        let mismatch = native_test_operation(
            "std/test::expectFailure",
            vec![debug(&debug_calls)],
            vec![
                predicate.clone(),
                native_operation("effect.fail", vec![V::text("other")]),
            ],
        );
        assert_eq!(
            native_test_render_failure(&typed(mismatch)),
            "typed failure did not match: debug:other"
        );
        assert_eq!(debug_calls.read_reference().render(), "1");
        for source in [
            native_effect_die(V::text("defect")),
            V::leaf(vec![], |_, _| Err(NativeExit::Cancelled)),
        ] {
            let plan = native_test_operation(
                "std/test::expectFailure",
                vec![debug(&debug_calls)],
                vec![predicate.clone(), source],
            );
            assert!(matches!(
                NativeContext::default().root(plan),
                Err(NativeExit::Defect(_)) | Err(NativeExit::Cancelled)
            ));
        }
        assert_eq!(predicate_calls.read_reference().render(), "3");
        drop((plan, predicate, source_calls, predicate_calls, debug_calls));
        finish();
    }

    #[test]
    fn failure_constructors_and_boolean_boundaries_keep_the_exact_failure_payloads() {
        enable_cycle_collection();
        for (name, input, expected) in [
            (
                "std/test::isTrue",
                false,
                "expected true; expected true; actual false",
            ),
            (
                "std/test::isFalse",
                true,
                "expected false; expected false; actual true",
            ),
        ] {
            assert_eq!(
                native_test_render_failure(&typed(operation(name, vec![V::Bool(input)]))),
                expected
            );
            assert!(matches!(
                NativeContext::default().root(operation(name, vec![V::Bool(!input)])),
                Ok(V::Unit)
            ));
        }
        let message = V::text("雪\n\0");
        let failure = typed(operation("std/test::fail", vec![message.clone()]));
        assert_eq!(failure.payload().node_id(), message.node_id());
        assert_eq!(native_test_render_failure(&failure), "雪\n\0");
        let payload = V::Record(Shared::new(BTreeMap::from([
            ("message".into(), V::text("manual")),
            ("expected".into(), V::adt("std/prelude::Nothing", V::Unit)),
            ("actual".into(), V::adt("std/prelude::Nothing", V::Unit)),
        ])));
        assert_eq!(
            native_test_render_failure(&operation("std/test::AssertionFailed", vec![payload])),
            "manual"
        );
        drop((failure, message));
        finish();
    }

    #[test]
    fn discovery_preserves_wrapper_precedence_order_and_validates_before_selection() {
        enable_cycle_collection();
        let case = || operation("std/test::test", vec![V::text("case"), succeed(V::Unit)]);
        let tree = operation(
            "std/test::skip",
            vec![
                V::text("outer"),
                operation(
                    "std/test::skip",
                    vec![
                        V::text("inner"),
                        operation(
                            "std/test::timeout",
                            vec![
                                native_duration(9_000_000),
                                operation(
                                    "std/test::timeout",
                                    vec![native_duration(999_999), case()],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        );
        let cases = native_test_discover(vec![("module".into(), tree)])
            .ok()
            .unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].name, "module::case");
        assert_eq!(cases[0].skip.as_deref(), Some("inner"));
        assert_eq!(cases[0].timeout, Some(0));
        drop(cases);
        for name in ["", "a::b", "a\nb", "a\rb"] {
            assert!(native_test_validate_name("case", name).is_err());
        }
        assert!(native_test_validate_name("case", " 雪🙂 ").is_ok());
        let duplicate =
            native_test_discover(vec![("module".into(), case()), ("module".into(), case())])
                .err()
                .unwrap();
        assert_eq!(duplicate, "duplicate test name module::case");
        finish();
    }
}
