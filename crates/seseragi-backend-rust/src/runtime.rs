// Single-thread native values. Captures and mutable edges are always explicit.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

thread_local! {
    static DROP_QUEUE: RefCell<Vec<Box<dyn Any>>> = const { RefCell::new(Vec::new()) };
    static DRAINING: Cell<bool> = const { Cell::new(false) };
    static DICTIONARIES: RefCell<BTreeMap<&'static str, V>> = const { RefCell::new(BTreeMap::new()) };
    static GLOBALS: RefCell<Vec<V>> = const { RefCell::new(Vec::new()) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static SHARES: Cell<usize> = const { Cell::new(0) };
}

struct Node<T: TraceValue> {
    value: T,
}
impl<T: TraceValue> Drop for Node<T> {
    fn drop(&mut self) {
        let _ = LIVE.try_with(|n| n.set(n.get() - 1));
        if CYCLE_TRACKING.try_with(Cell::get).unwrap_or(false) {
            let id = self as *const Self as usize;
            let _ = NODES.try_with(|nodes| nodes.borrow_mut().remove(&id));
        }
    }
}
struct Shared<T: TraceValue + 'static>(Option<Rc<Node<T>>>);
impl<T: TraceValue + 'static> Shared<T> {
    fn new(value: T) -> Self {
        // Initialize teardown dependencies before dictionaries can own nodes.
        DROP_QUEUE.with(|_| {});
        DRAINING.with(|_| {});
        LIVE.with(|n| {
            n.set(n.get() + 1);
            PEAK.with(|p| p.set(p.get().max(n.get())));
        });
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        let node = Rc::new(Node { value });
        if CYCLE_TRACKING.with(Cell::get) {
            let erased: Rc<dyn TraceNode> = node.clone();
            NODES.with(|nodes| {
                nodes
                    .borrow_mut()
                    .insert(Rc::as_ptr(&node) as usize, Rc::downgrade(&erased));
            });
        }
        Self(Some(node))
    }
    fn get(&self) -> &T {
        &self.0.as_ref().expect("live shared handle").value
    }
    fn id(&self) -> usize {
        Rc::as_ptr(self.0.as_ref().expect("live shared handle")) as usize
    }
}
impl<T: TraceValue + 'static> Clone for Shared<T> {
    fn clone(&self) -> Self {
        SHARES.with(|n| n.set(n.get() + 1));
        Self(Some(Rc::clone(
            self.0.as_ref().expect("live shared handle"),
        )))
    }
}

// Destroy last owners through a work queue. Dropping a parent can enqueue
// children, but cannot recurse through a long immutable value graph.
impl<T: TraceValue + 'static> Drop for Shared<T> {
    fn drop(&mut self) {
        let node = self.0.take().expect("live shared handle");
        if Rc::strong_count(&node) != 1 {
            drop(node);
            return;
        }
        DROP_QUEUE.with(|queue| queue.borrow_mut().push(Box::new(node)));
        if DRAINING.with(|flag| flag.replace(true)) {
            return;
        }
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                DRAINING.with(|flag| flag.set(false));
            }
        }
        let _reset = Reset;
        loop {
            let next = DROP_QUEUE.with(|queue| queue.borrow_mut().pop());
            match next {
                Some(node) => drop(node),
                None => break,
            }
        }
    }
}

fn cached(key: &'static str, factory: impl FnOnce() -> V) -> V {
    // Initialize teardown state first: TLS destructors run in reverse order.
    DROP_QUEUE.with(|_| {});
    DRAINING.with(|_| {});
    LIVE.with(|_| {});
    if let Some(value) = DICTIONARIES.with(|cache| cache.borrow().get(key).cloned()) {
        return value;
    }
    let value = factory(); // No cache borrow survives nested factory evaluation.
    DICTIONARIES.with(|cache| cache.borrow_mut().insert(key, value.clone()));
    value
}
fn clear_dictionaries() {
    let entries = DICTIONARIES.with(|cache| std::mem::take(&mut *cache.borrow_mut()));
    drop(entries);
}

