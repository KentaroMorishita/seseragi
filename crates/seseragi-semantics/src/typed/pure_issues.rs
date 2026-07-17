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
pub(crate) enum MonadDoIssue {
    ResultTypeNotMonadic {
        expression: ByteSpan,
        actual: TypedType,
    },
    ConstructorMismatch {
        expression: ByteSpan,
        expected: TypedType,
        actual: TypedType,
    },
    RefutableBindPattern {
        pattern: ByteSpan,
    },
    UnsupportedBindPattern {
        pattern: ByteSpan,
    },
    MissingFinalExpression {
        do_block: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArrayIssue {
    EmptyWithoutExpectedType {
        collection: &'static str,
        literal: ByteSpan,
    },
    ElementTypeMismatch {
        collection: &'static str,
        element: ByteSpan,
        index: usize,
        expected: TypedType,
        actual: TypedType,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RecordIssue {
    DuplicateField {
        field: ByteSpan,
        name: String,
    },
    MissingField {
        field: ByteSpan,
        name: String,
    },
    AccessOnNonRecord {
        receiver: ByteSpan,
        actual: TypedType,
    },
    SpreadOnNonRecord {
        spread: ByteSpan,
        actual: TypedType,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RangeIssue {
    pub(crate) endpoint: ByteSpan,
    pub(crate) position: &'static str,
    pub(crate) actual: TypedType,
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
