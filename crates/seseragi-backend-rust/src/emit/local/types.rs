use super::*;

/// A type argument with a written `_` in its last slots, after at least one
/// written argument, is a partly applied constructor: `Effect<R, E, _>` in
/// `MaybeT<Effect<R, E, _>, A>` is `Effect<R, E>`. Only argument positions
/// are read this way, and a type with only holes, such as `Array<_>`, keeps
/// them.
pub(in crate::emit) fn partly_applied(argument: &CoreType) -> CoreType {
    let strip = |arguments: &[CoreType]| {
        let holes = arguments
            .iter()
            .rev()
            .take_while(|argument| matches!(argument, CoreType::Hole))
            .count();
        let written = if holes == arguments.len() {
            arguments.len()
        } else {
            arguments.len() - holes
        };
        arguments[..written].to_vec()
    };
    match argument {
        CoreType::Named { name, arguments } => CoreType::Named {
            name: name.clone(),
            arguments: strip(arguments),
        },
        CoreType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => CoreType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: strip(arguments),
        },
        other => other.clone(),
    }
}

fn nominal_parts(ty: &CoreType) -> Option<(CoreType, Vec<CoreType>)> {
    match ty {
        CoreType::Named { name, arguments } => Some((
            CoreType::Named {
                name: name.clone(),
                arguments: vec![],
            },
            arguments.iter().map(partly_applied).collect(),
        )),
        CoreType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => Some((
            CoreType::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: vec![],
            },
            arguments.iter().map(partly_applied).collect(),
        )),
        _ => None,
    }
}

pub(in crate::emit) fn argument_type(expr: &CoreExpr) -> Option<CoreType> {
    let primitive = match expr {
        CoreExpr::Unit { .. } => "Unit",
        CoreExpr::Integer { .. } => "Int",
        CoreExpr::Float64 { .. } => "Float",
        CoreExpr::Boolean { .. } => "Bool",
        CoreExpr::String { .. } | CoreExpr::Template { .. } => "String",
        CoreExpr::Char { .. } => "Char",
        CoreExpr::Variable { type_ref, .. }
        | CoreExpr::Call { type_ref, .. }
        | CoreExpr::Lambda { type_ref, .. }
        | CoreExpr::Tuple { type_ref, .. }
        | CoreExpr::Array { type_ref, .. }
        | CoreExpr::List { type_ref, .. }
        | CoreExpr::Record { type_ref, .. }
        | CoreExpr::FieldAccess { type_ref, .. }
        | CoreExpr::OptionalFieldAccess { type_ref, .. }
        | CoreExpr::ArrayComprehension { type_ref, .. }
        | CoreExpr::ListComprehension { type_ref, .. }
        | CoreExpr::If { type_ref, .. }
        | CoreExpr::Decision { type_ref, .. }
        | CoreExpr::Unary { type_ref, .. }
        | CoreExpr::Binary { type_ref, .. }
        | CoreExpr::MonadDo { type_ref, .. } => return Some(type_ref.clone()),
        CoreExpr::Sequence { result, .. } => return argument_type(result),
        CoreExpr::EffectOperation {
            requirements,
            failure,
            success,
            ..
        }
        | CoreExpr::EffectInvoke {
            requirements,
            failure,
            success,
            ..
        } => {
            return Some(CoreType::ExternalNamed {
                name: "Effect".into(),
                canonical: "std/prelude::Effect".into(),
                arguments: vec![requirements.clone(), failure.clone(), success.clone()],
            })
        }
    };
    Some(CoreType::ExternalNamed {
        name: primitive.into(),
        canonical: format!("std/prelude::{primitive}"),
        arguments: vec![],
    })
}

