use super::*;
use seseragi_lowering::{CoreComprehensionClause, CorePattern};

fn pattern_type(pattern: &CorePattern) -> Result<&CoreType> {
    Ok(match pattern {
        CorePattern::Integer { type_ref, .. }
        | CorePattern::Char { type_ref, .. }
        | CorePattern::String { type_ref, .. }
        | CorePattern::Boolean { type_ref, .. }
        | CorePattern::Binding { type_ref, .. }
        | CorePattern::Wildcard { type_ref, .. }
        | CorePattern::Constructor { type_ref, .. }
        | CorePattern::Tuple { type_ref, .. }
        | CorePattern::Array { type_ref, .. }
        | CorePattern::List { type_ref, .. }
        | CorePattern::Record { type_ref, .. } => type_ref,
        CorePattern::Invalid { .. } => {
            return Err("Rust backend: invalid comprehension pattern".into())
        }
    })
}

impl Backend<'_> {
    pub(super) fn comprehension(
        &mut self,
        element: &CoreExpr,
        clauses: &[CoreComprehensionClause],
        type_ref: &CoreType,
        list: bool,
        scope: &Scope,
    ) -> Result<String> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native comprehension needs a target-specific ABI proof".into(),
            );
        }
        if !matches!(
            clauses.first(),
            Some(CoreComprehensionClause::Generator { .. })
        ) {
            return Err("Rust backend: comprehension requires an initial generator".into());
        }
        let element_type = local::method_argument_type(element)
            .ok_or("Rust backend: missing comprehension element type")?;
        let expected = CoreType::Named {
            name: if list {
                "std/prelude::List"
            } else {
                "std/prelude::Array"
            }
            .into(),
            arguments: vec![self.local_canonical_type(&element_type, scope)?],
        };
        if self.declared_method_type_kind(type_ref, scope) != Some(0)
            || self.local_canonical_type(type_ref, scope)? != expected
        {
            return Err("Rust backend: comprehension result type mismatch".into());
        }
        self.capabilities.insert("iterable".into());
        if list {
            self.capabilities.insert("list".into());
        }
        let output = self.fresh();
        let body = self.comprehension_clauses(element, clauses, &output, scope)?;
        Ok(format!(
            "{{ let mut {output} = Vec::new(); {body} native_sequence_collect({output}, {list}) }}"
        ))
    }

    fn comprehension_clauses(
        &mut self,
        element: &CoreExpr,
        clauses: &[CoreComprehensionClause],
        output: &str,
        scope: &Scope,
    ) -> Result<String> {
        let Some((clause, rest)) = clauses.split_first() else {
            return Ok(format!("{output}.push({});", self.expr(element, scope)?));
        };
        match clause {
            CoreComprehensionClause::Guard { condition, .. } => {
                let ty = local::method_argument_type(condition)
                    .ok_or("Rust backend: missing comprehension guard type")?;
                if self.local_canonical_type(&ty, scope)?
                    != (CoreType::Named {
                        name: "std/prelude::Bool".into(),
                        arguments: vec![],
                    })
                {
                    return Err("Rust backend: comprehension guard requires Bool".into());
                }
                Ok(format!(
                    "if ({}).boolean() {{ {} }}",
                    self.expr(condition, scope)?,
                    self.comprehension_clauses(element, rest, output, scope)?
                ))
            }
            CoreComprehensionClause::Generator {
                pattern,
                source,
                evidence,
                ..
            } => {
                self.validate_iterable_evidence(evidence, scope)?;
                let source_type = local::method_argument_type(source)
                    .ok_or("Rust backend: missing comprehension source type")?;
                if self.local_canonical_type(&source_type, scope)?
                    != self.local_canonical_type(&evidence.constraint.arguments[0], scope)?
                    || self.local_canonical_type(pattern_type(pattern)?, scope)?
                        != self.local_canonical_type(&evidence.constraint.arguments[1], scope)?
                {
                    return Err(
                        "Rust backend: comprehension Iterable source/element mismatch".into(),
                    );
                }
                let dictionary = self.evidence(std::slice::from_ref(evidence), scope)?;
                let source = self.expr(source, scope)?;
                let iterator = self.fresh();
                let value = self.fresh();
                let next = self.fresh();
                let label = self.fresh();
                let mut inner = scope.clone();
                let pattern = self.comprehension_pattern(pattern, &value, &label, &mut inner)?;
                let body = self.comprehension_clauses(element, rest, output, &inner)?;
                Ok(format!("{{ let mut {iterator} = ({dictionary}).remove(0).field(\"iterate\").apply({source}); while let Some(({value}, {next})) = native_prelude_pull({iterator}) {{ {iterator} = {next}; '{label}: {{ {pattern} {body} }} }} }}"))
            }
        }
    }

    fn comprehension_pattern(
        &mut self,
        pattern: &CorePattern,
        value: &str,
        label: &str,
        scope: &mut Scope,
    ) -> Result<String> {
        let mut body = String::new();
        let condition = match pattern {
            CorePattern::Binding { name, .. } => {
                body.push_str(&format!("let {} = {value}.clone();", id(name)));
                scope.insert(name.clone());
                None
            }
            CorePattern::Wildcard { .. } => None,
            CorePattern::Integer { value: literal, .. } => Some(format!(
                "matches!({value}, V::Int(n) if n == {}_i64)",
                literal
                    .parse::<i64>()
                    .map_err(|_| "Rust backend: invalid comprehension integer")?
            )),
            CorePattern::Boolean { value: literal, .. } => {
                Some(format!("matches!({value}, V::Bool(n) if n == {literal})"))
            }
            CorePattern::String { value: literal, .. }
            | CorePattern::Char { value: literal, .. } => Some(format!(
                "native_textual_literal_matches(&{value}, {})",
                quoted(literal)
            )),
            CorePattern::Constructor {
                symbol, argument, ..
            } => {
                body.push_str(&format!(
                    "if !{value}.tagged({}) {{ break '{label}; }}",
                    quoted(symbol)
                ));
                if let Some(argument) = argument {
                    let payload = self.fresh();
                    body.push_str(&format!("let {payload} = {value}.payload();"));
                    body.push_str(&self.comprehension_pattern(argument, &payload, label, scope)?);
                }
                None
            }
            CorePattern::Tuple { elements, .. } => {
                for (index, pattern) in elements.iter().enumerate() {
                    let item = self.fresh();
                    body.push_str(&format!("let {item} = {value}.index({index});"));
                    body.push_str(&self.comprehension_pattern(pattern, &item, label, scope)?);
                }
                None
            }
            CorePattern::Array { elements, rest, .. } => {
                let values = self.fresh();
                let operator = if rest.is_some() { ">=" } else { "==" };
                body.push_str(&format!("let V::Array({values}) = &{value} else {{ panic!(\"Rust ABI: comprehension Array pattern\") }}; if !({values}.get().len() {operator} {}) {{ break '{label}; }}", elements.len()));
                for (index, pattern) in elements.iter().enumerate() {
                    let item = self.fresh();
                    body.push_str(&format!("let {item} = {values}.get()[{index}].clone();"));
                    body.push_str(&self.comprehension_pattern(pattern, &item, label, scope)?);
                }
                if let Some(rest) = rest {
                    let tail = self.fresh();
                    body.push_str(&format!(
                        "let {tail} = V::Array(Shared::new({values}.get()[{}..].to_vec()));",
                        elements.len()
                    ));
                    body.push_str(&self.comprehension_pattern(rest, &tail, label, scope)?);
                }
                None
            }
            CorePattern::List { elements, rest, .. } => {
                let mut tail = value.to_owned();
                for pattern in elements {
                    let cell = self.fresh();
                    let item = self.fresh();
                    let next = self.fresh();
                    body.push_str(&format!("let Some({cell}) = {tail}.list_cell() else {{ break '{label}; }}; let {item} = {cell}.head.clone(); let {next} = {cell}.tail.clone();"));
                    body.push_str(&self.comprehension_pattern(pattern, &item, label, scope)?);
                    tail = next;
                }
                if let Some(rest) = rest {
                    body.push_str(&self.comprehension_pattern(rest, &tail, label, scope)?);
                } else {
                    body.push_str(&format!(
                        "if {tail}.list_cell().is_some() {{ break '{label}; }}"
                    ));
                }
                None
            }
            CorePattern::Record { fields, .. } => {
                for field in fields {
                    let item = self.fresh();
                    body.push_str(&format!(
                        "let {item} = {value}.field({});",
                        quoted(&field.name)
                    ));
                    body.push_str(&self.comprehension_pattern(
                        &field.pattern,
                        &item,
                        label,
                        scope,
                    )?);
                }
                None
            }
            CorePattern::Invalid { .. } => {
                return Err("Rust backend: invalid comprehension pattern".into())
            }
        };
        if let Some(condition) = condition {
            body.push_str(&format!("if !({condition}) {{ break '{label}; }}"));
        }
        Ok(body)
    }
}
