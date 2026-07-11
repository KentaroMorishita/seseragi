#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectEntryContract {
    pub(crate) failure_renderer: FailureRenderer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FailureRenderer {
    Never,
    Show { dictionary_export: String },
}
