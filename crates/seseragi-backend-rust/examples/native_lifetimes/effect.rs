//! Synchronous, cold Effect / Ref / resource research slice. No fibers or I/O.
use super::{
    heap::{Handle, Heap},
    scope::Exit,
};

#[derive(Default)]
pub struct Context {
    frames: Vec<Vec<Handle>>,
    pub output: Vec<String>,
    pub cancelled: bool,
    masked: bool,
    pub checkpoints: usize,
    pub reclaimed: usize,
    pub candidate_collection: bool,
}
impl Context {
    pub fn checkpoint(&mut self, heap: &Heap) -> Result<(), Exit> {
        self.checkpoints += 1;
        // Deliberately stress roots at EVERY checkpoint, including release.
        self.reclaimed += (if self.candidate_collection {
            heap.collect_candidates()
        } else {
            heap.collect()
        })
        .map_err(|e| Exit::Defect(vec![e.into()]))?
        .reclaimed;
        if self.cancelled && !self.masked {
            Err(Exit::Cancelled)
        } else {
            Ok(())
        }
    }
    fn finish(&mut self, heap: &Heap, original: Result<Handle, Exit>) -> Result<Handle, Exit> {
        let previous = std::mem::replace(&mut self.masked, true);
        let mut defects = Vec::new();
        loop {
            let next = self.frames.last_mut().expect("active scope").pop();
            let Some(finalizer) = next else { break };
            match finalizer.execute(heap, self) {
                Ok(_) => {}
                Err(Exit::Defect(errors)) => defects.extend(errors),
                Err(_) => defects.push("finalizer produced failure/cancellation".into()),
            }
        }
        self.frames.pop();
        self.masked = previous;
        if defects.is_empty() {
            original
        } else {
            Err(Exit::Defect(defects))
        }
    }
    pub fn root(&mut self, heap: &Heap, effect: &Handle) -> Result<Handle, Exit> {
        self.frames.push(Vec::new());
        let result = effect.execute(heap, self);
        self.finish(heap, result)
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
        ("std/effect::scoped", 1) => heap.effect(args, |h, c, ctx| {
            ctx.frames.push(Vec::new());
            let result = c[0].execute(h, ctx);
            ctx.finish(h, result)
        }),
        ("std/effect::acquireRelease", 2) => heap.effect(args, |h, c, ctx| {
            let resource = c[0].execute(h, ctx)?;
            // Invoke the user's release callback during finalization, not
            // during acquisition. Its pure prefix may itself produce a defect.
            let release = h.effect(vec![c[1].clone(), resource.clone()], |h, c, ctx| {
                c[0].call(h, c[1].clone())?.execute(h, ctx)
            });
            ctx.frames.last_mut().expect("active scope").push(release);
            // Registration happened before cancellation can be observed.
            ctx.checkpoint(h)?;
            Ok(resource)
        }),
        ("std/effect::recover", 2) => heap.effect(args, |h, c, ctx| match c[1].execute(h, ctx) {
            Err(Exit::Failure(error)) => c[0].call(h, h.text(&error))?.execute(h, ctx),
            result => result,
        }),
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
}
