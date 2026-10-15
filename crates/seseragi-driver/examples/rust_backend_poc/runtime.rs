// Experimental single-thread runtime. Only the documented pure slice is supported.
use std::{cell::Cell, collections::BTreeMap, rc::Rc};

thread_local! {
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static SHARES: Cell<usize> = const { Cell::new(0) };
}

struct Node<T> {
    value: T,
}
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        LIVE.with(|n| n.set(n.get() - 1));
    }
}
struct Shared<T>(Rc<Node<T>>);
impl<T> Shared<T> {
    fn new(value: T) -> Self {
        LIVE.with(|n| {
            n.set(n.get() + 1);
            PEAK.with(|p| p.set(p.get().max(n.get())));
        });
        ALLOCATIONS.with(|n| n.set(n.get() + 1));
        Self(Rc::new(Node { value }))
    }
    fn get(&self) -> &T {
        &self.0.value
    }
}
impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        SHARES.with(|n| n.set(n.get() + 1));
        Self(Rc::clone(&self.0))
    }
}

#[derive(Clone)]
enum V {
    Unit,
    Int(i64),
    Bool(bool),
    Text(Shared<String>),
    Tuple(Shared<Vec<V>>),
    Array(Shared<Vec<V>>),
    Record(Shared<BTreeMap<String, V>>),
    Adt(Shared<(String, V)>),
    Fun(Shared<Box<dyn Fn(V) -> V>>),
    Dict(Shared<BTreeMap<String, V>>),
}

impl V {
    fn text(value: &str) -> Self {
        Self::Text(Shared::new(value.to_owned()))
    }
    fn fun(f: impl Fn(V) -> V + 'static) -> Self {
        Self::Fun(Shared::new(Box::new(f)))
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
            Self::Fun(f) => (f.get())(arg),
            _ => panic!("PoC ABI: expected function"),
        }
    }
    fn field(&self, key: &str) -> V {
        match self {
            Self::Dict(v) | Self::Record(v) => {
                v.get().get(key).expect("PoC ABI: missing field").clone()
            }
            _ => panic!("PoC ABI: expected dictionary or record"),
        }
    }
    fn index(&self, index: usize) -> V {
        match self {
            Self::Tuple(v) | Self::Array(v) => v.get()[index].clone(),
            _ => panic!("PoC ABI: expected tuple or array"),
        }
    }
    fn payload(&self) -> V {
        match self {
            Self::Adt(v) => v.get().1.clone(),
            _ => panic!("PoC ABI: expected ADT"),
        }
    }
    fn tagged(&self, tag: &str) -> bool {
        matches!(self, Self::Adt(v) if v.get().0 == tag)
    }
    fn boolean(&self) -> bool {
        match self {
            Self::Bool(v) => *v,
            _ => panic!("PoC ABI: expected Bool"),
        }
    }
    fn render(&self) -> String {
        match self {
            Self::Unit => "()".into(),
            Self::Int(v) => v.to_string(),
            Self::Bool(v) => if *v { "True" } else { "False" }.into(),
            Self::Text(v) => v.get().clone(),
            Self::Tuple(v) => format!(
                "({})",
                v.get().iter().map(V::render).collect::<Vec<_>>().join(", ")
            ),
            Self::Array(v) => format!(
                "[{}]",
                v.get().iter().map(V::render).collect::<Vec<_>>().join(", ")
            ),
            Self::Adt(v) => format!("{} {}", v.get().0, v.get().1.render()),
            _ => panic!("PoC render: unsupported value"),
        }
    }
}

fn apply(mut f: V, args: Vec<V>) -> V {
    for arg in args {
        f = f.apply(arg);
    }
    f
}
fn checked_int(value: i64) -> V {
    assert!(
        (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value),
        "Seseragi Int overflow"
    );
    V::Int(value)
}
fn binary(op: &str, a: V, b: V) -> V {
    match (op, a, b) {
        ("+", V::Int(a), V::Int(b)) => checked_int(a.checked_add(b).expect("Int overflow")),
        ("-", V::Int(a), V::Int(b)) => checked_int(a.checked_sub(b).expect("Int overflow")),
        ("*", V::Int(a), V::Int(b)) => checked_int(a.checked_mul(b).expect("Int overflow")),
        ("==", V::Int(a), V::Int(b)) => V::Bool(a == b),
        ("<", V::Int(a), V::Int(b)) => V::Bool(a < b),
        ("<=", V::Int(a), V::Int(b)) => V::Bool(a <= b),
        ("+", V::Text(a), V::Text(b)) => V::Text(Shared::new(format!("{}{}", a.get(), b.get()))),
        _ => panic!("PoC ABI: unsupported binary operation"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn cloning_large_values_shares_storage() {
        let text = V::text(&"x".repeat(1_000_000));
        let before = ALLOCATIONS.with(Cell::get);
        let copies = (0..10_000).map(|_| text.clone()).collect::<Vec<_>>();
        assert_eq!(before, ALLOCATIONS.with(Cell::get));
        let (V::Text(a), V::Text(b)) = (&text, &copies[0]) else {
            unreachable!()
        };
        assert!(Rc::ptr_eq(&a.0, &b.0));
        drop(copies);
        drop(text);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn cold_effect_factory_is_reusable_and_releases_captures() {
        let runs = Shared::new(Cell::new(0));
        let weak = Rc::downgrade(&runs.0);
        let captured = runs.clone();
        let effect = V::fun(move |_| {
            captured.get().set(captured.get().get() + 1);
            V::Int(captured.get().get())
        });
        assert_eq!(runs.get().get(), 0);
        assert_eq!(effect.apply(V::Unit).render(), "1");
        assert_eq!(effect.apply(V::Unit).render(), "2");
        drop(runs);
        assert!(weak.upgrade().is_some());
        drop(effect);
        assert!(weak.upgrade().is_none());
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn strong_cycle_survives_until_explicitly_broken() {
        let owner = Shared::new(RefCell::new(Vec::<V>::new()));
        let weak = Rc::downgrade(&owner.0);
        let captured = owner.clone();
        owner.get().borrow_mut().push(V::fun(
            move |_| V::Int(captured.get().borrow().len() as i64),
        ));
        drop(owner);
        assert_eq!(
            LIVE.with(Cell::get),
            2,
            "owner -> callback -> owner is not collected"
        );
        let recovered = weak.upgrade().unwrap();
        recovered.value.borrow_mut().clear();
        drop(recovered);
        assert!(weak.upgrade().is_none());
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn weak_back_edge_is_reclaimed() {
        let owner = Shared::new(RefCell::new(Vec::<V>::new()));
        let weak = Rc::downgrade(&owner.0);
        let captured = weak.clone();
        owner.get().borrow_mut().push(V::fun(move |_| {
            let owner = captured.upgrade().expect("owner alive during callback");
            let count = owner.value.borrow().len();
            V::Int(count as i64)
        }));
        assert_eq!(owner.get().borrow()[0].apply(V::Unit).render(), "1");
        drop(owner);
        assert!(weak.upgrade().is_none());
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
