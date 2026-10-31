use super::*;
use seseragi_lowering::CoreMonadDoStatement;

// Only a directly forwarded continuation result can carry a private resume
// marker. A closure inside the monadic result can run after this do returns.
pub(super) fn tail_bind(instance: &seseragi_lowering::CoreInstance) -> bool {
    if !instance.identity.starts_with("std/prelude::Monad<") {
        return false;
    }
    let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
        return false;
    };
    let Some(method) = methods.iter().find(|method| method.name == "flatMap") else {
        return false;
    };
    let Some(callback) = method.parameters.first() else {
        return false;
    };
    fn absent(expr: &CoreExpr, callback: &str) -> bool {
        core_free_variables(expr).is_some_and(|free| !free.names.contains(callback))
    }
    fn tail(expr: &CoreExpr, callback: &str, source_parameters: usize) -> bool {
        if absent(expr, callback) {
            return true;
        }
        match expr {
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                trait_dispatch,
                ..
            } if callee == callback && source_parameters == 0 => {
                arguments.len() == 1
                    && evidence.is_empty()
                    && trait_dispatch.is_none()
                    && absent(&arguments[0], callback)
            }
            CoreExpr::Lambda {
                parameter, body, ..
            } => {
                source_parameters > 0
                    && parameter.id != callback
                    && tail(body, callback, source_parameters - 1)
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => {
                statements.iter().all(|statement| match statement {
                    CoreStatement::PureLet { name, value, .. } => {
                        name != callback && absent(value, callback)
                    }
                    _ => false,
                }) && tail(result, callback, source_parameters)
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                absent(condition, callback)
                    && tail(then_branch, callback, source_parameters)
                    && tail(else_branch, callback, source_parameters)
            }
            CoreExpr::Decision {
                scrutinee,
                branches,
                ..
            } => {
                absent(scrutinee, callback)
                    && branches.iter().all(|branch| {
                        branch
                            .bindings
                            .iter()
                            .all(|binding| binding.name != callback)
                            && branch
                                .guard
                                .as_ref()
                                .is_none_or(|guard| absent(guard, callback))
                            && tail(&branch.value, callback, source_parameters)
                    })
            }
            _ => false,
        }
    }
    method.parameters.len() <= 2
        && tail(
            &method.body,
            &callback.id,
            2usize.saturating_sub(method.parameters.len()),
        )
}

