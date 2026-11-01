#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeSource {
    file: &'static str,
    line: usize,
    column: usize,
}
impl std::fmt::Display for NativeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
#[derive(Clone, Debug)]
struct NativeDiagnostic<T> {
    value: T,
    source: Option<&'static NativeSource>,
}
impl<T> From<T> for NativeDiagnostic<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            source: None,
        }
    }
}
impl From<&str> for NativeDiagnostic<String> {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}
impl<T> std::ops::Deref for NativeDiagnostic<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl PartialEq<&str> for NativeDiagnostic<String> {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}
impl std::fmt::Display for NativeDiagnostic<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl NativeExit {
    fn defects(values: Vec<String>) -> Self {
        Self::Defect(values.into_iter().map(Into::into).collect())
    }
    fn failure(value: V) -> Self {
        Self::Failure(value.into())
    }
    fn at(mut self, source: Option<&'static NativeSource>) -> Self {
        match &mut self {
            Self::Failure(error) => {
                error.source = error.source.or(source);
            }
            Self::Defect(errors) => {
                for error in errors {
                    error.source = error.source.or(source);
                }
            }
            Self::Cancelled => {}
        }
        self
    }
}

#[cold]
#[inline(never)]
fn native_defect_at(message: &'static str, source: Option<&'static NativeSource>) -> ! {
    match source {
        Some(source) => std::panic::panic_any(NativeDiagnostic {
            value: message.to_owned(),
            source: Some(source),
        }),
        None => panic!("{message}"),
    }
}

