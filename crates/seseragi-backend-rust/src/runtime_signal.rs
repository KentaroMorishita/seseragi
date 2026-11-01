const SIGNAL_SOURCE: i64 = 0;
const SIGNAL_MAP: i64 = 1;
const SIGNAL_COMBINE: i64 = 2;
const SIGNAL_CONSTANT: i64 = 3;
const SIGNAL_DISTINCT: i64 = 4;
const SIGNAL_SWITCH: i64 = 5;
const SIGNAL_APPLY: i64 = 6;

// Signals own their upstream values. Scopes own subscriptions, while the
// publication registry keeps weak handles. Unsubscribe clears captured values.
thread_local! {
    static SIGNAL_SUBSCRIPTION_ORDER: Cell<i64> = const { Cell::new(0) };
    static SIGNAL_EPOCH: Cell<i64> = const { Cell::new(0) };
    static SIGNAL_PUBLICATION: Cell<bool> = const { Cell::new(false) };
    static SIGNAL_SUBSCRIBERS: RefCell<Vec<Weak<Node<RefCell<V>>>>> = const { RefCell::new(Vec::new()) };
    static SIGNAL_TRANSACTIONS: RefCell<std::collections::VecDeque<V>> = const { RefCell::new(std::collections::VecDeque::new()) };
}

