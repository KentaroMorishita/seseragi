use crate::{ExternalTraitBinding, ExternalTypeBinding};
use seseragi_syntax::{ByteSpan, InterfaceMethod, SurfaceMethod};

pub(super) enum TraitContract<'a> {
    Local {
        parameters: &'a [String],
        methods: &'a [SurfaceMethod],
        span: ByteSpan,
    },
    Imported {
        name: &'a str,
        canonical: &'a str,
        parameters: &'a [String],
        methods: &'a [InterfaceMethod],
        bindings: &'a [ExternalTypeBinding],
        trait_bindings: &'a [ExternalTraitBinding],
        import_span: ByteSpan,
    },
}

pub(super) enum TraitMethodContract<'a> {
    Local(&'a SurfaceMethod),
    Imported(&'a InterfaceMethod),
}

impl TraitMethodContract<'_> {
    pub(super) fn name(&self) -> &str {
        match self {
            Self::Local(method) => &method.name,
            Self::Imported(method) => &method.name,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InstanceContractIssue {
    ArityMismatch {
        trait_name: String,
        expected: usize,
        actual: usize,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    MissingMethod {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    UnexpectedMethod {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    DuplicateMethod {
        method: String,
        primary: ByteSpan,
        declaration: ByteSpan,
    },
    SignatureMismatch {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
    MissingBody {
        method: String,
        primary: ByteSpan,
        contract: ByteSpan,
    },
}