fn set_global(index: usize, value: V) {
    // Evaluate the initializer before borrowing storage. Calls can read earlier
    // bindings, but no initializer or user callback runs under this borrow.
    GLOBALS.with(|globals| {
        let mut globals = globals.borrow_mut();
        assert_eq!(
            globals.len(),
            index,
            "Rust ABI: global initialization order"
        );
        globals.push(value);
    });
}
// Only owned clones or immediate scalar extraction run under this borrow;
// source callbacks and initializer evaluation stay outside global storage.
fn read_global<T>(index: usize, read: impl FnOnce(&V) -> T) -> T {
    GLOBALS.with(|globals| {
        let globals = globals.borrow();
        read(globals.get(index).expect("Rust ABI: uninitialized global"))
    })
}
fn get_global(index: usize) -> V {
    read_global(index, V::clone)
}
fn get_global_int(index: usize) -> i64 {
    read_global(index, |value| match value {
        V::Int(value) => *value,
        _ => panic!("Rust ABI: expected Int"),
    })
}
fn get_global_bool(index: usize) -> bool {
    read_global(index, |value| match value {
        V::Bool(value) => *value,
        _ => panic!("Rust ABI: expected Bool"),
    })
}
fn clear_globals() {
    let values = GLOBALS.with(|globals| std::mem::take(&mut *globals.borrow_mut()));
    drop(values);
}

