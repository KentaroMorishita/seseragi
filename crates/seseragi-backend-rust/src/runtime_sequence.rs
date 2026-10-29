// Borrow immutable sequence edges while invoking callbacks; no Ref/collector
// borrow survives user code. Only included edges enter each strict result.
enum NativeSequenceIter<'a> {
    Array(std::slice::Iter<'a, V>),
    List(&'a V),
}
impl<'a> Iterator for NativeSequenceIter<'a> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Array(values) => values.next(),
            Self::List(cursor) => {
                let value = *cursor;
                let cell = value.list_cell()?;
                *cursor = &cell.tail;
                Some(&cell.head)
            }
        }
    }
}
fn native_sequence_iter(value: &V, list: bool) -> NativeSequenceIter<'_> {
    if list {
        NativeSequenceIter::List(value)
    } else {
        let V::Array(values) = value else {
            panic!("Rust ABI: expected Array")
        };
        NativeSequenceIter::Array(values.get().iter())
    }
}
fn native_sequence_collect(values: Vec<V>, list: bool) -> V {
    if list {
        V::list_from_values(values)
    } else {
        V::Array(Shared::new(values))
    }
}
fn native_sequence_operation(name: &str, arguments: Vec<V>) -> V {
    if name == "std/collection::NonPositiveSize" {
        let [V::Int(size)] = arguments.as_slice() else {
            panic!("Rust ABI: SizeError constructor")
        };
        return V::adt("std/collection::NonPositiveSize", V::Int(*size));
    }
    let (owner, operation) = name.split_once("::").expect("canonical sequence name");
    let list = match owner {
        "std/list" => true,
        "std/array" => false,
        _ => panic!("Rust ABI: sequence owner"),
    };
    let input = arguments.last().expect("sequence input");
    let nothing = || V::adt("std/prelude::Nothing", V::Unit);
    let just = |value| V::adt("std/prelude::Just", value);
    match operation {
        "last" => {
            let value = if list {
                native_sequence_iter(input, true).last()
            } else {
                let V::Array(values) = input else {
                    panic!("Rust ABI: expected Array")
                };
                values.get().last()
            };
            value.map_or_else(nothing, |v| just(v.clone()))
        }
        "init" => {
            let mut iter = native_sequence_iter(input, list).peekable();
            if iter.peek().is_none() {
                return nothing();
            }
            let mut output = Vec::new();
            while let Some(value) = iter.next() {
                if iter.peek().is_some() {
                    output.push(value.clone());
                }
            }
            just(native_sequence_collect(output, list))
        }
        "findIndex" | "takeWhile" | "dropWhile" => {
            let predicate = &arguments[0];
            let mut output = Vec::new();
            let mut iter = native_sequence_iter(input, list).enumerate();
            while let Some((index, value)) = iter.next() {
                let matched = predicate.apply(value.clone()).boolean();
                if operation == "findIndex" {
                    if matched {
                        return just(checked_int(
                            i64::try_from(index).expect("sequence index fits Int"),
                        ));
                    }
                } else if !matched {
                    if operation == "takeWhile" {
                        break;
                    }
                    if list {
                        return input.list_rest(index);
                    }
                    output.push(value.clone());
                    output.extend(iter.map(|(_, v)| v.clone()));
                    return native_sequence_collect(output, list);
                } else if operation == "takeWhile" {
                    output.push(value.clone());
                }
            }
            if operation == "findIndex" {
                nothing()
            } else {
                native_sequence_collect(output, list)
            }
        }
        "reduceRight" => {
            let values: Vec<_> = native_sequence_iter(input, list).collect();
            let mut result = arguments[0].clone();
            for value in values.into_iter().rev() {
                result = arguments[1].apply(value.clone()).apply(result);
            }
            result
        }
        "zip" | "zipWith" => {
            let right = &arguments[if operation == "zip" { 0 } else { 1 }];
            let mut result = Vec::new();
            for (a, b) in native_sequence_iter(input, list).zip(native_sequence_iter(right, list)) {
                result.push(if operation == "zip" {
                    V::Tuple(Shared::new(vec![a.clone(), b.clone()]))
                } else {
                    arguments[0].apply(a.clone()).apply(b.clone())
                });
            }
            native_sequence_collect(result, list)
        }
        "unzip" => {
            let mut left = Vec::new();
            let mut right = Vec::new();
            for pair in native_sequence_iter(input, list) {
                let V::Tuple(parts) = pair else {
                    panic!("Rust ABI: unzip requires pair")
                };
                let [a, b] = parts.get().as_slice() else {
                    panic!("Rust ABI: unzip pair arity")
                };
                left.push(a.clone());
                right.push(b.clone());
            }
            V::Tuple(Shared::new(vec![
                native_sequence_collect(left, list),
                native_sequence_collect(right, list),
            ]))
        }
        "chunksOf" | "windows" => {
            let V::Int(size) = arguments[0] else {
                panic!("Rust ABI: sequence size")
            };
            if size <= 0 {
                return native_left(V::adt("std/collection::NonPositiveSize", V::Int(size)));
            }
            let size = usize::try_from(size).unwrap_or(usize::MAX);
            let mut result = Vec::new();
            if operation == "chunksOf" {
                let mut chunk = Vec::new();
                for value in native_sequence_iter(input, list) {
                    chunk.push(value.clone());
                    if chunk.len() == size {
                        result.push(native_sequence_collect(std::mem::take(&mut chunk), list));
                    }
                }
                if !chunk.is_empty() {
                    result.push(native_sequence_collect(chunk, list));
                }
            } else {
                let values: Vec<_> = native_sequence_iter(input, list).collect();
                if size <= values.len() {
                    for window in values.windows(size) {
                        result.push(native_sequence_collect(
                            window.iter().map(|v| (*v).clone()).collect(),
                            list,
                        ));
                    }
                }
            }
            native_right(native_sequence_collect(result, list))
        }
        _ => panic!("Rust ABI: unsupported sequence operation"),
    }
}

