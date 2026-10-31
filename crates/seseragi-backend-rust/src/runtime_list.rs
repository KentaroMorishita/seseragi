// Persistent lists own one head and one shared tail per cons. Empty is immediate.
struct ListCell {
    head: V,
    tail: V,
}
impl V {
    fn list_cons(head: V, tail: V) -> V {
        // Do not inspect, traverse, or copy the tail when constructing a node.
        V::List(Shared::new(ListCell { head, tail }))
    }
    fn list_from_values(values: Vec<V>) -> V {
        values
            .into_iter()
            .rev()
            .fold(V::ListEmpty, |tail, head| Self::list_cons(head, tail))
    }
    fn list_cell(&self) -> Option<&ListCell> {
        match self {
            V::ListEmpty => None,
            V::List(cell) => Some(cell.get()),
            _ => panic!("Rust ABI: expected List"),
        }
    }
    fn list_rest_ref(&self, count: usize) -> &V {
        let mut cursor = self;
        for _ in 0..count {
            cursor = &cursor.list_cell().expect("checked List projection").tail;
        }
        cursor
    }
    fn list_rest(&self, count: usize) -> V {
        self.list_rest_ref(count).clone()
    }
    fn list_element(&self, index: usize) -> V {
        self.list_rest_ref(index)
            .list_cell()
            .expect("checked List element")
            .head
            .clone()
    }
    fn list_length_matches(&self, length: usize, minimum: bool) -> bool {
        let mut cursor = self;
        for _ in 0..length {
            let Some(cell) = cursor.list_cell() else {
                return false;
            };
            cursor = &cell.tail;
        }
        minimum || cursor.list_cell().is_none()
    }
    fn list_values(&self) -> Vec<V> {
        let mut values = Vec::new();
        let mut cursor = self;
        while let Some(cell) = cursor.list_cell() {
            values.push(cell.head.clone());
            cursor = &cell.tail;
        }
        values
    }
}

fn native_list_operation(name: &str, arguments: Vec<V>) -> V {
    match name {
        "std/prelude:::" => V::list_cons(arguments[0].clone(), arguments[1].clone()),
        "std/list::empty" => V::ListEmpty,
        "std/list::singleton" => V::list_cons(arguments[0].clone(), V::ListEmpty),
        "std/array::toList" => {
            let V::Array(values) = &arguments[0] else {
                panic!("Rust ABI: expected Array")
            };
            V::list_from_values(values.get().clone())
        }
        "std/list::toArray" => V::Array(Shared::new(arguments[0].list_values())),
        "std/list::isEmpty" => V::Bool(arguments[0].list_cell().is_none()),
        "std/list::length" => {
            let mut length = 0;
            let mut cursor = &arguments[0];
            while let Some(cell) = cursor.list_cell() {
                length += 1;
                cursor = &cell.tail;
            }
            checked_int(length)
        }
        "std/list::head" | "std/list::tail" => match arguments[0].list_cell() {
            None => V::adt("std/prelude::Nothing", V::Unit),
            Some(cell) => V::adt(
                "std/prelude::Just",
                if name == "std/list::head" {
                    cell.head.clone()
                } else {
                    cell.tail.clone()
                },
            ),
        },
        "std/list::get" | "std/list::take" | "std/list::drop" => {
            let V::Int(count) = arguments[0] else {
                panic!("Rust ABI: expected Int")
            };
            let mut cursor = &arguments[1];
            let mut remaining = count.max(0);
            let mut prefix = Vec::new();
            while remaining > 0 {
                let Some(cell) = cursor.list_cell() else {
                    break;
                };
                if name == "std/list::take" {
                    prefix.push(cell.head.clone());
                }
                remaining -= 1;
                cursor = &cell.tail;
            }
            match name {
                "std/list::take" => V::list_from_values(prefix),
                "std/list::drop" => cursor.clone(),
                _ => {
                    if count >= 0 && remaining == 0 {
                        if let Some(cell) = cursor.list_cell() {
                            return V::adt("std/prelude::Just", cell.head.clone());
                        }
                    }
                    V::adt("std/prelude::Nothing", V::Unit)
                }
            }
        }
        "std/list::reverse" => {
            let mut result = V::ListEmpty;
            let mut cursor = &arguments[0];
            while let Some(cell) = cursor.list_cell() {
                result = V::list_cons(cell.head.clone(), result);
                cursor = &cell.tail;
            }
            result
        }
        "std/list::append" => {
            // Source API is suffix first, then the original prefix.
            let values = arguments[1].list_values();
            values
                .into_iter()
                .rev()
                .fold(arguments[0].clone(), |tail, head| V::list_cons(head, tail))
        }
        "std/list::filter" | "std/list::find" => {
            let callback = &arguments[0];
            let mut cursor = &arguments[1];
            let mut kept = Vec::new();
            while let Some(cell) = cursor.list_cell() {
                if callback.apply(cell.head.clone()).boolean() {
                    if name == "std/list::find" {
                        return V::adt("std/prelude::Just", cell.head.clone());
                    }
                    kept.push(cell.head.clone());
                }
                cursor = &cell.tail;
            }
            if name == "std/list::find" {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::list_from_values(kept)
            }
        }
        _ => panic!("Rust ABI: unsupported List operation"),
    }
}

