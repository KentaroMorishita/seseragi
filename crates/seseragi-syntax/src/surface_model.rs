use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceModule {
    pub schema: u32,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<SurfaceImport>,
    pub declarations: Vec<SurfaceDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceImport {
    pub specifier: String,
    pub items: Vec<SurfaceImportItem>,
    pub span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceImportItem {
    pub namespace: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
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
    Newtype {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        representation: TypeRef,
        span: ByteSpan,
    },
    Trait {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<String>,
        span: ByteSpan,
    },
    Operator {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        fixity: String,
        precedence: u32,
        spelling: String,
        parameters: Vec<SurfaceParameter>,
        return_type: TypeRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<String>,
        span: ByteSpan,
    },
    Instance {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        trait_name: String,
        arguments: Vec<TypeRef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<String>,
        span: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceParameter {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
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
    Named {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<TypeRef>,
        span: ByteSpan,
    },
}