#[derive(Clone)]
enum V {
    Unit,
    Int(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Text(Shared<String>),
    Tuple(Shared<Vec<V>>),
    Array(Shared<Vec<V>>),
    ListEmpty,
    List(Shared<ListCell>),
    Record(Shared<BTreeMap<String, V>>),
    Adt(Shared<(String, V)>),
    Fun(Shared<Function>),
    Dict(Shared<BTreeMap<String, V>>),
    Reference(Shared<RefCell<V>>),
    Effect(Shared<EffectPlan>),
    Fiber(Shared<NativeFiber>),
    Deferred(Shared<NativeDeferred>),
}

struct Function {
    captures: Vec<V>,
    code: fn(&[V], V) -> V,
}

impl V {
    fn text(value: &str) -> Self {
        Self::Text(Shared::new(value.to_owned()))
    }
    fn fun(captures: Vec<V>, code: fn(&[V], V) -> V) -> Self {
        Self::Fun(Shared::new(Function { captures, code }))
    }
    fn adt(tag: &str, payload: V) -> Self {
        Self::Adt(Shared::new((tag.into(), payload)))
    }
    fn dict(methods: Vec<(&str, V)>) -> Self {
        Self::Dict(Shared::new(
            methods.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        ))
    }
    fn apply(&self, arg: V) -> V {
        match self {
            Self::Fun(f) => (f.get().code)(&f.get().captures, arg),
            _ => panic!("Rust ABI: expected function"),
        }
    }
    fn field(&self, key: &str) -> V {
        match self {
            Self::Dict(v) | Self::Record(v) => {
                v.get().get(key).expect("Rust ABI: missing field").clone()
            }
            _ => panic!("Rust ABI: expected dictionary or record"),
        }
    }
    fn index(&self, index: usize) -> V {
        match self {
            Self::Tuple(v) | Self::Array(v) => v.get()[index].clone(),
            _ => panic!("Rust ABI: expected tuple or array"),
        }
    }
    fn payload(&self) -> V {
        match self {
            Self::Adt(v) => v.get().1.clone(),
            _ => panic!("Rust ABI: expected ADT"),
        }
    }
    fn tagged(&self, tag: &str) -> bool {
        matches!(self, Self::Adt(v) if v.get().0 == tag)
    }
    fn boolean(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            _ => panic!("Rust ABI: expected Bool"),
        }
    }
    // Inspection output, not the language's Show instance implementation.
    // Use an explicit stack for nested collections and ADTs too.
    fn render(&self) -> String {
        enum Task<'a> {
            Value(&'a V),
            Text(String),
            ListTail(&'a V),
        }
        let mut work = vec![Task::Value(self)];
        let mut output = String::new();
        while let Some(task) = work.pop() {
            let value = match task {
                Task::ListTail(cursor) => {
                    if let Some(cell) = cursor.list_cell() {
                        output.push_str(", ");
                        work.push(Task::ListTail(&cell.tail));
                        work.push(Task::Value(&cell.head));
                    } else {
                        output.push(']');
                    }
                    continue;
                }
                Task::Text(text) => {
                    output.push_str(&text);
                    continue;
                }
                Task::Value(v) => v,
            };
            match value {
                V::Unit => output.push_str("()"),
                V::Int(n) => output.push_str(&n.to_string()),
                V::Float(n) => output.push_str(&native_float_format(*n)),
                V::Char(c) => output.push(*c),
                V::Bool(v) => output.push_str(if *v { "True" } else { "False" }),
                V::Text(v) => output.push_str(v.get()),
                V::ListEmpty => output.push_str("`[]"),
                V::List(v) => {
                    output.push_str("`[");
                    work.push(Task::ListTail(&v.get().tail));
                    work.push(Task::Value(&v.get().head));
                }
                V::Tuple(v) | V::Array(v) => {
                    let array = matches!(value, V::Array(_));
                    output.push_str(if array { "[" } else { "(" });
                    work.push(Task::Text(if array { "]" } else { ")" }.into()));
                    for (index, item) in v.get().iter().enumerate().rev() {
                        work.push(Task::Value(item));
                        if index != 0 {
                            work.push(Task::Text(", ".into()));
                        }
                    }
                }
                V::Record(v) => {
                    output.push('{');
                    work.push(Task::Text("}".into()));
                    for (index, (name, item)) in v.get().iter().enumerate().rev() {
                        work.push(Task::Value(item));
                        work.push(Task::Text(format!("{name}: ")));
                        if index != 0 {
                            work.push(Task::Text(", ".into()));
                        }
                    }
                }
                V::Adt(v) => {
                    output.push_str(&v.get().0);
                    output.push(' ');
                    work.push(Task::Value(&v.get().1));
                }
                V::Fun(_) => output.push_str("<function>"),
                V::Dict(_) => output.push_str("<dictionary>"),
                V::Reference(_) => output.push_str("<ref>"),
                V::Fiber(_) => output.push_str("<Fiber>"),
                V::Deferred(_) => output.push_str("<Deferred>"),
                V::Effect(_) => output.push_str("<effect>"),
            }
        }
        output
    }
}

fn apply(mut f: V, args: Vec<V>) -> V {
    for arg in args {
        f = f.apply(arg);
    }
    f
}
fn checked_int(value: i64) -> V {
    V::Int(checked_int_value(value))
}
fn checked_int_value(value: i64) -> i64 {
    assert!(
        (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value),
        "Seseragi Int overflow"
    );
    value
}
fn negate(value: V) -> V {
    match value {
        V::Int(n) => checked_int(n.checked_neg().expect("Int overflow")),
        V::Float(n) => V::Float(-n),
        _ => panic!("Rust ABI: expected Int"),
    }
}
fn int_add_value(a: i64, b: i64) -> i64 {
    checked_int_value(a.checked_add(b).expect("Int overflow"))
}
fn int_subtract_value(a: i64, b: i64) -> i64 {
    checked_int_value(a.checked_sub(b).expect("Int overflow"))
}
fn int_multiply_value(a: i64, b: i64) -> i64 {
    checked_int_value(a.checked_mul(b).expect("Int overflow"))
}
fn int_divide_value(a: i64, b: i64) -> i64 {
    if b == 0 {
        panic!("Int division by zero");
    }
    checked_int_value(a);
    checked_int_value(b);
    checked_int_value(a.checked_div(b).expect("Int overflow"))
}
fn int_remainder_value(a: i64, b: i64) -> i64 {
    if b == 0 {
        panic!("Int remainder by zero");
    }
    checked_int_value(a);
    checked_int_value(b);
    checked_int_value(a.checked_rem(b).expect("Int overflow"))
}
fn int_equal_value(a: i64, b: i64) -> bool {
    a == b
}
fn int_less_value(a: i64, b: i64) -> bool {
    a < b
}
fn int_less_equal_value(a: i64, b: i64) -> bool {
    a <= b
}
fn binary(op: &str, a: V, b: V) -> V {
    match (op, a, b) {
        ("+", V::Int(a), V::Int(b)) => V::Int(int_add_value(a, b)),
        ("-", V::Int(a), V::Int(b)) => V::Int(int_subtract_value(a, b)),
        ("*", V::Int(a), V::Int(b)) => V::Int(int_multiply_value(a, b)),
        ("/", V::Int(a), V::Int(b)) => V::Int(int_divide_value(a, b)),
        ("%", V::Int(a), V::Int(b)) => V::Int(int_remainder_value(a, b)),
        ("==", V::Int(a), V::Int(b)) => V::Bool(int_equal_value(a, b)),
        ("==", V::Bool(a), V::Bool(b)) => V::Bool(a == b),
        ("==", V::Unit, V::Unit) => V::Bool(true),
        ("==", V::Text(a), V::Text(b)) => V::Bool(a.get() == b.get()),
        ("<", V::Int(a), V::Int(b)) => V::Bool(int_less_value(a, b)),
        ("<=", V::Int(a), V::Int(b)) => V::Bool(int_less_equal_value(a, b)),
        ("+", V::Text(a), V::Text(b)) => V::Text(Shared::new(format!("{}{}", a.get(), b.get()))),
        _ => panic!("Rust ABI: unsupported binary operation"),
    }
}

fn native_operator_dictionary(identity: &'static str) -> V {
    cached(identity, || {
        macro_rules! arithmetic {
            ($method:literal, $operator:literal) => {
                V::dict(vec![(
                    $method,
                    V::fun(vec![], |_, left| {
                        V::fun(vec![left], |c, right| {
                            binary($operator, c[0].clone(), right)
                        })
                    }),
                )])
            };
        }
        match identity {
            "std/int::Add" | "std/string::Add" => arithmetic!("add", "+"),
            "std/int::Sub" => arithmetic!("sub", "-"),
            "std/int::Mul" => arithmetic!("mul", "*"),
            "std/int::Div" => arithmetic!("div", "/"),
            "std/int::Rem" => arithmetic!("rem", "%"),
            "std/int::Eq" | "std/bool::Eq" | "std/unit::Eq" | "std/string::Eq" => {
                arithmetic!("eq", "==")
            }
            _ => panic!("Rust ABI: unsupported arithmetic dictionary"),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn string_equality_compares_contents_and_releases_partial_captures() {
        clear_dictionaries();
        let equal = native_operator_dictionary("std/string::Eq").field("eq");
        let saved = equal.apply(V::text("雫\0🙂"));
        let separate = V::text(&format!("{}{}", "雫\0", "🙂"));
        assert!(saved.apply(separate).boolean());
        assert!(!saved.apply(V::text("雫\0🦀")).boolean());
        assert!(binary("==", V::text(""), V::text("")).boolean());
        assert!(!binary("==", V::text("é"), V::text("é")).boolean());
        drop(saved);
        drop(equal);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn bool_and_unit_equality_preserve_curried_value_stages() {
        clear_dictionaries();
        let bool_eq = native_operator_dictionary("std/bool::Eq").field("eq");
        let true_stage = bool_eq.apply(V::Bool(true));
        assert!(true_stage.apply(V::Bool(true)).boolean());
        assert!(!true_stage.apply(V::Bool(false)).boolean());
        let unit_eq = native_operator_dictionary("std/unit::Eq").field("eq");
        let unit_stage = unit_eq.apply(V::Unit);
        assert!(matches!(&unit_stage, V::Fun(_)));
        assert!(unit_stage.apply(V::Unit).boolean());
        assert!(unit_stage.apply(V::Unit).boolean());
        drop((unit_stage, unit_eq, true_stage, bool_eq));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn scalar_global_reads_keep_checked_access_and_release_storage_borrows() {
        clear_globals();
        assert!(std::panic::catch_unwind(|| get_global_int(0)).is_err());
        assert!(std::panic::catch_unwind(|| get_global_bool(0)).is_err());
        set_global(0, V::Int(42));
        set_global(1, V::Bool(true));
        let before = ALLOCATIONS.with(Cell::get);
        for _ in 0..1000 {
            assert_eq!(get_global_int(0), 42);
            assert!(get_global_bool(1));
        }
        assert_eq!(ALLOCATIONS.with(Cell::get), before);
        assert!(std::panic::catch_unwind(|| get_global_int(1)).is_err());
        assert!(std::panic::catch_unwind(|| get_global_bool(0)).is_err());
        GLOBALS.with(|globals| assert!(globals.try_borrow_mut().is_ok()));
        assert_eq!(get_global(0).render(), "42");
        clear_globals();
        assert!(std::panic::catch_unwind(|| get_global_int(0)).is_err());
        set_global(0, V::Int(7));
        assert_eq!(get_global_int(0), 7);
        clear_globals();
    }

    #[test]
    fn deep_values_and_closure_captures_drop_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut chain = V::Unit;
                for _ in 0..100_000 {
                    chain = V::adt("Cons", chain);
                }
                let retained = chain.clone();
                let callback = V::fun(vec![retained], |captures, _| captures[0].clone());
                drop(chain);
                assert!(LIVE.with(Cell::get) >= 100_000);
                drop(callback);
                assert_eq!(LIVE.with(Cell::get), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn large_values_share_storage_and_release_last_owner() {
        let text = V::text(&"x".repeat(1_000_000));
        let before = ALLOCATIONS.with(Cell::get);
        let copies = (0..10_000).map(|_| text.clone()).collect::<Vec<_>>();
        assert_eq!(ALLOCATIONS.with(Cell::get), before);
        drop(text);
        drop(copies);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn lazy_dictionary_cache_releases_on_scope_clear() {
        let calls = Cell::new(0);
        for _ in 0..100 {
            drop(cached("closed", || {
                calls.set(calls.get() + 1);
                V::dict(vec![("read", V::fun(vec![], |_, v| v))])
            }));
        }
        assert_eq!(calls.get(), 1);
        assert_eq!(LIVE.with(Cell::get), 2);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn cached_captures_are_safe_during_thread_teardown() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut chain = V::Unit;
                for _ in 0..20_000 {
                    chain = V::adt("Cons", chain);
                }
                drop(cached("deep", || {
                    V::fun(vec![chain], |captures, _| captures[0].clone())
                }));
                // Deliberately leave the dictionary cache populated for TLS Drop.
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