thread_local! {
    static DISPLAY_DEPTH: Cell<usize> = const { Cell::new(0) };
}
// Standard display dictionaries share TS's 128-level document construction
// bound. User methods keep their own recursion policy, as on the TS target.
fn native_display(render: impl FnOnce() -> V) -> V {
    if DISPLAY_DEPTH.with(Cell::get) >= 128 {
        return V::text("…");
    }
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            DISPLAY_DEPTH.with(|depth| depth.set(depth.get() - 1));
        }
    }
    DISPLAY_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let _restore = Restore;
    render()
}

fn native_list_display_dictionary(element: V, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    let child = element.field(method);
    let render = V::fun(vec![child], |capture, values| {
        native_display(|| {
            let mut output = String::from("`[");
            let mut cursor = &values;
            let mut first = true;
            while let Some(cell) = cursor.list_cell() {
                if !first {
                    output.push_str(", ");
                }
                first = false;
                output.push_str(&capture[0].apply(cell.head.clone()).render());
                cursor = &cell.tail;
            }
            output.push(']');
            V::text(&output)
        })
    });
    V::dict(vec![(method, render)])
}

fn native_primitive_display_dictionary(identity: &'static str) -> V {
    if identity.ends_with("::Never>") {
        return V::dict(vec![(
            if identity.starts_with("Debug<") {
                "debug"
            } else {
                "show"
            },
            V::fun(vec![], |_, _| {
                panic!("Rust ABI: Never display received a value")
            }),
        )]);
    }
    let debug = identity.starts_with("Debug<");
    let method = if debug { "debug" } else { "show" };
    let string = identity.ends_with("::String>");
    V::dict(vec![(
        method,
        V::fun(vec![V::Bool(debug && string)], |capture, value| {
            native_display(|| {
                if !capture[0].boolean() {
                    return V::text(&value.render());
                }
                let V::Text(text) = value else {
                    panic!("Rust ABI: expected String")
                };
                let mut output = String::from("\"");
                for c in text.get().chars() {
                    match c {
                        '\\' => output.push_str("\\\\"),
                        '"' => output.push_str("\\\""),
                        '\0' => output.push_str("\\0"),
                        '\u{8}' => output.push_str("\\b"),
                        '\t' => output.push_str("\\t"),
                        '\n' => output.push_str("\\n"),
                        '\u{c}' => output.push_str("\\f"),
                        '\r' => output.push_str("\\r"),
                        c if u32::from(c) < 0x20 || (0x7f..=0x9f).contains(&u32::from(c)) => {
                            output.push_str(&format!("\\u{{{:X}}}", u32::from(c)));
                        }
                        c => output.push(c),
                    }
                }
                output.push('"');
                V::text(&output)
            })
        }),
    )])
}

#[cfg(test)]
mod list_tests {
    use super::*;

