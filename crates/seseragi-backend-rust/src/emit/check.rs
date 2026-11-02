use super::*;

pub(crate) fn diagnose(
    module: &CoreModule,
    owners: &BTreeMap<String, String>,
    scalar_globals: &seseragi_lowering::CoreScalarGlobals,
    type_names: &BTreeMap<(String, String), String>,
    trait_names: &BTreeMap<(String, String), String>,
    options: &crate::RustTargetOptions,
    target: crate::RustDiagnosticTarget,
) -> Vec<RustBackendError> {
    let mut backend = Backend::new(
        module,
        owners,
        scalar_globals,
        type_names,
        trait_names,
        options,
        target == crate::RustDiagnosticTarget::ProgramWasm,
        false,
    );
    backend.diagnostics = Some(Vec::new());
    for binding in &module.bindings {
        backend.owner = &owners[&binding.symbol];
        let mut scope = Scope::new();
        scope.type_parameters(
            binding
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &binding.symbol,
        );
        let result = backend.expr(&binding.value, &scope);
        backend.record_diagnostic(result, &binding.origin);
    }
    for instance in &module.instances {
        let result = backend.instance(instance);
        backend.record_diagnostic(result, &instance.origin);
    }
    for function in &module.functions {
        let result = backend.function_definition(function);
        backend.record_diagnostic(result, &function.origin);
    }
    let mut diagnostics = backend.diagnostics.unwrap();
    diagnostics.sort_by(|a, b| {
        a.origin
            .as_ref()
            .map(|span| (&span.source, span.start, span.end))
            .cmp(
                &b.origin
                    .as_ref()
                    .map(|span| (&span.source, span.start, span.end)),
            )
            .then(a.message.cmp(&b.message))
    });
    diagnostics.dedup();
    diagnostics
}

impl Backend<'_> {
    fn record_diagnostic<T>(&mut self, result: Result<T>, origin: &seseragi_lowering::SourceSpan) {
        if let Err(mut error) = result {
            if error.origin.is_none() {
                error.origin = Some(origin.clone());
            }
            if error.module.is_none() {
                error.module = Some(self.owner.to_owned());
            }
            self.diagnostics.as_mut().unwrap().push(error);
        }
    }
}
