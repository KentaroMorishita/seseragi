// A language tuple dictionary keeps the selected method for every position.
// Captures are ordinary managed graph edges, including generic/user methods.
fn native_tuple_display_dictionary(elements: Vec<V>, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    let children = elements.iter().map(|child| child.field(method)).collect();
    let render = V::fun(children, |capture, value| {
        native_display(|| {
            let V::Tuple(elements) = value else {
                panic!("Rust ABI: expected tuple")
            };
            assert_eq!(
                elements.get().len(),
                capture.len(),
                "Rust ABI: tuple display arity"
            );
            let mut output = String::from("(");
            for (index, (child, value)) in capture.iter().zip(elements.get()).enumerate() {
                if index != 0 {
                    output.push_str(", ");
                }
                // Apply one selected method completely before the next. No
                // global, Ref, or collector storage is borrowed by this call.
                let rendered = child.apply(value.clone());
                let V::Text(text) = rendered else {
                    panic!("Rust ABI: display method must return String")
                };
                output.push_str(text.get());
            }
            output.push(')');
            V::text(&output)
        })
    });
    V::dict(vec![(method, render)])
}

#[cfg(test)]
mod tuple_tests {
    use super::*;

    #[test]
    fn tuple_methods_visit_in_order_stop_at_defects_and_release_captured_refs() {
        enable_cycle_collection();
        let log = V::reference(V::Int(0));
        let child = V::dict(vec![(
            "show",
            V::fun(vec![log.clone()], |capture, value| {
                let V::Int(previous) = capture[0].read_reference() else {
                    panic!()
                };
                let V::Int(n) = value else { panic!() };
                capture[0].write_reference(V::Int(previous * 10 + n));
                if n == 2 {
                    panic!("second child defect")
                }
                V::text(&format!("selected:{n}"))
            }),
        )]);
        let dictionary = native_tuple_display_dictionary(vec![child.clone(); 3], false);
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dictionary.field("show").apply(V::Tuple(Shared::new(vec![
                V::Int(1),
                V::Int(2),
                V::Int(3),
            ])))
        }));
        assert!(failed.is_err());
        assert_eq!(log.read_reference().render(), "12");
        assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
        log.write_reference(V::Int(0));
        let shown = dictionary.field("show").apply(V::Tuple(Shared::new(vec![
            V::Int(3),
            V::Int(4),
            V::Int(5),
        ])));
        assert_eq!(shown.render(), "(selected:3, selected:4, selected:5)");
        assert_eq!(log.read_reference().render(), "345");
        // A Ref may retain the dictionary whose methods capture that same Ref.
        // Collection must see those edges; explicit method captures keep them visible.
        log.write_reference(V::Tuple(Shared::new(vec![
            dictionary.clone(),
            child.clone(),
        ])));
        drop((shown, dictionary, child, log));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn tuple_and_list_display_share_the_standard_depth_bound() {
        fn expected(remaining: usize, depth: usize) -> String {
            if depth >= 128 {
                return "…".into();
            }
            if remaining == 0 {
                return "1".into();
            }
            let child = expected(remaining - 1, depth + 1);
            if remaining % 2 == 0 {
                format!("({child}, {})", if depth + 1 >= 128 { "…" } else { "0" })
            } else {
                format!("`[{child}]")
            }
        }
        for debug in [false, true] {
            let method = if debug { "debug" } else { "show" };
            let primitive = native_primitive_display_dictionary(if debug {
                "Debug<std/prelude::Int>"
            } else {
                "Show<std/prelude::Int>"
            });
            let mut value = V::Int(1);
            let mut dictionary = primitive.clone();
            for depth in 1..=130 {
                if depth % 2 == 0 {
                    value = V::Tuple(Shared::new(vec![value, V::Int(0)]));
                    dictionary =
                        native_tuple_display_dictionary(vec![dictionary, primitive.clone()], debug);
                } else {
                    value = V::list_cons(value, V::ListEmpty);
                    dictionary = native_list_display_dictionary(dictionary, debug);
                }
            }
            let output = dictionary.field(method).apply(value);
            assert_eq!(output.render(), expected(130, 0));
            assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
            drop((primitive, dictionary, output));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
