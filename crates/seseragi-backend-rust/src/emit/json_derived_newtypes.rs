use super::*;

impl Backend<'_> {
    pub(super) fn json_derived_dictionary(
        &mut self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceImplementation::DerivedJson {
            adt_symbol,
            payload_evidence,
            transparent_newtype: true,
        } = &instance.implementation
        else {
            return self.json_derived_struct_dictionary(instance, scope);
        };
        self.json_codec_target()?;
        let adt = self
            .module
            .adts
            .iter()
            .find(|a| a.symbol == *adt_symbol && a.newtype)
            .ok_or("Rust backend: derived JSON requires a checked newtype")?
            .clone();
        let [variant] = adt.variants.as_slice() else {
            return Err("Rust backend: JSON newtype requires one constructor".into());
        };
        let Some(payload) = &variant.payload else {
            return Err("Rust backend: JSON newtype requires a payload".into());
        };
        let [selected] = payload_evidence.as_slice() else {
            return Err("Rust backend: JSON newtype requires one child dictionary".into());
        };
        let method = match instance.trait_name.as_str() {
            "JsonEncode" => "encodeJson",
            "JsonDecode" => "decodeJson",
            _ => return Err("Rust backend: unproven derived JSON newtype trait".into()),
        };
        if !self.json_derived_identity(instance, &format!("std/prelude::{}", instance.trait_name))
            || instance.supertrait_count != 0
            || variant.symbol != adt.symbol
            || variant.name != adt.name
            || selected.variant_symbol != variant.symbol
            || instance.arguments.len() != 1
            || instance.type_parameters.len() != adt.type_parameters.len()
            || instance
                .type_parameters
                .iter()
                .zip(&adt.type_parameters)
                .any(|(a, b)| a.name != b.name || a.arity != b.arity)
            || self.declared_method_type_kind(payload, scope) != Some(0)
        {
            return Err("Rust backend: unproven JSON newtype owner, kind or evidence".into());
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
            return Err("Rust backend: derived JSON newtype head mismatch".into());
        }
        let child = CoreCallEvidence {
            constraint: seseragi_lowering::CoreInstanceConstraint {
                name: instance.trait_name.clone(),
                trait_identity: Some(format!("std/prelude::{}", instance.trait_name)),
                arguments: vec![payload.clone()],
            },
            evidence: selected.evidence.clone(),
        };
        let dictionaries = self.evidence(&[child], scope)?;
        let parameter = "\0json_newtype_value";
        let value = id(parameter);
        let body = if instance.trait_name == "JsonEncode" {
            format!("{{ let _frame = NativeJsonCodecFrame::enter(); assert!({value}.tagged({})); ({dictionaries}).remove(0).field(\"encodeJson\").apply({value}.payload()) }}",quoted(&variant.symbol))
        } else {
            format!("{{ let _frame = NativeJsonCodecFrame::enter(); let decoded=({dictionaries}).remove(0).field(\"decodeJson\").apply({value}); if decoded.tagged(\"std/prelude::Left\") {{ decoded }} else {{ native_right(V::adt({},decoded.payload())) }} }}",quoted(&variant.symbol))
        };
        let function = Self::closure(parameter, &body, scope);
        self.capabilities.insert("json".into());
        Ok(Some(format!(
            "V::dict(vec![({}, {function})])",
            quoted(method)
        )))
    }
}
