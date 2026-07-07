use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceModule {
    pub schema: u32,
    pub source: String,
    pub declarations: Vec<SurfaceDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceDecl {
    Let {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        type_ref: Option<TypeRef>,
        span: ByteSpan,
    },
    EffectFn {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        return_type: Option<TypeRef>,
        span: ByteSpan,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Private,
    Public,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TypeRef {
    Named { name: String, span: ByteSpan },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_let_surface_module() {
        let module = SurfaceModule {
            schema: 1,
            source: "main.ssrg".to_owned(),
            declarations: vec![SurfaceDecl::Let {
                visibility: Visibility::Public,
                name: "answer".to_owned(),
                name_span: ByteSpan { start: 8, end: 14 },
                type_ref: Some(TypeRef::Named {
                    name: "Int".to_owned(),
                    span: ByteSpan { start: 16, end: 19 },
                }),
                span: ByteSpan { start: 0, end: 24 },
            }],
        };

        let json = serde_json::to_value(&module).expect("surface module serializes");

        assert_eq!(json["schema"], 1);
        assert_eq!(json["source"], "main.ssrg");
        assert_eq!(json["declarations"][0]["kind"], "let");
        assert_eq!(json["declarations"][0]["visibility"], "public");
        assert_eq!(json["declarations"][0]["typeRef"]["kind"], "named");
    }

    #[test]
    fn constructs_effect_function_surface_decl() {
        let decl = SurfaceDecl::EffectFn {
            visibility: Visibility::Private,
            name: "main".to_owned(),
            name_span: ByteSpan { start: 10, end: 14 },
            return_type: Some(TypeRef::Named {
                name: "Unit".to_owned(),
                span: ByteSpan { start: 18, end: 22 },
            }),
            span: ByteSpan { start: 0, end: 72 },
        };

        let json = serde_json::to_value(&decl).expect("surface decl serializes");

        assert_eq!(json["kind"], "effectFn");
        assert_eq!(json["visibility"], "private");
        assert_eq!(json["returnType"]["name"], "Unit");
    }
}
