use super::*;

impl Backend<'_> {
    pub(super) fn json_derived_identity(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        trait_identity: &str,
    ) -> bool {
        if !matches!(
            trait_identity,
            "std/prelude::JsonEncode" | "std/prelude::JsonDecode"
        ) || trait_identity != format!("std/prelude::{}", instance.trait_name)
        {
            return false;
        }
        let CoreInstanceImplementation::DerivedJson {
            adt_symbol,
            transparent_newtype,
            ..
        } = &instance.implementation
        else {
            return false;
        };
        let declaration = if *transparent_newtype {
            self.module
                .adts
                .iter()
                .find(|a| a.symbol == *adt_symbol && a.newtype)
                .map(|a| (&a.symbol, &a.type_parameters))
        } else {
            self.module
                .structs
                .iter()
                .find(|s| s.symbol == *adt_symbol)
                .map(|s| (&s.symbol, &s.type_parameters))
        };
        let Some((symbol, parameters)) = declaration else {
            return false;
        };
        let head = if parameters.is_empty() {
            symbol.clone()
        } else {
            format!(
                "{}<{}>",
                symbol,
                (0..parameters.len())
                    .map(|n| format!("${n}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        // Derived identities use the checked trait label plus canonical head,
        // unlike user instances, whose identity includes the trait's owner.
        instance.type_identity.as_deref() == Some(head.as_str())
            && instance.identity == format!("{}<{head}>", instance.trait_name)
    }

    pub(super) fn json_derived_struct_dictionary(
        &mut self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceImplementation::DerivedJson {
            adt_symbol,
            payload_evidence,
            transparent_newtype,
        } = &instance.implementation
        else {
            return Ok(None);
        };
        self.json_codec_target()?;
        let structure = self
            .module
            .structs
            .iter()
            .find(|s| s.symbol == *adt_symbol)
            .ok_or("Rust backend: derived JSON requires a supported named struct")?
            .clone();
        let method = match instance.trait_name.as_str() {
            "JsonEncode" => "encodeJson",
            "JsonDecode" => "decodeJson",
            _ => return Err("Rust backend: unproven derived JSON trait".into()),
        };
        if *transparent_newtype
            || !self
                .json_derived_identity(instance, &format!("std/prelude::{}", instance.trait_name))
            || instance.supertrait_count != 0
            || instance.type_parameters.len() != structure.type_parameters.len()
            || instance
                .type_parameters
                .iter()
                .zip(&structure.type_parameters)
                .any(|(a, b)| a.name != b.name || a.arity != b.arity)
            || instance.arguments.len() != 1
            || payload_evidence.len() != structure.fields.len()
        {
            return Err("Rust backend: unproven derived JSON struct owner or arity".into());
        }
        let expected = CoreType::Named {
            name: structure.symbol.clone(),
            arguments: structure
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
            return Err("Rust backend: derived JSON struct head mismatch".into());
        }
        let mut names = BTreeSet::new();
        let mut children = Vec::new();
        for field in &structure.fields {
            if !names.insert(&field.name)
                || self.declared_method_type_kind(&field.type_ref, scope) != Some(0)
            {
                return Err("Rust backend: unproven derived JSON struct field".into());
            }
            let mut selected = payload_evidence
                .iter()
                .filter(|p| p.variant_symbol == field.name);
            let payload = selected
                .next()
                .ok_or("Rust backend: missing derived JSON field evidence")?;
            if selected.next().is_some() {
                return Err("Rust backend: duplicate derived JSON field evidence".into());
            }
            children.push(CoreCallEvidence {
                constraint: seseragi_lowering::CoreInstanceConstraint {
                    name: instance.trait_name.clone(),
                    trait_identity: Some(format!("std/prelude::{}", instance.trait_name)),
                    arguments: vec![field.type_ref.clone()],
                },
                evidence: payload.evidence.clone(),
            });
        }
        let fields = structure
            .fields
            .iter()
            .map(|f| format!("({}, false)", quoted(&f.name)))
            .collect::<Vec<_>>()
            .join(",");
        let children = self.evidence(&children, scope)?;
        let parameter = "\0json_derived_value";
        // Materialize child dictionaries inside the traced method. A recursive
        // struct can refer to its own factory without expanding it at creation.
        let body = format!(
            "native_json_container_dictionary(\"Record\", {}, {children}, vec![{fields}]).field({}).apply({})",
            instance.trait_name == "JsonEncode", quoted(method), id(parameter)
        );
        let value = Self::closure(parameter, &body, scope);
        self.capabilities.insert("json".into());
        Ok(Some(format!(
            "V::dict(vec![({}, {value})])",
            quoted(method)
        )))
    }
}
