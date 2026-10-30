use super::*;

impl Backend<'_> {
    pub(super) fn json_derived_adt_dictionary(
        &mut self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<String> {
        self.json_codec_target()?;
        let CoreInstanceImplementation::DerivedJson {
            adt_symbol,
            payload_evidence,
            transparent_newtype: false,
        } = &instance.implementation
        else {
            return Err("Rust backend: unproven derived JSON ADT".into());
        };
        let adt = self
            .module
            .adts
            .iter()
            .find(|a| a.symbol == *adt_symbol && !a.newtype)
            .ok_or("Rust backend: missing derived JSON ADT")?
            .clone();
        let encode = match instance.trait_name.as_str() {
            "JsonEncode" => true,
            "JsonDecode" => false,
            _ => return Err("Rust backend: unproven derived JSON ADT trait".into()),
        };
        if !self.json_derived_identity(instance, &format!("std/prelude::{}", instance.trait_name))
            || instance.supertrait_count != 0
            || instance.arguments.len() != 1
            || adt.variants.is_empty()
            || instance.type_parameters.len() != adt.type_parameters.len()
            || instance
                .type_parameters
                .iter()
                .zip(&adt.type_parameters)
                .any(|(a, b)| a.name != b.name || a.arity != b.arity)
            || payload_evidence.len() != adt.variants.iter().filter(|v| v.payload.is_some()).count()
        {
            return Err("Rust backend: unproven derived JSON ADT owner or arity".into());
        }
        let expected = CoreType::Named {
            name: adt.symbol.clone(),
            arguments: adt
                .type_parameters
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
            return Err("Rust backend: derived JSON ADT head mismatch".into());
        }
        let owner = adt
            .symbol
            .rsplit_once("::")
            .ok_or("Rust backend: missing JSON ADT owner")?
            .0;
        let mut names = BTreeSet::new();
        let mut cases = Vec::new();
        for variant in &adt.variants {
            if !names.insert(&variant.name)
                || variant.symbol != format!("{owner}::{}", variant.name)
            {
                return Err("Rust backend: invalid derived JSON variant owner or tag".into());
            }
            let thunk = if let Some(payload) = &variant.payload {
                if self.declared_method_type_kind(payload, scope) != Some(0) {
                    return Err("Rust backend: unproven JSON ADT payload kind".into());
                }
                let mut selected = payload_evidence
                    .iter()
                    .filter(|p| p.variant_symbol == variant.symbol);
                let child = selected
                    .next()
                    .ok_or("Rust backend: missing JSON ADT child evidence")?;
                if selected.next().is_some() {
                    return Err("Rust backend: duplicate JSON ADT child evidence".into());
                }
                let selected = CoreCallEvidence {
                    constraint: seseragi_lowering::CoreInstanceConstraint {
                        name: instance.trait_name.clone(),
                        trait_identity: Some(format!("std/prelude::{}", instance.trait_name)),
                        arguments: vec![payload.clone()],
                    },
                    evidence: child.evidence.clone(),
                };
                let children = self.evidence(&[selected], scope)?;
                let thunk = Self::closure(
                    "\0json_adt_dictionary_unit",
                    &format!("({children}).remove(0)"),
                    scope,
                );
                format!("Some({thunk})")
            } else {
                "None".into()
            };
            cases.push(format!(
                "({}, {}, {thunk})",
                quoted(&variant.name),
                quoted(&variant.symbol)
            ));
        }
        self.capabilities.insert("json".into());
        Ok(format!(
            "native_json_derived_adt_dictionary({encode}, vec![{}])",
            cases.join(",")
        ))
    }
}
