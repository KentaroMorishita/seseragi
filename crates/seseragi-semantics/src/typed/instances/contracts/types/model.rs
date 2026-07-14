use crate::SymbolId;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum ContractType {
    Binder(u32),
    Parameter(SymbolId),
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