impl Backend<'_> {
    pub(super) fn monad_supertrait(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<Option<Vec<&'static str>>> {
        let (parent_name, fields) = if instance.identity.starts_with("std/prelude::Applicative<") {
            ("Functor", vec!["map"])
        } else if instance.identity.starts_with("std/prelude::Monad<") {
            ("Applicative", vec!["map", "pure", "apply"])
        } else {
            return Ok(None);
        };
        let Some(parent) = instance.constraints.first() else {
            return Err("Rust backend: missing monad supertrait".into());
        };
        if self.program_wasm_library
            || instance.trait_name
                != if parent_name == "Functor" {
                    "Applicative"
                } else {
                    "Monad"
                }
            || instance.supertrait_count != 1
            || parent.name != parent_name
            || parent.trait_identity.as_deref()
                != Some(format!("std/prelude::{parent_name}").as_str())
            || instance.arguments.len() != 1
            || parent.arguments.len() != 1
            || self.declared_method_type_kind(&instance.arguments[0], scope) != Some(1)
            || self.declared_method_type_kind(&parent.arguments[0], scope) != Some(1)
            || self.local_canonical_type(&instance.arguments[0], scope)?
                != self.local_canonical_type(&parent.arguments[0], scope)?
        {
            return Err("Rust backend: unproven monad supertrait".into());
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven monad implementation".into());
        };
        let expected: &[&str] = if parent_name == "Functor" {
            &["pure", "apply"]
        } else {
            &["flatMap"]
        };
        if methods.len() != expected.len()
            || expected
                .iter()
                .any(|name| methods.iter().filter(|method| method.name == *name).count() != 1)
        {
            return Err("Rust backend: monad instance method count mismatch".into());
        }
        for method in methods {
            let count = if method.name == "pure" { 1 } else { 2 };
            if method.type_parameters.len() != count
                || method
                    .type_parameters
                    .iter()
                    .any(|parameter| parameter.arity != 0)
                || !method.constraints.is_empty()
                || method.parameters.is_empty()
            {
                return Err("Rust backend: unproven monad method scheme".into());
            }
            let mut inner = scope.clone();
            inner.type_parameters(
                method
                    .type_parameters
                    .iter()
                    .map(|p| (p.name.clone(), p.arity)),
                &format!("{}::{}", instance.identity, method.name),
            );
            let variables = method
                .type_parameters
                .iter()
                .map(|parameter| CoreType::Named {
                    name: parameter.name.clone(),
                    arguments: vec![],
                })
                .collect::<Vec<_>>();
            let function = |parameter, result| CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            };
            let apply = |argument: CoreType| -> Result<CoreType> {
                let mut constructor = instance.arguments[0].clone();
                match &mut constructor {
                    CoreType::Named { arguments, .. }
                    | CoreType::ExternalNamed { arguments, .. } => arguments.push(argument),
                    CoreType::TypeFunction { .. } => {
                        return seseragi_syntax::type_functions::apply_type_function(
                            &constructor,
                            &[argument],
                        )
                        .ok_or_else(|| "Rust backend: unproven monad method constructor".into())
                    }
                    _ => return Err("Rust backend: unproven monad method constructor".into()),
                }
                Ok(constructor)
            };
            let expected = if method.name == "pure" {
                function(variables[0].clone(), apply(variables[0].clone())?)
            } else {
                let next = if method.name == "apply" {
                    apply(function(variables[0].clone(), variables[1].clone()))?
                } else {
                    function(variables[0].clone(), apply(variables[1].clone())?)
                };
                function(
                    next,
                    function(apply(variables[0].clone())?, apply(variables[1].clone())?),
                )
            };
            let actual = method.parameters.iter().rev().fold(
                local::method_argument_type(&method.body)
                    .ok_or("Rust backend: missing monad method result")?,
                |result, parameter| function(parameter.type_ref.clone(), result),
            );
            if self.declared_method_type_kind(&actual, &inner) != Some(0)
                || self.local_canonical_type(&actual, &inner)?
                    != self.local_canonical_type(&expected, &inner)?
            {
                return Err("Rust backend: monad method signature mismatch".into());
            }
        }
        Ok(Some(fields))
    }

    pub(super) fn monad_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let Some((provider, method)) = identity.rsplit_once("::") else {
            return Ok(None);
        };
        if !matches!(method, "Functor" | "Applicative" | "Monad") {
            return Ok(None);
        }
        let (owner, slots, kind) = match provider {
            "std/array" => ("std/prelude::Array", 1, 0),
            "std/list" => ("std/prelude::List", 1, 1),
            "std/maybe" => ("std/prelude::Maybe", 1, 2),
            "std/either" => ("std/prelude::Either", 2, 3),
            "std/non-empty-list" => ("std/non-empty-list::NonEmptyList", 1, 4),
            "std/effect" => ("std/prelude::Effect", 3, 5),
            "std/validation" if method != "Monad" => ("std/validation::Validation", 2, 6),
            "std/map" if method == "Functor" => ("std/map::Map", 2, 7),
            "std/stream" => ("std/stream::Stream", 3, 8),
            "std/transformer/maybe" => ("std/transformer/maybe::MaybeT", 2, 9),
            "std/transformer/either" => ("std/transformer/either::EitherT", 3, 10),
            "std/transformer/reader" => ("std/transformer/reader::ReaderT", 3, 11),
            "std/transformer/state" => ("std/transformer/state::StateT", 3, 12),
            "std/transformer/writer" => ("std/transformer/writer::WriterT", 3, 13),
            _ => return Ok(None),
        };
        if self.program_wasm_library {
            return Err(
                "program WASM: native monad dictionary needs a target-specific ABI proof".into(),
            );
        }
        let [constructor] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: monad dictionary constraint arity mismatch".into());
        };
        if selected.constraint.name != method
            || selected.constraint.trait_identity.as_deref()
                != Some(format!("std/prelude::{method}").as_str())
            || (kind < 9 && (!type_arguments.is_empty() || !evidence_arguments.is_empty()))
            || self.declared_method_type_kind(constructor, scope) != Some(1)
        {
            return Err("Rust backend: unproven standard monad dictionary".into());
        }
        let marker = CoreType::Named {
            name: "Int".into(),
            arguments: vec![],
        };
        let applied = if matches!(constructor, CoreType::TypeFunction { .. }) {
            seseragi_syntax::type_functions::apply_type_function(
                constructor,
                std::slice::from_ref(&marker),
            )
            .ok_or("Rust backend: unproven monad type function")?
        } else {
            let mut applied = constructor.clone();
            match &mut applied {
                CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                    arguments.push(marker)
                }
                _ => return Err("Rust backend: unproven monad constructor".into()),
            }
            applied
        };
        let applied = self.local_canonical_type(&applied, scope)?;
        if !matches!(&applied, CoreType::Named { name, arguments } if name == owner && arguments.len() == slots && arguments.last() == Some(&CoreType::Named { name: "std/prelude::Int".into(), arguments: vec![] }))
        {
            return Err("Rust backend: monad provider constructor mismatch".into());
        }
        self.capabilities.insert("monad".into());
        if matches!(kind, 5 | 8) {
            self.capabilities.insert("effect-sync".into());
        }
        if kind >= 9 {
            let CoreType::Named { arguments, .. } = applied else {
                unreachable!()
            };
            if type_arguments.len() != slots - 1
                || type_arguments
                    .iter()
                    .map(|ty| self.local_canonical_type(ty, scope))
                    .collect::<Result<Vec<_>>>()?
                    != arguments[..slots - 1]
                || evidence_arguments.len() != if kind == 13 { 2 } else { 1 }
            {
                return Err(
                    "Rust backend: transformer dictionary factory arguments mismatch".into(),
                );
            }
            let base = if kind == 9 { 0 } else { 1 };
            for (child, (label, ty)) in evidence_arguments.iter().zip(
                std::iter::once(("Monad", &arguments[base]))
                    .chain((kind == 13).then_some(("Monoid", &arguments[0]))),
            ) {
                if child.constraint.name != label
                    || child.constraint.trait_identity.as_deref()
                        != Some(format!("std/prelude::{label}").as_str())
                    || child.constraint.arguments.len() != 1
                    || self.local_canonical_type(&child.constraint.arguments[0], scope)? != *ty
                {
                    return Err("Rust backend: transformer child dictionary mismatch".into());
                }
            }
            let children = self.evidence(evidence_arguments, scope)?;
            return Ok(Some(format!(
                "native_transformer_dictionary({kind}, {children})"
            )));
        }
        Ok(Some(format!(
            "native_monad_dictionary({kind}, {})",
            quoted(method)
        )))
    }

    pub(super) fn monad_do(
        &mut self,
        statements: &[CoreMonadDoStatement],
        result: &CoreExpr,
        evidence: &CoreCallEvidence,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<String> {
        if evidence.constraint.name != "Monad"
            || evidence.constraint.trait_identity.as_deref() != Some("std/prelude::Monad")
        {
            return Err("Rust backend: do requires a Monad dictionary".into());
        }
        let [constructor] = evidence.constraint.arguments.as_slice() else {
            return Err("Rust backend: do dictionary constructor arity".into());
        };
        if self.declared_method_type_kind(constructor, scope) != Some(1) {
            return Err("Rust backend: do dictionary requires a unary constructor".into());
        }
        let check = |actual: &CoreType, payload: Option<&CoreType>| -> Result<()> {
            let item = "\0monad_do_item".to_owned();
            let argument = CoreType::Named {
                name: item.clone(),
                arguments: vec![],
            };
            let mut template = constructor.clone();
            match &mut template {
                CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                    arguments.push(argument)
                }
                CoreType::TypeFunction { .. } => {
                    template =
                        seseragi_syntax::type_functions::apply_type_function(&template, &[argument])
                            .ok_or("Rust backend: unproven do constructor application")?
                }
                _ => return Err("Rust backend: unproven do constructor".into()),
            }
            let mut bindings = BTreeMap::new();
            if self.declared_method_type_kind(actual, scope) != Some(0) {
                return Err("Rust backend: unproven do value kind".into());
            }
            self.match_local_type(
                &template,
                actual,
                &BTreeMap::from([(item.clone(), 0)]),
                &mut bindings,
                scope,
            )?;
            if let Some(payload) = payload {
                if bindings.get(&item) != Some(&self.local_canonical_type(payload, scope)?) {
                    return Err("Rust backend: do bind payload mismatch".into());
                }
            }
            Ok(())
        };
        check(type_ref, None)?;
        let result_type =
            local::method_argument_type(result).ok_or("Rust backend: missing do result type")?;
        check(&result_type, None)?;
        if self.local_canonical_type(type_ref, scope)?
            != self.local_canonical_type(&result_type, scope)?
        {
            return Err("Rust backend: do result type mismatch".into());
        }
        for statement in statements {
            let (value, payload) = match statement {
                CoreMonadDoStatement::Bind {
                    value, type_ref, ..
                } => (value, Some(type_ref)),
                CoreMonadDoStatement::Expression { value } => (value, None),
                CoreMonadDoStatement::PureLet { .. } => continue,
            };
            check(
                &local::method_argument_type(value)
                    .ok_or("Rust backend: missing do source type")?,
                payload,
            )?;
        }
        let dictionary = self.evidence(std::slice::from_ref(evidence), scope)?;
        let mut inner = scope.clone();
        let mut bindings: BTreeMap<String, usize> = BTreeMap::new();
        let mut count = 0;
        let mut steps = Vec::new();
        for statement in statements.iter().map(Some).chain(std::iter::once(None)) {
            let (kind, name, value) = match statement {
                Some(CoreMonadDoStatement::Expression { value }) => (1, None, value),
                Some(CoreMonadDoStatement::Bind { name, value, .. }) => (2, Some(name), value),
                Some(CoreMonadDoStatement::PureLet { name, value, .. }) => (0, Some(name), value),
                None => (3, None, result),
            };
            let mut next_callable = None;
            let body = if let Some(CoreMonadDoStatement::PureLet {
                type_parameters,
                constraints,
                value,
                ..
            }) = statement.filter(|statement| matches!(statement, CoreMonadDoStatement::PureLet { type_parameters, constraints, .. } if !type_parameters.is_empty() || !constraints.is_empty()))
            {
                let (parameters, body) = match value {
                    CoreExpr::Lambda {
                        parameter, body, ..
                    } => (std::slice::from_ref(parameter), body.as_ref()),
                    _ => (&[][..], value),
                };
                let mut callable = LocalCallable::new(
                    type_parameters.iter().map(|p| (p.name.clone(), p.arity)),
                    constraints,
                    parameters,
                );
                if parameters.is_empty() {
                    callable.method_value = local::method_argument_type(value)
                        .filter(|ty| matches!(ty, CoreType::Function { .. }));
                }
                let emitted = self.local_function(&callable, body, &inner)?;
                next_callable = Some(callable);
                emitted
            } else {
                self.expr(value, &inner)?
            };
            let state = self.fresh();
            let free = core_free_variables(value);
            let needed = |name: &str| free.as_ref().is_none_or(|free| free.names.contains(name));
            let mut captures = scope.clone();
            if let Some(mut free) = free.clone() {
                for index in free.evidence_parameters {
                    free.names.insert(format!("\0dictionary_{index}"));
                }
                captures.retain_free(&free.names);
            }
            captures.retain(|name| !bindings.contains_key(name));
            let restore = bindings
                .iter()
                .filter(|(name, _)| needed(name))
                .map(|(name, slot)| {
                    format!(
                        "let {} = {}.list_element({});",
                        id(name),
                        id(&state),
                        count - slot - 1
                    )
                })
                .collect::<String>();
            let step = Self::closure(&state, &format!("{{ {restore} {body} }}"), &captures);
            steps.push(format!(
                "V::Tuple(Shared::new(vec![V::Int({kind}), {step}]))"
            ));
            if let Some(name) = name {
                bindings.insert(name.clone(), count);
                count += 1;
                if let Some(callable) = next_callable {
                    inner.insert_callable(name.clone(), callable);
                } else {
                    inner.insert(name.clone());
                }
            }
        }
        Ok(format!("native_monad_do(({dictionary}).remove(0), V::Array(Shared::new(vec![{}])), 0, V::ListEmpty)", steps.join(",")))
    }
}