fn signal_record(fields: Vec<(&str, V)>) -> V {
    V::Record(Shared::new(
        fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
    ))
}
fn signal_field(signal: &V, key: &str) -> V {
    signal.read_reference().field(key)
}
fn signal_fields(signal: &V, fields: Vec<(&str, V)>) {
    let V::Record(record) = signal.read_reference() else {
        panic!("Rust ABI: expected Signal state")
    };
    let mut record = record.get().clone();
    for (key, value) in fields {
        record.insert(key.into(), value);
    }
    signal.write_reference(V::Record(Shared::new(record)));
}
fn signal_int(signal: &V, key: &str) -> i64 {
    let V::Int(value) = signal_field(signal, key) else {
        panic!("Rust ABI: Signal integer")
    };
    value
}
fn signal_dependencies(signal: &V) -> Vec<V> {
    let V::Array(values) = signal_field(signal, "dependencies") else {
        panic!("Rust ABI: Signal dependencies")
    };
    values.get().clone()
}
fn signal_node(kind: i64, mapper: V, dependencies: Vec<V>, cached: V) -> V {
    V::reference(signal_record(vec![
        ("kind", V::Int(kind)),
        ("mapper", mapper),
        ("dependencies", V::Array(Shared::new(dependencies))),
        ("cached", cached),
        ("epoch", V::Int(-1)),
        ("revision", V::Int(0)),
        ("sourceRevision", V::Int(0)),
        ("innerRevision", V::Int(0)),
        ("published", V::Unit),
        ("subscriptionOrder", V::Int(0)),
    ]))
}
fn signal_revision(signal: &V) -> i64 {
    match signal_int(signal, "kind") {
        SIGNAL_MAP | SIGNAL_COMBINE | SIGNAL_CONSTANT | SIGNAL_APPLY => signal_dependencies(signal)
            .iter()
            .map(signal_revision)
            .sum(),
        SIGNAL_DISTINCT | SIGNAL_SWITCH => {
            signal_current(signal);
            signal_int(signal, "revision")
        }
        _ => signal_int(signal, "revision"),
    }
}
fn signal_current(signal: &V) -> V {
    let kind = signal_int(signal, "kind");
    let epoch = SIGNAL_EPOCH.with(Cell::get);
    if kind == SIGNAL_SOURCE || signal_int(signal, "epoch") == epoch {
        return signal_field(signal, "cached");
    }
    let mut dependencies = signal_dependencies(signal);
    let mapper = signal_field(signal, "mapper");
    let value = match kind {
        SIGNAL_MAP => mapper.apply(signal_current(&dependencies[0])),
        SIGNAL_COMBINE => mapper
            .apply(signal_current(&dependencies[0]))
            .apply(signal_current(&dependencies[1])),
        SIGNAL_CONSTANT => signal_field(signal, "cached"),
        SIGNAL_APPLY => signal_current(&dependencies[0]).apply(signal_current(&dependencies[1])),
        SIGNAL_DISTINCT => {
            let next_revision = signal_revision(&dependencies[0]);
            let mut cached = signal_field(signal, "cached");
            if next_revision != signal_int(signal, "sourceRevision") {
                let next = signal_current(&dependencies[0]);
                let equal = mapper.apply(cached.clone()).apply(next.clone()).boolean();
                signal_fields(signal, vec![("sourceRevision", V::Int(next_revision))]);
                if !equal {
                    cached = next;
                    signal_fields(
                        signal,
                        vec![("revision", V::Int(signal_int(signal, "revision") + 1))],
                    );
                }
            }
            cached
        }
        SIGNAL_SWITCH => {
            let outer_revision = signal_revision(&dependencies[0]);
            let changed = if outer_revision != signal_int(signal, "sourceRevision") {
                dependencies[1] = mapper.apply(signal_current(&dependencies[0]));
                signal_fields(
                    signal,
                    vec![
                        ("sourceRevision", V::Int(outer_revision)),
                        ("innerRevision", V::Int(signal_revision(&dependencies[1]))),
                        ("dependencies", V::Array(Shared::new(dependencies.clone()))),
                    ],
                );
                true
            } else {
                let revision = signal_revision(&dependencies[1]);
                let changed = revision != signal_int(signal, "innerRevision");
                signal_fields(signal, vec![("innerRevision", V::Int(revision))]);
                changed
            };
            let value = signal_current(&dependencies[1]);
            if changed {
                signal_fields(
                    signal,
                    vec![("revision", V::Int(signal_int(signal, "revision") + 1))],
                );
            }
            value
        }
        _ => panic!("Rust ABI: Signal kind"),
    };
    signal_fields(
        signal,
        vec![("cached", value.clone()), ("epoch", V::Int(epoch))],
    );
    value
}
fn signal_depth(signal: &V) -> usize {
    signal_dependencies(signal)
        .iter()
        .map(signal_depth)
        .max()
        .map_or(0, |depth| depth + 1)
}
fn signal_affected(
    signal: &V,
    changed: &std::collections::BTreeSet<usize>,
    memo: &mut BTreeMap<usize, bool>,
) -> bool {
    let id = signal.node_id().unwrap();
    if changed.contains(&id) {
        return true;
    }
    if let Some(affected) = memo.get(&id) {
        return *affected;
    }
    let affected = signal_dependencies(signal)
        .iter()
        .any(|dependency| signal_affected(dependency, changed, memo));
    memo.insert(id, affected);
    affected
}
fn signal_subscribers() -> Vec<V> {
    SIGNAL_SUBSCRIBERS.with(|subscribers| {
        let mut subscribers = subscribers.borrow_mut();
        subscribers.retain(|weak| weak.strong_count() > 0);
        subscribers
            .iter()
            .filter_map(Weak::upgrade)
            .map(|node| V::Reference(Shared(Some(node))))
            .collect()
    })
}
fn signal_unsubscribe(subscription: &V) {
    subscription.write_reference(V::Unit);
    let id = subscription.node_id().unwrap();
    SIGNAL_SUBSCRIBERS.with(|subscribers| {
        subscribers
            .borrow_mut()
            .retain(|weak| weak.strong_count() > 0 && weak.as_ptr() as usize != id)
    });
}
fn signal_subscribe(source: V, observer: V, context: &mut NativeContext) -> V {
    let subscribers = signal_subscribers();
    if !subscribers
        .iter()
        .any(|s| signal_field(s, "source").node_id() == source.node_id())
    {
        let order = SIGNAL_SUBSCRIPTION_ORDER.with(|order| {
            order.set(order.get() + 1);
            order.get()
        });
        signal_fields(
            &source,
            vec![
                ("published", V::Int(signal_revision(&source))),
                ("subscriptionOrder", V::Int(order)),
            ],
        );
    }
    let subscription = V::reference(signal_record(vec![
        ("source", source),
        ("observer", observer),
    ]));
    let V::Reference(handle) = &subscription else {
        unreachable!()
    };
    SIGNAL_SUBSCRIBERS.with(|subscribers| {
        subscribers
            .borrow_mut()
            .push(Rc::downgrade(handle.0.as_ref().unwrap()))
    });
    let scope = context
        .scopes
        .last_mut()
        .expect("Signal subscription scope");
    scope
        .subscriptions
        .retain(|s| !matches!(s.read_reference(), V::Unit));
    scope.subscriptions.push(subscription.clone());
    subscription
}

