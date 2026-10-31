// A persistent pull iterator is a traced closure over its step and initial
// state. Constructing a value performs no step; observing it never consumes it.
fn native_iterator_unfold(step: V, initial: V) -> V {
    V::dict(vec![(
        "next",
        V::fun(vec![step, initial], |captures, unit| {
            assert!(matches!(unit, V::Unit), "Rust ABI: Iterator next Unit");
            let result = captures[0].apply(captures[1].clone());
            if result.tagged("std/prelude::Nothing") {
                return result;
            }
            assert!(
                result.tagged("std/prelude::Just"),
                "Rust ABI: Iterator step Maybe"
            );
            let V::Tuple(pair) = result.payload() else {
                panic!("Rust ABI: Iterator step pair")
            };
            let [value, state] = pair.get().as_slice() else {
                panic!("Rust ABI: Iterator step pair arity")
            };
            V::adt(
                "std/prelude::Just",
                V::Tuple(Shared::new(vec![
                    value.clone(),
                    native_iterator_unfold(captures[0].clone(), state.clone()),
                ])),
            )
        }),
    )])
}
fn native_iterator_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/iterator::unfold" | "std/prelude::unfold", [step, initial]) => {
            native_iterator_unfold(step.clone(), initial.clone())
        }
        ("std/iterator::next" | "std/prelude::next", [iterator]) => {
            iterator.field("next").apply(V::Unit)
        }
        _ => panic!("Rust ABI: Iterator operation or source arity"),
    }
}

#[cfg(test)]
mod iterator_tests {
    use super::*;
    fn step(value: V) -> Option<(V, V)> {
        let result = native_iterator_operation("std/iterator::next", vec![value]);
        if result.tagged("std/prelude::Nothing") {
            return None;
        }
        let V::Tuple(pair) = result.payload() else {
            panic!()
        };
        Some((pair.get()[0].clone(), pair.get()[1].clone()))
    }
    #[test]
    fn unfold_is_lazy_next_replays_the_same_state_and_only_returned_tails_advance() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let callback = V::fun(vec![calls.clone()], |c, state| {
            let V::Int(count) = c[0].read_reference() else {
                panic!()
            };
            c[0].write_reference(V::Int(count + 1));
            let V::Int(n) = state else { panic!() };
            if n == 3 {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::adt(
                    "std/prelude::Just",
                    V::Tuple(Shared::new(vec![V::text(&format!("雪{n}")), V::Int(n + 1)])),
                )
            }
        });
        let initial =
            native_iterator_operation("std/iterator::unfold", vec![callback.clone(), V::Int(0)]);
        assert_eq!(calls.read_reference().render(), "0");
        let (first, rest) = step(initial.clone()).unwrap();
        assert_eq!(first.render(), "雪0");
        let (again, replayed) = step(initial.clone()).unwrap();
        assert_eq!(again.render(), "雪0");
        let (second, last) = step(rest.clone()).unwrap();
        assert_eq!(second.render(), "雪1");
        let (third, end) = step(last.clone()).unwrap();
        assert_eq!(third.render(), "雪2");
        assert!(step(end.clone()).is_none());
        assert!(step(end.clone()).is_none());
        assert_eq!(calls.read_reference().render(), "6");
        drop((
            calls, callback, initial, first, rest, again, replayed, second, last, third, end,
        ));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn iterator_captures_and_yielded_functions_are_traced_and_defects_release_ref_cycles() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let callback = V::fun(vec![cell.clone()], |c, state| {
            let V::Int(n) = state else { panic!() };
            assert_ne!(n, 1, "deliberate step defect");
            let retained = V::fun(vec![c[0].clone()], |_, _| V::Int(42));
            V::adt(
                "std/prelude::Just",
                V::Tuple(Shared::new(vec![retained, V::Int(1)])),
            )
        });
        let initial = native_iterator_unfold(callback.clone(), V::Int(0));
        cell.write_reference(initial.clone());
        let (value, rest) = step(initial.clone()).unwrap();
        assert_eq!(value.apply(V::Unit).render(), "42");
        let rest_id = rest.node_id().unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| step(rest.clone()))).is_err()
        );
        drop((cell, callback, initial, rest));
        collect_cycles().unwrap();
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&rest_id)));
        assert_eq!(value.apply(V::Unit).render(), "42");
        drop(value);
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn long_pull_chain_keeps_bounded_live_state_and_releases_previous_steps() {
        enable_cycle_collection();
        let callback = V::fun(vec![], |_, state| {
            let V::Int(n) = state else { panic!() };
            V::adt(
                "std/prelude::Just",
                V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
            )
        });
        let mut iterator = native_iterator_unfold(callback, V::Int(0));
        for n in 0..100_000 {
            let (value, next) = step(iterator).unwrap();
            assert_eq!(value.render(), n.to_string());
            iterator = next;
            assert!(LIVE.with(Cell::get) < 16);
        }
        drop(iterator);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
