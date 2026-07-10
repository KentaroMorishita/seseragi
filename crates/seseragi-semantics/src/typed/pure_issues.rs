use crate::TypedType;
use seseragi_syntax::ByteSpan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PureCallIssue {
    Arity {
        callee: ByteSpan,
        expected: usize,
        actual: usize,
    },
    ArgumentType {
        argument: ByteSpan,
        index: usize,
        expected: TypedType,
        actual: TypedType,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConditionalIssue {
    ConditionNotBool {
        condition: ByteSpan,
        actual: TypedType,
    },
    BranchTypeMismatch {
        then_branch: ByteSpan,
        else_branch: ByteSpan,
        then_type: TypedType,
        else_type: TypedType,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnresolvedNameIssue {
    pub(crate) origin: ByteSpan,
}
