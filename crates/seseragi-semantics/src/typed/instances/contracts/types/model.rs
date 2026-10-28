use crate::SymbolId;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ContractType {
    TypeFunction {
        parameter_kinds: Vec<u32>,
        body: Box<ContractType>,
    },
    TypeFunctionParameter {
        depth: u32,
        index: u32,
    },
    Binder(u32),
    Parameter(SymbolId),
    // Private template variables scoped to one alias proof. These must all
    // disappear before returning a contract, identity, or semantic type.
    AliasSlot {
        owner: TypeIdentity,
        index: u32,
    },
    Named(TypeIdentity),
    Apply {
        constructor: Box<ContractType>,
        arguments: Vec<ContractType>,
    },
    Function {
        parameter: Box<ContractType>,
        result: Box<ContractType>,
    },
    Record {
        closed: bool,
        fields: Vec<(String, bool, ContractType)>,
    },
    Tuple(Vec<ContractType>),
    RequirementMerge(Vec<ContractType>),
    Hole,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum TypeIdentity {
    Canonical(String),
    Symbol(SymbolId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ContractConstraint {
    pub(super) trait_identity: TypeIdentity,
    pub(super) arguments: Vec<ContractType>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ContractMethod {
    pub(super) type_ref: ContractType,
    pub(super) constraints: Vec<ContractConstraint>,
}

pub(super) fn apply_arguments(
    constructor: ContractType,
    arguments: Vec<ContractType>,
) -> Option<ContractType> {
    if arguments.is_empty() {
        return Some(constructor);
    }
    if matches!(constructor, ContractType::TypeFunction { .. }) {
        return seseragi_syntax::type_functions::apply_type_function(&constructor, &arguments);
    }
    let (constructor, existing) = match constructor {
        ContractType::Apply {
            constructor,
            arguments,
        } => (*constructor, arguments),
        constructor => (constructor, Vec::new()),
    };
    if existing.contains(&ContractType::Hole) {
        let mut supplied = arguments.into_iter();
        let filled = existing
            .into_iter()
            .map(|argument| {
                if argument == ContractType::Hole {
                    supplied.next()
                } else {
                    Some(argument)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        if supplied.next().is_some() {
            return None;
        }
        return Some(ContractType::Apply {
            constructor: Box::new(constructor),
            arguments: filled,
        });
    }
    let mut combined = existing;
    combined.extend(arguments);
    Some(ContractType::Apply {
        constructor: Box::new(constructor),
        arguments: combined,
    })
}

impl seseragi_syntax::type_functions::TypeFunctionTerm for ContractType {
    fn type_function(&self) -> Option<(&[u32], &Self)> {
        match self {
            Self::TypeFunction {
                parameter_kinds,
                body,
            } => Some((parameter_kinds, body)),
            _ => None,
        }
    }
    fn type_function_parameter(&self) -> Option<(u32, u32)> {
        match self {
            Self::TypeFunctionParameter { depth, index } => Some((*depth, *index)),
            _ => None,
        }
    }
    fn from_type_function(parameter_kinds: Vec<u32>, body: Self) -> Self {
        Self::TypeFunction {
            parameter_kinds,
            body: Box::new(body),
        }
    }
    fn from_type_function_parameter(depth: u32, index: u32) -> Self {
        Self::TypeFunctionParameter { depth, index }
    }
    fn is_type_hole(&self) -> bool {
        matches!(self, Self::Hole)
    }
    fn map_type_children(&self, map: &mut dyn FnMut(&Self) -> Option<Self>) -> Option<Self> {
        Some(match self {
            Self::Apply {
                constructor,
                arguments,
            } => Self::Apply {
                constructor: Box::new(map(constructor)?),
                arguments: arguments.iter().map(map).collect::<Option<_>>()?,
            },
            Self::Function { parameter, result } => Self::Function {
                parameter: Box::new(map(parameter)?),
                result: Box::new(map(result)?),
            },
            Self::Record { closed, fields } => Self::Record {
                closed: *closed,
                fields: fields
                    .iter()
                    .map(|(name, optional, field)| Some((name.clone(), *optional, map(field)?)))
                    .collect::<Option<_>>()?,
            },
            Self::Tuple(elements) => Self::Tuple(elements.iter().map(map).collect::<Option<_>>()?),
            Self::RequirementMerge(operands) => {
                Self::RequirementMerge(operands.iter().map(map).collect::<Option<_>>()?)
            }
            leaf => leaf.clone(),
        })
    }
}