    #[test]
    fn predicates_visit_source_order_and_find_stops_before_later_values() {
        enable_cycle_collection();
        let log = V::reference(V::Int(0));
        let predicate = V::fun(vec![log.clone()], |capture, value| {
            let V::Int(n) = value else { panic!() };
            let V::Int(previous) = capture[0].read_reference() else {
                panic!()
            };
            capture[0].write_reference(V::Int(previous * 10 + n));
            V::Bool(n == 1)
        });
        let values = V::list_from_values(vec![V::Int(3), V::Int(1), V::Int(2)]);
        let found =
            native_list_operation("std/list::find", vec![predicate.clone(), values.clone()]);
        assert_eq!(found.payload().render(), "1");
        assert_eq!(log.read_reference().render(), "31");
        log.write_reference(V::Int(0));
        let filtered =
            native_list_operation("std/list::filter", vec![predicate.clone(), values.clone()]);
        assert_eq!(filtered.render(), "`[1]");
        assert_eq!(log.read_reference().render(), "312");
        drop((found, filtered, predicate, values, log));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn standard_list_display_bounds_depth_and_restores_it_after_defects() {
        let mut value = V::Int(1);
        let mut dictionary = native_primitive_display_dictionary("Show<std/prelude::Int>");
        for _ in 0..130 {
            value = V::list_cons(value, V::ListEmpty);
            dictionary = native_list_display_dictionary(dictionary, false);
        }
        let output = dictionary.field("show").apply(value).render();
        assert_eq!(output, format!("{}…{}", "`[".repeat(128), "]".repeat(128)));
        assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
        let failed = std::panic::catch_unwind(|| native_display(|| panic!("render failed")));
        assert!(failed.is_err());
        assert_eq!(DISPLAY_DEPTH.with(Cell::get), 0);
        drop(dictionary);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn cons_and_pattern_rest_share_the_existing_tail_without_allocating_it_again() {
        let tail = V::list_from_values(vec![V::Int(2), V::Int(3)]);
        let before = ALLOCATIONS.with(Cell::get);
        let value = V::list_cons(V::Int(1), tail.clone());
        assert_eq!(ALLOCATIONS.with(Cell::get) - before, 1);
        let rest = value.list_rest(1);
        assert_eq!(rest.node_id(), tail.node_id());
        assert_eq!(value.list_rest(0).node_id(), value.node_id());
        assert_eq!(ALLOCATIONS.with(Cell::get) - before, 1);
        assert!(value.list_length_matches(2, true));
        assert!(!value.list_length_matches(2, false));
        assert!(value.list_length_matches(3, false));
        assert!(!value.list_length_matches(4, true));
        drop(value);
        assert_eq!(rest.render(), "`[2, 3]");
        drop((rest, tail));
        // Cons's operation is independent of its tail's representation. This
        // opaque sentinel is never traversed as a checked List at runtime.
        drop(V::list_cons(V::Int(1), V::Unit));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn iterative_list_render_and_drop_release_deep_and_shared_graphs() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut values = V::ListEmpty;
                for _ in 0..100000 {
                    values = V::list_cons(V::Int(1), values);
                }
                let suffix = values.list_rest(99999);
                assert_eq!(values.render().len(), 300001);
                drop(values);
                assert_eq!(suffix.render(), "`[1]");
                drop(suffix);
                let mut nested = V::Int(1);
                for _ in 0..100000 {
                    nested = V::list_cons(nested, V::ListEmpty);
                }
                assert_eq!(nested.render().len(), 300001);
                drop(nested);
                assert_eq!(LIVE.with(Cell::get), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn list_edges_keep_reachable_ref_cycles_and_defer_collection_while_borrowed() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let list = V::list_from_values(vec![cell.clone(), cell.clone()]);
        cell.write_reference(list.clone());
        drop(list);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        let callback = V::fun(vec![cell.clone()], |capture, _| capture[0].read_reference());
        let garbage = V::reference(V::Unit);
        garbage.write_reference(V::list_cons(garbage.clone(), V::ListEmpty));
        drop(garbage);
        let V::Reference(slot) = &cell else {
            unreachable!()
        };
        let borrow = slot.get().borrow();
        assert!(collect_cycles().is_err());
        drop(borrow);
        assert_eq!(collect_cycles().unwrap().reclaimed, 2);
        drop(cell);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(
            callback.apply(V::Unit).list_element(1).node_id(),
            callback.apply(V::Unit).list_element(0).node_id()
        );
        drop(callback);
        assert_eq!(collect_cycles().unwrap().reclaimed, 3);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
