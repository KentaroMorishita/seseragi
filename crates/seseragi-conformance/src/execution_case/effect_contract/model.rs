use seseragi_lowering::GeneratedModule;
use seseragi_semantics::TypedModuleInterface;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectEntryContract {
    pub(crate) failure_renderer: FailureRenderer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FailureRenderer {
    Never,
    Show { dictionary: DictionaryImport },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DictionaryImport {
    pub(crate) module: String,
    pub(crate) export: String,
}

/// Project-owned metadata needed to import a failure dictionary from the
/// module that actually generated it.
///
/// Module identity and wrapper-root import specifiers remain separate.
/// Consumers must not recover either value by parsing an instance identity or
/// type spelling.
pub(crate) struct ProjectFailureRendererCatalog<'a> {
    generated_modules: BTreeMap<String, &'a GeneratedModule>,
    typed_interfaces: BTreeMap<String, &'a TypedModuleInterface>,
    wrapper_module_specifiers: BTreeMap<String, String>,
}

impl<'a> ProjectFailureRendererCatalog<'a> {
    pub(crate) fn new(
        generated_modules: impl IntoIterator<Item = (String, &'a GeneratedModule)>,
        typed_interfaces: impl IntoIterator<Item = (String, &'a TypedModuleInterface)>,
        wrapper_module_specifiers: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        Self {
            generated_modules: generated_modules.into_iter().collect(),
            typed_interfaces: typed_interfaces.into_iter().collect(),
            wrapper_module_specifiers: wrapper_module_specifiers.into_iter().collect(),
        }
    }

    pub(super) fn generated_module(&self, module: &str) -> Option<&GeneratedModule> {
        self.generated_modules.get(module).copied()
    }

    pub(super) fn typed_interface(&self, module: &str) -> Option<&TypedModuleInterface> {
        self.typed_interfaces.get(module).copied()
    }

    pub(super) fn wrapper_module_specifier(&self, module: &str) -> Option<&str> {
        self.wrapper_module_specifiers
            .get(module)
            .map(String::as_str)
    }
}