impl Backend<'_> {
    /// The type checker leaves a type that nothing constrains, such as the
    /// element of `[]` or of `maps.empty ()`, as a hole or as the callee's own
    /// type parameter name, which names no type here. No value of that type
    /// exists, so an operation whose result does not depend on it may use any
    /// type; it uses `Unit`. Only type arguments are grounded, such as the
    /// element of `Array<_>`: a hole in place of a whole parameter or result
    /// stays and is rejected. Trailing holes after a written argument also
    /// stay: they mark a partly applied constructor, as `partly_applied` reads
    /// them.
    pub(in crate::emit) fn ground_unconstrained(&self, ty: &CoreType, scope: &Scope) -> CoreType {
        self.ground_unconstrained_in(ty, scope, false)
    }

    fn ground_unconstrained_in(&self, ty: &CoreType, scope: &Scope, argument: bool) -> CoreType {
        let unit = || CoreType::Named {
            name: "std/prelude::Unit".into(),
            arguments: vec![],
        };
        let ground = |arguments: &[CoreType]| {
            let holes = arguments
                .iter()
                .rev()
                .take_while(|argument| matches!(argument, CoreType::Hole))
                .count();
            let written = if holes == arguments.len() {
                arguments.len()
            } else {
                arguments.len() - holes
            };
            arguments
                .iter()
                .enumerate()
                .map(|(index, argument)| {
                    if index < written {
                        self.ground_unconstrained_in(argument, scope, true)
                    } else {
                        argument.clone()
                    }
                })
                .collect()
        };
        let recur = |ty: &CoreType| self.ground_unconstrained_in(ty, scope, argument);
        match ty {
            CoreType::Hole if argument => unit(),
            CoreType::Named { name, arguments }
                if argument && arguments.is_empty() && self.names_no_type(name, scope) =>
            {
                unit()
            }
            CoreType::Named { name, arguments } => CoreType::Named {
                name: name.clone(),
                arguments: ground(arguments),
            },
            CoreType::ExternalNamed {
                name,
                canonical,
                arguments,
            } => CoreType::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: ground(arguments),
            },
            CoreType::Function { parameter, result } => CoreType::Function {
                parameter: Box::new(recur(parameter)),
                result: Box::new(recur(result)),
            },
            CoreType::Tuple { elements } => CoreType::Tuple {
                elements: elements.iter().map(recur).collect(),
            },
            CoreType::Record { closed, fields } => CoreType::Record {
                closed: *closed,
                fields: fields
                    .iter()
                    .map(|field| seseragi_lowering::CoreRecordField {
                        type_ref: recur(&field.type_ref),
                        ..field.clone()
                    })
                    .collect(),
            },
            other => other.clone(),
        }
    }

    /// Whether an unqualified type name resolves to nothing: not a type
    /// parameter in scope, a prelude or imported type, nor a type this module
    /// declares.
    fn names_no_type(&self, name: &str, scope: &Scope) -> bool {
        if scope.type_parameter_arity(name).is_some() {
            return false;
        }
        let ty = CoreType::Named {
            name: name.into(),
            arguments: vec![],
        };
        let Ok(CoreType::Named {
            name: canonical, ..
        }) = self.local_canonical_type(&ty, scope)
        else {
            return false;
        };
        canonical == format!("{}::{name}", self.owner)
            && !self
                .type_names
                .contains_key(&(self.owner.to_owned(), name.to_owned()))
            && !self.module.adts.iter().any(|ty| ty.symbol == canonical)
            && !self.module.structs.iter().any(|ty| ty.symbol == canonical)
            && !self.module.aliases.iter().any(|ty| ty.symbol == canonical)
    }

    pub(in crate::emit) fn local_canonical_type(
        &self,
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<CoreType> {
        let parameters = scope
            .type_parameters
            .iter()
            .map(|(name, (_, identity))| (name.clone(), identity.clone()))
            .collect();
        display::canonical_type(ty, self.owner, &parameters, self.type_names, false)
            .ok_or_else(|| "Rust backend: unproven local callable type".into())
    }

    pub(in crate::emit) fn local_constructor_arity(
        &self,
        ty: &CoreType,
        scope: &Scope,
    ) -> Option<usize> {
        if matches!(ty, CoreType::TypeFunction { .. }) {
            return self.declared_method_type_kind(ty, scope);
        }
        if let CoreType::Named { name, arguments } = ty {
            if arguments.is_empty() {
                if let Some(arity) = scope.type_parameter_arity(name) {
                    return Some(arity as usize);
                }
            }
        }
        let CoreType::Named { name, .. } = self.local_canonical_type(ty, scope).ok()? else {
            return None;
        };
        self.module
            .adts
            .iter()
            .find(|ty| ty.symbol == name)
            .map(|ty| ty.type_parameters.len())
            .or_else(|| {
                self.module
                    .structs
                    .iter()
                    .find(|ty| ty.symbol == name)
                    .map(|ty| ty.type_parameters.len())
            })
            .or_else(|| match name.as_str() {
                "std/signal::Signal"
                | "std/signal::MutableSignal"
                | "std/prelude::Sum"
                | "std/prelude::Product"
                | "std/web/html::Html"
                | "std/web/html::EventAction"
                    if !self.program_wasm_library =>
                {
                    Some(1)
                }
                "std/prelude::Array" | "std/prelude::List" | "std/prelude::Range"
                | "std/prelude::Maybe" | "std/ref::Ref" => Some(1),
                "std/prelude::Either"
                | "std/effect::Fiber"
                | "std/effect::FiberExit"
                | "std/deferred::Deferred" => Some(2),
                _ => None,
            })
    }

    pub(in crate::emit) fn match_local_type(
        &self,
        template: &CoreType,
        actual: &CoreType,
        variables: &BTreeMap<String, u32>,
        bindings: &mut BTreeMap<String, CoreType>,
        scope: &Scope,
    ) -> Result<()> {
        if let CoreType::Named { name, arguments } = template {
            if let Some(arity) = variables.get(name) {
                if let Some(bound @ CoreType::TypeFunction { .. }) = bindings.get(name) {
                    let applied =
                        seseragi_syntax::type_functions::apply_type_function(bound, arguments)
                            .ok_or("Rust backend: unproven type-function application")?;
                    return self.match_local_type(&applied, actual, variables, bindings, scope);
                }
                if arguments.is_empty() && matches!(actual, CoreType::TypeFunction { .. }) {
                    if self.declared_method_type_kind(actual, scope) != Some(*arity as usize) {
                        return Err("Rust backend: type-function binder kind mismatch".into());
                    }
                    let actual = actual
                        .normalized_type_function()
                        .ok_or("Rust backend: unproven closed type function")?;
                    if bindings.get(name).is_some_and(|bound| bound != &actual) {
                        return Err("Rust backend: local callable type parameter mismatch".into());
                    }
                    bindings.insert(name.clone(), actual);
                    return Ok(());
                }
                let actual = if *arity == 0 && arguments.is_empty() {
                    self.local_canonical_type(actual, scope)?
                } else {
                    if arguments.len() > *arity as usize {
                        return Err(
                            "Rust backend: local constructor application arity mismatch".into()
                        );
                    }
                    let (mut constructor, actual_arguments) = nominal_parts(actual)
                        .ok_or("Rust backend: unproven local constructor parameter")?;
                    let prefix = actual_arguments
                        .len()
                        .checked_sub(arguments.len())
                        .ok_or("Rust backend: local constructor argument arity mismatch")?;
                    let kind = self
                        .local_constructor_arity(&constructor, scope)
                        .ok_or("Rust backend: unproven local constructor identity or kind")?;
                    if kind.checked_sub(prefix) != Some(*arity as usize) {
                        return Err("Rust backend: local constructor kind mismatch".into());
                    }
                    for (template, actual) in arguments.iter().zip(&actual_arguments[prefix..]) {
                        self.match_local_type(template, actual, variables, bindings, scope)?;
                    }
                    match &mut constructor {
                        CoreType::Named { arguments, .. }
                        | CoreType::ExternalNamed { arguments, .. } => {
                            *arguments = actual_arguments[..prefix].to_vec()
                        }
                        _ => unreachable!(),
                    }
                    self.local_canonical_type(&constructor, scope)?
                };
                if bindings.get(name).is_some_and(|bound| bound != &actual) {
                    return Err("Rust backend: local callable type parameter mismatch".into());
                }
                bindings.insert(name.clone(), actual);
                return Ok(());
            }
        }
        if let (Some((left, left_args)), Some((right, right_args))) =
            (nominal_parts(template), nominal_parts(actual))
        {
            if self.local_canonical_type(&left, scope)?
                == self.local_canonical_type(&right, scope)?
                && left_args.len() == right_args.len()
            {
                for (left, right) in left_args.iter().zip(&right_args) {
                    self.match_local_type(left, right, variables, bindings, scope)?;
                }
                return Ok(());
            }
        } else {
            let children = match (template, actual) {
                (CoreType::Tuple { elements: left }, CoreType::Tuple { elements: right })
                    if left.len() == right.len() =>
                {
                    Some((left.clone(), right.clone()))
                }
                (
                    CoreType::Function {
                        parameter: lp,
                        result: lr,
                    },
                    CoreType::Function {
                        parameter: rp,
                        result: rr,
                    },
                ) => Some((
                    vec![*lp.clone(), *lr.clone()],
                    vec![*rp.clone(), *rr.clone()],
                )),
                (
                    CoreType::Record {
                        closed: left_closed,
                        fields: left,
                    },
                    CoreType::Record {
                        closed: right_closed,
                        fields: right,
                    },
                ) if left_closed == right_closed && left.len() == right.len() => {
                    let mut left = left.iter().collect::<Vec<_>>();
                    let mut right = right.iter().collect::<Vec<_>>();
                    left.sort_by(|l, r| l.name.cmp(&r.name));
                    right.sort_by(|l, r| l.name.cmp(&r.name));
                    if !left
                        .iter()
                        .zip(&right)
                        .all(|(l, r)| l.name == r.name && l.optional == r.optional)
                    {
                        return Err("Rust backend: local callable constraint type mismatch".into());
                    }
                    Some((
                        left.iter().map(|field| field.type_ref.clone()).collect(),
                        right.iter().map(|field| field.type_ref.clone()).collect(),
                    ))
                }
                (
                    CoreType::RequirementMerge { operands: left },
                    CoreType::RequirementMerge { operands: right },
                ) if left.len() == right.len() => Some((left.clone(), right.clone())),
                _ => None,
            };
            if let Some((left, right)) = children {
                for (left, right) in left.iter().zip(&right) {
                    self.match_local_type(left, right, variables, bindings, scope)?;
                }
                return Ok(());
            }
            if self.local_canonical_type(template, scope)?
                == self.local_canonical_type(actual, scope)?
            {
                return Ok(());
            }
        }
        Err("Rust backend: local callable constraint type mismatch".into())
    }
}
