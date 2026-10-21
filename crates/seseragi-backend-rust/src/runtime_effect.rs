// Cold, resumable Effect machine. User callbacks return values/plans; they
// never recursively enter this interpreter. Frames and scopes own ordinary V
// handles, so the collector can discover every live continuation as a root.
type NativeOutcome = Result<V, NativeExit>;
#[derive(Clone)]
enum NativeExit {
    Failure(V),
    Defect(Vec<String>),
    Cancelled,
}
type NativeCode = fn(&[V], &mut NativeContext) -> NativeOutcome;
#[derive(Clone, Copy)]
enum Instruction {
    Native(NativeCode),
    Bind,
    Defer,
    Scoped,
    AcquireRelease,
    Recover,
    MapError,
    Release,
    Concurrent(ConcurrentOperation),
    FileSystem(NativeFileOperation),
}
struct EffectPlan {
    instruction: Instruction,
    captures: Vec<V>,
}
impl V {
    fn effect(instruction: Instruction, captures: Vec<V>) -> Self {
        Self::Effect(Shared::new(EffectPlan {
            instruction,
            captures,
        }))
    }
    fn leaf(captures: Vec<V>, code: NativeCode) -> Self {
        Self::effect(Instruction::Native(code), captures)
    }
    fn bind(source: V, next: V) -> Self {
        Self::effect(Instruction::Bind, vec![source, next])
    }
    fn plan(&self) -> (Instruction, Vec<V>) {
        let Self::Effect(value) = self else {
            panic!("Rust ABI: expected Effect")
        };
        (value.get().instruction, value.get().captures.clone())
    }
}

fn native_panic_message(error: Box<dyn Any + Send>) -> String {
    error
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| error.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_else(|| "native host panic".into())
}
fn native_panic(error: Box<dyn Any + Send>) -> NativeExit {
    NativeExit::Defect(vec![native_panic_message(error)])
}
fn catch_native(action: impl FnOnce() -> NativeOutcome) -> NativeOutcome {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(action))
        .unwrap_or_else(|error| Err(native_panic(error)))
}
enum NativeFrame {
    Bind(V),
    Recover(V),
    MapError(V),
    Register(V),
    Scope,
    Cleanup {
        original: NativeOutcome,
        defects: Vec<String>,
        previous_mask: bool,
    },
}
enum NativeState {
    Evaluate(V),
    Return(NativeOutcome),
}
struct NativeContext {
    scopes: Vec<NativeScope>,
    cancelled: bool,
    masked: bool,
    checkpoints: usize,
    gc_every: usize,
    reclaimed: usize,
    peak_frames: usize,
}
impl Default for NativeContext {
    fn default() -> Self {
        Self {
            scopes: Vec::new(),
            cancelled: false,
            masked: false,
            checkpoints: 0,
            gc_every: 512,
            reclaimed: 0,
            peak_frames: 0,
        }
    }
}
impl NativeContext {
    fn checkpoint(&mut self) -> Result<(), NativeExit> {
        self.checkpoints += 1;
        if self.gc_every > 0 && self.checkpoints % self.gc_every == 0 {
            if let Ok(collection) = collect_cycles() {
                self.reclaimed += collection.reclaimed;
            }
        }
        if self.cancelled && !self.masked {
            Err(NativeExit::Cancelled)
        } else {
            Ok(())
        }
    }
    fn root(&mut self, plan: V) -> NativeOutcome {
        assert!(
            self.scopes.is_empty(),
            "cannot reenter the native Effect machine"
        );
        NativeScheduler::run(self, plan)
    }
    fn cleanup(
        &mut self,
        stack: &mut Vec<NativeFrame>,
        original: NativeOutcome,
        defects: Vec<String>,
        previous_mask: bool,
    ) -> NativeState {
        if let Some(next) = self
            .scopes
            .last_mut()
            .expect("closing native scope")
            .finalizers
            .pop()
        {
            stack.push(NativeFrame::Cleanup {
                original,
                defects,
                previous_mask,
            });
            NativeState::Evaluate(next)
        } else {
            let root = self.scopes.pop().expect("closing native scope").root;
            self.masked = previous_mask;
            NativeState::Return(if defects.is_empty() {
                if root {
                    // Root source evaluation has already produced its terminal
                    // outcome. Interrupt still waits for masked cleanup, but
                    // cannot replace an outcome committed before that request.
                    original
                } else {
                    original.and_then(|value| self.checkpoint().map(|_| value))
                }
            } else {
                Err(NativeExit::Defect(defects))
            })
        }
    }
}

