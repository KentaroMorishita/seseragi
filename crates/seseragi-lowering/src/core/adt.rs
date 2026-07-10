use crate::{source_span, CoreType, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::TypedAdtVariant;
use seseragi_syntax::{ByteSpan, Visibility};

use super::types::lower_typed_type;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreAdt {
    pub symbol: String,
    pub name: String,
    pub visibility: Visibility,
    pub opaque: bool,
    pub type_parameters: Vec<String>,
    pub variants: Vec<CoreAdtVariant>,
    pub origin: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreAdtVariant {
    pub symbol: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<CoreType>,
    pub origin: SourceSpan,
}

pub(super) struct AdtDeclInput {
    pub(super) symbol: String,
    pub(super) name: String,
    pub(super) visibility: Visibility,
    pub(super) opaque: bool,
    pub(super) type_parameters: Vec<String>,
    pub(super) variants: Vec<TypedAdtVariant>,
    pub(super) origin: ByteSpan,
}

pub(super) fn lower_adt(source: &str, input: AdtDeclInput) -> CoreAdt {
    CoreAdt {
        symbol: input.symbol,
        name: input.name,
        visibility: input.visibility,
        opaque: input.opaque,
        type_parameters: input.type_parameters,
        variants: input
            .variants
            .into_iter()
            .map(|variant| lower_variant(source, variant))
            .collect(),
        origin: source_span(source, input.origin),
    }
}

fn lower_variant(source: &str, variant: TypedAdtVariant) -> CoreAdtVariant {
    CoreAdtVariant {
        symbol: variant.symbol,
        name: variant.name,
        payload: variant.payload.map(lower_typed_type),
        origin: source_span(source, variant.origin),
    }
}

#[cfg(test)]
mod tests {
    use crate::{lower_typed_module, CoreType};
    use seseragi_semantics::type_module;

    #[test]
    fn lowers_public_adt_constructor_contracts_and_payloads() {
        let source = "\
pub type Hand =
  | Rock
  | Paper

pub type Label =
  | Missing
  | Present String
";
        let core = lower_typed_module(type_module("artifact/adts/main.ssrg", source));

        assert_eq!(core.adts.len(), 2);
        let hand = &core.adts[0];
        assert_eq!(hand.symbol, "artifact/adts::Hand");
        assert_eq!(hand.name, "Hand");
        assert_eq!(hand.visibility, seseragi_syntax::Visibility::Public);
        assert!(!hand.opaque);
        assert_eq!(hand.origin.source, "main.ssrg");
        assert_eq!(
            &source[hand.origin.start..hand.origin.end],
            "pub type Hand =\n  | Rock\n  | Paper"
        );

        let present = &core.adts[1].variants[1];
        assert_eq!(present.symbol, "artifact/adts::Present");
        assert_eq!(present.name, "Present");
        assert_eq!(
            present.payload,
            Some(CoreType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            })
        );
        assert_eq!(&source[present.origin.start..present.origin.end], "Present");
    }

    #[test]
    fn freezes_opaque_and_private_constructor_export_boundaries() {
        let source = "\
pub opaque type Token =
  | Token String

type Internal =
  | Hidden
";
        let core = lower_typed_module(type_module("artifact/adts/main.ssrg", source));

        let token = &core.adts[0];
        assert_eq!(token.visibility, seseragi_syntax::Visibility::Public);
        assert!(token.opaque);

        let internal = &core.adts[1];
        assert_eq!(internal.visibility, seseragi_syntax::Visibility::Private);
        assert!(!internal.opaque);
    }
}
