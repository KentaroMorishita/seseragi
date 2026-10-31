use super::*;
use seseragi_lowering::{CoreInstanceMethod, CoreTraitDispatch, CoreTraitMethodContract};

mod values;

impl Backend<'_> {
    pub(super) fn validate_bare_method(
        &self,
        dispatch: &CoreTraitDispatch,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let [selected] = evidence else {
            return Err("Rust backend: bare method requires one primary dictionary".into());
        };
        let mut declared = selected.constraint.clone();
        declared.trait_identity = Some(dispatch.trait_identity.clone());
        if !self.valid_constraint_label(&declared)
            || selected
                .constraint
                .trait_identity
                .as_deref()
                .is_some_and(|identity| identity != dispatch.trait_identity)
        {
            return Err("Rust backend: bare method dictionary trait mismatch".into());
        }
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[declared], &[]),
            evidence,
            scope,
        )?;
        if !self.program_wasm_library && self.validate_standard_error_evidence(selected, scope)? {
            return self.validate_derived_method(dispatch, selected, type_ref, scope);
        }
        self.validate_signal_method(dispatch, selected, type_ref, scope)?;
        self.validate_decimal_show_value(dispatch, selected, type_ref, scope)?;
        self.validate_decimal_constant_value(dispatch, selected, type_ref, scope)?;
        self.validate_big_int_display_value(dispatch, selected, type_ref, scope)?;
        self.validate_big_int_constant_value(dispatch, selected, type_ref, scope)?;
        self.validate_primitive_operator_value(dispatch, selected, type_ref, scope)?;
        if dispatch.trait_identity == "std/prelude::Ord" {
            self.validate_ord_method_value(dispatch, selected, type_ref, scope)?;
        }
        let (identity, type_arguments) = match &selected.evidence {
            CoreInstanceEvidence::Parameter { .. } => {
                return self.validate_parameter_method_value(dispatch, selected, type_ref, scope);
            }
            CoreInstanceEvidence::Local {
                identity,
                type_arguments,
                ..
            }
            | CoreInstanceEvidence::Imported {
                identity,
                type_arguments,
                ..
            } => (identity, type_arguments),
            _ => return Ok(()), // Standard dictionaries have their own validators.
        };
        let instance = self
            .module
            .instances
            .iter()
            .find(|i| &i.identity == identity)
            .ok_or("Rust backend: unknown bare method instance")?;
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            if self.derived_identity(instance, &dispatch.trait_identity) {
                return self.validate_derived_method(dispatch, selected, type_ref, scope);
            }
            if matches!(
                instance.implementation,
                CoreInstanceImplementation::DerivedJson { .. }
            ) {
                let operation = match (dispatch.trait_identity.as_str(), dispatch.method.as_str()) {
                    ("std/prelude::JsonEncode", "encodeJson") => {
                        "std/prelude::JsonEncode::encodeJson"
                    }
                    ("std/prelude::JsonDecode", "decodeJson") => {
                        "std/prelude::JsonDecode::decodeJson"
                    }
                    _ => return Err("Rust backend: unproven derived JSON method".into()),
                };
                self.json_codec_signature(operation, evidence, type_ref, scope)?;
                return Ok(());
            }
            return if dispatch.trait_identity == "std/prelude::Show" && dispatch.method == "show" {
                Ok(())
            } else {
                Err("Rust backend: unproven derived method reference".into())
            };
        };
        let method = methods
            .iter()
            .find(|m| m.name == dispatch.method)
            .ok_or("Rust backend: unknown selected method")?;
        if !method.constraints.is_empty() {
            return Err("Rust backend: missing bare method constraint stages".into());
        }
        for (parameter, actual) in instance.type_parameters.iter().zip(type_arguments) {
            if self.declared_method_type_kind(actual, scope) != Some(parameter.arity as usize) {
                return Err("Rust backend: selected method instance argument kind mismatch".into());
            }
        }
        let mut substitutions = instance
            .type_parameters
            .iter()
            .zip(type_arguments)
            .map(|(p, ty)| {
                self.local_canonical_type(ty, scope)
                    .map(|ty| (p.name.clone(), ty))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut variables = BTreeMap::new();
        for (index, parameter) in method.type_parameters.iter().enumerate() {
            if substitutions.contains_key(&parameter.name) {
                return Err("Rust backend: unproven overlapping method type scope".into());
            }
            let name = format!("\0bare_method_type_{index}");
            variables.insert(name.clone(), parameter.arity);
            substitutions.insert(
                parameter.name.clone(),
                CoreType::Named {
                    name,
                    arguments: vec![],
                },
            );
        }
        // Reconstruct source stages from the implementation's parameters and
        // checked body result, including a function returned by that body.
        // Dictionary stages are deliberately absent from this value type.
        let mut method_type = local::method_argument_type(&method.body)
            .ok_or("Rust backend: unproven selected method result")?;
        for parameter in method.parameters.iter().rev() {
            method_type = CoreType::Function {
                parameter: Box::new(parameter.type_ref.clone()),
                result: Box::new(method_type),
            };
        }
        let method_type = display::canonical_type(
            &method_type,
            &self.owners[identity],
            &substitutions,
            self.type_names,
            true,
        )
        .ok_or("Rust backend: unproven selected method value type")?;
        self.validate_method_value_type(&method_type, type_ref, &variables, scope)
    }

    fn validate_parameter_method_value(
        &self,
        dispatch: &CoreTraitDispatch,
        selected: &CoreCallEvidence,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if dispatch.trait_identity == "std/prelude::Hash" && dispatch.method == "hash" {
            return Ok(()); // The complete A -> Int value is checked by validate_hash_call.
        }
        if dispatch.trait_identity == "std/prelude::Ord" {
            return self.validate_ord_method_value(dispatch, selected, type_ref, scope);
        }
        if self.module.trait_methods.iter().any(|declaration| {
            declaration.trait_identity == dispatch.trait_identity
                && declaration.method == dispatch.method
        }) {
            return self.validate_declared_parameter_method(dispatch, selected, type_ref, scope);
        }
        if matches!(
            (dispatch.trait_identity.as_str(), dispatch.method.as_str()),
            ("std/prelude::Functor", "map")
                | ("std/prelude::Applicative", "apply" | "pure")
                | ("std/prelude::Monad", "flatMap")
        ) {
            return self.validate_hkt_parameter_method_value(dispatch, selected, type_ref, scope);
        }
        // Eq is checked by equality_call. The remaining admitted source
        // parameter methods have the canonical A -> B -> C arithmetic ABI.
        if !matches!(
            (dispatch.trait_identity.as_str(), dispatch.method.as_str()),
            ("std/prelude::Add", "add")
                | ("std/prelude::Sub", "sub")
                | ("std/prelude::Mul", "mul")
                | ("std/prelude::Div", "div")
                | ("std/prelude::Rem", "rem")
                | ("std/prelude::Pow", "pow")
        ) || selected.constraint.arguments.len() != 3
        {
            return Err("Rust backend: unproven scoped method value signature".into());
        }
        let [left, right, result] = selected.constraint.arguments.as_slice() else {
            unreachable!()
        };
        let expected = CoreType::Function {
            parameter: Box::new(left.clone()),
            result: Box::new(CoreType::Function {
                parameter: Box::new(right.clone()),
                result: Box::new(result.clone()),
            }),
        };
        self.match_local_type(
            &expected,
            type_ref,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn validate_declared_parameter_method(
        &self,
        dispatch: &CoreTraitDispatch,
        selected: &CoreCallEvidence,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let declaration = self
            .module
            .trait_methods
            .iter()
            .find(|declaration| {
                declaration.trait_identity == dispatch.trait_identity
                    && declaration.method == dispatch.method
            })
            .ok_or("Rust backend: missing abstract method declaration")?;
        let signature = &declaration.signature;
        if !declaration.constraints.is_empty()
            || dispatch.method_contract.is_some()
            || signature.parameters.is_empty()
            || !signature.argument_types.is_empty()
            || declaration.trait_parameters.len() != selected.constraint.arguments.len()
        {
            return Err(
                "Rust backend: unproven abstract method declaration staging or kind".into(),
            );
        }
        let owner = declaration
            .trait_identity
            .rsplit_once("::")
            .map(|(owner, _)| owner)
            .ok_or("Rust backend: missing abstract method declaration owner")?;
        let mut names = BTreeSet::new();
        let mut substitutions = BTreeMap::new();
        for (parameter, actual) in declaration
            .trait_parameters
            .iter()
            .zip(&selected.constraint.arguments)
        {
            if !names.insert(parameter.name.clone()) {
                return Err("Rust backend: overlapping abstract method declaration binder".into());
            }
            if self.declared_method_type_kind(actual, scope) != Some(parameter.arity as usize) {
                return Err("Rust backend: abstract method trait argument kind mismatch".into());
            }
            substitutions.insert(
                parameter.name.clone(),
                self.local_canonical_type(actual, scope)?,
            );
        }
        let mut variables = BTreeMap::new();
        for (index, parameter) in signature.type_parameters.iter().enumerate() {
            if !names.insert(parameter.name.clone()) {
                return Err("Rust backend: overlapping abstract method declaration binder".into());
            }
            let name = format!(
                "\0declared_method::{}::{}::{index}",
                dispatch.trait_identity, dispatch.method
            );
            variables.insert(name.clone(), parameter.arity);
            substitutions.insert(
                parameter.name.clone(),
                CoreType::Named {
                    name,
                    arguments: vec![],
                },
            );
        }
        let mut expected = signature.result.clone();
        for parameter in signature.parameters.iter().rev() {
            expected = CoreType::Function {
                parameter: Box::new(parameter.clone()),
                result: Box::new(expected),
            };
        }
        let expected =
            display::canonical_type(&expected, owner, &substitutions, self.type_names, true)
                .ok_or("Rust backend: unproven abstract method declaration type")?;
        self.validate_method_value_type(&expected, type_ref, &variables, scope)
    }

    fn validate_method_value_type(
        &self,
        expected: &CoreType,
        type_ref: &CoreType,
        variables: &BTreeMap<String, u32>,
        scope: &Scope,
    ) -> Result<()> {
        // Check the complete selected value before matching declaration-local
        // constructor variables. Counting an ADT's parameters alone does not
        // prove the kinds or order of an applied constructor's fixed prefix.
        if self.declared_method_type_kind(type_ref, scope) != Some(0) {
            return Err("Rust backend: abstract method value kind mismatch".into());
        }
        let mut bindings = BTreeMap::new();
        self.match_local_type(expected, type_ref, variables, &mut bindings, scope)?;
        for (name, arity) in variables {
            match bindings.get(name) {
                Some(actual)
                    if self.declared_method_type_kind(actual, scope) != Some(*arity as usize) =>
                {
                    return Err("Rust backend: abstract method binder kind mismatch".into());
                }
                None if *arity > 0 => {
                    return Err("Rust backend: unproven abstract method constructor binder".into());
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Kind-check the selected trait arguments before substituting them into
    /// declaration slots. Fixed prefixes consume slots from the left; scoped
    /// constructors use their binder identity, never their displayed spelling.
    pub(super) fn declared_method_type_kind(&self, ty: &CoreType, scope: &Scope) -> Option<usize> {
        self.declared_method_type_kind_scoped(ty, scope, &[])
    }

    fn declared_method_type_kind_scoped(
        &self,
        ty: &CoreType,
        scope: &Scope,
        functions: &[Vec<u32>],
    ) -> Option<usize> {
        match ty {
            CoreType::TypeFunction {
                parameter_kinds,
                body,
            } => {
                if self.program_wasm_library
                    || parameter_kinds.is_empty()
                    || parameter_kinds.iter().any(|kind| *kind != 0)
                {
                    return None;
                }
                let mut nested = functions.to_vec();
                nested.push(parameter_kinds.clone());
                (self.declared_method_type_kind_scoped(body, scope, &nested) == Some(0))
                    .then_some(parameter_kinds.len())
            }
            CoreType::TypeFunctionParameter { depth, index } => functions
                .iter()
                .rev()
                .nth(*depth as usize)
                .and_then(|function| function.get(*index as usize))
                .map(|arity| *arity as usize),
            CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                if let CoreType::Named { name, .. } = ty {
                    if let Some(arity) = scope.type_parameter_arity(name) {
                        if arguments.iter().any(|a| {
                            self.declared_method_type_kind_scoped(a, scope, functions) != Some(0)
                        }) {
                            return None;
                        }
                        return (arity as usize).checked_sub(arguments.len());
                    }
                }
                let mut head = ty.clone();
                match &mut head {
                    CoreType::Named { arguments, .. }
                    | CoreType::ExternalNamed { arguments, .. } => arguments.clear(),
                    _ => unreachable!(),
                }
                let slots = self.declared_method_constructor_slots(&head, scope)?;
                let remaining = slots.get(arguments.len()..)?;
                if arguments.iter().zip(&slots).any(|(argument, kind)| {
                    self.declared_method_type_kind_scoped(argument, scope, functions)
                        != Some(*kind as usize)
                }) || remaining.iter().any(|kind| *kind != 0)
                {
                    return None;
                }
                // TypeParameter::arity describes a constructor whose remaining
                // slots all take values. A higher-order residual kind cannot
                // be certified just by counting its unfilled slots.
                Some(remaining.len())
            }
            CoreType::Function { parameter, result } => {
                (self.declared_method_type_kind_scoped(parameter, scope, functions) == Some(0)
                    && self.declared_method_type_kind_scoped(result, scope, functions) == Some(0))
                .then_some(0)
            }
            CoreType::Tuple { elements } => elements
                .iter()
                .all(|e| self.declared_method_type_kind_scoped(e, scope, functions) == Some(0))
                .then_some(0),
            CoreType::Record { closed, fields } => (*closed
                && fields.iter().all(|f| {
                    self.declared_method_type_kind_scoped(&f.type_ref, scope, functions) == Some(0)
                }))
            .then_some(0),
            CoreType::Hole | CoreType::RequirementMerge { .. } => None,
        }
    }

    fn declared_method_constructor_slots(
        &self,
        head: &CoreType,
        scope: &Scope,
    ) -> Option<Vec<u32>> {
        let CoreType::Named { name, .. } = self.local_canonical_type(head, scope).ok()? else {
            return None;
        };
        // Native codec error types are new trusted nominal kinds. Do not let
        // that trust widen program WASM through an ignored user dictionary
        // whose body only constructs an Either and never calls a JSON API.
        if self.program_wasm_library
            && matches!(
                name.as_str(),
                "std/json::DecodeErrorKind" | "std/json::DecodeError" | "std/json::JsonReadError"
            )
        {
            return None;
        }
        if let Some(parameters) = self
            .module
            .adts
            .iter()
            .find(|ty| ty.symbol == name)
            .map(|ty| &ty.type_parameters)
            .or_else(|| {
                self.module
                    .structs
                    .iter()
                    .find(|ty| ty.symbol == name)
                    .map(|ty| &ty.type_parameters)
            })
        {
            return Some(parameters.iter().map(|p| p.arity).collect());
        }
        if !self.program_wasm_library {
            match name.as_str() {
                "std/signal::Signal" | "std/signal::MutableSignal" => return Some(vec![0]),
                "std/signal::SignalChange" | "std/signal::Subscription" => return Some(vec![]),
                _ => {}
            }
            if let Some(slots) = super::standard_errors::constructor_slots(&name) {
                return Some(slots);
            }
        }
        match name.as_str() {
            "std/path::Path"
            | "std/bytes::Byte"
            | "std/bytes::Bytes"
            | "std/bytes::ByteError"
            | "std/big-int::BigInt"
            | "std/big-int::BigIntParseError"
            | "std/big-int::BigIntConversionError"
            | "std/big-int::BigIntDivisionError"
            | "std/big-int::BigIntPowerError"
            | "std/decimal::Decimal"
            | "std/decimal::DecimalParseError"
            | "std/decimal::DecimalContext"
            | "std/decimal::DecimalContextError"
            | "std/decimal::DecimalConversionError"
            | "std/decimal::DecimalArithmeticError"
            | "std/json::Json"
            | "std/json::JsonPathSegment"
            | "std/json::JsonParseError"
            | "std/json::DecodeErrorKind"
            | "std/json::DecodeError"
            | "std/json::JsonReadError"
            | "std/collection::SizeError"
            | "std/time::Duration"
            | "std/time::DurationError"
            | "std/clock::Clock"
            | "std/path::PathError"
            | "std/fs::FileSystem"
            | "std/fs::FileSystemError"
            | "std/fs::FileSystemErrorKind"
            | "std/fs::FileSystemOperation"
            | "std/fs::FileTextError"
            | "std/fs::WriteMode"
            | "std/text::Utf8DecodeError"
            | "std/prelude::Unit"
            | "std/prelude::Never"
            | "std/prelude::Int"
            | "std/prelude::Float"
            | "std/prelude::Bool"
            | "std/prelude::Char"
            | "std/prelude::String"
            | "std/prelude::Ordering"
            | "std/prelude::Console"
            | "std/prelude::ConsoleError"
            | "std/prelude::Stdin"
            | "std/prelude::StdinError" => Some(vec![]),
            "std/prelude::Array"
            | "std/prelude::List"
            | "std/prelude::Range"
            | "std/prelude::Maybe"
            | "std/ref::Ref"
            | "std/set::Set"
            | "std/prelude::Iterator"
            | "std/collection::ReduceStep" => Some(vec![0]),
            "std/prelude::Either"
            | "std/map::Map"
            | "std/effect::Fiber"
            | "std/effect::FiberExit"
            | "std/deferred::Deferred" => Some(vec![0, 0]),
            "std/prelude::Effect" => Some(vec![0, 0, 0]),
            _ => number_parse::constructor_slots(&name),
        }
    }

    pub(super) fn validate_hkt_parameter_method_value(
        &self,
        dispatch: &CoreTraitDispatch,
        selected: &CoreCallEvidence,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let [constructor] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: HKT method dictionary arity mismatch".into());
        };
        let mut head = constructor.clone();
        let prefix = match &mut head {
            CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                std::mem::take(arguments)
            }
            _ => return Err("Rust backend: unproven HKT method constructor".into()),
        };
        if self
            .local_constructor_arity(&head, scope)
            .and_then(|arity| arity.checked_sub(prefix.len()))
            != Some(1)
        {
            return Err("Rust backend: HKT method constructor kind mismatch".into());
        }
        let variable = |name: &str| CoreType::Named {
            name: name.to_owned(),
            arguments: vec![],
        };
        let a = variable("\0hkt_method_a");
        let b = variable("\0hkt_method_b");
        let applied = |argument: CoreType| {
            let mut ty = constructor.clone();
            match &mut ty {
                CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                    arguments.push(argument)
                }
                _ => unreachable!(),
            }
            ty
        };
        let arrow = |parameter, result| CoreType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        };
        if dispatch.method == "pure" {
            if self.program_wasm_library {
                return Err(
                    "program WASM: native Applicative pure requires a target-specific ABI proof"
                        .into(),
                );
            }
            return self.match_local_type(
                &arrow(a.clone(), applied(a)),
                type_ref,
                &BTreeMap::from([("\0hkt_method_a".into(), 0)]),
                &mut BTreeMap::new(),
                scope,
            );
        }
        let (first, output) = match dispatch.method.as_str() {
            "map" => (arrow(a.clone(), b.clone()), applied(b.clone())),
            "apply" => (applied(arrow(a.clone(), b.clone())), applied(b.clone())),
            "flatMap" => (arrow(a.clone(), applied(b.clone())), applied(b.clone())),
            _ => unreachable!("canonical HKT method was checked by the caller"),
        };
        let expected = arrow(first, arrow(applied(a), output));
        self.match_local_type(
            &expected,
            type_ref,
            &BTreeMap::from([("\0hkt_method_a".into(), 0), ("\0hkt_method_b".into(), 0)]),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn method_function(
        &mut self,
        method: &CoreInstanceMethod,
        scope: &Scope,
    ) -> Result<String> {
        self.method_function_with_staging(method, scope, false)
    }

    pub(super) fn method_value_function(
        &mut self,
        method: &CoreInstanceMethod,
        scope: &Scope,
    ) -> Result<String> {
        self.method_function_with_staging(method, scope, true)
    }

    fn method_function_with_staging(
        &mut self,
        method: &CoreInstanceMethod,
        scope: &Scope,
        dictionaries_first: bool,
    ) -> Result<String> {
        if !method.constraints.is_empty() && scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven overlapping method type scope".into());
        }
        let mut inner = scope.clone();
        let mut parameters = method.parameters.clone();
        for constraint in &method.constraints {
            if !self.valid_constraint_label(constraint) {
                return Err("Rust backend: unproven method constraint identity".into());
            }
            let index = inner.dictionaries.len();
            inner.dictionaries.push(constraint.clone());
            // Implementation-only stages follow the source parameters. They
            // use the combined instance/method frame, but never source arity.
            parameters.push(CoreParameter {
                id: format!("\0dictionary_{index}"),
                kind: "evidence".into(),
                type_ref: CoreType::Hole,
            });
        }
        if dictionaries_first {
            parameters.rotate_right(method.constraints.len());
        }
        self.function(&parameters, &method.body, &inner)
    }

    pub(super) fn reject_missing_method_contract(
        &self,
        dispatch: &CoreTraitDispatch,
        evidence: &[CoreCallEvidence],
    ) -> Result<()> {
        let constrained = evidence
            .first()
            .and_then(|selected| match &selected.evidence {
                CoreInstanceEvidence::Local { identity, .. }
                | CoreInstanceEvidence::Imported { identity, .. } => self
                    .module
                    .instances
                    .iter()
                    .find(|i| &i.identity == identity),
                _ => None,
            })
            .and_then(|instance| match &instance.implementation {
                CoreInstanceImplementation::UserDefined { methods } => {
                    methods.iter().find(|method| method.name == dispatch.method)
                }
                _ => None,
            })
            .is_some_and(|method| !method.constraints.is_empty());
        if constrained {
            return Err("Rust backend: missing constrained method contract".into());
        }
        Ok(())
    }

    fn validate_method_contract(
        &self,
        dispatch: &CoreTraitDispatch,
        contract: &CoreTraitMethodContract,
        arguments: &[CoreExpr],
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if contract.parameters.is_empty()
            || contract.constraints.len() < 2
            || contract
                .constraints
                .first()
                .and_then(|c| c.trait_identity.as_deref())
                != Some(dispatch.trait_identity.as_str())
            || contract
                .constraints
                .iter()
                .any(|c| !self.valid_constraint_label(c))
        {
            return Err(
                "Rust backend: invalid constrained method contract identity or arity".into(),
            );
        }
        let callable = LocalCallable::new(std::iter::empty(), &contract.constraints, &[]);
        self.validate_local_evidence(&callable, evidence, scope)?;
        for (expected, actual) in contract.parameters.iter().zip(arguments) {
            let actual = local::method_argument_type(actual)
                .ok_or("Rust backend: unproven method argument type")?;
            self.match_local_type(
                expected,
                &actual,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        if let Some(CoreCallEvidence {
            evidence:
                CoreInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    ..
                }
                | CoreInstanceEvidence::Imported {
                    identity,
                    type_arguments,
                    ..
                },
            ..
        }) = evidence.first()
        {
            let instance = self
                .module
                .instances
                .iter()
                .find(|i| &i.identity == identity)
                .ok_or("Rust backend: unknown method instance")?;
            let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
            else {
                return Err("Rust backend: unproven constrained method implementation".into());
            };
            let method = methods
                .iter()
                .find(|method| method.name == dispatch.method)
                .ok_or("Rust backend: unknown constrained method")?;
            if method.parameters.len() != contract.parameters.len()
                || method.constraints.len() + 1 != evidence.len()
                || instance.type_parameters.len() != type_arguments.len()
            {
                return Err(
                    "Rust backend: constrained method source or dictionary arity mismatch".into(),
                );
            }
            let mut substitutions = instance
                .type_parameters
                .iter()
                .zip(type_arguments)
                .map(|(parameter, ty)| {
                    self.local_canonical_type(ty, scope)
                        .map(|ty| (parameter.name.clone(), ty))
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            let mut variables = Vec::new();
            for (index, parameter) in method.type_parameters.iter().enumerate() {
                if substitutions.contains_key(&parameter.name) {
                    return Err("Rust backend: unproven overlapping method type scope".into());
                }
                let name = format!("\0method_type_{index}");
                variables.push((name.clone(), parameter.arity));
                substitutions.insert(
                    parameter.name.clone(),
                    CoreType::Named {
                        name,
                        arguments: vec![],
                    },
                );
            }
            let canonical = |ty| {
                display::canonical_type(
                    ty,
                    &self.owners[identity],
                    &substitutions,
                    self.type_names,
                    true,
                )
                .ok_or_else(|| {
                    RustBackendError::from("Rust backend: unproven selected method type")
                })
            };
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| {
                    Ok(CoreParameter {
                        type_ref: canonical(&parameter.type_ref)?,
                        ..parameter.clone()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let constraints = method
                .constraints
                .iter()
                .map(|constraint| {
                    Ok(seseragi_lowering::CoreInstanceConstraint {
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(canonical)
                            .collect::<Result<Vec<_>>>()?,
                        ..constraint.clone()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let callable = LocalCallable::new(variables.into_iter(), &constraints, &parameters);
            self.validate_local_arguments(&callable, arguments, &evidence[1..], scope)?;
        }
        Ok(())
    }

    pub(super) fn method_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            unreachable!()
        };
        let contract = dispatch
            .method_contract
            .as_ref()
            .expect("checked method contract");
        if arguments.is_empty() {
            return self.constrained_method_value(expr, scope);
        }
        let arity = contract.parameters.len();
        if !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || *evidence_argument_index != (arguments.len() > arity).then_some(arity)
        {
            return Err("Rust backend: unproven constrained method source staging".into());
        }
        if arguments.len() < arity {
            let mut completed = expr.clone();
            let CoreExpr::Call {
                arguments: completed_arguments,
                ..
            } = &mut completed
            else {
                unreachable!()
            };
            let parameters = contract
                .parameters
                .iter()
                .skip(arguments.len())
                .map(|type_ref| {
                    let id = format!("\0method_partial_{}", self.fresh());
                    completed_arguments.push(CoreExpr::Variable {
                        callable_signature: None,
                        name: id.clone(),
                        evidence: vec![],
                        type_ref: type_ref.clone(),
                        origin: core_expr_origin(expr).clone(),
                    });
                    CoreParameter {
                        id,
                        kind: "named".into(),
                        type_ref: type_ref.clone(),
                    }
                })
                .collect::<Vec<_>>();
            let mut inner = scope.clone();
            for parameter in &parameters {
                inner.insert(parameter.id.clone());
            }
            let CoreExpr::Call {
                arguments: completed_arguments,
                ..
            } = &completed
            else {
                unreachable!()
            };
            self.validate_method_contract(
                dispatch,
                contract,
                completed_arguments,
                evidence,
                &inner,
            )?;
            let primary = self.evidence(&evidence[..1], scope)?;
            let stage = format!("\0method_stage_{}", self.fresh());
            let mut output = format!(
                "{{ let mut {} = ({primary}).remove(0).field({});",
                id(&stage),
                quoted(&dispatch.method)
            );
            for argument in arguments {
                output.push_str(&format!(
                    "{0} = {0}.apply({1});",
                    id(&stage),
                    self.expr(argument, scope)?
                ));
            }
            let result = self.fresh();
            let mut body = format!("{{ let mut {result} = {}.clone();", id(&stage));
            for parameter in &parameters {
                body.push_str(&format!(
                    "{result} = {result}.apply({}.clone());",
                    id(&parameter.id)
                ));
            }
            for selected in &evidence[1..] {
                let dictionary = self.evidence(std::slice::from_ref(selected), &inner)?;
                body.push_str(&format!(
                    "{result} = {result}.apply(({dictionary}).remove(0));"
                ));
            }
            body.push_str(&format!("{result} }}"));
            let mut captures = scope.clone();
            captures.retain(|name| name.starts_with("\0dictionary_"));
            captures.insert(stage.clone());
            for (index, parameter) in parameters.iter().enumerate().rev() {
                let mut previous = captures.clone();
                for earlier in &parameters[..index] {
                    previous.insert(earlier.id.clone());
                }
                body = Self::closure(&parameter.id, &body, &previous);
            }
            output.push_str(&body);
            output.push_str(" }");
            return Ok(output);
        }
        self.validate_method_contract(dispatch, contract, arguments, evidence, scope)?;
        let primary = self.evidence(&evidence[..1], scope)?;
        let temporary = self.fresh();
        let mut value = format!(
            "{{ let mut {temporary} = ({primary}).remove(0).field({});",
            quoted(&dispatch.method)
        );
        // Source stages can fail or return closures. Run them at the same
        // position as TS, before materializing each method dictionary stage.
        for argument in &arguments[..arity] {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        for selected in &evidence[1..] {
            let dictionary = self.evidence(std::slice::from_ref(selected), scope)?;
            value.push_str(&format!(
                "{temporary} = {temporary}.apply(({dictionary}).remove(0));"
            ));
        }
        for argument in &arguments[arity..] {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(value)
    }
}