#[derive(Default)]
struct NativeScope {
    root: bool,
    finalizers: Vec<V>,
    children: Vec<V>,
}
struct NativeMachine {
    context: NativeContext,
    stack: Vec<NativeFrame>,
    state: Option<NativeState>,
}
enum NativeTurn {
    Runnable,
    Operation(ConcurrentOperation, Vec<V>),
    FileSystem(NativeFileOperation, Vec<V>),
    CloseScope(NativeOutcome, bool),
    Complete(NativeOutcome),
}
impl NativeMachine {
    fn new(mut context: NativeContext, plan: V) -> Self {
        context.scopes.push(NativeScope {
            root: true,
            ..NativeScope::default()
        });
        let state = match context.checkpoint() {
            Ok(()) => NativeState::Evaluate(plan),
            Err(exit) => NativeState::Return(Err(exit)),
        };
        Self {
            context,
            stack: vec![NativeFrame::Scope],
            state: Some(state),
        }
    }
    fn resume(&mut self, outcome: NativeOutcome) {
        self.state = Some(NativeState::Return(outcome));
    }
    fn turn(&mut self) -> NativeTurn {
        let context = &mut self.context;
        let stack = &mut self.stack;
        let before = context.checkpoints;
        let mut state = self.state.take().expect("runnable native continuation");
        loop {
            context.peak_frames = context.peak_frames.max(stack.len());
            state = match state {
                NativeState::Evaluate(plan) => {
                    // Checked plans are immutable; no Ref borrow survives an
                    // instruction boundary or a call to user code.
                    let (instruction, c) = plan.plan();
                    match instruction {
                        Instruction::Native(code) => NativeState::Return(
                            context
                                .checkpoint()
                                .and_then(|_| catch_native(|| code(&c, context))),
                        ),
                        Instruction::Concurrent(operation) => {
                            if let Err(exit) = context.checkpoint() {
                                NativeState::Return(Err(exit))
                            } else {
                                return NativeTurn::Operation(operation, c);
                            }
                        }
                        Instruction::FileSystem(operation) => {
                            if let Err(exit) = context.checkpoint() {
                                NativeState::Return(Err(exit))
                            } else {
                                return NativeTurn::FileSystem(operation, c);
                            }
                        }
                        Instruction::Bind => {
                            stack.push(NativeFrame::Bind(c[1].clone()));
                            NativeState::Evaluate(c[0].clone())
                        }
                        Instruction::Defer => match context
                            .checkpoint()
                            .and_then(|_| catch_native(|| Ok(c[0].apply(V::Unit))))
                        {
                            Ok(next) => NativeState::Evaluate(next),
                            Err(exit) => NativeState::Return(Err(exit)),
                        },
                        Instruction::Scoped => {
                            context.scopes.push(NativeScope::default());
                            stack.push(NativeFrame::Scope);
                            NativeState::Evaluate(c[0].clone())
                        }
                        Instruction::AcquireRelease => {
                            stack.push(NativeFrame::Register(c[1].clone()));
                            NativeState::Evaluate(c[0].clone())
                        }
                        Instruction::Recover => {
                            stack.push(NativeFrame::Recover(c[0].clone()));
                            NativeState::Evaluate(c[1].clone())
                        }
                        Instruction::MapError => {
                            stack.push(NativeFrame::MapError(c[0].clone()));
                            NativeState::Evaluate(c[1].clone())
                        }
                        Instruction::Release => match catch_native(|| Ok(c[0].apply(c[1].clone())))
                        {
                            Ok(next) => NativeState::Evaluate(next),
                            Err(exit) => NativeState::Return(Err(exit)),
                        },
                    }
                }
                NativeState::Return(result) => {
                    let Some(frame) = stack.pop() else {
                        return NativeTurn::Complete(result);
                    };
                    match frame {
                        NativeFrame::Bind(next) => match result {
                            Ok(value) => match context
                                .checkpoint()
                                .and_then(|_| catch_native(|| Ok(next.apply(value))))
                            {
                                Ok(plan) => NativeState::Evaluate(plan),
                                Err(exit) => NativeState::Return(Err(exit)),
                            },
                            error => NativeState::Return(error),
                        },
                        NativeFrame::Recover(handler) => match result {
                            Err(NativeExit::Failure(error)) => match context
                                .checkpoint()
                                .and_then(|_| catch_native(|| Ok(handler.apply(error))))
                            {
                                Ok(plan) => NativeState::Evaluate(plan),
                                Err(exit) => NativeState::Return(Err(exit)),
                            },
                            result => NativeState::Return(result),
                        },
                        NativeFrame::MapError(handler) => match result {
                            Err(NativeExit::Failure(error)) => {
                                NativeState::Return(context.checkpoint().and_then(|_| {
                                    catch_native(|| Err(NativeExit::Failure(handler.apply(error))))
                                }))
                            }
                            result => NativeState::Return(result),
                        },
                        NativeFrame::Register(release) => match result {
                            Ok(resource) => {
                                context
                                    .scopes
                                    .last_mut()
                                    .expect("owning native scope")
                                    .finalizers
                                    .push(V::effect(
                                        Instruction::Release,
                                        vec![release, resource.clone()],
                                    ));
                                // A successful acquisition owns its finalizer
                                // BEFORE the next cancellation checkpoint.
                                NativeState::Return(context.checkpoint().map(|_| resource))
                            }
                            error => NativeState::Return(error),
                        },
                        NativeFrame::Scope => {
                            let previous_mask = std::mem::replace(&mut context.masked, true);
                            return NativeTurn::CloseScope(result, previous_mask);
                        }
                        NativeFrame::Cleanup {
                            original,
                            mut defects,
                            previous_mask,
                        } => {
                            match result {
                                Ok(_) => {}
                                Err(NativeExit::Defect(errors)) => defects.extend(errors),
                                Err(NativeExit::Failure(_)) => {
                                    defects.push("Effect finalizer produced a typed failure".into())
                                }
                                Err(NativeExit::Cancelled) => {
                                    defects.push("Effect finalizer was cancelled".into())
                                }
                            }
                            context.cleanup(stack, original, defects, previous_mask)
                        }
                    }
                }
            };

            // Only Effect checkpoints yield. A pure callback runs to completion.
            if context.checkpoints != before {
                self.state = Some(state);
                return NativeTurn::Runnable;
            }
        }
    }
}

