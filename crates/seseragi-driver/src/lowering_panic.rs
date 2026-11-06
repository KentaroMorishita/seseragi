use seseragi_lowering::{LoweringInternalError, TypeScriptLoweringError};
use seseragi_syntax::{
    ByteRange, Diagnostic, DiagnosticArtifact, DiagnosticSeverity, RelatedDiagnostic,
};
use std::panic::{catch_unwind, UnwindSafe};

/// The closure owns intermediate compiler state; only an independent copy of
/// the frontend diagnostics survives an unwind. Abort-mode panics cannot unwind.
pub(crate) fn catch_lowering_panic<T>(
    diagnostics: DiagnosticArtifact,
    lower: impl FnOnce() -> T + UnwindSafe,
) -> Result<T, DiagnosticArtifact> {
    catch_unwind(lower)
        .map_err(|_| push_internal_error(diagnostics, ByteRange { start: 0, end: 0 }, None))
}

/// Reports a lowering failure as an internal compiler error. A failed lowering
/// invariant points at its Typed HIR, Core or TypeScript source owner. A label names the step,
/// the missing fact and the invariant. Ordinary type errors never reach here.
pub(crate) fn lowering_error_diagnostics(
    diagnostics: DiagnosticArtifact,
    error: &TypeScriptLoweringError,
) -> DiagnosticArtifact {
    match error {
        TypeScriptLoweringError::Internal(error) => internal_error_diagnostics(diagnostics, error),
        other => push_internal_error(
            diagnostics,
            ByteRange { start: 0, end: 0 },
            Some((None, format!("TypeScript output plan failed: {other:?}"))),
        ),
    }
}

pub(crate) fn internal_error_diagnostics(
    diagnostics: DiagnosticArtifact,
    error: &LoweringInternalError,
) -> DiagnosticArtifact {
    let (primary, source) = match &error.origin {
        Some(origin) => (
            ByteRange {
                start: origin.start,
                end: origin.end,
            },
            (origin.source != diagnostics.source).then(|| origin.source.clone()),
        ),
        None => (ByteRange { start: 0, end: 0 }, None),
    };
    push_internal_error(diagnostics, primary, Some((source, error.to_string())))
}

fn push_internal_error(
    mut diagnostics: DiagnosticArtifact,
    primary: ByteRange,
    label: Option<(Option<String>, String)>,
) -> DiagnosticArtifact {
    let related = label
        .map(|(source, message)| RelatedDiagnostic {
            source,
            message,
            primary,
        })
        .into_iter()
        .collect();
    diagnostics.diagnostics.push(Diagnostic {
        id: format!("d{}", diagnostics.diagnostics.len() + 1),
        code: "SES-I0001".to_owned(),
        severity: DiagnosticSeverity::Error,
        message_key: "internal.compiler-error".to_owned(),
        primary,
        related,
        fixes: Vec::new(),
        type_difference: None,
    });
    diagnostics
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

    thread_local! {
        static CORE_EDIT: Cell<Option<fn(&mut seseragi_lowering::CoreModule)>> =
            const { Cell::new(None) };
    }

    /// Runs `action` while `edit` removes checker facts from Core IR right
    /// before TypeScript lowering, as a compiler defect would.
    pub(crate) fn with_core_edit<T>(
        edit: fn(&mut seseragi_lowering::CoreModule),
        action: impl FnOnce() -> T,
    ) -> T {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                CORE_EDIT.with(|slot| slot.set(None));
            }
        }
        let _reset = Reset;
        CORE_EDIT.with(|slot| slot.set(Some(edit)));
        action()
    }

    pub(crate) fn edit_core(
        mut core: seseragi_lowering::CoreModule,
    ) -> seseragi_lowering::CoreModule {
        if let Some(edit) = CORE_EDIT.with(Cell::get) {
            edit(&mut core);
        }
        core
    }
    thread_local! {
        static TYPED_EDIT: Cell<Option<fn(&mut seseragi_semantics::TypedModule)>> = const { Cell::new(None) };
        static TYPESCRIPT_EDIT: Cell<Option<fn(&mut seseragi_lowering::TypeScriptModule)>> = const { Cell::new(None) };
    }

    pub(crate) fn with_typed_edit<T>(
        edit: fn(&mut seseragi_semantics::TypedModule),
        action: impl FnOnce() -> T,
    ) -> T {
        struct Reset(Option<fn(&mut seseragi_semantics::TypedModule)>);
        impl Drop for Reset {
            fn drop(&mut self) {
                TYPED_EDIT.with(|slot| slot.set(self.0));
            }
        }
        let _reset = Reset(TYPED_EDIT.with(|slot| slot.replace(Some(edit))));
        action()
    }

    pub(crate) fn edit_typed(module: &mut seseragi_semantics::TypedModule) {
        if let Some(edit) = TYPED_EDIT.with(Cell::get) {
            edit(module);
        }
    }

    pub(crate) fn with_typescript_edit<T>(
        edit: fn(&mut seseragi_lowering::TypeScriptModule),
        action: impl FnOnce() -> T,
    ) -> T {
        struct Reset(Option<fn(&mut seseragi_lowering::TypeScriptModule)>);
        impl Drop for Reset {
            fn drop(&mut self) {
                TYPESCRIPT_EDIT.with(|slot| slot.set(self.0));
            }
        }
        let _reset = Reset(TYPESCRIPT_EDIT.with(|slot| slot.replace(Some(edit))));
        action()
    }

    pub(crate) fn edit_typescript(
        mut module: seseragi_lowering::TypeScriptModule,
    ) -> seseragi_lowering::TypeScriptModule {
        if let Some(edit) = TYPESCRIPT_EDIT.with(Cell::get) {
            edit(&mut module);
        }
        module
    }
}

#[cfg(test)]
mod tests;
