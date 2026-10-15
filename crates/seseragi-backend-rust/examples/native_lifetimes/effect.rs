//! Cold Effect instruction machine. All managed continuations are explicit roots.
use super::{
    heap::{EffectCode, Handle, Heap},
    scope::Exit,
};
type Outcome = Result<Handle, Exit>;

#[derive(Clone, Copy)]
pub enum Instruction {
    Native(EffectCode),
    Bind,
    Scoped,
    AcquireRelease,
    Recover,
    Release,
    Defer,
}

enum Frame {
    Bind(Handle),
    Recover(Handle),
    Register(Handle),
    Scope,
    Cleanup {
        original: Outcome,
        defects: Vec<String>,
        previous_mask: bool,
    },
}
enum State {
    Evaluate(Handle),
    Return(Outcome),
}

pub struct Context {
    frames: Vec<Vec<Handle>>,
    pub output: Vec<String>,
    pub cancelled: bool,
    masked: bool,
    pub checkpoints: usize,
    pub reclaimed: usize,
    pub candidate_collection: bool,
    pub gc_every: usize,
    pub peak_frames: usize,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            output: Vec::new(),
            cancelled: false,
            masked: false,
            checkpoints: 0,
            reclaimed: 0,
            candidate_collection: false,
            gc_every: 1,
            peak_frames: 0,
        }
    }
}
impl Context {
    pub fn checkpoint(&mut self, heap: &Heap) -> Result<(), Exit> {
        self.checkpoints += 1;
        if self.gc_every > 0 && self.checkpoints % self.gc_every == 0 {
            self.reclaimed += (if self.candidate_collection {
                heap.collect_candidates()
            } else {
                heap.collect()
            })
            .map_err(|e| Exit::Defect(vec![e.into()]))?
            .reclaimed;
        }
        if self.cancelled && !self.masked {
            Err(Exit::Cancelled)
        } else {
            Ok(())
        }
    }
    pub fn root(&mut self, heap: &Heap, effect: &Handle) -> Outcome {
        assert!(
            self.frames.is_empty(),
            "native leaf cannot recursively enter the interpreter"
        );
        self.frames.push(Vec::new());
        let initial = match self.checkpoint(heap) {
            Ok(()) => State::Evaluate(effect.clone()),
            Err(exit) => State::Return(Err(exit)),
        };
        self.run(heap, initial, vec![Frame::Scope])
    }
    fn run(&mut self, heap: &Heap, mut state: State, mut stack: Vec<Frame>) -> Outcome {
        loop {
            self.peak_frames = self.peak_frames.max(stack.len());
            state = match state {
                State::Evaluate(effect) => {
                    match effect.plan() {
                        Err(error) => State::Return(Err(error)),
                        Ok((c, instruction)) => match instruction {
                            Instruction::Native(code) => {
                                let result =
                                    self.checkpoint(heap).and_then(|_| code(heap, &c, self));
                                State::Return(result)
                            }
                            Instruction::Bind => {
                                stack.push(Frame::Bind(c[1].clone()));
                                State::Evaluate(c[0].clone())
                            }
                            Instruction::Recover => {
                                stack.push(Frame::Recover(c[0].clone()));
                                State::Evaluate(c[1].clone())
                            }
                            Instruction::Scoped => {
                                self.frames.push(Vec::new());
                                stack.push(Frame::Scope);
                                State::Evaluate(c[0].clone())
                            }
                            Instruction::AcquireRelease => {
                                stack.push(Frame::Register(c[1].clone()));
                                State::Evaluate(c[0].clone())
                            }
                            Instruction::Release => {
                                // The callback's pure prefix runs at FINALIZATION.
                                match c[0].call(heap, c[1].clone()) {
                                    Ok(plan) => State::Evaluate(plan),
                                    Err(exit) => State::Return(Err(exit)),
                                }
                            }
                            Instruction::Defer => match self
                                .checkpoint(heap)
                                .and_then(|_| c[0].call(heap, heap.unit()))
                            {
                                Ok(plan) => State::Evaluate(plan),
                                Err(exit) => State::Return(Err(exit)),
                            },
                        },
                    }
                }
                State::Return(result) => {
                    let Some(frame) = stack.pop() else {
                        return result;
                    };
                    match frame {
                        Frame::Bind(next) => match result {
                            Ok(value) => {
                                match self.checkpoint(heap).and_then(|_| next.call(heap, value)) {
                                    Ok(plan) => State::Evaluate(plan),
                                    Err(exit) => State::Return(Err(exit)),
                                }
                            }
                            Err(exit) => State::Return(Err(exit)),
                        },
                        Frame::Recover(handler) => match result {
                            Err(Exit::Failure(error)) => {
                                match self
                                    .checkpoint(heap)
                                    .and_then(|_| handler.call(heap, heap.text(&error)))
                                {
                                    Ok(plan) => State::Evaluate(plan),
                                    Err(exit) => State::Return(Err(exit)),
                                }
                            }
                            result => State::Return(result),
                        },
                        Frame::Register(release) => match result {
                            Ok(resource) => {
                                let finalizer = heap.instruction(
                                    Instruction::Release,
                                    vec![release, resource.clone()],
                                );
                                self.frames
                                    .last_mut()
                                    .expect("owning scope")
                                    .push(finalizer);
                                // No checkpoint between acquisition and registration.
                                State::Return(self.checkpoint(heap).map(|_| resource))
                            }
                            Err(exit) => State::Return(Err(exit)),
                        },
                        Frame::Scope => {
                            let previous_mask = std::mem::replace(&mut self.masked, true);
                            self.cleanup(heap, &mut stack, result, Vec::new(), previous_mask)
                        }
                        Frame::Cleanup {
                            original,
                            mut defects,
                            previous_mask,
                        } => {
                            match result {
                                Ok(_) => {}
                                Err(Exit::Defect(errors)) => defects.extend(errors),
                                Err(_) => {
                                    defects.push("finalizer produced failure/cancellation".into())
                                }
                            }
                            self.cleanup(heap, &mut stack, original, defects, previous_mask)
                        }
                    }
                }
            };
        }
    }
    fn cleanup(
        &mut self,
        _heap: &Heap,
        stack: &mut Vec<Frame>,
        original: Outcome,
        defects: Vec<String>,
        previous_mask: bool,
    ) -> State {
        let next = self.frames.last_mut().expect("owning scope").pop();
        if let Some(finalizer) = next {
            stack.push(Frame::Cleanup {
                original,
                defects,
                previous_mask,
            });
            State::Evaluate(finalizer)
        } else {
            self.frames.pop();
            self.masked = previous_mask;
            State::Return(if defects.is_empty() {
                original
            } else {
                Err(Exit::Defect(defects))
            })
        }
    }
    #[cfg(test)]
    fn finish(&mut self, heap: &Heap, original: Outcome) -> Outcome {
        self.run(heap, State::Return(original), vec![Frame::Scope])
    }
}