fn native_operation(name: &str, args: Vec<V>) -> V {
    if let Some(operation) = ConcurrentOperation::named(name) {
        return V::effect(Instruction::Concurrent(operation), args);
    }
    match name {
        "std/effect::fromEither" => V::leaf(args, |c, _| {
            let V::Adt(value) = &c[0] else {
                panic!("Rust ABI: expected Either")
            };
            match value.get().0.as_str() {
                "std/prelude::Left" => Err(NativeExit::Failure(value.get().1.clone())),
                "std/prelude::Right" => Ok(value.get().1.clone()),
                _ => panic!("Rust ABI: Either owner"),
            }
        }),
        "std/effect::succeed" | "effect.succeed" => V::leaf(args, |c, _| Ok(c[0].clone())),
        "std/effect::fail" | "effect.fail" => {
            V::leaf(args, |c, _| Err(NativeExit::Failure(c[0].clone())))
        }
        "std/effect::defer" => V::effect(Instruction::Defer, args),
        "std/effect::scoped" => V::effect(Instruction::Scoped, args),
        "std/effect::acquireRelease" => V::effect(Instruction::AcquireRelease, args),
        "std/effect::recover" => V::effect(Instruction::Recover, args),
        "std/effect::mapError" | "effect.mapError" => V::effect(Instruction::MapError, args),
        "std/effect::flatMap" => V::bind(args[0].clone(), args[1].clone()),
        "std/ref::make" => V::leaf(args, |c, _| Ok(V::reference(c[0].clone()))),
        "std/ref::get" => V::leaf(args, |c, _| Ok(c[0].read_reference())),
        "std/ref::set" => V::leaf(args, |c, _| {
            c[1].write_reference(c[0].clone());
            Ok(V::Unit)
        }),
        "std/ref::update" => V::leaf(args, |c, _| {
            let next = c[0].apply(c[1].read_reference());
            c[1].write_reference(next);
            Ok(V::Unit)
        }),
        "std/ref::modify" => V::leaf(args, |c, _| {
            let pair = c[0].apply(c[1].read_reference());
            let result = pair.index(0);
            c[1].write_reference(pair.index(1));
            Ok(result)
        }),
        "std/prelude::println" | "console.println" | "std/console::println" => {
            V::leaf(args, |c, _| {
                let text = format!("{}\n", console_text(&c[0]));
                write_native_console(&mut std::io::stdout().lock(), &text)
            })
        }
        "std/prelude::print" | "console.print" | "std/console::print" => V::leaf(args, |c, _| {
            write_native_console(&mut std::io::stdout().lock(), &console_text(&c[0]))
        }),
        "std/console::error" => V::leaf(args, |c, _| {
            write_native_console(&mut std::io::stderr().lock(), &console_text(&c[0]))
        }),
        "std/console::errorLine" => V::leaf(args, |c, _| {
            write_native_console(
                &mut std::io::stderr().lock(),
                &format!("{}\n", console_text(&c[0])),
            )
        }),
        "std/console::flush" => V::leaf(args, |_, _| {
            write_native_console(&mut std::io::stdout().lock(), "")?;
            write_native_console(&mut std::io::stderr().lock(), "")
        }),
        _ => panic!("Rust ABI: unsupported native operation {name}"),
    }
}

