fn native_range(start: V, end: V, inclusive: bool) -> V {
    assert!(
        matches!((&start, &end), (V::Int(_), V::Int(_))),
        "Rust ABI: Range Int bounds"
    );
    V::Record(Shared::new(BTreeMap::from([
        ("start".into(), start),
        ("end".into(), end),
        ("inclusive".into(), V::Bool(inclusive)),
    ])))
}

fn native_range_iterator(range: V) -> V {
    let start = range.field("start");
    native_iterator_unfold(
        V::fun(
            vec![range.field("end"), range.field("inclusive")],
            |c, state| {
                if matches!(state, V::Unit) {
                    return V::adt("std/prelude::Nothing", V::Unit);
                }
                let (V::Int(current), V::Int(end)) = (&state, &c[0]) else {
                    panic!("Rust ABI: Range Int cursor")
                };
                if if c[1].boolean() {
                    current > end
                } else {
                    current >= end
                } {
                    return V::adt("std/prelude::Nothing", V::Unit);
                }
                let rest = if current == end {
                    V::Unit
                } else {
                    checked_int(current + 1)
                };
                V::adt(
                    "std/prelude::Just",
                    V::Tuple(Shared::new(vec![state, rest])),
                )
            },
        ),
        start,
    )
}

fn native_range_display_dictionary(bound: V, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    V::dict(vec![(
        method,
        V::fun(vec![bound.field(method)], |c, range| {
            native_display(|| {
                let start = c[0].apply(range.field("start"));
                let end = c[0].apply(range.field("end"));
                let (V::Text(start), V::Text(end)) = (start, end) else {
                    panic!("Rust ABI: display method must return String")
                };
                V::text(&format!(
                    "{}{}{}",
                    start.get(),
                    if range.field("inclusive").boolean() {
                        "..="
                    } else {
                        ".."
                    },
                    end.get()
                ))
            })
        }),
    )])
}

#[cfg(test)]
mod range_tests {
    use super::*;

    #[test]
    fn cursors_are_lazy_persistent_and_stop_at_safe_int_bounds() {
        for (start, end, inclusive, expected) in [
            (2, 2, false, vec![]),
            (2, 2, true, vec![2]),
            (3, 1, false, vec![]),
            (3, 1, true, vec![]),
            (-1, 2, false, vec![-1, 0, 1]),
            (-1, 2, true, vec![-1, 0, 1, 2]),
            (
                9007199254740990,
                9007199254740991,
                true,
                vec![9007199254740990, 9007199254740991],
            ),
            (
                -9007199254740991,
                -9007199254740990,
                false,
                vec![-9007199254740991],
            ),
        ] {
            let range = native_range(V::Int(start), V::Int(end), inclusive);
            let mut iterator = native_range_iterator(range);
            for value in expected {
                for _ in 0..2 {
                    let (head, _) = native_prelude_pull(iterator.clone()).unwrap();
                    let V::Int(head) = head else { panic!() };
                    assert_eq!(head, value);
                }
                iterator = native_prelude_pull(iterator).unwrap().1;
            }
            assert!(native_prelude_pull(iterator.clone()).is_none());
            assert!(native_prelude_pull(iterator).is_none());
            assert_eq!(LIVE.with(Cell::get), 0);
        }
        let before = ALLOCATIONS.with(Cell::get);
        let cursor = native_range_iterator(native_range(
            V::Int(-9007199254740991),
            V::Int(9007199254740991),
            true,
        ));
        assert!(ALLOCATIONS.with(Cell::get) - before <= 4);
        let (head, rest) = native_prelude_pull(cursor).unwrap();
        assert_eq!(head.render(), "-9007199254740991");
        assert_eq!(
            native_prelude_pull(rest).unwrap().0.render(),
            "-9007199254740990"
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn hundred_thousand_elements_bound_frames_and_live_memory() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                enable_cycle_collection();
                let total = V::reference(V::Int(0));
                let count = V::reference(V::Int(0));
                let plan = native_prelude_collection(
                    "std/prelude::forEach",
                    vec![native_iterable_dictionary("std/range::Iterable")],
                    vec![
                        V::fun(vec![total.clone(), count.clone()], |c, value| {
                            V::leaf(vec![c[0].clone(), c[1].clone(), value], |c, _| {
                                c[0].write_reference(binary(
                                    "+",
                                    c[0].read_reference(),
                                    c[2].clone(),
                                ));
                                c[1].write_reference(binary("+", c[1].read_reference(), V::Int(1)));
                                Ok(V::Unit)
                            })
                        }),
                        native_range(V::Int(1), V::Int(100_000), true),
                    ],
                );
                let baseline = LIVE.with(Cell::get);
                PEAK.with(|peak| peak.set(baseline));
                let mut context = NativeContext::default();
                assert!(matches!(context.root(plan), Ok(V::Unit)));
                assert_eq!(total.read_reference().render(), "5000050000");
                assert_eq!(count.read_reference().render(), "100000");
                assert!(context.peak_frames <= 2, "{}", context.peak_frames);
                assert!(
                    PEAK.with(Cell::get) <= baseline + 32,
                    "peak={} baseline={baseline}",
                    PEAK.with(Cell::get)
                );
                drop((context, total, count));
                clear_dictionaries();
                assert_eq!(collect_cycles().unwrap().reclaimed, 0);
                assert_eq!(LIVE.with(Cell::get), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
