// Trial deletion for the mutable native slice. Pure programs keep ordinary Rc
// costs: the registry is enabled before their first allocation only when the
// checked module uses Ref. All cycles in this slice cross a mutable Ref edge;
// local recursive closures and opaque foreign captures remain unsupported.
thread_local! {
    static CYCLE_TRACKING: Cell<bool> = const { Cell::new(false) };
    static NODES: RefCell<BTreeMap<usize, Weak<dyn TraceNode>>> = const { RefCell::new(BTreeMap::new()) };
}

trait TraceValue {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str>;
    fn sever_reference(&self) -> Option<V> {
        None
    }
}
trait TraceNode {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str>;
    fn sever_reference(&self) -> Option<V>;
}
impl<T: TraceValue> TraceNode for Node<T> {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        self.value.edges(visit)
    }
    fn sever_reference(&self) -> Option<V> {
        self.value.sever_reference()
    }
}
impl TraceValue for String {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}
impl TraceValue for Vec<V> {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        self.iter().for_each(visit);
        Ok(())
    }
}
impl TraceValue for BTreeMap<String, V> {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        self.values().for_each(visit);
        Ok(())
    }
}
impl TraceValue for (String, V) {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        visit(&self.1);
        Ok(())
    }
}
impl TraceValue for Function {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        self.captures.edges(visit)
    }
}
impl TraceValue for RefCell<V> {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        // Even an immutable outstanding borrow defers the entire collection.
        // No edge is severed until every node has passed this preflight.
        let value = self.try_borrow_mut().map_err(|_| "borrowed native Ref")?;
        visit(&value);
        Ok(())
    }
    fn sever_reference(&self) -> Option<V> {
        Some(self.replace(V::Unit))
    }
}
impl TraceValue for EffectPlan {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        self.captures.edges(visit)
    }
}
impl V {
    fn node_id(&self) -> Option<usize> {
        Some(match self {
            Self::Unit | Self::Int(_) | Self::Bool(_) | Self::Char(_) => return None,
            Self::Text(v) => v.id(),
            Self::Tuple(v) | Self::Array(v) => v.id(),
            Self::Record(v) | Self::Dict(v) => v.id(),
            Self::Adt(v) => v.id(),
            Self::Fun(v) => v.id(),
            Self::Reference(v) => v.id(),
            Self::Effect(v) => v.id(),
        })
    }
    fn reference(value: V) -> V {
        assert!(
            CYCLE_TRACKING.with(Cell::get),
            "Ref requires a tracked heap"
        );
        V::Reference(Shared::new(RefCell::new(value)))
    }
    fn read_reference(&self) -> V {
        let Self::Reference(value) = self else {
            panic!("Rust ABI: expected Ref")
        };
        value.get().borrow().clone()
    }
    fn write_reference(&self, value: V) {
        let Self::Reference(slot) = self else {
            panic!("Rust ABI: expected Ref")
        };
        let old = slot.get().replace(value);
        drop(old); // Release the borrow before destroying any previous value.
    }
}
fn enable_cycle_collection() {
    if CYCLE_TRACKING.with(Cell::get) {
        return;
    }
    assert_eq!(LIVE.with(Cell::get), 0, "enable tracing before allocation");
    DROP_QUEUE.with(|_| {});
    DRAINING.with(|_| {});
    LIVE.with(|_| {});
    CYCLE_TRACKING.with(|tracking| tracking.set(true));
    NODES.with(|_| {});
}