pub fn operation(heap: &Heap, name: &str, args: Vec<Handle>) -> Result<Handle, Exit> {
    let effect = match (name, args.len()) {
        ("std/effect::succeed" | "effect.succeed", 1) => {
            heap.effect(args, |_, c, _| Ok(c[0].clone()))
        }
        ("std/effect::fail" | "effect.fail", 1) => {
            heap.effect(args, |_, c, _| Err(Exit::Failure(c[0].show())))
        }
        ("std/ref::make", 1) => heap.effect(args, |h, c, _| Ok(h.reference(c[0].clone()))),
        ("std/ref::get", 1) => heap.effect(args, |_, c, _| Ok(c[0].read())),
        ("std/ref::set", 2) => heap.effect(args, |h, c, _| {
            c[1].write(c[0].clone());
            Ok(h.unit())
        }),
        ("std/effect::scoped", 1) => heap.instruction(Instruction::Scoped, args),
        ("std/effect::acquireRelease", 2) => heap.instruction(Instruction::AcquireRelease, args),
        ("std/effect::recover", 2) => heap.instruction(Instruction::Recover, args),
        ("std/effect::defer", 1) => heap.instruction(Instruction::Defer, args),
        ("std/prelude::println" | "console.println", 1) => heap.effect(args, |h, c, ctx| {
            ctx.output.push(c[0].show());
            Ok(h.unit())
        }),
        _ => {
            return Err(Exit::Defect(vec![format!(
                "unsupported research operation: {name}"
            )]))
        }
    };
    Ok(effect)
}

