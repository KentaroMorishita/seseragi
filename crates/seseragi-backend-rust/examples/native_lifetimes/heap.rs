//! Opt-in lifetime model, not the emitted pure ABI. No user finalizers run here.
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

#[derive(Default)]
struct State {
    nodes: RefCell<BTreeMap<usize, Weak<Node>>>,
    suspects: RefCell<BTreeMap<usize, Weak<Node>>>,
    queue: RefCell<Vec<Rc<Node>>>,
    draining: Cell<bool>,
    live: Cell<usize>,
}
struct Node {
    state: Rc<State>,
    value: RefCell<Option<Value>>,
}
impl Drop for Node {
    fn drop(&mut self) {
        self.state.live.set(self.state.live.get() - 1);
        let id = self as *const Self as usize;
        self.state.nodes.borrow_mut().remove(&id);
        self.state.suspects.borrow_mut().remove(&id);
    }
}

// External handles and graph edges have identical owning semantics. The
// collector derives roots from strong counts, rather than asking source code
// to tag roots or weakening arbitrary closure captures.
pub struct Handle(Option<Rc<Node>>);
impl Clone for Handle {
    fn clone(&self) -> Self {
        Self(Some(self.node().clone()))
    }
}
impl Handle {
    fn node(&self) -> &Rc<Node> {
        self.0.as_ref().expect("live handle")
    }
    fn id(&self) -> usize {
        Rc::as_ptr(self.node()) as usize
    }
    pub fn integer(&self) -> i64 {
        let value = self.node().value.borrow();
        let Some(Value::Int(n)) = &*value else {
            panic!("expected Int")
        };
        *n
    }
    pub fn apply(&self, input: i64) -> i64 {
        let value = self.node().value.borrow();
        let Some(Value::Function { captures, code }) = &*value else {
            panic!("expected function")
        };
        code(captures, input)
    }
    pub fn call(&self, heap: &Heap, input: Handle) -> Result<Handle, super::scope::Exit> {
        let (captures, code) = {
            let value = self.node().value.borrow();
            let Some(Value::Callable { captures, code }) = &*value else {
                return Err(super::scope::Exit::Defect(vec!["expected callable".into()]));
            };
            (captures.clone(), *code)
        };
        code(heap, &captures, input)
    }
    pub fn execute(
        &self,
        heap: &Heap,
        context: &mut super::effect::Context,
    ) -> Result<Handle, super::scope::Exit> {
        let (captures, code) = {
            let value = self.node().value.borrow();
            let Some(Value::Effect { captures, code }) = &*value else {
                return Err(super::scope::Exit::Defect(vec!["expected Effect".into()]));
            };
            (captures.clone(), *code)
        };
        context.checkpoint(heap)?;
        code(heap, &captures, context)
    }
    pub fn boolean(&self) -> bool {
        let value = self.node().value.borrow();
        let Some(Value::Bool(value)) = &*value else {
            panic!("expected Bool")
        };
        *value
    }
    pub fn show(&self) -> String {
        match self.node().value.borrow().as_ref().expect("live node") {
            Value::Unit => "()".into(),
            Value::Int(n) => n.to_string(),
            Value::Bool(v) => if *v { "True" } else { "False" }.into(),
            Value::Text(v) => v.clone(),
            _ => panic!("only primitive Show is supported by the research runner"),
        }
    }
    pub fn read(&self) -> Handle {
        let value = self.node().value.borrow();
        let Some(Value::Reference(child)) = &*value else {
            panic!("expected reference")
        };
        child.clone()
    }
    pub fn write(&self, child: Handle) {
        assert!(Rc::ptr_eq(&self.node().state, &child.node().state));
        let mut value = self.node().value.borrow_mut();
        let Some(Value::Reference(slot)) = value.as_mut() else {
            panic!("expected reference")
        };
        let old = std::mem::replace(slot, child);
        drop(value);
        drop(old);
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        let node = self.0.take().expect("live handle");
        if Rc::strong_count(&node) != 1 {
            // A non-zero decrement may have removed the last EXTERNAL root
            // of a cycle. Deduplicate weak candidates; keep ordinary Rc Drop.
            node.state
                .suspects
                .borrow_mut()
                .insert(Rc::as_ptr(&node) as usize, Rc::downgrade(&node));
            return;
        }
        let state = node.state.clone();
        state.queue.borrow_mut().push(node);
        if state.draining.replace(true) {
            return;
        }
        loop {
            let next = state.queue.borrow_mut().pop();
            match next {
                Some(node) => drop(node),
                None => break,
            }
        }
        state.draining.set(false);
    }
}