#[derive(Default, Debug)]
struct CycleCollection {
    scanned: usize,
    reclaimed: usize,
}
fn collect_cycles() -> Result<CycleCollection, &'static str> {
    if !CYCLE_TRACKING.with(Cell::get) {
        return Ok(CycleCollection::default());
    }
    // Weak registry entries cannot keep a user value alive. Each snapshot has
    // exactly one temporary strong owner per node, deducted below.
    let nodes: Vec<_> = NODES.with(|registry| {
        registry
            .borrow()
            .iter()
            .filter_map(|(id, weak)| weak.upgrade().map(|n| (*id, n)))
            .collect()
    });
    let indices: BTreeMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (*id, i))
        .collect();
    let mut incoming = vec![0usize; nodes.len()];
    let mut outgoing = vec![Vec::new(); nodes.len()];
    for (i, (_, node)) in nodes.iter().enumerate() {
        let mut missing = false;
        node.edges(&mut |value| {
            if let Some(id) = value.node_id() {
                if let Some(target) = indices.get(&id) {
                    incoming[*target] += 1;
                    outgoing[i].push(*target);
                } else {
                    missing = true;
                }
            }
        })?;
        if missing {
            return Err("untracked native value edge");
        }
    }
    let mut work = Vec::new();
    for (i, (_, node)) in nodes.iter().enumerate() {
        let roots = Rc::strong_count(node)
            .checked_sub(1 + incoming[i])
            .ok_or("native ownership invariant")?;
        if roots > 0 {
            work.push(i);
        }
    }
    let mut reachable = vec![false; nodes.len()];
    while let Some(i) = work.pop() {
        if !std::mem::replace(&mut reachable[i], true) {
            work.extend(outgoing[i].iter().copied());
        }
    }
    let before = LIVE.with(Cell::get);
    let mut severed = Vec::new();
    for (i, (_, node)) in nodes.iter().enumerate() {
        if !reachable[i] {
            if let Some(value) = node.sever_reference() {
                severed.push(value);
            }
        }
    }
    let scanned = nodes.len();
    // Collector reclamation never calls language finalizers. Those belong to
    // Effect scopes, which remain ordinary external roots during cleanup.
    drop(severed);
    drop(nodes);
    Ok(CycleCollection {
        scanned,
        reclaimed: before - LIVE.with(Cell::get),
    })
}

#[cfg(test)]
mod trace_tests {
    use super::*;
    #[test]
    fn rooted_callbacks_keep_cycles_and_duplicate_edges_alive() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let tail = V::Array(Shared::new(vec![V::Int(42)]));
        let callback = V::fun(vec![cell.clone(), cell.clone(), tail.clone()], |c, _| {
            c[2].clone()
        });
        cell.write_reference(callback.clone());
        drop(cell);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(callback.apply(V::Unit).index(0).render(), "42");
        let alias = callback.clone();
        drop(callback);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop(alias);
        assert_eq!(collect_cycles().unwrap().reclaimed, 2);
        assert_eq!(tail.index(0).render(), "42");
        drop(tail);
        assert_eq!(LIVE.with(Cell::get), 0);
        assert!(NODES.with(|nodes| nodes.borrow().is_empty()));
    }
    #[test]
    fn borrowed_references_defer_collection_without_partial_reclamation() {
        enable_cycle_collection();
        let rooted = V::reference(V::Unit);
        let garbage = V::reference(V::Unit);
        garbage.write_reference(garbage.clone());
        drop(garbage);
        let V::Reference(value) = &rooted else {
            unreachable!()
        };
        let guard = value.get().borrow();
        assert!(collect_cycles().is_err());
        assert_eq!(LIVE.with(Cell::get), 2);
        drop(guard);
        assert_eq!(collect_cycles().unwrap().reclaimed, 1);
        drop(rooted);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn collection_matches_an_independent_reachability_oracle() {
        enable_cycle_collection();
        let mut seed = 0x517c_c1b7_u64;
        for _ in 0..200 {
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as usize
            };
            let cells: Vec<_> = (0..12).map(|_| V::reference(V::Unit)).collect();
            let mut edges = vec![Vec::new(); cells.len()];
            for (i, cell) in cells.iter().enumerate() {
                edges[i] = (0..next() % 5).map(|_| next() % cells.len()).collect();
                cell.write_reference(V::Array(Shared::new(
                    edges[i].iter().map(|i| cells[*i].clone()).collect(),
                )));
            }
            let roots: Vec<_> = (0..next() % 4).map(|_| next() % cells.len()).collect();
            let retained: Vec<_> = roots.iter().map(|i| cells[*i].clone()).collect();
            let mut reachable = [false; 12];
            let mut work = roots;
            while let Some(i) = work.pop() {
                if !std::mem::replace(&mut reachable[i], true) {
                    work.extend(edges[i].iter().copied());
                }
            }
            drop(cells);
            collect_cycles().unwrap();
            assert_eq!(
                LIVE.with(Cell::get),
                reachable.iter().filter(|v| **v).count() * 2
            );
            drop(retained);
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