// Preserve the established prelude's primitive console coercions separately
// from language Show and the pure target's inspection format. In particular,
// direct Bool output is `true`, while Show Bool is `True`.
fn console_text(value: &V) -> String {
    match value {
        V::Unit => "undefined".into(),
        V::Bool(value) => value.to_string(),
        V::Float(value) => native_float_console(*value),
        V::Text(_) | V::Int(_) | V::Char(_) => value.render(),
        _ => panic!("Rust ABI: render non-scalar Console values with Show"),
    }
}

fn write_native_console(writer: &mut impl std::io::Write, text: &str) -> NativeOutcome {
    writer
        .write_all(text.as_bytes())
        .and_then(|_| writer.flush())
        .map(|_| V::Unit)
        .map_err(|error| {
            NativeExit::Failure(V::Record(Shared::new(BTreeMap::from([
                ("kind".into(), V::text("console-error")),
                ("message".into(), V::text(&error.to_string())),
            ]))))
        })
}

fn run_native_entry(entry: impl FnOnce() -> V, trace: bool) -> i32 {
    run_native_program(entry, trace, true)
}
fn run_native_pure_entry(entry: impl FnOnce() -> V, trace: bool) -> i32 {
    run_native_program(entry, trace, false)
}
fn run_native_program(entry: impl FnOnce() -> V, trace: bool, effect: bool) -> i32 {
    // This executable owns its single-thread runtime. Defects are reported
    // after managed scope cleanup, without the host panic hook printing early.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    if trace {
        enable_cycle_collection();
    }
    let mut context = NativeContext::default();
    let result = catch_native(|| {
        let value = entry();
        if effect {
            context.root(value)
        } else {
            println!("{}", value.render());
            Ok(V::Unit)
        }
    });
    let code = match &result {
        Ok(_) => 0,
        Err(NativeExit::Failure(error)) => {
            eprintln!("{}", error.render());
            1
        }
        Err(NativeExit::Defect(errors)) => {
            for (i, error) in errors.iter().enumerate() {
                eprintln!(
                    "Seseragi {}: {error}",
                    if i == 0 {
                        "Defect"
                    } else {
                        "suppressed Defect"
                    }
                );
            }
            70
        }
        Err(NativeExit::Cancelled) => {
            eprintln!("Seseragi cancelled");
            130
        }
    };
    drop(result);
    drop(context);
    // Global roots outlive scope finalizers and returned values, including on
    // initialization failure. Only then can Ref cycles become unreachable.
    clear_globals();
    clear_dictionaries();
    let collection = collect_cycles().expect("native root released its Ref borrows");
    if std::env::var_os("SESERAGI_NATIVE_METRICS").is_some() {
        eprintln!(
            "native managed-live={} collected={} scanned={}",
            LIVE.with(Cell::get),
            collection.reclaimed,
            collection.scanned
        );
    }
    std::panic::set_hook(previous_hook);
    code
}

