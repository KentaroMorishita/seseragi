//! Scoped substitution shared by checked type representations.
//!
//! This module does not infer a kind or resolve a source spelling. Producers
//! must supply a checked body and consumers retain their own admission rules.

pub trait TypeFunctionTerm: Clone {
    fn type_function(&self) -> Option<(&[u32], &Self)>;
    fn type_function_parameter(&self) -> Option<(u32, u32)>;
    fn from_type_function(parameter_kinds: Vec<u32>, body: Self) -> Self;
    fn from_type_function_parameter(depth: u32, index: u32) -> Self;
    fn is_type_hole(&self) -> bool;
    fn map_type_children(&self, map: &mut dyn FnMut(&Self) -> Option<Self>) -> Option<Self>;
}

pub fn apply_type_function<T: TypeFunctionTerm>(value: &T, arguments: &[T]) -> Option<T> {
    let (kinds, body) = value.type_function()?;
    if arguments.len() > kinds.len() || arguments.iter().any(T::is_type_hole) {
        return None;
    }
    if arguments.is_empty() {
        return Some(value.clone());
    }
    let remaining = &kinds[arguments.len()..];
    let body = substitute(body, arguments, kinds.len(), 0, !remaining.is_empty())?;
    Some(if remaining.is_empty() {
        body
    } else {
        T::from_type_function(remaining.to_vec(), body)
    })
}

fn substitute<T: TypeFunctionTerm>(
    value: &T,
    arguments: &[T],
    arity: usize,
    depth: u32,
    keep_binder: bool,
) -> Option<T> {
    if let Some((actual, index)) = value.type_function_parameter() {
        return if actual == depth {
            if index as usize >= arity {
                None
            } else if let Some(argument) = arguments.get(index as usize) {
                shift_type_function_parameters(
                    argument,
                    depth.checked_add(u32::from(keep_binder))?,
                    0,
                )
            } else if keep_binder {
                Some(T::from_type_function_parameter(
                    depth,
                    index.checked_sub(arguments.len() as u32)?,
                ))
            } else {
                None
            }
        } else {
            Some(T::from_type_function_parameter(
                if actual > depth && !keep_binder {
                    actual.checked_sub(1)?
                } else {
                    actual
                },
                index,
            ))
        };
    }
    if let Some((kinds, body)) = value.type_function() {
        return Some(T::from_type_function(
            kinds.to_vec(),
            substitute(body, arguments, arity, depth.checked_add(1)?, keep_binder)?,
        ));
    }
    value.map_type_children(&mut |child| substitute(child, arguments, arity, depth, keep_binder))
}

pub fn shift_type_function_parameters<T: TypeFunctionTerm>(
    value: &T,
    amount: u32,
    cutoff: u32,
) -> Option<T> {
    if let Some((depth, index)) = value.type_function_parameter() {
        return Some(T::from_type_function_parameter(
            if depth >= cutoff {
                depth.checked_add(amount)?
            } else {
                depth
            },
            index,
        ));
    }
    if let Some((kinds, body)) = value.type_function() {
        return Some(T::from_type_function(
            kinds.to_vec(),
            shift_type_function_parameters(body, amount, cutoff.checked_add(1)?)?,
        ));
    }
    value.map_type_children(&mut |child| shift_type_function_parameters(child, amount, cutoff))
}
