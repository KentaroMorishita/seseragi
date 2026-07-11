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
