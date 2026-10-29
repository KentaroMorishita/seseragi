use super::*;

fn operation(name: &str) -> Option<(bool, bool)> {
    Some(match name {
        "std/json::encodeString" => (true, true),
        "std/json::decodeString" => (false, true),
        "std/prelude::JsonEncode::encodeJson" => (true, false),
        "std/prelude::JsonDecode::decodeJson" => (false, false),
        _ => return None,
    })
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn function(parameter: CoreType, result: CoreType) -> CoreType {
    CoreType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}

impl Backend<'_> {
    fn json_codec_target(&self) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native JSON codecs require a target-specific ABI proof".into(),
            );
        }
        Ok(())
    }
    pub(super) fn json_codec_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if matches!(&selected.evidence, CoreInstanceEvidence::Local { identity, .. } | CoreInstanceEvidence::Imported { identity, .. }
            if identity.starts_with("std/prelude::JsonEncode<") || identity.starts_with("std/prelude::JsonDecode<"))
            || (matches!(
                selected.evidence,
                CoreInstanceEvidence::Local { .. } | CoreInstanceEvidence::Imported { .. }
            ) && matches!(
                selected.constraint.trait_identity.as_deref(),
                Some("std/prelude::JsonEncode" | "std/prelude::JsonDecode")
            ))
        {
            self.validate_json_codec_user_dictionary(selected, scope)?;
        }
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let Some((owner, trait_name)) = identity.split_once("::") else {
            return Ok(None);
        };
        if !matches!(trait_name, "JsonEncode" | "JsonDecode") {
            return Ok(None);
        }
        let (kind, canonical) = match owner {
            "std/bool" => ("Bool", "std/prelude::Bool"),
            "std/string" => ("String", "std/prelude::String"),
            "std/int" => ("Int", "std/prelude::Int"),
            "std/decimal" => ("Decimal", "std/decimal::Decimal"),
            "std/unit" => ("Unit", "std/prelude::Unit"),
            "std/json" => ("Json", "std/json::Json"),
            _ => return Err("Rust backend: unsupported standard JSON codec dictionary".into()),
        };
        self.json_codec_target()?;
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: JSON codec constraint arity".into());
        };
        if !lists::standard_trait(selected, trait_name, actual)
            || !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != named(canonical, vec![])
        {
            return Err(
                "Rust backend: unproven standard JSON codec owner, kind or evidence".into(),
            );
        }
        self.capabilities.insert("json".into());
        Ok(Some(format!(
            "native_json_codec_dictionary({}, {})",
            quoted(kind),
            trait_name == "JsonEncode"
        )))
    }

    fn validate_json_codec_user_dictionary(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        let encode = selected.constraint.name == "JsonEncode";
        let trait_name = if encode { "JsonEncode" } else { "JsonDecode" };
        let [value] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: JSON codec constraint arity".into());
        };
        if !lists::standard_trait(selected, trait_name, value)
            || self.declared_method_type_kind(value, scope) != Some(0)
        {
            return Err("Rust backend: unproven selected JSON codec type".into());
        }
        let identity = match &selected.evidence {
            CoreInstanceEvidence::Local { identity, .. }
            | CoreInstanceEvidence::Imported { identity, .. } => identity,
            _ => unreachable!(),
        };
        let instance = self
            .module
            .instances
            .iter()
            .find(|i| &i.identity == identity)
            .ok_or("Rust backend: unknown JSON codec dictionary")?;
        let method_name = if encode { "encodeJson" } else { "decodeJson" };
        if let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation {
            let method = methods
                .iter()
                .find(|m| m.name == method_name)
                .ok_or("Rust backend: missing selected JSON codec method")?;
            if !method.type_parameters.is_empty() {
                return Err("Rust backend: unproven JSON codec method type stages".into());
            }
        }
        let json = named("std/json::Json", vec![]);
        let method_type = if encode {
            function(value.clone(), json)
        } else {
            function(
                json,
                named(
                    "std/prelude::Either",
                    vec![named("std/json::DecodeError", vec![]), value.clone()],
                ),
            )
        };
        self.validate_bare_method(
            &seseragi_lowering::CoreTraitDispatch {
                trait_identity: format!("std/prelude::{trait_name}"),
                method: method_name.into(),
                method_contract: None,
            },
            std::slice::from_ref(selected),
            &method_type,
            scope,
        )
    }

    fn json_codec_signature(
        &self,
        name: &str,
        evidence: &[CoreCallEvidence],
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<(bool, bool)> {
        self.json_codec_target()?;
        let (encode, string) = operation(name).expect("known JSON codec");
        let [selected] = evidence else {
            return Err("Rust backend: JSON codec requires one dictionary".into());
        };
        let trait_name = if encode { "JsonEncode" } else { "JsonDecode" };
        let [value] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: JSON codec constraint arity".into());
        };
        if !lists::standard_trait(selected, trait_name, value)
            || self.declared_method_type_kind(value, scope) != Some(0)
        {
            return Err("Rust backend: unproven canonical JSON codec constraint".into());
        }
        let mut declared = selected.constraint.clone();
        declared.trait_identity = Some(format!("std/prelude::{trait_name}"));
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[declared], &[]),
            evidence,
            scope,
        )?;
        let json = named("std/json::Json", vec![]);
        let text = named("std/prelude::String", vec![]);
        let expected = if encode {
            function(value.clone(), if string { text } else { json })
        } else {
            function(
                if string { text } else { json },
                named(
                    "std/prelude::Either",
                    vec![
                        named(
                            if string {
                                "std/json::JsonReadError"
                            } else {
                                "std/json::DecodeError"
                            },
                            vec![],
                        ),
                        value.clone(),
                    ],
                ),
            )
        };
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)?
                != self.local_canonical_type(&expected, scope)?
        {
            return Err("Rust backend: unproven canonical JSON codec argument/result type".into());
        }
        Ok((encode, string))
    }

    pub(super) fn json_codec_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            type_ref,
            ..
        } = expr
        else {
            return Ok(None);
        };
        let Some((encode, string)) = operation(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > 1
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven JSON codec source stages".into());
        }
        if string {
            if trait_dispatch.is_some() {
                return Err("Rust backend: unexpected JSON string codec dispatch".into());
            }
        } else {
            let dispatch = trait_dispatch
                .as_ref()
                .ok_or("Rust backend: missing JSON codec dispatch")?;
            if dispatch.trait_identity
                != format!(
                    "std/prelude::{}",
                    if encode { "JsonEncode" } else { "JsonDecode" }
                )
                || dispatch.method != if encode { "encodeJson" } else { "decodeJson" }
                || dispatch.method_contract.is_some()
            {
                return Err("Rust backend: unproven JSON codec dispatch".into());
            }
        }
        let full = if let Some(argument) = arguments.first() {
            function(
                local::method_argument_type(argument)
                    .ok_or("Rust backend: missing JSON codec argument type")?,
                type_ref.clone(),
            )
        } else {
            type_ref.clone()
        };
        self.json_codec_signature(callee, evidence, &full, scope)?;
        self.capabilities.insert("json".into());
        let dictionary = self.evidence(evidence, scope)?;
        let method = if encode { "encodeJson" } else { "decodeJson" };
        if let Some(argument) = arguments.first() {
            self.direct_calls += 1;
            let value = self.expr(argument, scope)?;
            let d = self.fresh();
            let v = self.fresh();
            // String wrappers take the source value before the selected dictionary;
            // trait methods select their dictionary before applying the value.
            Ok(Some(if string {
                format!("{{let {v}={value};let {d}=({dictionary}).remove(0);native_json_codec_string({encode},{v},{d})}}")
            } else {
                format!(
                    "{{let {d}=({dictionary}).remove(0);{d}.field({}).apply({value})}}",
                    quoted(method)
                )
            }))
        } else {
            Ok(Some(Self::json_codec_callable(encode, string, &dictionary)))
        }
    }
    fn json_codec_callable(encode: bool, string: bool, dictionary: &str) -> String {
        if string {
            format!("V::fun(vec![({dictionary}).remove(0)],|captures,value|native_json_codec_string({encode},value,captures[0].clone()))")
        } else {
            format!(
                "({dictionary}).remove(0).field({})",
                quoted(if encode { "encodeJson" } else { "decodeJson" })
            )
        }
    }
    pub(super) fn json_codec_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if operation(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let (encode, string) = self.json_codec_signature(name, evidence, type_ref, scope)?;
        self.capabilities.insert("json".into());
        let dictionary = self.evidence(evidence, scope)?;
        Ok(Some(Self::json_codec_callable(encode, string, &dictionary)))
    }
}