fn native_source<T>(source: &'static NativeSource, action: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)).unwrap_or_else(|error| {
        let diagnostic = match native_panic(error).at(Some(source)) {
            NativeExit::Defect(mut errors) => errors.remove(0),
            _ => unreachable!(),
        };
        std::panic::resume_unwind(Box::new(diagnostic))
    })
}
fn native_source_value(source: &'static NativeSource, action: impl FnOnce() -> V) -> V {
    native_locate_value(source, native_source(source, action))
}
fn native_locate_value(source: &'static NativeSource, mut value: V) -> V {
    if let V::Effect(plan) = &mut value {
        if plan.get().source.is_none() {
            if let Some(node) = Rc::get_mut(plan.0.as_mut().expect("live shared handle")) {
                node.value.source = Some(source);
                return value;
            }
            let mut located = plan.get().clone();
            located.source = Some(source);
            return V::Effect(Shared::new(located));
        }
    }
    value
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    const INNER: &NativeSource = &NativeSource {
        file: "inner.ssrg",
        line: 3,
        column: 7,
    };
    const OUTER: &NativeSource = &NativeSource {
        file: "outer.ssrg",
        line: 9,
        column: 2,
    };

    #[test]
    fn arithmetic_attaches_source_only_to_the_operation_that_fails() {
        for operation in [
            || {
                int_add_value_at(9_007_199_254_740_991, 1, Some(INNER));
            },
            || {
                int_subtract_value_at(-9_007_199_254_740_991, 1, Some(INNER));
            },
            || {
                int_multiply_value_at(9_007_199_254_740_991, 2, Some(INNER));
            },
            || {
                int_divide_value_at(1, 0, Some(INNER));
            },
            || {
                int_remainder_value_at(1, 0, Some(INNER));
            },
            || {
                int_power_value_at(2, -1, Some(INNER));
            },
            || {
                int_negate_value_at(i64::MIN, Some(INNER));
            },
            || {
                get_global_at(0, INNER);
            },
        ] {
            let result = catch_native(|| {
                operation();
                Ok(V::Unit)
            });
            let Err(NativeExit::Defect(errors)) = result else {
                panic!()
            };
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].source, Some(INNER));
        }
        assert_eq!(int_add_value_at(20, 22, Some(INNER)), 42);
    }

    #[test]
    fn fresh_effect_locations_do_not_allocate_a_second_plan() {
        let plan = native_operation("effect.fail", vec![V::Int(7)]);
        let allocations = ALLOCATIONS.with(Cell::get);
        let plan = native_locate_value(INNER, plan);
        assert_eq!(ALLOCATIONS.with(Cell::get), allocations);
        let Err(NativeExit::Failure(error)) = NativeContext::default().root(plan) else {
            panic!()
        };
        assert_eq!(error.source, Some(INNER));
        drop(error);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn unwinding_keeps_the_innermost_location() {
        let result =
            catch_native(|| native_source(OUTER, || native_source(INNER, || panic!("boom"))));
        let Err(NativeExit::Defect(errors)) = result else {
            panic!()
        };
        assert_eq!(errors[0].value, "boom");
        assert_eq!(errors[0].source, Some(INNER));
    }

    #[test]
    fn saved_plans_are_cold_replayable_and_do_not_accumulate_source_frames() {
        let plan = native_source_value(INNER, || native_operation("effect.fail", vec![V::Int(7)]));
        let forwarded = native_source_value(OUTER, || plan.clone());
        for _ in 0..2 {
            let Err(NativeExit::Failure(error)) = NativeContext::default().root(forwarded.clone())
            else {
                panic!()
            };
            assert_eq!(error.source, Some(INNER));
            assert!(matches!(error.value, V::Int(7)));
        }
        let succeed =
            native_source_value(OUTER, || native_operation("effect.succeed", vec![V::Unit]));
        let mut context = NativeContext::default();
        assert!(context.root(succeed).is_ok());
        assert_eq!(context.peak_frames, 2);
        drop((plan, forwarded, context));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn located_defer_loops_keep_bounded_frames() {
        fn step(c: &[V], _: V) -> V {
            let V::Int(remaining) = c[0] else { panic!() };
            native_source_value(INNER, || {
                if remaining == 0 {
                    native_operation("effect.succeed", vec![V::Unit])
                } else {
                    native_operation(
                        "std/effect::defer",
                        vec![V::fun(vec![V::Int(remaining - 1)], step)],
                    )
                }
            })
        }
        let mut context = NativeContext::default();
        assert!(context.root(step(&[V::Int(10_000)], V::Unit)).is_ok());
        assert_eq!(context.peak_frames, 2);
        drop(context);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn recovered_failures_do_not_supply_the_location_of_a_later_failure() {
        let first = native_source_value(INNER, || native_operation("effect.fail", vec![V::Int(1)]));
        let handler = V::fun(vec![], |_, _| {
            native_source_value(OUTER, || native_operation("effect.fail", vec![V::Int(2)]))
        });
        let plan = native_operation("std/effect::recover", vec![handler, first]);
        let Err(NativeExit::Failure(error)) = NativeContext::default().root(plan) else {
            panic!()
        };
        assert!(matches!(error.value, V::Int(2)));
        assert_eq!(error.source, Some(OUTER));
    }

    #[test]
    fn cleanup_keeps_each_defect_location() {
        let mut plan = native_operation("effect.succeed", vec![V::Unit]);
        for source in [INNER, OUTER] {
            let release = V::fun(vec![V::Bool(source == INNER)], |c, _| {
                let source = if c[0].boolean() { INNER } else { OUTER };
                native_source_value(source, || {
                    V::leaf(vec![V::text(source.file)], |c, _| {
                        Err(NativeExit::defects(vec![c[0].render()]))
                    })
                })
            });
            let acquire = native_operation(
                "std/effect::acquireRelease",
                vec![native_operation("effect.succeed", vec![V::Unit]), release],
            );
            plan = V::bind(acquire, V::fun(vec![plan], |c, _| c[0].clone()));
        }
        let Err(NativeExit::Defect(errors)) = NativeContext::default().root(plan) else {
            panic!()
        };
        assert_eq!(
            errors.iter().map(|error| error.source).collect::<Vec<_>>(),
            [Some(INNER), Some(OUTER)]
        );
        assert_eq!(
            errors
                .iter()
                .map(|error| error.value.as_str())
                .collect::<Vec<_>>(),
            ["inner.ssrg", "outer.ssrg"]
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
