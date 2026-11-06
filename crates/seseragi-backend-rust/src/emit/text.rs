use super::*;

fn text_signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let string = CoreType::Named {
        name: "std/prelude::String".into(),
        arguments: vec![],
    };
    let strings = CoreType::Named {
        name: "std/prelude::Array".into(),
        arguments: vec![string.clone()],
    };
    let boolean = CoreType::Named {
        name: "std/prelude::Bool".into(),
        arguments: vec![],
    };
    let int = CoreType::Named {
        name: "std/prelude::Int".into(),
        arguments: vec![],
    };
    let bytes = CoreType::Named {
        name: "std/bytes::Bytes".into(),
        arguments: vec![],
    };
    let scalar = CoreType::Named {
        name: "std/prelude::Char".into(),
        arguments: vec![],
    };
    let form = CoreType::Named {
        name: "std/text/unicode::NormalizationForm".into(),
        arguments: vec![],
    };
    match name {
        "std/text/grapheme::length" => Some((vec![string], int)),
        "std/text/grapheme::clusters" => Some((vec![string], strings)),
        "std/text/grapheme::byteBoundaries" => Some((
            vec![string],
            CoreType::Named {
                name: "std/prelude::Array".into(),
                arguments: vec![int],
            },
        )),
        "std/text/grapheme::at" => Some((
            vec![int, string.clone()],
            CoreType::Named {
                name: "std/prelude::Maybe".into(),
                arguments: vec![string],
            },
        )),
        "std/text/grapheme::slice" => Some((
            vec![int.clone(), int, string.clone()],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![
                    CoreType::Named {
                        name: "std/text/grapheme::GraphemeSliceError".into(),
                        arguments: vec![],
                    },
                    string,
                ],
            },
        )),
        "std/text/unicode::version" => Some((
            vec![CoreType::Named {
                name: "std/prelude::Unit".into(),
                arguments: vec![],
            }],
            string,
        )),
        "std/text/unicode::normalize" => Some((vec![form, string.clone()], string)),
        "std/text/unicode::isNormalized" => Some((vec![form, string], boolean)),
        "std/text/unicode::generalCategory" => Some((
            vec![scalar],
            CoreType::Named {
                name: "std/text/unicode::UnicodeGeneralCategory".into(),
                arguments: vec![],
            },
        )),
        "std/text/unicode::simpleCaseFold" => Some((vec![scalar.clone()], scalar)),
        "std/text/unicode::fullCaseFold" => Some((vec![string.clone()], string)),
        "std/text/unicode::isAlphabetic"
        | "std/text/unicode::isWhitespace"
        | "std/text/unicode::isDecimalDigit"
        | "std/text/unicode::isMark" => Some((vec![scalar], boolean)),
        "std/text::encodeUtf8" => Some((vec![string], bytes)),
        "std/text::decodeUtf8" => Some((
            vec![bytes],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![
                    CoreType::Named {
                        name: "std/text::Utf8DecodeError".into(),
                        arguments: vec![],
                    },
                    string,
                ],
            },
        )),
        "std/text::decodeUtf8Lossy" => Some((vec![bytes], string)),
        "std/text::isEmpty" => Some((vec![string], boolean)),
        "std/text::concat" => Some((vec![strings], string)),
        "std/text::lengthScalars" | "std/text::lengthBytes" => Some((vec![string], int)),
        "std/text::scalarAt" => Some((
            vec![int, string],
            CoreType::Named {
                name: "std/prelude::Maybe".into(),
                arguments: vec![CoreType::Named {
                    name: "std/prelude::Char".into(),
                    arguments: vec![],
                }],
            },
        )),
        "std/text::sliceScalars" => Some((
            vec![int.clone(), int, string.clone()],
            CoreType::Named {
                name: "std/prelude::Either".into(),
                arguments: vec![
                    CoreType::Named {
                        name: "std/text::TextSliceError".into(),
                        arguments: vec![],
                    },
                    string,
                ],
            },
        )),
        "std/text::lines" | "std/text::words" => Some((vec![string], strings)),
        "std/text::join" => Some((vec![string.clone(), strings], string)),
        "std/text::split" => Some((vec![string.clone(), string], strings)),
        "std/text::trim"
        | "std/text::trimStart"
        | "std/text::trimEnd"
        | "std/text::toLower"
        | "std/text::toUpper"
        | "std/text::caseFold" => Some((vec![string.clone()], string)),
        "std/text::contains" | "std/text::startsWith" | "std/text::endsWith" => {
            Some((vec![string.clone(), string], boolean))
        }
        "std/text::replace" | "std/text::replaceAll" => Some((vec![string.clone(); 3], string)),
        _ => None,
    }
}

fn remaining_type(parameters: &[CoreType], result: &CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result.clone(), |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

impl Backend<'_> {
    fn text_target(&self, name: &str) -> Result<()> {
        // `std/text` operations on String values are pure and match TS in
        // program WASM. Bytes, the Unicode tables and the standard error ADT
        // of `sliceScalars` need their own proof.
        if self.program_wasm_library
            && (name.starts_with("std/text/unicode::")
                || matches!(
                    name,
                    "std/text::encodeUtf8"
                        | "std/text::decodeUtf8"
                        | "std/text::decodeUtf8Lossy"
                        | "std/text::sliceScalars"
                ))
        {
            return Err(
                "program WASM: native text extension requires a target-specific proof".into(),
            );
        }
        Ok(())
    }

    fn text_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity).map(|i| format!("\0text_argument_{i}")).collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!("native_text_operation({}, vec![{arguments}])", quoted(name));
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    fn text_type_matches(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical text argument/result type".into());
        }
        Ok(())
    }

    pub(super) fn text_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameters, result)) = text_signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        self.text_target(name)?;
        if !evidence.is_empty() {
            return Err("Rust backend: text operation with unsupported evidence".into());
        }
        self.text_type_matches(type_ref, &remaining_type(&parameters, &result), scope)?;
        self.capabilities.insert("text".into());
        Ok(Some(Self::text_callable(name, parameters.len())))
    }

    pub(super) fn text_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
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
        let Some((parameters, result)) = text_signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        self.text_target(callee)?;
        if arguments.len() > parameters.len()
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven text source application".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing text argument type")?;
            self.text_type_matches(&actual, expected, scope)?;
        }
        self.text_type_matches(
            type_ref,
            &remaining_type(&parameters[arguments.len()..], &result),
            scope,
        )?;
        self.capabilities.insert("text".into());
        if arguments.len() == parameters.len() {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_text_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::text_callable(callee, parameters.len())
        );
        for argument in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(Some(output))
    }
}
