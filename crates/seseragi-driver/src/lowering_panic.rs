use seseragi_syntax::{ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticSeverity};
use std::panic::{catch_unwind, UnwindSafe};

/// The closure owns intermediate compiler state; only an independent copy of
/// the frontend diagnostics survives an unwind. Abort-mode panics cannot unwind.
pub(crate) fn catch_lowering_panic<T>(
    mut diagnostics: DiagnosticArtifact,
    lower: impl FnOnce() -> T + UnwindSafe,
) -> Result<T, DiagnosticArtifact> {
    catch_unwind(lower).map_err(|_| {
        diagnostics.diagnostics.push(Diagnostic {
            id: format!("d{}", diagnostics.diagnostics.len() + 1),
            code: "SES-I0001".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: "internal.compiler-error".to_owned(),
            primary: ByteRange { start: 0, end: 0 },
            related: Vec::new(),
            fixes: Vec::new(),
            type_difference: None,
        });
        diagnostics
    })
}

#[cfg(test)]
pub(crate) mod fault {
    use std::cell::Cell;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum Stage {
        Core,
        TypeScript,
        Emission,
    }

    thread_local! {
        static PANIC_AT: Cell<Option<(Stage, usize)>> = const { Cell::new(None) };
    }

    pub(crate) fn with_panic_at<T>(stage: Stage, skip: usize, action: impl FnOnce() -> T) -> T {
        struct Reset(Option<(Stage, usize)>);
        impl Drop for Reset {
            fn drop(&mut self) {
                PANIC_AT.with(|slot| slot.set(self.0));
            }
        }
        let _reset = Reset(PANIC_AT.with(|slot| slot.replace(Some((stage, skip)))));
        let result = action();
        PANIC_AT.with(|slot| assert_eq!(slot.get(), None, "panic stage was not reached"));
        result
    }

    fn after(stage: Stage) {
        PANIC_AT.with(|slot| match slot.get() {
            Some((target, 0)) if target == stage => {
                slot.set(None);
                panic!("injected {stage:?} panic");
            }
            Some((target, skip)) if target == stage => slot.set(Some((target, skip - 1))),
            _ => {}
        });
    }

    pub(crate) fn after_core() {
        after(Stage::Core);
    }

    pub(crate) fn after_typescript() {
        after(Stage::TypeScript);
    }

    pub(crate) fn after_emission() {
        after(Stage::Emission);
    }
}

#[cfg(test)]
mod tests;