// Calculate every staged value before mutating a source. A defective mapper
// leaves all source snapshots unchanged.
fn signal_commit(changes: V, context: &mut NativeContext) -> NativeOutcome {
    let V::Array(changes) = changes else {
        panic!("Rust ABI: SignalChange array")
    };
    let mut staged: Vec<(V, V)> = Vec::new();
    for change in changes.get() {
        let target = change.field("target");
        let index = staged
            .iter()
            .position(|(s, _)| s.node_id() == target.node_id());
        let value = if change.field("update").boolean() {
            let previous = index.map_or_else(|| signal_current(&target), |i| staged[i].1.clone());
            change.field("value").apply(previous)
        } else {
            change.field("value")
        };
        if let Some(index) = index {
            staged[index].1 = value;
        } else {
            staged.push((target, value));
        }
    }
    context.checkpoint()?;
    let changed = staged
        .iter()
        .map(|(s, _)| s.node_id().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    for (target, value) in staged {
        signal_fields(
            &target,
            vec![
                ("cached", value),
                ("revision", V::Int(signal_int(&target, "revision") + 1)),
            ],
        );
    }
    if !changed.is_empty() {
        SIGNAL_EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
    }
    let subscribers = signal_subscribers();
    let mut candidates = Vec::<V>::new();
    let mut affected = BTreeMap::new();
    for subscription in &subscribers {
        let source = signal_field(subscription, "source");
        if !candidates.iter().any(|s| s.node_id() == source.node_id())
            && signal_affected(&source, &changed, &mut affected)
        {
            candidates.push(source);
        }
    }
    for source in &candidates {
        signal_current(source);
    }
    candidates.retain(|source| {
        let revision = signal_revision(source);
        if matches!(signal_field(source, "published"), V::Int(old) if old == revision) {
            return false;
        }
        signal_fields(source, vec![("published", V::Int(revision))]);
        true
    });
    candidates.sort_by_key(|source| {
        (
            signal_depth(source),
            signal_int(source, "subscriptionOrder"),
        )
    });
    Ok(V::Array(Shared::new(candidates)))
}

struct SignalPublication {
    sources: std::collections::VecDeque<V>,
    notifications: std::collections::VecDeque<V>,
    current: Option<V>,
    first_error: Option<NativeExit>,
    previous_mask: bool,
}
impl SignalPublication {
    fn resume(
        mut self,
        result: NativeOutcome,
        context: &mut NativeContext,
        stack: &mut Vec<NativeFrame>,
    ) -> NativeState {
        if let Err(error) = result {
            if let Some(subscriber) = self.current.take() {
                signal_unsubscribe(&subscriber);
            }
            if self.first_error.is_none() {
                self.first_error = Some(error);
            }
        }
        self.current = None;
        loop {
            if let Some(notification) = self.notifications.pop_front() {
                let subscriber = notification.index(0);
                if matches!(subscriber.read_reference(), V::Unit) {
                    continue;
                }
                let plan = catch_native(|| {
                    Ok(signal_field(&subscriber, "observer").apply(notification.index(1)))
                });
                self.current = Some(subscriber);
                stack.push(NativeFrame::SignalPublication(self));
                return match plan {
                    Ok(plan) => NativeState::Evaluate(plan),
                    Err(error) => NativeState::Return(Err(error)),
                };
            }
            if let Some(source) = self.sources.pop_front() {
                let value = signal_current(&source);
                for subscriber in signal_subscribers() {
                    if signal_field(&subscriber, "source").node_id() == source.node_id() {
                        self.notifications
                            .push_back(V::Tuple(Shared::new(vec![subscriber, value.clone()])));
                    }
                }
                continue;
            }
            let queued = SIGNAL_TRANSACTIONS.with(|queue| queue.borrow_mut().pop_front());
            let Some(changes) = queued else {
                SIGNAL_PUBLICATION.with(|active| active.set(false));
                context.masked = self.previous_mask;
                return NativeState::Return(self.first_error.map_or(Ok(V::Unit), Err));
            };
            match catch_native(|| signal_commit(changes, context)) {
                Ok(V::Array(sources)) => self.sources.extend(sources.get().iter().cloned()),
                Err(error) => {
                    if self.first_error.is_none() {
                        self.first_error = Some(error);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
fn signal_transaction(
    changes: V,
    context: &mut NativeContext,
    stack: &mut Vec<NativeFrame>,
) -> NativeState {
    if let Err(error) = context.checkpoint() {
        return NativeState::Return(Err(error));
    }
    if SIGNAL_PUBLICATION.with(Cell::get) {
        SIGNAL_TRANSACTIONS.with(|queue| queue.borrow_mut().push_back(changes));
        return NativeState::Return(Ok(V::Unit));
    }
    match catch_native(|| signal_commit(changes, context)) {
        Ok(V::Array(sources)) => {
            let previous_mask = std::mem::replace(&mut context.masked, true);
            SIGNAL_PUBLICATION.with(|active| active.set(true));
            SignalPublication {
                sources: sources.get().iter().cloned().collect(),
                notifications: std::collections::VecDeque::new(),
                current: None,
                first_error: None,
                previous_mask,
            }
            .resume(Ok(V::Unit), context, stack)
        }
        Err(error) => NativeState::Return(Err(error)),
        _ => unreachable!(),
    }
}

fn native_signal_operation(name: &str, mut args: Vec<V>) -> V {
    match name {
        "std/signal::make" => V::leaf(args, |c, _| {
            Ok(signal_node(SIGNAL_SOURCE, V::Unit, vec![], c[0].clone()))
        }),
        "std/signal::read" => V::leaf(args, |c, _| Ok(signal_current(&c[0]))),
        "std/signal::set" | "std/signal::update" => {
            let change = native_signal_operation(
                if name.ends_with("::set") {
                    "std/signal::planSet"
                } else {
                    "std/signal::planUpdate"
                },
                args,
            );
            V::effect(
                Instruction::SignalTransaction,
                vec![V::Array(Shared::new(vec![change]))],
            )
        }
        "std/signal::planSet" | "std/signal::planUpdate" => signal_record(vec![
            ("target", args.remove(1)),
            ("value", args.remove(0)),
            ("update", V::Bool(name.ends_with("::planUpdate"))),
        ]),
        "std/signal::transaction" => V::effect(Instruction::SignalTransaction, args),
        "std/signal::subscribe" => V::effect(Instruction::SignalSubscribe, args),
        "std/signal::unsubscribe" => V::leaf(args, |c, context| {
            signal_unsubscribe(&c[0]);
            for scope in &mut context.scopes {
                scope
                    .subscriptions
                    .retain(|s| !matches!(s.read_reference(), V::Unit));
            }
            Ok(V::Unit)
        }),
        "std/signal::constant" => signal_node(SIGNAL_CONSTANT, V::Unit, vec![], args.remove(0)),
        "std/signal::map" | "std/signal::combine" | "std/signal::apply" => {
            let kind = match name {
                "std/signal::map" => SIGNAL_MAP,
                "std/signal::combine" => SIGNAL_COMBINE,
                _ => SIGNAL_APPLY,
            };
            let mapper = if kind == SIGNAL_APPLY {
                V::Unit
            } else {
                args.remove(0)
            };
            signal_node(kind, mapper, args, V::Unit)
        }
        "std/signal::distinct" | "std/signal::switchMap" => {
            let mapper = args.remove(0);
            let source = args.remove(0);
            let revision = signal_revision(&source);
            let mut dependencies = vec![source.clone()];
            let kind = if name.ends_with("::distinct") {
                SIGNAL_DISTINCT
            } else {
                SIGNAL_SWITCH
            };
            let value = if kind == SIGNAL_DISTINCT {
                signal_current(&source)
            } else {
                let inner = mapper.apply(signal_current(&source));
                dependencies.push(inner.clone());
                signal_current(&inner)
            };
            let inner_revision = if kind == SIGNAL_SWITCH {
                signal_revision(&dependencies[1])
            } else {
                0
            };
            let signal = signal_node(kind, mapper, dependencies, value);
            signal_fields(
                &signal,
                vec![
                    ("sourceRevision", V::Int(revision)),
                    ("innerRevision", V::Int(inner_revision)),
                    ("epoch", V::Int(SIGNAL_EPOCH.with(Cell::get))),
                ],
            );
            signal
        }
        _ => panic!("Rust ABI: unsupported Signal operation {name}"),
    }
}
fn native_signal_dictionary() -> V {
    V::dict(vec![
        (
            "map",
            V::fun(vec![], |_, mapper| {
                V::fun(vec![mapper], |c, source| {
                    native_signal_operation("std/signal::map", vec![c[0].clone(), source])
                })
            }),
        ),
        (
            "pure",
            V::fun(vec![], |_, value| {
                native_signal_operation("std/signal::constant", vec![value])
            }),
        ),
        (
            "apply",
            V::fun(vec![], |_, functions| {
                V::fun(vec![functions], |c, values| {
                    native_signal_operation("std/signal::apply", vec![c[0].clone(), values])
                })
            }),
        ),
    ])
}

#[cfg(test)]
mod signal_tests {
    use super::*;

    fn make(value: i64) -> V {
        signal_node(SIGNAL_SOURCE, V::Unit, vec![], V::Int(value))
    }
    fn changes(changes: Vec<V>) -> V {
        V::Array(Shared::new(changes))
    }
    fn set(value: V, target: V) -> V {
        native_signal_operation("std/signal::planSet", vec![value, target])
    }
    fn transaction(values: Vec<V>) -> V {
        native_signal_operation("std/signal::transaction", vec![changes(values)])
    }
    fn then(first: V, second: V) -> V {
        V::bind(first, V::fun(vec![second], |c, _| c[0].clone()))
    }
    fn observer(log: V) -> V {
        V::fun(vec![log], |c, value| {
            V::leaf(vec![c[0].clone(), value], |c, _| {
                let V::Array(values) = c[0].read_reference() else {
                    panic!()
                };
                let mut values = values.get().clone();
                values.push(c[1].clone());
                c[0].write_reference(changes(values));
                Ok(V::Unit)
            })
        })
    }
    fn subscribe(observer: V, source: V) -> V {
        native_signal_operation("std/signal::subscribe", vec![observer, source])
    }
    fn clean() {
        assert!(signal_subscribers().is_empty());
        assert!(!SIGNAL_PUBLICATION.with(Cell::get));
        assert!(SIGNAL_TRANSACTIONS.with(|q| q.borrow().is_empty()));
        clear_dictionaries();
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn diamond_settles_once_and_stages_repeated_updates_in_order() {
        enable_cycle_collection();
        let source = make(1);
        let calls = V::reference(V::Int(0));
        let mapper = V::fun(vec![calls.clone()], |c, v| {
            c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
            v
        });
        let left = native_signal_operation("std/signal::map", vec![mapper.clone(), source.clone()]);
        let right = native_signal_operation("std/signal::map", vec![mapper, source.clone()]);
        let sum = native_signal_operation(
            "std/signal::combine",
            vec![
                V::fun(vec![], |_, left| {
                    V::fun(vec![left], |c, right| binary("+", c[0].clone(), right))
                }),
                left.clone(),
                right.clone(),
            ],
        );
        let log = V::reference(changes(vec![]));
        let increment = V::fun(vec![], |_, value| binary("+", value, V::Int(1)));
        let plan = then(
            subscribe(observer(log.clone()), sum.clone()),
            transaction(vec![
                set(V::Int(5), source.clone()),
                native_signal_operation(
                    "std/signal::planUpdate",
                    vec![increment.clone(), source.clone()],
                ),
                native_signal_operation("std/signal::planUpdate", vec![increment, source.clone()]),
            ]),
        );
        let mut context = NativeContext::default();
        assert!(matches!(context.root(plan), Ok(V::Unit)));
        assert_eq!(log.read_reference().render(), "[2, 14]");
        assert_eq!(calls.read_reference().render(), "4");
        assert_eq!(signal_current(&sum).render(), "14");
        drop((source, left, right, sum, calls, log, context));
        clean();
    }

    #[test]
    fn observer_transactions_are_fifo_and_never_reenter() {
        enable_cycle_collection();
        let source = make(0);
        let log = V::reference(changes(vec![]));
        let active = V::reference(V::Bool(false));
        let callback = V::fun(
            vec![source.clone(), log.clone(), active.clone()],
            |c, value| {
                let begin = V::leaf(vec![c[1].clone(), c[2].clone(), value.clone()], |c, _| {
                    assert!(!c[1].read_reference().boolean());
                    c[1].write_reference(V::Bool(true));
                    let V::Array(values) = c[0].read_reference() else {
                        panic!()
                    };
                    let mut values = values.get().clone();
                    values.push(c[2].clone());
                    c[0].write_reference(changes(values));
                    Ok(V::Unit)
                });
                let V::Int(value) = value else { panic!() };
                let updates = if (1..3).contains(&value) {
                    transaction(vec![set(V::Int(value + 1), c[0].clone())])
                } else {
                    native_operation("std/effect::succeed", vec![V::Unit])
                };
                then(
                    begin,
                    then(
                        updates,
                        V::leaf(vec![c[2].clone()], |c, _| {
                            c[0].write_reference(V::Bool(false));
                            Ok(V::Unit)
                        }),
                    ),
                )
            },
        );
        let mut context = NativeContext::default();
        assert!(matches!(
            context.root(then(
                subscribe(callback, source.clone()),
                transaction(vec![set(V::Int(1), source.clone())])
            )),
            Ok(V::Unit)
        ));
        assert_eq!(log.read_reference().render(), "[0, 1, 2, 3]");
        assert!(!active.read_reference().boolean());
        drop((source, log, active, context));
        clean();
    }

    #[test]
    fn switch_releases_old_inner_and_distinct_uses_selected_equality() {
        enable_cycle_collection();
        let old = make(10);
        let new = make(20);
        let outer = make(0);
        let selected = native_signal_operation(
            "std/signal::switchMap",
            vec![
                V::fun(vec![old.clone(), new.clone()], |c, value| {
                    if value.render() == "0" {
                        c[0].clone()
                    } else {
                        c[1].clone()
                    }
                }),
                outer.clone(),
            ],
        );
        let equal = V::fun(vec![], |_, left| {
            V::fun(vec![left], |c, right| {
                V::Bool(c[0].render().len() == right.render().len())
            })
        });
        let distinct =
            native_signal_operation("std/signal::distinct", vec![equal, selected.clone()]);
        let log = V::reference(changes(vec![]));
        let mut context = NativeContext::default();
        let plan = then(
            subscribe(observer(log.clone()), distinct.clone()),
            then(
                transaction(vec![
                    set(V::Int(1), outer.clone()),
                    set(V::Int(30), old.clone()),
                    set(V::Int(21), new.clone()),
                ]),
                then(
                    transaction(vec![set(V::Int(100), old.clone())]),
                    transaction(vec![set(V::Int(200), new.clone())]),
                ),
            ),
        );
        assert!(matches!(context.root(plan), Ok(V::Unit)));
        assert_eq!(log.read_reference().render(), "[10, 200]");
        assert_eq!(signal_current(&selected).render(), "200");
        drop((old, new, outer, selected, distinct, log, context));
        clean();

        let outer = make(0);
        let selected = native_signal_operation(
            "std/signal::switchMap",
            vec![
                V::fun(vec![], |_, value| {
                    make(match value {
                        V::Int(v) => v,
                        _ => panic!(),
                    })
                }),
                outer.clone(),
            ],
        );
        signal_current(&selected);
        let baseline = LIVE.with(Cell::get);
        for i in 1..1000 {
            signal_fields(&outer, vec![("cached", V::Int(i)), ("revision", V::Int(i))]);
            SIGNAL_EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
            signal_current(&selected);
            assert_eq!(LIVE.with(Cell::get), baseline);
        }
        drop((outer, selected));
        clean();
    }

    #[test]
    fn unsubscribe_and_scope_outcomes_release_observers_and_bound_storage() {
        enable_cycle_collection();
        for outcome in 0..4 {
            let source = make(0);
            let captured = V::text("observer capture");
            let weak = match &captured {
                V::Text(v) => Rc::downgrade(v.0.as_ref().unwrap()),
                _ => unreachable!(),
            };
            let callback = V::fun(vec![captured], |_, _| {
                native_operation("std/effect::succeed", vec![V::Unit])
            });
            let terminal = V::leaf(
                vec![],
                match outcome {
                    0 => |_, _| Ok(V::Unit),
                    1 => |_, _| Err(NativeExit::failure(V::Int(1))),
                    2 => |_, _| panic!("observer scope defect"),
                    _ => |_, context| {
                        context.cancelled = true;
                        context.checkpoint().map(|_| V::Unit)
                    },
                },
            );
            let plan = then(subscribe(callback, source.clone()), terminal);
            let mut context = NativeContext::default();
            let result = context.root(plan);
            assert_eq!(result.is_ok(), outcome == 0);
            assert!(weak.upgrade().is_none());
            assert!(signal_subscribers().is_empty());
            drop((result, source, context));
            clean();
        }
        let source = make(0);
        let mut context = NativeContext::default();
        context.scopes.push(NativeScope::default());
        let before = LIVE.with(Cell::get);
        for _ in 0..2000 {
            let subscription = signal_subscribe(
                source.clone(),
                V::fun(vec![], |_, _| {
                    native_operation("std/effect::succeed", vec![V::Unit])
                }),
                &mut context,
            );
            signal_unsubscribe(&subscription);
            signal_unsubscribe(&subscription);
            assert!(LIVE.with(Cell::get) <= before + 3);
            assert!(context.scopes[0].subscriptions.len() <= 1);
        }
        drop((source, context));
        clean();
    }

    #[test]
    fn defective_staging_is_atomic_and_defective_observer_does_not_skip_others() {
        enable_cycle_collection();
        let source = make(0);
        let plan = transaction(vec![
            set(V::Int(1), source.clone()),
            native_signal_operation(
                "std/signal::planUpdate",
                vec![
                    V::fun(vec![], |_, _| panic!("staging defect")),
                    source.clone(),
                ],
            ),
        ]);
        let mut context = NativeContext::default();
        assert!(matches!(context.root(plan), Err(NativeExit::Defect(_))));
        assert_eq!(signal_current(&source).render(), "0");
        let log = V::reference(changes(vec![]));
        let bad = V::fun(vec![source.clone()], |c, value| {
            if value.render() == "0" {
                return native_operation("std/effect::succeed", vec![V::Unit]);
            }
            then(
                transaction(vec![set(V::Int(2), c[0].clone())]),
                V::leaf(vec![], |_, _| panic!("observer defect")),
            )
        });
        let plan = then(
            subscribe(bad, source.clone()),
            then(
                subscribe(observer(log.clone()), source.clone()),
                transaction(vec![set(V::Int(1), source.clone())]),
            ),
        );
        assert!(matches!(context.root(plan), Err(NativeExit::Defect(_))));
        assert_eq!(log.read_reference().render(), "[0, 1, 2]");
        drop((source, log, context));
        clean();
    }
}

#[cfg(test)]
mod signal_lifetime_tests {
    use super::*;

    #[test]
    fn many_subscribers_release_captures_and_registry_entries_on_unsubscribe() {
        enable_cycle_collection();
        let source = signal_node(SIGNAL_SOURCE, V::Unit, vec![], V::Int(0));
        let mut context = NativeContext::default();
        context.scopes.push(NativeScope::default());
        let baseline = LIVE.with(Cell::get);
        for _round in 0..3 {
            let allocations = ALLOCATIONS.with(Cell::get);
            PEAK.with(|peak| peak.set(baseline));
            let mut subscriptions = Vec::new();
            for _ in 0..1024 {
                let observer = V::fun(vec![V::text("capture")], |_, _| {
                    native_operation("std/effect::succeed", vec![V::Unit])
                });
                subscriptions.push(signal_subscribe(source.clone(), observer, &mut context));
            }
            assert_eq!(SIGNAL_SUBSCRIBERS.with(|s| s.borrow().len()), 1024);
            assert!(ALLOCATIONS.with(Cell::get) - allocations <= 4097);
            assert!(PEAK.with(Cell::get) <= baseline + 4098);
            assert!(LIVE.with(Cell::get) <= baseline + 4096);
            for subscription in &subscriptions {
                signal_unsubscribe(subscription);
            }
            assert!(SIGNAL_SUBSCRIBERS.with(|s| s.borrow().is_empty()));
            assert_eq!(LIVE.with(Cell::get), baseline + 1024);
            drop(subscriptions);
            context.scopes[0].subscriptions.clear();
            assert_eq!(LIVE.with(Cell::get), baseline);
        }
        drop((context, source));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn cancellation_before_commit_preserves_sources() {
        enable_cycle_collection();
        let source = signal_node(SIGNAL_SOURCE, V::Unit, vec![], V::Int(0));
        let changes = V::Array(Shared::new(vec![native_signal_operation(
            "std/signal::planSet",
            vec![V::Int(1), source.clone()],
        )]));
        let mut context = NativeContext {
            cancelled: true,
            ..NativeContext::default()
        };
        assert!(matches!(
            signal_commit(changes, &mut context),
            Err(NativeExit::Cancelled)
        ));
        assert_eq!(signal_current(&source).render(), "0");
        drop((context, source));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}

#[cfg(test)]
mod signal_publication_order_tests {
    use super::*;

    #[test]
    fn removing_one_observer_preserves_the_sources_publication_order() {
        enable_cycle_collection();
        let first = signal_node(SIGNAL_SOURCE, V::Unit, vec![], V::Int(0));
        let second = signal_node(SIGNAL_SOURCE, V::Unit, vec![], V::Int(0));
        let mut context = NativeContext::default();
        context.scopes.push(NativeScope::default());
        let observer = || {
            V::fun(vec![], |_, _| {
                native_operation("std/effect::succeed", vec![V::Unit])
            })
        };
        let removed = signal_subscribe(first.clone(), observer(), &mut context);
        signal_subscribe(second.clone(), observer(), &mut context);
        signal_subscribe(first.clone(), observer(), &mut context);
        signal_unsubscribe(&removed);
        let changes = V::Array(Shared::new(vec![
            native_signal_operation("std/signal::planSet", vec![V::Int(1), first.clone()]),
            native_signal_operation("std/signal::planSet", vec![V::Int(1), second.clone()]),
        ]));
        let V::Array(sources) = signal_commit(changes, &mut context).unwrap_or_else(|_| panic!())
        else {
            panic!()
        };
        assert_eq!(sources.get()[0].node_id(), first.node_id());
        assert_eq!(sources.get()[1].node_id(), second.node_id());
        for scope in &context.scopes {
            for subscription in &scope.subscriptions {
                signal_unsubscribe(subscription);
            }
        }
        drop((first, second, removed, context, sources));
        assert!(SIGNAL_SUBSCRIBERS.with(|s| s.borrow().is_empty()));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
