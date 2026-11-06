use crate::SourceSpan;

/// A compiler invariant that TypeScript lowering could not rely on. These are
/// not source type errors: the checker accepted the program, but the facts it
/// handed to lowering (selected trait evidence, partial-application types,
/// derived instance payloads, ...) were incomplete or inconsistent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeScriptInternalError {
    pub stage: TypeScriptLoweringStage,
    pub kind: TypeScriptInternalErrorKind,
    /// The invariant that failed, written for compiler contributors.
    pub detail: &'static str,
    /// The innermost Core node that owns the failure. Helpers that do not see
    /// a node leave it empty; the enclosing expression or instance fills it.
    pub origin: Option<SourceSpan>,
}

/// The lowering step that was running when the invariant failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeScriptLoweringStage {
    Expression,
    TraitDictionary,
    Instance,
}

/// What kind of checker-provided fact was missing or inconsistent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeScriptInternalErrorKind {
    /// Selected trait evidence is absent or cannot become a dictionary.
    MissingTraitEvidence,
    /// A derived instance lost its checked nominal type or field evidence.
    MissingDerivedEvidence,
    /// A runtime operation or standard definition is not registered.
    MissingRuntimeDefinition,
    /// A checked type or Core shape differs from what the checker guarantees.
    UnexpectedCoreShape,
}

impl TypeScriptInternalError {
    pub(crate) fn new(
        stage: TypeScriptLoweringStage,
        kind: TypeScriptInternalErrorKind,
        detail: &'static str,
    ) -> Self {
        Self {
            stage,
            kind,
            detail,
            origin: None,
        }
    }

    /// Attaches `origin` unless a more precise inner node already did.
    pub(crate) fn at(mut self, origin: &SourceSpan) -> Self {
        if self.origin.is_none() {
            self.origin = Some(origin.clone());
        }
        self
    }
}

pub(crate) type Lowered<T> = Result<T, TypeScriptInternalError>;

pub(crate) fn expression_error(
    kind: TypeScriptInternalErrorKind,
    detail: &'static str,
) -> TypeScriptInternalError {
    TypeScriptInternalError::new(TypeScriptLoweringStage::Expression, kind, detail)
}

/// Missing selected evidence while lowering an expression.
pub(crate) fn missing_evidence(detail: &'static str) -> TypeScriptInternalError {
    expression_error(TypeScriptInternalErrorKind::MissingTraitEvidence, detail)
}

/// A checked expression shape that the checker should have excluded.
pub(crate) fn unexpected_shape(detail: &'static str) -> TypeScriptInternalError {
    expression_error(TypeScriptInternalErrorKind::UnexpectedCoreShape, detail)
}

/// A runtime helper or standard definition that is not registered.
pub(crate) fn missing_runtime(detail: &'static str) -> TypeScriptInternalError {
    expression_error(
        TypeScriptInternalErrorKind::MissingRuntimeDefinition,
        detail,
    )
}

/// A derived instance whose checked nominal type or evidence is missing.
pub(crate) fn missing_derived_evidence(detail: &'static str) -> TypeScriptInternalError {
    TypeScriptInternalError::new(
        TypeScriptLoweringStage::Instance,
        TypeScriptInternalErrorKind::MissingDerivedEvidence,
        detail,
    )
}

/// Selected evidence that cannot become a trait dictionary reference.
pub(crate) fn missing_dictionary(detail: &'static str) -> TypeScriptInternalError {
    TypeScriptInternalError::new(
        TypeScriptLoweringStage::TraitDictionary,
        TypeScriptInternalErrorKind::MissingTraitEvidence,
        detail,
    )
}

/// A standard dictionary that is not registered with the runtime.
pub(crate) fn missing_dictionary_runtime(detail: &'static str) -> TypeScriptInternalError {
    TypeScriptInternalError::new(
        TypeScriptLoweringStage::TraitDictionary,
        TypeScriptInternalErrorKind::MissingRuntimeDefinition,
        detail,
    )
}

impl TypeScriptLoweringStage {
    pub fn description(self) -> &'static str {
        match self {
            Self::Expression => "TypeScript lowering of an expression",
            Self::TraitDictionary => "TypeScript lowering of a trait dictionary",
            Self::Instance => "TypeScript lowering of a trait instance",
        }
    }
}

impl TypeScriptInternalErrorKind {
    pub fn description(self) -> &'static str {
        match self {
            Self::MissingTraitEvidence => "missing selected trait evidence",
            Self::MissingDerivedEvidence => "missing derived instance evidence",
            Self::MissingRuntimeDefinition => "missing runtime definition",
            Self::UnexpectedCoreShape => "unexpected checked Core shape",
        }
    }
}

impl std::fmt::Display for TypeScriptInternalError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} during {}: {}",
            self.kind.description(),
            self.stage.description(),
            self.detail
        )
    }
}

impl std::error::Error for TypeScriptInternalError {}
