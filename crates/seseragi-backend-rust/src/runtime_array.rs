fn native_array_index(values: V, offset: V) -> V {
    let (V::Array(values), V::Int(offset)) = (values, offset) else {
        panic!("Rust ABI: expected Array and Int")
    };
    // Clone the selected edge before dropping the container. Functions, Refs,
    // and cold Effects keep their existing managed capture/collector graph.
    match usize::try_from(offset)
        .ok()
        .and_then(|index| values.get().get(index))
    {
        Some(value) => V::adt("std/prelude::Just", value.clone()),
        None => V::adt("std/prelude::Nothing", V::Unit),
    }
}

// Maybe display uses the checked child dictionary, including user/generic
// methods. Nothing skips the child; Just invokes it once at the shared depth.
fn native_maybe_display_dictionary(element: V, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    let render = V::fun(vec![element.field(method)], |capture, value| {
        native_display(|| {
            let V::Adt(value) = value else {
                panic!("Rust ABI: expected Maybe")
            };
            match value.get().0 {
                "std/prelude::Nothing" => V::text("Nothing"),
                "std/prelude::Just" => {
                    let rendered = capture[0].apply(value.get().1.clone());
                    let V::Text(text) = rendered else {
                        panic!("Rust ABI: display method must return String")
                    };
                    V::text(&format!("Just {}", text.get()))
                }
                _ => panic!("Rust ABI: expected canonical Maybe constructor"),
            }
        })
    });
    V::dict(vec![(method, render)])
}

// The selected element method is an explicit managed capture. Traversing an
// immutable Array holds no Ref, global, or collector borrow across user code.
fn native_array_display_dictionary(element: V, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    let render = V::fun(vec![element.field(method)], |capture, value| {
        native_display(|| {
            let V::Array(elements) = value else {
                panic!("Rust ABI: expected Array")
            };
            let mut output = String::from("[");
            for (index, value) in elements.get().iter().enumerate() {
                if index != 0 {
                    output.push_str(", ");
                }
                let rendered = capture[0].apply(value.clone());
                let V::Text(text) = rendered else {
                    panic!("Rust ABI: display method must return String")
                };
                output.push_str(text.get());
            }
            output.push(']');
            V::text(&output)
        })
    });
    V::dict(vec![(method, render)])
}

#[cfg(test)]
mod array_tests {
    use super::*;

