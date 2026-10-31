use super::*;
use seseragi_lowering::{CoreInstance, CoreInstanceConstraint, CoreShowPayloadEvidence};

fn method(class: &str) -> Option<&'static str> {
    match class {
        "Eq" => Some("eq"),
        "Ord" => Some("compare"),
        "Hash" => Some("hash"),
        "Show" => Some("show"),
        "Debug" => Some("debug"),
        _ => None,
    }
}

impl Backend<'_> {
    pub(super) fn validate_derived_method(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let class = selected.constraint.name.as_str();
        if dispatch.trait_identity != format!("std/prelude::{class}")
            || method(class) != Some(dispatch.method.as_str())
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: derived method identity mismatch".into());
        }
        let [argument] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: derived method argument arity".into());
        };
        let result = match class {
            "Eq" => "Bool",
            "Ord" => "Ordering",
            "Hash" => "Int",
            _ => "String",
        };
        let mut expected = CoreType::Named {
            name: format!("std/prelude::{result}"),
            arguments: vec![],
        };
        for _ in 0..if matches!(class, "Eq" | "Ord") { 2 } else { 1 } {
            expected = CoreType::Function {
                parameter: Box::new(argument.clone()),
                result: Box::new(expected),
            };
        }
        self.match_local_type(
            &expected,
            actual,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

    pub(super) fn validate_derived_structural_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<bool> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(false);
        };
        let Some((owner, class)) = identity.split_once("::") else {
            return Ok(false);
        };
        if !matches!(
            (owner, class),
            ("std/tuple", "Eq" | "Ord" | "Hash") | ("std/record", "Eq" | "Show" | "Debug")
        ) {
            return Ok(false);
        }
        let [ty] = type_arguments.as_slice() else {
            return Err("Rust backend: structural derived type arity mismatch".into());
        };
        if !lists::standard_trait(selected, class, ty)
            || self.declared_method_type_kind(ty, scope) != Some(0)
            || (class == "Hash"
                && selected.constraint.trait_identity.as_deref() != Some("std/prelude::Hash"))
        {
            return Err("Rust backend: structural derived identity or type mismatch".into());
        }
        let children = match (owner, ty) {
            ("std/tuple", CoreType::Tuple { elements }) if elements.len() >= 2 => elements.clone(),
            (
                "std/record",
                CoreType::Record {
                    closed: true,
                    fields,
                },
            ) => {
                let mut fields = fields.iter().collect::<Vec<_>>();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                if fields.windows(2).any(|p| p[0].name == p[1].name) {
                    return Err("Rust backend: duplicate structural derived field".into());
                }
                fields.iter().map(|f| f.type_ref.clone()).collect()
            }
            _ => return Err("Rust backend: structural derived owner mismatch".into()),
        };
        if children.len() != evidence_arguments.len() {
            return Err("Rust backend: structural derived child arity mismatch".into());
        }
        for (ty, child) in children.iter().zip(evidence_arguments) {
            if !lists::standard_trait(child, class, ty) {
                return Err("Rust backend: structural derived child mismatch".into());
            }
            self.validate_derived_child(child, scope)?;
        }
        Ok(true)
    }

    pub(super) fn derived_structural_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !self.validate_derived_structural_evidence(selected, scope)? {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native structural instances require a target-specific ABI proof"
                    .into(),
            );
        }
        let CoreInstanceEvidence::Standard {
            type_arguments,
            evidence_arguments,
            ..
        } = &selected.evidence
        else {
            unreachable!()
        };
        let fields = if let CoreType::Record { fields, .. } = &type_arguments[0] {
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(|a, b| a.name.cmp(&b.name));
            format!(
                "Some(vec![{}])",
                fields
                    .iter()
                    .map(|f| format!("({}, {})", quoted(&f.name), f.optional))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        } else {
            "None".into()
        };
        let children = self.evidence(evidence_arguments, scope)?;
        Ok(Some(format!(
            "native_derived_structural_dictionary({}, {children}, {fields})",
            quoted(method(&selected.constraint.name).unwrap())
        )))
    }

    pub(super) fn derived_identity(&self, instance: &CoreInstance, trait_id: &str) -> bool {
        let symbol = match &instance.implementation {
            CoreInstanceImplementation::DerivedShow { adt_symbol, .. }
                if matches!(instance.trait_name.as_str(), "Show" | "Debug") =>
            {
                adt_symbol
            }
            CoreInstanceImplementation::DerivedStructural { adt_symbol, .. }
                if matches!(instance.trait_name.as_str(), "Eq" | "Ord" | "Hash") =>
            {
                adt_symbol
            }
            _ => return false,
        };
        if trait_id != format!("std/prelude::{}", instance.trait_name) {
            return false;
        }
        let parameters = self
            .module
            .adts
            .iter()
            .find(|a| &a.symbol == symbol)
            .map(|a| &a.type_parameters)
            .or_else(|| {
                self.module
                    .structs
                    .iter()
                    .find(|s| &s.symbol == symbol)
                    .map(|s| &s.type_parameters)
            });
        let Some(parameters) = parameters else {
            return false;
        };
        let head = if parameters.is_empty() {
            symbol.clone()
        } else {
            format!(
                "{symbol}<{}>",
                (0..parameters.len())
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        instance.type_identity.as_ref() == Some(&head)
            && instance.identity == format!("{}<{head}>", instance.trait_name)
    }

    pub(super) fn derived_dictionary(
        &mut self,
        instance: &CoreInstance,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (symbol, evidence, transparent) = match &instance.implementation {
            CoreInstanceImplementation::DerivedShow {
                adt_symbol,
                payload_evidence,
            } => (adt_symbol, payload_evidence, None),
            CoreInstanceImplementation::DerivedStructural {
                adt_symbol,
                payload_evidence,
                transparent_newtype,
            } => (adt_symbol, payload_evidence, Some(*transparent_newtype)),
            _ => return Ok(None),
        };
        if self.program_wasm_library {
            return Err(
                "program WASM: native derived instances require a target-specific ABI proof".into(),
            );
        }
        let class = instance.trait_name.as_str();
        let operation = method(class).ok_or("Rust backend: unsupported derived trait")?;
        if !self.derived_identity(instance, &format!("std/prelude::{class}"))
            || instance.arguments.len() != 1
        {
            return Err("Rust backend: unproven derived instance identity".into());
        }
        let adt = self
            .module
            .adts
            .iter()
            .find(|a| &a.symbol == symbol)
            .cloned();
        let structure = self
            .module
            .structs
            .iter()
            .find(|s| &s.symbol == symbol)
            .cloned();
        let parameters = adt
            .as_ref()
            .map(|a| &a.type_parameters)
            .or_else(|| structure.as_ref().map(|s| &s.type_parameters))
            .unwrap();
        if &instance.type_parameters != parameters
            || transparent.is_some_and(|flag| flag != adt.as_ref().is_some_and(|a| a.newtype))
            || instance.supertrait_count != usize::from(class == "Ord")
        {
            return Err("Rust backend: derived declaration or arity mismatch".into());
        }
        let expected = CoreType::Named {
            name: symbol.clone(),
            arguments: parameters
                .iter()
                .map(|p| CoreType::Named {
                    name: p.name.clone(),
                    arguments: vec![],
                })
                .collect(),
        };
        if self.declared_method_type_kind(&instance.arguments[0], scope) != Some(0)
            || self.local_canonical_type(&instance.arguments[0], scope)?
                != self.local_canonical_type(&expected, scope)?
        {
            return Err("Rust backend: derived instance head mismatch".into());
        }
        let mut inherited = String::new();
        if class == "Ord" {
            let parent = instance
                .constraints
                .first()
                .ok_or("Rust backend: missing derived Ord parent")?;
            if parent.name != "Eq"
                || parent.trait_identity.as_deref() != Some("std/prelude::Eq")
                || parent.arguments.len() != 1
                || self.local_canonical_type(&parent.arguments[0], scope)?
                    != self.local_canonical_type(&expected, scope)?
            {
                return Err("Rust backend: derived Ord parent mismatch".into());
            }
            inherited = format!("(\"eq\", {}.field(\"eq\")),", id("\0dictionary_0"));
        }
        let left = "\0derived_left";
        let right = "\0derived_right";
        let mut body_scope = scope.clone();
        body_scope.insert(left.into());
        if matches!(class, "Eq" | "Ord") {
            body_scope.insert(right.into());
        }
        let l = id(left);
        let r = id(right);
        let body = if let Some(adt) = adt {
            if adt.variants.is_empty()
                || (adt.newtype && (adt.variants.len() != 1 || adt.variants[0].payload.is_none()))
                || evidence.len() != adt.variants.iter().filter(|v| v.payload.is_some()).count()
            {
                return Err("Rust backend: invalid derived variants or payload count".into());
            }
            let owner = symbol
                .rsplit_once("::")
                .ok_or("Rust backend: missing derived owner")?
                .0;
            let mut names = BTreeSet::new();
            let mut branches = String::new();
            if class == "Eq" {
                branches.push_str(&format!("if !matches!((&{l}, &{r}), (V::Adt(a), V::Adt(b)) if a.get().0 == b.get().0) {{ return V::Bool(false); }}"));
            }
            if class == "Ord" && !adt.newtype {
                let tags = adt
                    .variants
                    .iter()
                    .map(|v| quoted(&v.symbol))
                    .collect::<Vec<_>>()
                    .join(",");
                branches.push_str(&format!("let tags = [{tags}]; let a = tags.iter().position(|t| {l}.tagged(t)).expect(\"Rust ABI: derived case\"); let b = tags.iter().position(|t| {r}.tagged(t)).expect(\"Rust ABI: derived case\"); if a != b {{ return V::adt(if a < b {{ \"std/prelude::Less\" }} else {{ \"std/prelude::Greater\" }}, V::Unit); }}"));
            }
            for (index, variant) in adt.variants.iter().enumerate() {
                if !names.insert(&variant.name)
                    || variant.symbol != format!("{owner}::{}", variant.name)
                {
                    return Err("Rust backend: invalid derived variant owner or tag".into());
                }
                let members = if let Some(ty) = &variant.payload {
                    vec![(
                        self.derived_child(class, ty, &variant.symbol, evidence, &body_scope)?,
                        format!("{l}.payload()"),
                        format!("{r}.payload()"),
                        String::new(),
                    )]
                } else {
                    vec![]
                };
                let prefix = if variant.payload.is_some() {
                    format!("{} ", variant.name)
                } else {
                    variant.name.clone()
                };
                let rendered = Self::derived_members(
                    operation,
                    &members,
                    if adt.newtype {
                        None
                    } else {
                        Some(index as u32)
                    },
                    &prefix,
                    "",
                );
                branches.push_str(&format!(
                    "if {l}.tagged({}) {{ return {{ {rendered} }}; }}",
                    quoted(&variant.symbol)
                ));
            }
            branches.push_str("panic!(\"Rust ABI: invalid derived value\")");
            branches
        } else {
            let structure = structure.unwrap();
            if evidence.len() != structure.fields.len() {
                return Err("Rust backend: derived field count mismatch".into());
            }
            let mut names = BTreeSet::new();
            let mut members = Vec::new();
            for field in &structure.fields {
                if !names.insert(&field.name) {
                    return Err("Rust backend: duplicate derived field".into());
                }
                members.push((
                    self.derived_child(class, &field.type_ref, &field.name, evidence, &body_scope)?,
                    format!("{l}.field({})", quoted(&field.name)),
                    format!("{r}.field({})", quoted(&field.name)),
                    format!("{}: ", field.name),
                ));
            }
            let name = symbol.rsplit_once("::").unwrap().1;
            let prefix = if members.is_empty() {
                format!("{name} {{}}")
            } else {
                format!("{name} {{ ")
            };
            Self::derived_members(
                operation,
                &members,
                Some(0),
                &prefix,
                if members.is_empty() { "" } else { " }" },
            )
        };
        let value = if matches!(class, "Eq" | "Ord") {
            let mut inner_scope = scope.clone();
            inner_scope.insert(left.into());
            Self::closure(left, &Self::closure(right, &body, &inner_scope), scope)
        } else if matches!(class, "Show" | "Debug") {
            Self::closure(left, &format!("native_display(|| {{ {body} }})"), scope)
        } else {
            Self::closure(left, &body, scope)
        };
        Ok(Some(format!(
            "V::dict(vec![{inherited}({}, {value})])",
            quoted(operation)
        )))
    }

    fn derived_child(
        &mut self,
        class: &str,
        ty: &CoreType,
        key: &str,
        payloads: &[CoreShowPayloadEvidence],
        scope: &Scope,
    ) -> Result<String> {
        if self.declared_method_type_kind(ty, scope) != Some(0) {
            return Err(format!(
                "Rust backend: derived {class} member {key} has unsupported native type {ty:?}"
            )
            .into());
        }
        let mut matches = payloads.iter().filter(|p| p.variant_symbol == key);
        let payload = matches
            .next()
            .ok_or("Rust backend: missing derived member evidence")?;
        if matches.next().is_some() {
            return Err("Rust backend: duplicate derived member evidence".into());
        }
        let selected = CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: class.into(),
                trait_identity: Some(format!("std/prelude::{class}")),
                arguments: vec![ty.clone()],
            },
            evidence: payload.evidence.clone(),
        };
        self.validate_derived_child(&selected, scope)?;
        Ok(format!(
            "({}).remove(0)",
            self.evidence(&[selected], scope)?
        ))
    }

    fn validate_derived_child(&self, child: &CoreCallEvidence, scope: &Scope) -> Result<()> {
        match child.constraint.name.as_str() {
            "Eq" => self.validate_eq_evidence(child, scope),
            "Hash" => self.validate_hash_evidence(child, scope),
            "Ord" => self.validate_ord_evidence(child, scope),
            "Show" | "Debug" => self.validate_display_child(child, &child.constraint.name, scope),
            _ => Err("Rust backend: unsupported derived child".into()),
        }
    }

    fn derived_members(
        operation: &str,
        members: &[(String, String, String, String)],
        seed: Option<u32>,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let mut body = match operation {
            "hash" => seed
                .map(|n| format!("let mut state = {n}u32;"))
                .unwrap_or_default(),
            "show" | "debug" => format!("let mut text = String::from({});", quoted(prefix)),
            _ => String::new(),
        };
        for (index, (dictionary, left, right, label)) in members.iter().enumerate() {
            let call = format!("{dictionary}.field({}).apply({left})", quoted(operation));
            match operation {
                "eq" => body.push_str(&format!("if !{call}.apply({right}).boolean() {{ return V::Bool(false); }}")),
                "compare" => body.push_str(&format!("let order = {call}.apply({right}); if !order.tagged(\"std/prelude::Equal\") {{ return order; }}")),
                "hash" if seed.is_none() => body.push_str(&format!("return {call};")),
                "hash" => body.push_str(&format!("let V::Int(hash) = {call} else {{ panic!(\"Rust ABI: Hash result\") }}; state = (state ^ hash as u32).wrapping_mul(16777619);")),
                _ => {
                    if index > 0 { body.push_str("text.push_str(\", \");"); }
                    body.push_str(&format!("text.push_str({}); let V::Text(child) = {call} else {{ panic!(\"Rust ABI: display result\") }}; text.push_str(child.get());", quoted(label)));
                }
            }
        }
        body.push_str(&match operation {
            "eq" => "V::Bool(true)".into(),
            "compare" => "V::adt(\"std/prelude::Equal\", V::Unit)".into(),
            "hash" if seed.is_none() => "unreachable!()".into(),
            "hash" => "V::Int(state as i32 as i64)".into(),
            _ => format!("text.push_str({}); V::text(&text)", quoted(suffix)),
        });
        body
    }
}
