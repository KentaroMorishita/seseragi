// Included only when checked source uses std/effect::die. A language Defect is
// an interpreter outcome, so WASM need not unwind a Rust panic to close scopes.
fn native_effect_die(message: V) -> V {
    V::leaf(vec![message], |captures, _| {
        let V::Text(message) = &captures[0] else {
            return Err(NativeExit::defects(vec![
                "Rust ABI: die expected String".into()
            ]));
        };
        Err(NativeExit::defects(vec![message.get().clone()]))
    })
}

#[cfg(test)]
mod die_tests {
    use super::*;

    #[test]
    fn die_is_cold_replayable_and_never_a_typed_failure() {
        let plan = native_effect_die(V::text("cold Defect"));
        for _ in 0..2 {
            let result = NativeContext::default().root(plan.clone());
            assert!(
                matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["cold Defect"])
            );
        }
        drop(plan);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn finalizer_die_keeps_lifo_and_attempts_remaining_cleanup() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let mut plan = native_operation("effect.fail", vec![V::text("typed failure")]);
        for name in ["inner", "outer"] {
            let release = V::fun(vec![log.clone(), V::text(name)], |captures, _| {
                captures[0].write_reference(binary(
                    "+",
                    captures[0].read_reference(),
                    captures[1].clone(),
                ));
                native_effect_die(captures[1].clone())
            });
            let acquire = native_operation(
                "std/effect::acquireRelease",
                vec![native_operation("effect.succeed", vec![V::Unit]), release],
            );
            plan = V::bind(
                acquire,
                V::fun(vec![plan], |captures, _| captures[0].clone()),
            );
        }
        let result = NativeContext::default().root(plan);
        assert!(
            matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["inner", "outer"])
        );
        assert_eq!(log.read_reference().render(), "innerouter");
        drop((result, log));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
