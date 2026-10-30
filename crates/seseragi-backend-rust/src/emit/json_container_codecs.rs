use super::*;

impl Backend<'_> {
    pub(super) fn json_container_codec_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        owner: &str,
        trait_name: &str,
        scope: &Scope,
    ) -> Result<String> {
        let CoreInstanceEvidence::Standard {
            type_arguments,
            evidence_arguments,
            ..
        } = &selected.evidence
        else {
            unreachable!()
        };
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: JSON container constraint arity".into());
        };
        if !lists::standard_trait(selected, trait_name, actual)
            || self.declared_method_type_kind(actual, scope) != Some(0)
        {
            return Err("Rust backend: unproven JSON container trait or kind".into());
        }
        let types = type_arguments
            .iter()
            .map(|ty| {
                if self.declared_method_type_kind(ty, scope) != Some(0) {
                    return Err("Rust backend: unproven JSON container child kind".into());
                }
                self.local_canonical_type(ty, scope)
            })
            .collect::<Result<Vec<_>>>()?;
        let encode = trait_name == "JsonEncode";
        let named = |name: &str, arguments| CoreType::Named {
            name: name.into(),
            arguments,
        };
        let (kind, expected, children) = match (owner, types.as_slice()) {
            ("std/array", [a]) => (
                "Array",
                named("std/prelude::Array", vec![a.clone()]),
                vec![a.clone()],
            ),
            ("std/list", [a]) => (
                "List",
                named("std/prelude::List", vec![a.clone()]),
                vec![a.clone()],
            ),
            ("std/maybe", [a]) => (
                "Maybe",
                named("std/prelude::Maybe", vec![a.clone()]),
                vec![a.clone()],
            ),
            ("std/either", [e, a]) => (
                "Either",
                named("std/prelude::Either", vec![e.clone(), a.clone()]),
                vec![e.clone(), a.clone()],
            ),
            ("std/map", [k, v]) => (
                "Map",
                named("std/map::Map", vec![k.clone(), v.clone()]),
                vec![k.clone(), v.clone()],
            ),
            ("std/set", [a]) => (
                "Set",
                named("std/set::Set", vec![a.clone()]),
                vec![a.clone()],
            ),
            ("std/tuple", [ty @ CoreType::Tuple { elements }]) if elements.len() >= 2 => {
                ("Tuple", ty.clone(), elements.clone())
            }
            (
                "std/record",
                [ty @ CoreType::Record {
                    closed: true,
                    fields,
                }],
            ) => {
                let mut fields = fields.iter().collect::<Vec<_>>();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                if fields.windows(2).any(|pair| pair[0].name == pair[1].name) {
                    return Err("Rust backend: duplicate JSON record field".into());
                }
                (
                    "Record",
                    ty.clone(),
                    fields.iter().map(|f| f.type_ref.clone()).collect(),
                )
            }
            _ => return Err("Rust backend: JSON container owner or type arity mismatch".into()),
        };
        if self.local_canonical_type(actual, scope)? != expected {
            return Err("Rust backend: JSON container selected type mismatch".into());
        }
        let mut contracts = Vec::new();
        if !encode && matches!(kind, "Map" | "Set") {
            contracts.push(("Eq", children[0].clone()));
            contracts.push(("Hash", children[0].clone()));
        }
        contracts.extend(children.into_iter().map(|ty| (trait_name, ty)));
        if contracts.len() != evidence_arguments.len() {
            return Err("Rust backend: JSON container dictionary child arity mismatch".into());
        }
        for ((name, ty), child) in contracts.iter().zip(evidence_arguments) {
            let [argument] = child.constraint.arguments.as_slice() else {
                return Err("Rust backend: JSON container child constraint arity".into());
            };
            if child.constraint.name != *name
                || child
                    .constraint
                    .trait_identity
                    .as_deref()
                    .is_some_and(|identity| identity != format!("std/prelude::{name}"))
                || self.declared_method_type_kind(argument, scope) != Some(0)
                || self.local_canonical_type(argument, scope)? != *ty
            {
                return Err(
                    "Rust backend: JSON container child owner, kind or type mismatch".into(),
                );
            }
            match *name {
                "Eq" => self.validate_eq_evidence(child, scope)?,
                "Hash" => self.validate_hash_evidence(child, scope)?,
                _ => {}
            }
        }
        let fields = if let CoreType::Record { fields, .. } = &expected {
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(|a, b| a.name.cmp(&b.name));
            fields
                .iter()
                .map(|f| format!("({}, {})", quoted(&f.name), f.optional))
                .collect::<Vec<_>>()
                .join(",")
        } else {
            String::new()
        };
        let children = self.evidence(evidence_arguments, scope)?;
        self.capabilities.insert("json".into());
        match kind {
            "List" => {
                self.capabilities.insert("list".into());
            }
            "Map" => {
                self.capabilities.insert("map".into());
            }
            "Set" => {
                self.capabilities.insert("set".into());
            }
            _ => {}
        }
        Ok(format!(
            "native_json_container_dictionary({}, {encode}, {children}, vec![{fields}])",
            quoted(kind)
        ))
    }
}