enum Value {
    Unit,
    Int(i64),
    Bool(bool),
    Text(String),
    Sequence(Vec<Handle>),
    Function {
        captures: Vec<Handle>,
        code: fn(&[Handle], i64) -> i64,
    },
    Reference(Handle),
    Callable {
        captures: Vec<Handle>,
        code: CallableCode,
    },
    Effect {
        captures: Vec<Handle>,
        code: EffectCode,
    },
}
type CallableCode = fn(&Heap, &[Handle], Handle) -> Result<Handle, super::scope::Exit>;
type EffectCode =
    fn(&Heap, &[Handle], &mut super::effect::Context) -> Result<Handle, super::scope::Exit>;
impl Value {
    fn edges(&self) -> &[Handle] {
        match self {
            Self::Unit | Self::Int(_) | Self::Bool(_) | Self::Text(_) => &[],
            Self::Sequence(children)
            | Self::Function {
                captures: children, ..
            } => children,
            Self::Reference(child) => std::slice::from_ref(child),
            Self::Callable { captures, .. } | Self::Effect { captures, .. } => captures,
        }
    }
}

#[derive(Default)]
pub struct Heap(Rc<State>);
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub scanned: usize,
    pub edges: usize,
    pub roots: usize,
    pub reclaimed: usize,
    pub registry: usize,
}
impl Heap {
    fn allocate(&self, value: Value) -> Handle {
        for edge in value.edges() {
            assert!(Rc::ptr_eq(&self.0, &edge.node().state), "cross-heap edge");
        }
        let node = Rc::new(Node {
            state: self.0.clone(),
            value: RefCell::new(Some(value)),
        });
        self.0
            .nodes
            .borrow_mut()
            .insert(Rc::as_ptr(&node) as usize, Rc::downgrade(&node));
        self.0.live.set(self.0.live.get() + 1);
        Handle(Some(node))
    }
    pub fn int(&self, n: i64) -> Handle {
        self.allocate(Value::Int(n))
    }
    pub fn unit(&self) -> Handle {
        self.allocate(Value::Unit)
    }
    pub fn boolean(&self, value: bool) -> Handle {
        self.allocate(Value::Bool(value))
    }
    pub fn text(&self, value: &str) -> Handle {
        self.allocate(Value::Text(value.into()))
    }
    pub fn callable(&self, captures: Vec<Handle>, code: CallableCode) -> Handle {
        self.allocate(Value::Callable { captures, code })
    }
    pub fn effect(&self, captures: Vec<Handle>, code: EffectCode) -> Handle {
        self.allocate(Value::Effect { captures, code })
    }
    pub fn sequence(&self, children: Vec<Handle>) -> Handle {
        self.allocate(Value::Sequence(children))
    }
    pub fn function(&self, captures: Vec<Handle>, code: fn(&[Handle], i64) -> i64) -> Handle {
        self.allocate(Value::Function { captures, code })
    }
    pub fn reference(&self, initial: Handle) -> Handle {
        self.allocate(Value::Reference(initial))
    }
    pub fn live(&self) -> usize {
        self.0.live.get()
    }
    pub fn registry(&self) -> usize {
        self.0.nodes.borrow().len()
    }
    // A complete, single-thread, stop-the-world pass. O((N + E) log N), not
    // Bacon/Rajan's localized/concurrent collector. No user code can run from
    // tracing or clearing. A live borrow aborts before ANY graph mutation.
    pub fn collect(&self) -> Result<Collection, &'static str> {
        let nodes: Vec<_> = self
            .0
            .nodes
            .borrow()
            .values()
            .filter_map(Weak::upgrade)
            .collect();
        let result = self.collect_nodes(nodes)?;
        self.0.suspects.borrow_mut().clear();
        Ok(result)
    }
    // Trial deletion of only the candidate's transitive outgoing subgraph.
    // Outside incoming owners count as roots. This still has an unbounded
    // pause for a large candidate graph; it is NOT an incremental collector.
    pub fn collect_candidates(&self) -> Result<Collection, &'static str> {
        let suspects = std::mem::take(&mut *self.0.suspects.borrow_mut());
        let result = (|| {
            let mut work: Vec<_> = suspects.values().filter_map(Weak::upgrade).collect();
            let mut selected = BTreeMap::new();
            while let Some(node) = work.pop() {
                let id = Rc::as_ptr(&node) as usize;
                if selected.contains_key(&id) {
                    continue;
                }
                {
                    let value = node.value.try_borrow().map_err(|_| "borrowed heap")?;
                    for edge in value.as_ref().expect("live node").edges() {
                        if !selected.contains_key(&edge.id()) {
                            work.push(edge.node().clone());
                        }
                    }
                }
                selected.insert(id, node);
            }
            self.collect_nodes(selected.into_values().collect())
        })();
        if result.is_err() {
            self.0.suspects.borrow_mut().extend(suspects);
        }
        result
    }
    fn collect_nodes(&self, nodes: Vec<Rc<Node>>) -> Result<Collection, &'static str> {
        let before = self.live();
        // Retain one snapshot owner per node. Guards also make the no-mutation
        // interval concrete in safe Rust; borrowed runtime operations defer GC.
        let mut values = nodes
            .iter()
            .map(|n| n.value.try_borrow_mut())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "borrowed heap")?;
        let indices: BTreeMap<_, _> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (Rc::as_ptr(n) as usize, i))
            .collect();
        let mut incoming = vec![0usize; nodes.len()];
        let mut outgoing = vec![Vec::new(); nodes.len()];
        let mut edges = 0;
        for (i, value) in values.iter().enumerate() {
            for edge in value.as_ref().expect("live node").edges() {
                let target = *indices.get(&edge.id()).expect("all edges tracked");
                incoming[target] += 1; // Count multiplicity, not just distinct edges.
                outgoing[i].push(target);
                edges += 1;
            }
        }
        let mut work = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            let external = Rc::strong_count(node)
                .checked_sub(1 + incoming[i])
                .expect("snapshot + internal counts cannot exceed all owners");
            if external > 0 {
                work.push(i);
            }
        }
        let roots = work.len();
        let mut reachable = vec![false; nodes.len()];
        while let Some(i) = work.pop() {
            if std::mem::replace(&mut reachable[i], true) {
                continue;
            }
            work.extend(outgoing[i].iter().copied());
        }
        for (i, value) in values.iter_mut().enumerate() {
            if !reachable[i] {
                // All nodes are kept alive by the snapshot while edges are
                // severed. This clears containers, never language finalizers.
                drop(value.take());
            }
        }
        let scanned = nodes.len();
        drop(values);
        drop(nodes);
        Ok(Collection {
            scanned,
            edges,
            roots,
            reclaimed: before - self.live(),
            registry: self.registry(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returned_closure_roots_preserve_ref_cycles_and_values() {
        let heap = Heap::default();
        let cell = heap.reference(heap.int(0));
        let callback = heap.function(vec![cell.clone(), heap.int(40)], |c, n| c[1].integer() + n);
        cell.write(callback.clone());
        drop(cell);
        assert_eq!(heap.collect().unwrap().reclaimed, 0);
        assert_eq!(callback.apply(2), 42);
        let alias = callback.clone();
        drop(callback);
        assert_eq!(heap.collect().unwrap().reclaimed, 0);
        assert_eq!(alias.apply(3), 43);
        drop(alias);
        assert_eq!(heap.live(), 3, "Rc alone leaks this cycle and its payload");
        assert_eq!(heap.collect().unwrap().reclaimed, 3);
        assert_eq!(heap.live(), 0);
        assert_eq!(heap.registry(), 0);
    }

    #[test]
    fn unreachable_cycle_cannot_reclaim_its_reachable_shared_tail() {
        let heap = Heap::default();
        let tail = heap.sequence(vec![heap.int(42)]);
        let cell = heap.reference(heap.int(0));
        let callback = heap.function(vec![cell.clone(), cell.clone(), tail.clone()], |_, n| n);
        cell.write(callback.clone());
        drop(callback);
        drop(cell);
        assert_eq!(heap.collect().unwrap().reclaimed, 2);
        assert_eq!(heap.live(), 2);
        drop(tail);
        assert_eq!(heap.live(), 0);
    }

    #[test]
    fn collection_is_atomic_when_any_node_is_borrowed() {
        let heap = Heap::default();
        let cell = heap.reference(heap.int(0));
        cell.write(cell.clone());
        let other = heap.reference(heap.int(0));
        other.write(other.clone());
        drop(other);
        let guard = cell.node().value.borrow();
        assert_eq!(heap.collect(), Err("borrowed heap"));
        assert_eq!(
            heap.live(),
            2,
            "no unreachable edge was cleared before abort"
        );
        drop(guard);
        assert_eq!(heap.collect().unwrap().reclaimed, 1);
        drop(cell);
        assert_eq!(heap.collect().unwrap().reclaimed, 1);
    }

    #[test]
    fn foreign_registration_roots_require_explicit_detach() {
        let heap = Heap::default();
        let reference = heap.reference(heap.int(0));
        let callback = heap.function(vec![reference.clone()], |_, n| n);
        reference.write(callback.clone());
        let mut host_callbacks = BTreeMap::from([(7usize, callback.clone())]);
        drop(callback);
        drop(reference);
        assert_eq!(heap.collect().unwrap().reclaimed, 0);
        // Models a scope-owned JS listener / reload dispose token. A local
        // collector cannot prove that the foreign host has abandoned this root.
        host_callbacks.remove(&7);
        assert_eq!(heap.collect().unwrap().reclaimed, 2);
    }

    #[test]
    fn candidate_collection_avoids_an_unrelated_large_live_graph() {
        let heap = Heap::default();
        let mut live = heap.int(42);
        for _ in 0..100_000 {
            live = heap.sequence(vec![live]);
        }
        let cell = heap.reference(heap.int(0));
        let callback = heap.function(vec![cell.clone()], |_, n| n);
        cell.write(callback);
        drop(cell);
        let collected = heap.collect_candidates().unwrap();
        assert_eq!(collected.scanned, 2);
        assert_eq!(collected.reclaimed, 2);
        assert_eq!(heap.live(), 100_001);
        drop(live);
        assert_eq!(heap.live(), 0);
        assert_eq!(
            heap.registry(),
            0,
            "ordinary Rc Drop removes registry entries"
        );
    }

    #[test]
    fn candidate_borrow_abort_keeps_work_for_the_next_safe_point() {
        let heap = Heap::default();
        let cell = heap.reference(heap.int(0));
        cell.write(cell.clone());
        drop(cell.clone()); // Queue a non-zero decrement while keeping this root.
        let guard = cell.node().value.borrow_mut();
        assert_eq!(heap.collect_candidates(), Err("borrowed heap"));
        drop(guard);
        let retried = heap.collect_candidates().unwrap();
        assert_eq!(retried.scanned, 1, "borrowed candidate was not lost");
        assert_eq!(retried.reclaimed, 0);
        drop(cell);
        assert_eq!(heap.collect_candidates().unwrap().reclaimed, 1);
    }

    #[test]
    fn deep_graphs_drop_and_collect_with_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                for candidates in [false, true] {
                    let heap = Heap::default();
                    let cell = heap.reference(heap.int(0));
                    let mut chain = cell.clone();
                    for _ in 0..100_000 {
                        chain = heap.sequence(vec![chain]);
                    }
                    cell.write(chain);
                    assert_eq!(
                        if candidates {
                            heap.collect_candidates()
                        } else {
                            heap.collect()
                        }
                        .unwrap()
                        .reclaimed,
                        0
                    );
                    drop(cell);
                    assert_eq!(
                        if candidates {
                            heap.collect_candidates()
                        } else {
                            heap.collect()
                        }
                        .unwrap()
                        .reclaimed,
                        100_001
                    );
                    let mut acyclic = heap.int(42);
                    for _ in 0..100_000 {
                        acyclic = heap.sequence(vec![acyclic]);
                    }
                    drop(acyclic);
                    assert_eq!(heap.live(), 0);
                    heap.collect().unwrap();
                    assert_eq!(heap.registry(), 0);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn an_independent_marking_oracle_checks_varied_graphs() {
        for candidates in [false, true] {
            for seed in 0..100usize {
                let heap = Heap::default();
                let mut nodes: Vec<_> = (0..24).map(|_| heap.reference(heap.int(0))).collect();
                let links: Vec<Vec<usize>> = (0..24)
                    .map(|i| vec![(i * 7 + seed) % 24, (i * 11 + seed / 3) % 24])
                    .collect();
                for (i, links) in links.iter().enumerate() {
                    nodes[i]
                        .write(heap.sequence(links.iter().map(|j| nodes[*j].clone()).collect()));
                }
                let root_indices: Vec<_> = (0..24).filter(|i| (i + seed) % 9 == 0).collect();
                let roots: Vec<_> = root_indices.iter().map(|i| nodes[*i].clone()).collect();
                // Expected reachability comes from integer adjacency, not Rc counts.
                let mut marked = [false; 24];
                let mut work = root_indices;
                while let Some(i) = work.pop() {
                    if !std::mem::replace(&mut marked[i], true) {
                        work.extend(links[i].iter().copied());
                    }
                }
                nodes.clear();
                if candidates {
                    heap.collect_candidates()
                } else {
                    heap.collect()
                }
                .unwrap();
                assert_eq!(
                    heap.live(),
                    marked.iter().filter(|v| **v).count() * 2,
                    "seed {seed}"
                );
                drop(roots);
                if candidates {
                    heap.collect_candidates()
                } else {
                    heap.collect()
                }
                .unwrap();
                assert_eq!(heap.live(), 0, "seed {seed}");
            }
        }
    }
}