#[cfg(test)]
mod effect_tests {
    use super::*;
    fn succeed(value: V) -> V {
        native_operation("effect.succeed", vec![value])
    }
    #[test]
    fn cold_ref_effect_replays_and_failure_retains_arbitrary_values() {
        enable_cycle_collection();
        let cell = V::reference(V::Int(0));
        let increment = native_operation(
            "std/ref::update",
            vec![
                V::fun(vec![], |_, n| binary("+", n, V::Int(1))),
                cell.clone(),
            ],
        );
        assert_eq!(cell.read_reference().render(), "0");
        for _ in 0..2 {
            assert!(NativeContext {
                gc_every: 1,
                ..NativeContext::default()
            }
            .root(increment.clone())
            .is_ok());
        }
        assert_eq!(cell.read_reference().render(), "2");
        let error = V::adt("Error", cell.clone());
        let failed = native_operation("effect.fail", vec![error.clone()]);
        let Err(NativeExit::Failure(same)) = NativeContext {
            gc_every: 1,
            ..NativeContext::default()
        }
        .root(failed) else {
            panic!()
        };
        assert_eq!(same.node_id(), error.node_id());
        drop((same, error, increment, cell));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn deep_binds_scopes_and_defer_do_not_use_the_host_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                for shape in 0..3 {
                    let mut plan = succeed(V::Int(42));
                    for _ in 0..100_000 {
                        plan = match shape {
                            0 => V::bind(plan, V::fun(vec![], |_, value| succeed(value))),
                            1 => V::effect(Instruction::Scoped, vec![plan]),
                            _ => V::effect(
                                Instruction::Defer,
                                vec![V::fun(vec![plan], |c, _| c[0].clone())],
                            ),
                        };
                    }
                    assert_eq!(
                        NativeContext {
                            gc_every: 1,
                            ..NativeContext::default()
                        }
                        .root(plan)
                        .ok()
                        .unwrap()
                        .render(),
                        "42"
                    );
                    assert_eq!(LIVE.with(Cell::get), 0);
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn acquisition_registers_before_cancellation_and_masks_release() {
        enable_cycle_collection();
        let cell = V::reference(V::Int(0));
        let acquire = V::leaf(vec![], |_, ctx| {
            ctx.cancelled = true;
            Ok(V::Int(42))
        });
        let release = V::fun(vec![cell.clone()], |c, value| {
            native_operation("std/ref::set", vec![value, c[0].clone()])
        });
        let plan = native_operation("std/effect::acquireRelease", vec![acquire, release]);
        assert!(matches!(
            NativeContext {
                gc_every: 1,
                ..NativeContext::default()
            }
            .root(plan),
            Err(NativeExit::Cancelled)
        ));
        assert_eq!(cell.read_reference().render(), "42");
        drop(cell);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn finalizer_defects_do_not_skip_remaining_lifo_cleanup() {
        enable_cycle_collection();
        let log = V::reference(V::Int(0));
        let mut plan = V::leaf(vec![], |_, _| Err(NativeExit::Failure(V::Int(7))));
        for digit in (1..=3).rev() {
            let release = V::fun(vec![log.clone(), V::Int(digit)], |c, _| {
                V::leaf(c.to_vec(), |c, _| {
                    c[0].write_reference(binary(
                        "+",
                        binary("*", c[0].read_reference(), V::Int(10)),
                        c[1].clone(),
                    ));
                    if c[1].render() == "2" {
                        Ok(V::Unit)
                    } else {
                        Err(NativeExit::Defect(vec![format!(
                            "release {}",
                            c[1].render()
                        )]))
                    }
                })
            });
            let acquire = native_operation(
                "std/effect::acquireRelease",
                vec![succeed(V::Unit), release],
            );
            plan = V::bind(acquire, V::fun(vec![plan], |c, _| c[0].clone()));
        }
        let Err(NativeExit::Defect(errors)) = NativeContext {
            gc_every: 1,
            ..NativeContext::default()
        }
        .root(plan) else {
            panic!()
        };
        assert_eq!(errors, vec!["release 3", "release 1"]);
        assert_eq!(log.read_reference().render(), "321");
        drop(log);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn console_io_errors_are_typed_failures_and_keep_the_provider_shape() {
        struct Broken;
        impl std::io::Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let Err(NativeExit::Failure(error)) = write_native_console(&mut Broken, "hello") else {
            panic!()
        };
        assert_eq!(error.field("kind").render(), "console-error");
        assert!(!error.field("message").render().is_empty());
    }
}
