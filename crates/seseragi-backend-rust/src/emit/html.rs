use super::*;

fn operation(name: &str) -> Option<&str> {
    let name = name.strip_prefix("std/web/html::")?;
    (seseragi_project::standard_html_tag(name).is_some()
        || matches!(
            name,
            "style" | "text" | "fragment" | "renderToString" | "renderDocument" | "parseWebUrl"
        ))
    .then_some(name)
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn html_action(ty: &CoreType) -> Result<&CoreType> {
    match ty {
        CoreType::Named { name, arguments }
            if name == "std/web/html::Html" && arguments.len() == 1 =>
        {
            Ok(&arguments[0])
        }
        _ => Err("Rust backend: unproven canonical Html type".into()),
    }
}

fn children(ty: &CoreType, action: &CoreType) -> bool {
    if *ty == named("std/prelude::String", vec![]) || html_action(ty).is_ok_and(|a| a == action) {
        return true;
    }
    matches!(ty, CoreType::Named { name, arguments }
        if matches!(name.as_str(), "std/prelude::Array" | "std/prelude::List")
        && matches!(arguments.as_slice(), [html] if html_action(html).is_ok_and(|a| a == action)))
}

fn style_record(ty: &CoreType, variables: bool) -> bool {
    let CoreType::Record {
        closed: true,
        fields,
    } = ty
    else {
        return false;
    };
    let mut names = BTreeSet::new();
    fields.iter().all(|f| {
        names.insert(&f.name)
            && if f.name == "variables" && !variables {
                style_record(&f.type_ref, true)
            } else {
                f.type_ref == named("std/prelude::String", vec![])
            }
    })
}

fn substitute(ty: CoreType, action: &CoreType) -> CoreType {
    match ty {
        CoreType::Named { name, arguments } if name == "Action" && arguments.is_empty() => {
            action.clone()
        }
        CoreType::Named { name, arguments } => named(
            &name,
            arguments
                .into_iter()
                .map(|a| substitute(a, action))
                .collect(),
        ),
        CoreType::Function { parameter, result } => CoreType::Function {
            parameter: Box::new(substitute(*parameter, action)),
            result: Box::new(substitute(*result, action)),
        },
        ty => ty,
    }
}

impl Backend<'_> {
    fn validate_html_function(
        &mut self,
        name: &str,
        ty: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native HTML requires a target-specific ABI proof".into());
        }
        let ty = self.local_canonical_type(ty, scope)?;
        let CoreType::Function { parameter, result } = &ty else {
            return Err("Rust backend: HTML callable contract".into());
        };
        let string = named("std/prelude::String", vec![]);
        let valid = match name {
            "style" => {
                style_record(parameter, false) && **result == named("std/web/html::Style", vec![])
            }
            "parseWebUrl" => {
                **parameter == string
                    && **result
                        == named(
                            "std/prelude::Either",
                            vec![
                                named("std/web/html::HtmlBuildError", vec![]),
                                named("std/web/html::WebUrl", vec![]),
                            ],
                        )
            }
            "renderToString" | "renderDocument" => {
                html_action(parameter).is_ok() && **result == string
            }
            "text" => **parameter == string && html_action(result).is_ok(),
            "fragment" => html_action(result).is_ok_and(|action| children(parameter, action)),
            tag => {
                let action = html_action(result)?;
                let CoreType::Record {
                    closed: true,
                    fields,
                } = &**parameter
                else {
                    return Err("Rust backend: HTML props must be a closed record".into());
                };
                let (_, allowed) = seseragi_project::standard_html_tag_props(tag).unwrap();
                let mut names = BTreeSet::new();
                let fields_valid = fields.iter().all(|field| {
                    if !names.insert(&field.name) {
                        return false;
                    }
                    let Some(expected) = allowed.iter().find(|f| f.name == field.name) else {
                        return false;
                    };
                    if !expected.optional && field.optional {
                        return false;
                    }
                    if field.name == "children" {
                        return children(&field.type_ref, action);
                    }
                    let Some(expected) = standard_errors::interface_type(
                        &expected.type_ref,
                        "std/web/html",
                        &BTreeSet::from(["Action".into()]),
                    ) else {
                        return false;
                    };
                    field.type_ref == substitute(expected, action)
                });
                fields_valid
                    && allowed.iter().filter(|f| !f.optional).all(|required| {
                        fields
                            .iter()
                            .any(|f| f.name == required.name && !f.optional)
                    })
            }
        };
        if !valid {
            return Err("Rust backend: unproven HTML argument/result type".into());
        }
        let child_type = if name == "fragment" {
            Some(&**parameter)
        } else if let CoreType::Record { fields, .. } = &**parameter {
            if name == "style" {
                None
            } else {
                fields
                    .iter()
                    .find(|f| f.name == "children")
                    .map(|f| &f.type_ref)
            }
        } else {
            None
        };
        let expected = if name == "style" {
            Some((
                "StyleRecord",
                "std/web/html::StyleRecord<Record>",
                vec![(**parameter).clone()],
            ))
        } else if let Some(child) = child_type {
            let identity = if *child == string {
                "std/web/html::IntoChildren<String>"
            } else if html_action(child).is_ok() {
                "std/web/html::IntoChildren<Html>"
            } else if matches!(child, CoreType::Named { name, .. } if name == "std/prelude::Array")
            {
                "std/web/html::IntoChildren<Array>"
            } else {
                "std/web/html::IntoChildren<List>"
            };
            Some((
                "IntoChildren",
                identity,
                vec![child.clone(), html_action(result)?.clone()],
            ))
        } else {
            None
        };
        match (expected, evidence) {
            (None, []) => {}
            (Some((class, identity, arguments)), [selected]) => {
                let CoreInstanceEvidence::Standard {
                    identity: actual_identity,
                    type_arguments,
                    evidence_arguments,
                } = &selected.evidence
                else {
                    return Err("Rust backend: unproven HTML instance".into());
                };
                if selected.constraint.name != class
                    || selected.constraint.trait_identity.as_deref()
                        != Some(format!("std/web/html::trait({class})").as_str())
                    || *actual_identity != identity
                    || !type_arguments.is_empty()
                    || !evidence_arguments.is_empty()
                    || selected
                        .constraint
                        .arguments
                        .iter()
                        .map(|a| self.local_canonical_type(a, scope))
                        .collect::<Result<Vec<_>>>()?
                        != arguments
                {
                    return Err("Rust backend: HTML instance identity/type mismatch".into());
                }
            }
            _ => return Err("Rust backend: HTML evidence arity mismatch".into()),
        }
        Ok(())
    }

    pub(super) fn html_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some(operation) = operation(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }

        self.validate_html_function(operation, ty, evidence, scope)?;
        self.capabilities.insert("html".into());
        let void = seseragi_project::is_standard_void_html_tag(operation);
        Ok(Some(format!(
            "V::fun(vec![], |_, value| native_html_operation({}, value, {void}))",
            quoted(operation)
        )))
    }

    pub(super) fn html_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(None);
        };
        let Some(name) = operation(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > 1
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven HTML source application".into());
        }
        if arguments.is_empty() {
            return self.html_reference(callee, evidence, type_ref, scope);
        }
        let mut argument = local::method_argument_type(&arguments[0])
            .ok_or("Rust backend: missing HTML argument type")?;
        if argument == CoreType::Hole && matches!(name, "renderToString" | "renderDocument") {
            if let CoreExpr::Variable { name, evidence, .. } = &arguments[0] {
                if evidence.is_empty() && !scope.contains(name) {
                    let symbol = if name.contains("::") {
                        name.clone()
                    } else {
                        format!("{}::{name}", self.owner)
                    };
                    if let Some(binding) = self
                        .module
                        .bindings
                        .iter()
                        .find(|b| b.symbol == symbol && b.type_parameters.is_empty())
                    {
                        let ty = local::method_argument_type(&binding.value)
                            .ok_or("Rust backend: missing global Html initializer type")?;
                        let owner = symbol.rsplit_once("::").unwrap().0;
                        // A global use can lack a type even when its checked initializer has one.
                        // Recover only that declared value; never infer from rendered contents.
                        argument = display::canonical_type(
                            &ty,
                            owner,
                            &BTreeMap::new(),
                            self.type_names,
                            false,
                        )
                        .ok_or("Rust backend: unproven global Html initializer type")?;
                        html_action(&argument)?;
                    }
                }
            }
        }
        self.validate_html_function(
            name,
            &CoreType::Function {
                parameter: Box::new(argument),
                result: Box::new(type_ref.clone()),
            },
            evidence,
            scope,
        )?;
        self.capabilities.insert("html".into());
        self.direct_calls += 1;
        Ok(Some(format!(
            "native_html_operation({}, {}, {})",
            quoted(name),
            self.expr(&arguments[0], scope)?,
            seseragi_project::is_standard_void_html_tag(name)
        )))
    }
}
