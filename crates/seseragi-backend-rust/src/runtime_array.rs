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