    #[test]
    fn maybe_display_skips_nothing_uses_selected_methods_and_restores_depth() {
        enable_cycle_collection();
        for debug in [false, true] {
            let method = if debug { "debug" } else { "show" };
            let calls = V::reference(V::Int(0));
            let child = V::dict(vec![(
                method,
                V::fun(vec![calls.clone()], |capture, value| {
                    let V::Int(previous) = capture[0].read_reference() else {
                        panic!()
                    };
                    capture[0].write_reference(V::Int(previous + 1));
                    let V::Int(n) = value else { panic!() };
                    if n == 0 {
                        panic!("selected child defect")
                    }
                    V::text(&format!("selected:{n}"))
                }),
            )]);
            let dictionary = native_maybe_display_dictionary(child, debug);
            assert_eq!(
                dictionary
                    .field(method)
                    .apply(V::adt("std/prelude::Nothing", V::Unit))
                    .render(),
                "Nothing"
            );
            assert_eq!(calls.read_reference().render(), "0");
            assert_eq!(
                dictionary
                    .field(method)
                    .apply(V::adt("std/prelude::Just", V::Int(42)))
                    .render(),
                "Just selected:42"
            );
            assert_eq!(calls.read_reference().render(), "1");
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                dictionary
                    .field(method)
                    .apply(V::adt("std/prelude::Just", V::Int(0)))
            }));
            assert!(failed.is_err());
            assert_eq!(calls.read_reference().render(), "2");
            assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
            drop((dictionary, calls));
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn array_index_values_outlive_containers_and_keep_collector_edges() {
        enable_cycle_collection();
        let cell = V::reference(V::Int(41));
        let callback = V::fun(vec![cell.clone()], |captures, _| {
            captures[0].read_reference()
        });
        let values = V::Array(Shared::new(vec![callback.clone(), cell.clone()]));
        let selected = native_array_index(values.clone(), V::Int(0)).payload();
        let reference = native_array_index(values.clone(), V::Int(1)).payload();
        for offset in [i64::MIN, -1, 2, i64::MAX] {
            assert!(
                native_array_index(values.clone(), V::Int(offset)).tagged("std/prelude::Nothing")
            );
        }
        assert!(native_array_index(V::Array(Shared::new(vec![])), V::Int(0))
            .tagged("std/prelude::Nothing"));
        drop((values, callback));
        assert_eq!(selected.apply(V::Unit).render(), "41");
        reference.write_reference(V::Int(42));
        assert_eq!(selected.apply(V::Unit).render(), "42");
        // The selected edge roots a duplicate cyclic capture graph. Borrowing
        // the Ref must still defer collection; dropping all roots reclaims it.
        cell.write_reference(V::Array(Shared::new(vec![
            selected.clone(),
            selected.clone(),
        ])));
        let V::Reference(shared) = &reference else {
            panic!()
        };
        let borrowed = shared.get().borrow();
        assert!(collect_cycles().is_err());
        drop(borrowed);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((cell, reference, selected));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn array_methods_stop_at_defects_restore_depth_and_release_captured_cycles() {
        enable_cycle_collection();
        let log = V::reference(V::Int(0));
        for debug in [false, true] {
            let method = if debug { "debug" } else { "show" };
            let child = V::dict(vec![(
                method,
                V::fun(vec![log.clone()], |capture, value| {
                    let V::Int(previous) = capture[0].read_reference() else {
                        panic!()
                    };
                    let V::Int(n) = value else { panic!() };
                    capture[0].write_reference(V::Int(previous * 10 + n));
                    if n == 2 {
                        panic!("second element defect")
                    }
                    V::text(&format!("selected:{n}"))
                }),
            )]);
            let dictionary = native_array_display_dictionary(child.clone(), debug);
            log.write_reference(V::Int(0));
            let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                dictionary.field(method).apply(V::Array(Shared::new(vec![
                    V::Int(1),
                    V::Int(2),
                    V::Int(3),
                ])))
            }));
            assert!(failed.is_err());
            assert_eq!(log.read_reference().render(), "12");
            assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
            log.write_reference(V::Int(0));
            assert_eq!(
                dictionary
                    .field(method)
                    .apply(V::Array(Shared::new(vec![])))
                    .render(),
                "[]"
            );
            assert_eq!(log.read_reference().render(), "0");
            assert_eq!(
                dictionary
                    .field(method)
                    .apply(V::Array(Shared::new(vec![V::Int(3), V::Int(4)])))
                    .render(),
                "[selected:3, selected:4]"
            );
            assert_eq!(log.read_reference().render(), "34");
            // A rooted dictionary retains this Ref even when it points back to
            // duplicate copies of the dictionary and its selected method.
            log.write_reference(V::Array(Shared::new(vec![
                dictionary.clone(),
                dictionary.clone(),
                child.clone(),
            ])));
            assert_eq!(collect_cycles().unwrap().reclaimed, 0);
            log.write_reference(V::Int(0));
            drop((dictionary, child));
        }
        drop(log);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);

        let cell = V::reference(V::Unit);
        let child = V::dict(vec![(
            "show",
            V::fun(vec![cell.clone()], |capture, _| capture[0].read_reference()),
        )]);
        let dictionary = native_array_display_dictionary(child, false);
        cell.write_reference(dictionary.clone());
        let V::Reference(reference) = &cell else {
            panic!()
        };
        let guard = reference.get().borrow();
        assert!(collect_cycles().is_err());
        drop(guard);
        drop((cell, dictionary));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn array_tuple_and_list_display_share_the_depth_bound() {
        fn expected(remaining: usize, depth: usize) -> String {
            if depth >= 128 {
                return "…".into();
            }
            if remaining == 0 {
                return "1".into();
            }
            let child = expected(remaining - 1, depth + 1);
            match remaining % 3 {
                0 => format!("[{child}]"),
                1 => format!("`[{child}]"),
                _ => format!("({child}, {})", if depth + 1 >= 128 { "…" } else { "0" }),
            }
        }
        for debug in [false, true] {
            let primitive = native_primitive_display_dictionary(if debug {
                "Debug<std/prelude::Int>"
            } else {
                "Show<std/prelude::Int>"
            });
            let mut value = V::Int(1);
            let mut dictionary = primitive.clone();
            for depth in 1..=130 {
                match depth % 3 {
                    0 => {
                        value = V::Array(Shared::new(vec![value]));
                        dictionary = native_array_display_dictionary(dictionary, debug);
                    }
                    1 => {
                        value = V::list_cons(value, V::ListEmpty);
                        dictionary = native_list_display_dictionary(dictionary, debug);
                    }
                    _ => {
                        value = V::Tuple(Shared::new(vec![value, V::Int(0)]));
                        dictionary = native_tuple_display_dictionary(
                            vec![dictionary, primitive.clone()],
                            debug,
                        );
                    }
                }
            }
            let output = dictionary
                .field(if debug { "debug" } else { "show" })
                .apply(value);
            assert_eq!(output.render(), expected(130, 0));
            assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
            drop((primitive, dictionary, output));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn deeply_nested_array_dictionary_captures_drop_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut dictionary = native_primitive_display_dictionary("Show<std/prelude::Int>");
                let mut value = V::Int(1);
                for _ in 0..100_000 {
                    dictionary = native_array_display_dictionary(dictionary, false);
                    value = V::Array(Shared::new(vec![value]));
                }
                // Display itself is bounded; destroy the much deeper value and
                // method-capture graphs through the existing iterative drop queue.
                drop((dictionary, value));
                assert_eq!(LIVE.with(Cell::get), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
