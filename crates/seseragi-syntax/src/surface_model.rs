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
    Fn {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        parameters: Vec<SurfaceParameter>,
        return_type: TypeRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<String>,
        span: ByteSpan,
    },
    Newtype {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "is_false")]
        opaque: bool,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deriving: Vec<String>,
        representation: TypeRef,
        span: ByteSpan,
    },
    Alias {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        target: TypeRef,
        span: ByteSpan,
    },
    Type {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "is_false")]
        opaque: bool,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deriving: Vec<String>,
        variants: Vec<SurfaceVariant>,
        span: ByteSpan,
    },
    Struct {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "is_false")]
        opaque: bool,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        deriving: Vec<String>,
        fields: Vec<SurfaceField>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceField {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceVariant {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypeRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeRecordField {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
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
    Record {
        closed: bool,
        fields: Vec<TypeRecordField>,
        span: ByteSpan,
    },
    Tuple {
        elements: Vec<TypeRef>,
        span: ByteSpan,
    },
    Function {
        parameter: Box<TypeRef>,
        result: Box<TypeRef>,
        span: ByteSpan,
    },
}

fn is_false(value: &bool) -> bool {
    !*value
}
