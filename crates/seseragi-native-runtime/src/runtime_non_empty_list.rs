fn native_non_empty_list_parts(head: V, tail: V) -> V {
    V::Record(Shared::new(BTreeMap::from([
        ("head".into(), head),
        ("tail".into(), tail),
    ])))
}

pub fn native_non_empty_list_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/non-empty-list::singleton", [head]) => {
            native_non_empty_list_parts(head.clone(), V::ListEmpty)
        }
        ("std/non-empty-list::cons", [head, tail]) => {
            native_non_empty_list_parts(head.clone(), tail.clone())
        }
        ("std/non-empty-list::fromList", [values]) => match values.list_cell() {
            None => V::adt("std/prelude::Nothing", V::Unit),
            Some(cell) => V::adt(
                "std/prelude::Just",
                native_non_empty_list_parts(cell.head.clone(), cell.tail.clone()),
            ),
        },
        ("std/non-empty-list::toList", [values]) => {
            V::list_cons(values.field("head"), values.field("tail"))
        }
        ("std/non-empty-list::head", [values]) => values.field("head"),
        ("std/non-empty-list::tail", [values]) => values.field("tail"),
        ("std/non-empty-list::reduce1", [step, values]) => {
            let mut accumulator = values.field("head");
            let tail = values.field("tail");
            let mut cursor = &tail;
            while let Some(cell) = cursor.list_cell() {
                accumulator = step.apply(accumulator).apply(cell.head.clone());
                cursor = &cell.tail;
            }
            accumulator
        }
        _ => panic!("Rust ABI: unsupported NonEmptyList operation or arity"),
    }
}

#[cfg(test)]
mod non_empty_list_tests {
    use super::*;

    fn operation(name: &str, arguments: Vec<V>) -> V {
        native_non_empty_list_operation(&format!("std/non-empty-list::{name}"), arguments)
    }

    #[test]
    fn conversions_share_the_original_tail_and_projection_releases_unselected_edges() {
        enable_cycle_collection();
        let head = V::reference(V::Int(1));
        let tail = V::list_from_values((2..100_002).map(V::Int).collect());
        let tail_id = tail.node_id().unwrap();
        let source = V::list_cons(head.clone(), tail.clone());
        let before = LIVE.with(Cell::get);
        let converted = operation("fromList", vec![source.clone()]);
        assert!(converted.tagged("std/prelude::Just"));
        assert_eq!(LIVE.with(Cell::get), before + 2);
        let values = converted.payload();
        assert_eq!(
            operation("tail", vec![values.clone()]).node_id(),
            Some(tail_id)
        );
        let rebuilt = operation("toList", vec![values.clone()]);
        assert_eq!(rebuilt.list_cell().unwrap().head.node_id(), head.node_id());
        assert_eq!(rebuilt.list_cell().unwrap().tail.node_id(), Some(tail_id));
        let cons = operation("cons", vec![head.clone(), tail.clone()]);
        assert_eq!(
            operation("tail", vec![cons.clone()]).node_id(),
            Some(tail_id)
        );
        let selected = operation("head", vec![values.clone()]);
        drop((source, converted, values, rebuilt, cons, tail, head));
        collect_cycles().unwrap();
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&tail_id)));
        assert_eq!(selected.read_reference().render(), "1");
        drop(selected);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn empty_conversion_and_singleton_preserve_unit_functions_and_skip_the_reducer() {
        assert!(operation("fromList", vec![V::ListEmpty]).tagged("std/prelude::Nothing"));
        for head in [V::Unit, V::fun(vec![], |_, value| value)] {
            let one = operation("singleton", vec![head]);
            assert!(matches!(operation("tail", vec![one.clone()]), V::ListEmpty));
            let result = operation(
                "reduce1",
                vec![
                    V::fun(vec![], |_, _| panic!("singleton reducer called")),
                    one,
                ],
            );
            if matches!(result, V::Unit) {
                assert_eq!(result.render(), "()");
            } else {
                assert_eq!(result.apply(V::Int(42)).render(), "42");
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    fn record(log: &V, event: V) {
        let mut events = log.read_reference().list_values();
        events.push(event);
        log.write_reference(V::list_from_values(events));
    }

    #[test]
    fn reduce1_calls_each_curried_stage_once_in_left_fold_order_and_stops_at_the_first_defect() {
        enable_cycle_collection();
        for defect in 0..3 {
            let log = V::reference(V::ListEmpty);
            let cycle = V::reference(V::Unit);
            let step = V::fun(
                vec![log.clone(), V::Int(defect), cycle.clone()],
                |captures, acc| {
                    record(&captures[0], V::text(&format!("acc:{}", acc.render())));
                    assert!(!matches!(captures[1], V::Int(1)), "first stage defect");
                    V::fun(
                        vec![captures[0].clone(), captures[1].clone(), acc],
                        |captures, value| {
                            record(&captures[0], V::text(&format!("value:{}", value.render())));
                            assert!(!matches!(captures[1], V::Int(2)), "second stage defect");
                            let (V::Int(acc), V::Int(value)) = (&captures[2], value) else {
                                panic!()
                            };
                            V::Int(acc - value)
                        },
                    )
                },
            );
            cycle.write_reference(step.clone());
            drop(cycle);
            let values = operation(
                "cons",
                vec![V::Int(10), V::list_from_values(vec![V::Int(3), V::Int(2)])],
            );
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                operation("reduce1", vec![step, values])
            }));
            let expected: &[&str] = match defect {
                0 => {
                    assert_eq!(result.unwrap().render(), "5");
                    &["acc:10", "value:3", "acc:7", "value:2"]
                }
                1 => {
                    assert!(result.is_err());
                    &["acc:10"]
                }
                _ => {
                    assert!(result.is_err());
                    &["acc:10", "value:3"]
                }
            };
            let events = log.read_reference();
            assert_eq!(
                events
                    .list_values()
                    .iter()
                    .map(V::render)
                    .collect::<Vec<_>>(),
                expected
            );
            drop((log, events));
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn reduce1_handles_large_lists_without_retaining_the_source() {
        enable_cycle_collection();
        let source = V::list_from_values((1..100_001).map(V::Int).collect());
        let values = operation("fromList", vec![source]).payload();
        let step = V::fun(vec![], |_, acc| {
            V::fun(vec![acc], |captures, value| {
                let (V::Int(acc), V::Int(value)) = (&captures[0], value) else {
                    panic!()
                };
                V::Int(acc + value)
            })
        });
        let total = operation("reduce1", vec![step, values]);
        assert_eq!(total.render(), "5000050000");
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