#[cfg(test)]
mod sequence_tests {
    use super::*;
    fn seq(list: bool, values: &[i64]) -> V {
        native_sequence_collect(values.iter().copied().map(V::Int).collect(), list)
    }
    fn values(value: &V, list: bool) -> Vec<i64> {
        native_sequence_iter(value, list)
            .map(|v| {
                let V::Int(v) = v else { panic!() };
                *v
            })
            .collect()
    }
    fn name(list: bool, operation: &str) -> String {
        format!("std/{}::{operation}", if list { "list" } else { "array" })
    }

    #[test]
    fn predicates_short_circuit_in_source_order_and_fold_runs_right_to_left() {
        enable_cycle_collection();
        for list in [false, true] {
            let input = seq(list, &[1, 2, 3, 4]);
            let log = V::reference(V::Int(0));
            for operation in ["findIndex", "takeWhile", "dropWhile"] {
                log.write_reference(V::Int(0));
                let predicate = V::fun(
                    vec![log.clone(), V::Bool(operation == "findIndex")],
                    |c, v| {
                        let (V::Int(old), V::Int(v)) = (c[0].read_reference(), v) else {
                            panic!()
                        };
                        c[0].write_reference(V::Int(old * 10 + v));
                        V::Bool(if c[1].boolean() { v == 2 } else { v < 3 })
                    },
                );
                let output = native_sequence_operation(
                    &name(list, operation),
                    vec![predicate.clone(), input.clone()],
                );
                if operation == "findIndex" {
                    assert_eq!(output.payload().render(), "1");
                    assert_eq!(log.read_reference().render(), "12");
                } else {
                    assert_eq!(
                        values(&output, list),
                        if operation == "takeWhile" {
                            vec![1, 2]
                        } else {
                            vec![3, 4]
                        }
                    );
                    assert_eq!(log.read_reference().render(), "123");
                }
                native_sequence_operation(&name(list, operation), vec![predicate, seq(list, &[])]);
            }
            log.write_reference(V::Int(0));
            let step = V::fun(vec![log.clone()], |c, v| {
                let (V::Int(old), V::Int(v)) = (c[0].read_reference(), v) else {
                    panic!()
                };
                c[0].write_reference(V::Int(old * 10 + v));
                V::fun(vec![V::Int(v)], |c, a| {
                    let (V::Int(v), V::Int(a)) = (&c[0], a) else {
                        panic!()
                    };
                    V::Int(*v - a)
                })
            });
            assert_eq!(
                native_sequence_operation(
                    &name(list, "reduceRight"),
                    vec![V::Int(0), step.clone(), input.clone()]
                )
                .render(),
                "-2"
            );
            assert_eq!(log.read_reference().render(), "4321");
            assert_eq!(
                native_sequence_operation(
                    &name(list, "reduceRight"),
                    vec![V::Int(9), step, seq(list, &[])]
                )
                .render(),
                "9"
            );
            assert_eq!(log.read_reference().render(), "4321");
            assert_eq!(values(&input, list), vec![1, 2, 3, 4]);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn zip_uses_shorter_length_and_left_then_right_tuple_order() {
        for list in [false, true] {
            let left = seq(list, &[1, 2, 3]);
            let right = seq(list, &[4, 5]);
            let zipped =
                native_sequence_operation(&name(list, "zip"), vec![right.clone(), left.clone()]);
            let unzipped = native_sequence_operation(&name(list, "unzip"), vec![zipped]);
            let V::Tuple(parts) = &unzipped else { panic!() };
            assert_eq!(values(&parts.get()[0], list), vec![1, 2]);
            assert_eq!(values(&parts.get()[1], list), vec![4, 5]);
            let f = V::fun(vec![], |_, a| {
                V::fun(vec![a], |c, b| {
                    let (V::Int(a), V::Int(b)) = (&c[0], b) else {
                        panic!()
                    };
                    V::Int(a * 10 + b)
                })
            });
            assert_eq!(
                values(
                    &native_sequence_operation(&name(list, "zipWith"), vec![f, right, left]),
                    list
                ),
                vec![14, 25]
            );
            assert!(values(
                &native_sequence_operation(
                    &name(list, "zip"),
                    vec![seq(list, &[]), seq(list, &[1])]
                ),
                list
            )
            .is_empty());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn chunks_windows_and_size_error_cover_empty_large_and_nonpositive_sizes() {
        for list in [false, true] {
            for operation in ["chunksOf", "windows"] {
                for size in [i64::MIN, -1, 0] {
                    let output = native_sequence_operation(
                        &name(list, operation),
                        vec![V::Int(size), seq(list, &[1, 2])],
                    );
                    assert!(output.tagged("std/prelude::Left"));
                    assert!(output.payload().tagged("std/collection::NonPositiveSize"));
                    assert_eq!(output.payload().payload().render(), size.to_string());
                }
                for size in [1, 2, 5, i64::MAX] {
                    let empty = native_sequence_operation(
                        &name(list, operation),
                        vec![V::Int(size), seq(list, &[])],
                    );
                    assert!(empty.tagged("std/prelude::Right"));
                    assert!(values(&empty.payload(), list).is_empty());
                }
                let output = native_sequence_operation(
                    &name(list, operation),
                    vec![V::Int(2), seq(list, &[1, 2, 3, 4, 5])],
                )
                .payload();
                let nested: Vec<_> = native_sequence_iter(&output, list)
                    .map(|v| values(v, list))
                    .collect();
                assert_eq!(
                    nested,
                    if operation == "chunksOf" {
                        vec![vec![1, 2], vec![3, 4], vec![5]]
                    } else {
                        vec![vec![1, 2], vec![2, 3], vec![3, 4], vec![4, 5]]
                    }
                );
            }
            for operation in ["last", "init"] {
                assert!(
                    native_sequence_operation(&name(list, operation), vec![seq(list, &[])])
                        .tagged("std/prelude::Nothing")
                );
            }
            assert_eq!(
                native_sequence_operation(&name(list, "last"), vec![seq(list, &[1, 2, 3])])
                    .payload()
                    .render(),
                "3"
            );
            assert_eq!(
                values(
                    &native_sequence_operation(&name(list, "init"), vec![seq(list, &[1, 2, 3])])
                        .payload(),
                    list
                ),
                vec![1, 2]
            );
            assert!(values(
                &native_sequence_operation(&name(list, "init"), vec![seq(list, &[1])]).payload(),
                list
            )
            .is_empty());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn selected_results_keep_no_excluded_edges_or_hidden_list_prefix_or_suffix() {
        enable_cycle_collection();
        for list in [false, true] {
            for operation in ["windows", "init", "last", "takeWhile", "dropWhile"] {
                let refs: Vec<_> = (0..5).map(|n| V::reference(V::Int(n))).collect();
                let ids: Vec<_> = refs.iter().map(|v| v.node_id().unwrap()).collect();
                let input = native_sequence_collect(refs.clone(), list);
                let predicate = V::fun(vec![], |_, value| {
                    let V::Int(n) = value.read_reference() else {
                        panic!()
                    };
                    V::Bool(n < 2)
                });
                let selected = match operation {
                    "windows" => {
                        let all = native_sequence_operation(
                            &name(list, operation),
                            vec![V::Int(2), input.clone()],
                        )
                        .payload();
                        native_sequence_iter(&all, list).next().unwrap().clone()
                    }
                    "init" | "last" => {
                        native_sequence_operation(&name(list, operation), vec![input.clone()])
                            .payload()
                    }
                    _ => native_sequence_operation(
                        &name(list, operation),
                        vec![predicate.clone(), input.clone()],
                    ),
                };
                drop((refs, input, predicate));
                collect_cycles().unwrap();
                for (index, id) in ids.iter().enumerate() {
                    let included = match operation {
                        "windows" | "takeWhile" => index < 2,
                        "dropWhile" => index >= 2,
                        "init" => index < 4,
                        "last" => index == 4,
                        _ => unreachable!(),
                    };
                    assert_eq!(
                        NODES.with(|n| n.borrow().contains_key(id)),
                        included,
                        "{list}/{operation}/{index}"
                    );
                }
                drop(selected);
                collect_cycles().unwrap();
                assert_eq!(LIVE.with(Cell::get), 0);
            }
        }
    }

    #[test]
    fn callback_defects_release_partial_results_and_captured_ref_cycles() {
        enable_cycle_collection();
        for list in [false, true] {
            let cell = V::reference(V::Unit);
            let callback = V::fun(vec![cell.clone()], |_, v| {
                let V::Int(v) = v else { panic!() };
                assert_ne!(v, 2, "deliberate callback defect");
                V::fun(vec![], |_, v| v)
            });
            let cycle = native_sequence_collect(vec![callback.clone()], list);
            cell.write_reference(cycle.clone());
            let input = seq(list, &[1, 2, 3]);
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_sequence_operation(
                    &name(list, "zipWith"),
                    vec![callback.clone(), input.clone(), input.clone()],
                )
            }));
            assert!(failed.is_err());
            drop((callback, cell, cycle, input));
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
