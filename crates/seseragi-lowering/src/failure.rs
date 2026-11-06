use crate::SourceSpan;

/// A compiler invariant that lowering or emission could not rely on. These are
/// not source type errors: the checker accepted the program, but the facts it
/// handed to lowering (selected trait evidence, partial-application types,
/// derived instance payloads, ...) were incomplete or inconsistent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweringInternalError {
    pub stage: LoweringStage,
    pub kind: LoweringInternalErrorKind,
    /// The invariant that failed, written for compiler contributors.
    pub detail: &'static str,
    /// The innermost Typed HIR, Core or TypeScript node that owns the failure.
    /// Helpers without a node leave this empty for their caller to fill.
    pub origin: Option<SourceSpan>,
}

/// The lowering step that was running when the invariant failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringStage {
    Core,
    Type,
    Emission,
    Expression,
    TraitDictionary,
    Instance,
}

/// What kind of checker-provided fact was missing or inconsistent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringInternalErrorKind {
    /// A referenced type has no validated import binding.
    MissingTypeInformation,
    /// The accepted Typed HIR shape cannot be converted to Core.
    UnexpectedTypedHirShape,
    /// TypeScript IR is inconsistent with the emission contract.
    UnexpectedTypeScriptShape,
    /// Selected trait evidence is absent or cannot become a dictionary.
    MissingTraitEvidence,
    /// A derived instance lost its checked nominal type or field evidence.
    MissingDerivedEvidence,
    /// A runtime operation or standard definition is not registered.
    MissingRuntimeDefinition,
    /// A checked type or Core shape differs from what the checker guarantees.
    UnexpectedCoreShape,
}

impl LoweringInternalError {
    pub(crate) fn new(
        stage: LoweringStage,
        kind: LoweringInternalErrorKind,
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

pub(crate) type Lowered<T> = Result<T, LoweringInternalError>;

pub(crate) fn core_error(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::Core,
        LoweringInternalErrorKind::UnexpectedTypedHirShape,
        detail,
    )
}

pub(crate) fn type_error(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::Type,
        LoweringInternalErrorKind::MissingTypeInformation,
        detail,
    )
}

pub(crate) fn emission_error(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::Emission,
        LoweringInternalErrorKind::UnexpectedTypeScriptShape,
        detail,
    )
}

pub(crate) fn emission_evidence_error(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::Emission,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        detail,
    )
}

pub(crate) fn expression_error(
    kind: LoweringInternalErrorKind,
    detail: &'static str,
) -> LoweringInternalError {
    LoweringInternalError::new(LoweringStage::Expression, kind, detail)
}

/// Missing selected evidence while lowering an expression.
pub(crate) fn missing_evidence(detail: &'static str) -> LoweringInternalError {
    expression_error(LoweringInternalErrorKind::MissingTraitEvidence, detail)
}

/// A checked expression shape that the checker should have excluded.
pub(crate) fn unexpected_shape(detail: &'static str) -> LoweringInternalError {
    expression_error(LoweringInternalErrorKind::UnexpectedCoreShape, detail)
}

/// A runtime helper or standard definition that is not registered.
pub(crate) fn missing_runtime(detail: &'static str) -> LoweringInternalError {
    expression_error(LoweringInternalErrorKind::MissingRuntimeDefinition, detail)
}

/// A derived instance whose checked nominal type or evidence is missing.
pub(crate) fn missing_derived_evidence(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::Instance,
        LoweringInternalErrorKind::MissingDerivedEvidence,
        detail,
    )
}

/// Selected evidence that cannot become a trait dictionary reference.
pub(crate) fn missing_dictionary(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::TraitDictionary,
        LoweringInternalErrorKind::MissingTraitEvidence,
        detail,
    )
}

/// A standard dictionary that is not registered with the runtime.
pub(crate) fn missing_dictionary_runtime(detail: &'static str) -> LoweringInternalError {
    LoweringInternalError::new(
        LoweringStage::TraitDictionary,
        LoweringInternalErrorKind::MissingRuntimeDefinition,
        detail,
    )
}

impl LoweringStage {
    pub fn description(self) -> &'static str {
        match self {
            Self::Core => "Core lowering of Typed HIR",
            Self::Type => "TypeScript lowering of a type",
            Self::Emission => "TypeScript emission",
            Self::Expression => "TypeScript lowering of an expression",
            Self::TraitDictionary => "TypeScript lowering of a trait dictionary",
            Self::Instance => "TypeScript lowering of a trait instance",
        }
    }
}

impl LoweringInternalErrorKind {
    pub fn description(self) -> &'static str {
        match self {
            Self::MissingTypeInformation => "missing checked type information",
            Self::UnexpectedTypedHirShape => "unexpected checked Typed HIR shape",
            Self::UnexpectedTypeScriptShape => "unexpected TypeScript IR shape",
            Self::MissingTraitEvidence => "missing selected trait evidence",
            Self::MissingDerivedEvidence => "missing derived instance evidence",
            Self::MissingRuntimeDefinition => "missing runtime definition",
            Self::UnexpectedCoreShape => "unexpected checked Core shape",
        }
    }
}

impl std::fmt::Display for LoweringInternalError {
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

impl std::error::Error for LoweringInternalError {}