pub fn binary(heap: &Heap, operator: &str, left: Handle, right: Handle) -> Result<Handle, Exit> {
    let a = left.integer();
    let b = right.integer();
    match operator {
        "+" | "-" | "*" => {
            let result = match operator {
                "+" => a.checked_add(b),
                "-" => a.checked_sub(b),
                _ => a.checked_mul(b),
            };
            match result.filter(|n| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(n)) {
                Some(n) => Ok(heap.int(n)),
                None => Err(Exit::Defect(vec!["Seseragi Int overflow".into()])),
            }
        }
        "==" => Ok(heap.boolean(a == b)),
        "<" => Ok(heap.boolean(a < b)),
        _ => Err(Exit::Defect(vec!["unsupported research binary".into()])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cold_effect_replays_and_releases_captures() {
        let heap = Heap::default();
        let reference = heap.reference(heap.int(0));
        let effect = heap.effect(vec![reference.clone()], |h, c, _| {
            let next = c[0].read().integer() + 1;
            c[0].write(h.int(next));
            Ok(h.unit())
        });
        assert_eq!(reference.read().integer(), 0);
        Context::default()
            .root(&heap, &effect)
            .unwrap_or_else(|e| panic!("{e:?}"));
        Context::default()
            .root(&heap, &effect)
            .unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(reference.read().integer(), 2);
        drop(effect);
        drop(reference);
        heap.collect().unwrap();
        assert_eq!(heap.live(), 0);
    }

    #[test]
    fn cancellation_is_masked_and_defects_do_not_skip_remaining_finalizers() {
        let heap = Heap::default();
        let order = heap.reference(heap.int(0));
        let first = heap.effect(vec![order.clone()], |h, c, ctx| {
            assert!(ctx.cancelled);
            assert!(ctx.masked);
            c[0].write(h.int(c[0].read().integer() * 10 + 1));
            Err(Exit::Defect(vec!["first-defect".into()]))
        });
        let second = heap.effect(vec![order.clone()], |h, c, ctx| {
            assert!(ctx.masked);
            c[0].write(h.int(c[0].read().integer() * 10 + 2));
            Err(Exit::Defect(vec!["second-defect".into()]))
        });
        let mut context = Context::default();
        context.frames.push(vec![first, second]);
        context.cancelled = true;
        let result = context.finish(&heap, Err(Exit::Failure("expected".into())));
        assert!(
            matches!(result, Err(Exit::Defect(errors)) if errors == ["second-defect", "first-defect"])
        );
        assert_eq!(order.read().integer(), 21);
        assert!(!context.masked);
        assert!(context.frames.is_empty());
        drop(order);
        assert_eq!(heap.live(), 0);
    }

    #[test]
    fn long_bind_and_scope_chains_run_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                for kind in ["left-bind", "nested-scope", "defer"] {
                    let h = Heap::default();
                    let mut plan = operation(&h, "std/effect::succeed", vec![h.int(42)])
                        .unwrap_or_else(|e| panic!("{e:?}"));
                    let next = h.callable(vec![], |h, _, value| {
                        operation(h, "std/effect::succeed", vec![value])
                    });
                    for _ in 0..100_000 {
                        plan = match kind {
                            "left-bind" => h.bind(plan, next.clone()),
                            "nested-scope" => h.instruction(Instruction::Scoped, vec![plan]),
                            _ => {
                                let thunk = h.callable(vec![plan], |_, c, _| Ok(c[0].clone()));
                                h.instruction(Instruction::Defer, vec![thunk])
                            }
                        };
                    }
                    // Stress graph-sized GC separately. This test isolates stack
                    // depth; its immutable original plan retains the whole chain.
                    let mut ctx = Context {
                        gc_every: usize::MAX,
                        ..Context::default()
                    };
                    assert_eq!(
                        ctx.root(&h, &plan)
                            .unwrap_or_else(|e| panic!("{e:?}"))
                            .integer(),
                        42
                    );
                    assert!(ctx.frames.is_empty());
                    drop(plan);
                    drop(next);
                    h.collect().unwrap();
                    assert_eq!(h.live(), 0, "{kind}");
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn continuation_roots_survive_gc_and_cancellation_skips_them() {
        for candidates in [false, true] {
            let h = Heap::default();
            let tail = h.text("retained");
            let next = h.callable(vec![tail.clone()], |h, c, _| {
                operation(h, "std/effect::succeed", vec![c[0].clone()])
            });
            let source = operation(&h, "std/effect::succeed", vec![h.unit()])
                .unwrap_or_else(|e| panic!("{e:?}"));
            let plan = h.bind(source, next);
            drop(tail);
            let mut ctx = Context {
                candidate_collection: candidates,
                ..Context::default()
            };
            assert_eq!(
                ctx.root(&h, &plan)
                    .unwrap_or_else(|e| panic!("{e:?}"))
                    .show(),
                "retained"
            );
            ctx.cancelled = true;
            assert!(matches!(ctx.root(&h, &plan), Err(Exit::Cancelled)));
            drop(plan);
            h.collect().unwrap();
            assert_eq!(h.live(), 0);
        }
    }
}
