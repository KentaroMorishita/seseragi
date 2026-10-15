// Cold synchronous Effect machine. User callbacks return values/plans; they
// never recursively enter this interpreter. Frames and scopes own ordinary V
// handles, so the collector can discover every live continuation as a root.
type NativeOutcome = Result<V, NativeExit>;
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

fn native_panic(error: Box<dyn Any + Send>) -> NativeExit {
    let message = error
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| error.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_else(|| "native host panic".into());
    NativeExit::Defect(vec![message])
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
    scopes: Vec<Vec<V>>,
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
        self.scopes.push(Vec::new());
        let mut stack = vec![NativeFrame::Scope];
        let mut state = match self.checkpoint() {
            Ok(()) => NativeState::Evaluate(plan),
            Err(error) => NativeState::Return(Err(error)),
        };
        loop {
            self.peak_frames = self.peak_frames.max(stack.len());
            state = match state {
                NativeState::Evaluate(plan) => {
                    // Checked plans are immutable; no Ref borrow survives an
                    // instruction boundary or a call to user code.
                    let (instruction, c) = plan.plan();
                    match instruction {
                        Instruction::Native(code) => NativeState::Return(
                            self.checkpoint()
                                .and_then(|_| catch_native(|| code(&c, self))),
                        ),
                        Instruction::Bind => {
                            stack.push(NativeFrame::Bind(c[1].clone()));
                            NativeState::Evaluate(c[0].clone())
                        }
                        Instruction::Defer => match self
                            .checkpoint()
                            .and_then(|_| catch_native(|| Ok(c[0].apply(V::Unit))))
                        {
                            Ok(next) => NativeState::Evaluate(next),
                            Err(exit) => NativeState::Return(Err(exit)),
                        },
                        Instruction::Scoped => {
                            self.scopes.push(Vec::new());
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
                        return result;
                    };
                    match frame {
                        NativeFrame::Bind(next) => match result {
                            Ok(value) => match self
                                .checkpoint()
                                .and_then(|_| catch_native(|| Ok(next.apply(value))))
                            {
                                Ok(plan) => NativeState::Evaluate(plan),
                                Err(exit) => NativeState::Return(Err(exit)),
                            },
                            error => NativeState::Return(error),
                        },
                        NativeFrame::Recover(handler) => match result {
                            Err(NativeExit::Failure(error)) => match self
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
                                NativeState::Return(self.checkpoint().and_then(|_| {
                                    catch_native(|| Err(NativeExit::Failure(handler.apply(error))))
                                }))
                            }
                            result => NativeState::Return(result),
                        },
                        NativeFrame::Register(release) => match result {
                            Ok(resource) => {
                                self.scopes.last_mut().expect("owning native scope").push(
                                    V::effect(
                                        Instruction::Release,
                                        vec![release, resource.clone()],
                                    ),
                                );
                                // A successful acquisition owns its finalizer
                                // BEFORE the next cancellation checkpoint.
                                NativeState::Return(self.checkpoint().map(|_| resource))
                            }
                            error => NativeState::Return(error),
                        },
                        NativeFrame::Scope => {
                            let previous_mask = std::mem::replace(&mut self.masked, true);
                            self.cleanup(&mut stack, result, Vec::new(), previous_mask)
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
                            self.cleanup(&mut stack, original, defects, previous_mask)
                        }
                    }
                }
            };
        }
    }
    fn cleanup(
        &mut self,
        stack: &mut Vec<NativeFrame>,
        original: NativeOutcome,
        defects: Vec<String>,
        previous_mask: bool,
    ) -> NativeState {
        if let Some(next) = self.scopes.last_mut().expect("closing native scope").pop() {
            stack.push(NativeFrame::Cleanup {
                original,
                defects,
                previous_mask,
            });
            NativeState::Evaluate(next)
        } else {
            self.scopes.pop();
            self.masked = previous_mask;
            NativeState::Return(if defects.is_empty() {
                original
            } else {
                Err(NativeExit::Defect(defects))
            })
        }
    }
}

fn native_operation(name: &str, args: Vec<V>) -> V {
    match name {
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
    // This executable owns its single-thread runtime. Defects are reported
    // after managed scope cleanup, without the host panic hook printing early.
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    if trace {
        enable_cycle_collection();
    }
    let mut context = NativeContext::default();
    let result = catch_native(|| Ok(entry())).and_then(|plan| context.root(plan));
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
