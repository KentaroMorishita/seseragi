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
    MissingInstance {
        callee: ByteSpan,
        constraint: crate::TypedConstraint,
    },
    TraitMethodAmbiguous {
        callee: ByteSpan,
    },
    TraitMethodNoMatch {
        callee: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArrayIssue {
    EmptyWithoutExpectedType {
        array: ByteSpan,
    },
    ElementTypeMismatch {
        element: ByteSpan,
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
pub(crate) enum MatchIssue {
    PatternMismatch {
        pattern: ByteSpan,
        message: String,
    },
    GuardNotBool {
        guard: ByteSpan,
        actual: TypedType,
    },
    BranchTypeMismatch {
        expected_branch: ByteSpan,
        actual_branch: ByteSpan,
        expected: TypedType,
        actual: TypedType,
    },
    NonExhaustive {
        expression: ByteSpan,
        missing: Vec<String>,
    },
    Unreachable {
        arm: ByteSpan,
    },
}
