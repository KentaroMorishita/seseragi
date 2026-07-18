use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    /// Number of `Type` arguments accepted by this parameter. Zero denotes a
    /// regular value type parameter (`A`); one denotes `F<_>`.
    pub arity: u32,
}

impl TypeParameter {
    pub fn value(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arity: 0,
        }
    }

    pub fn constructor(name: impl Into<String>, arity: u32) -> Self {
        debug_assert!(arity > 0);
        Self {
            name: name.into(),
            arity,
        }
    }

    pub fn is_constructor(&self) -> bool {
        self.arity > 0
    }
}

impl PartialEq<String> for TypeParameter {
    fn eq(&self, other: &String) -> bool {
        self.arity == 0 && self.name == *other
    }
}

impl PartialEq<&str> for TypeParameter {
    fn eq(&self, other: &&str) -> bool {
        self.arity == 0 && self.name == *other
    }
}

// Keep existing schema-1 artifacts stable for ordinary type parameters while
// giving higher-kinded parameters a structural representation.
impl Serialize for TypeParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.arity == 0 {
            serializer.serialize_str(&self.name)
        } else {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Constructor<'a> {
                name: &'a str,
                arity: u32,
            }

            Constructor {
                name: &self.name,
                arity: self.arity,
            }
            .serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for TypeParameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Value(String),
            Constructor { name: String, arity: u32 },
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Value(name) => Self::value(name),
            Repr::Constructor { name, arity } => Self { name, arity },
        })
    }
}

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
    pub name_span: ByteSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias_span: Option<ByteSpan>,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<SurfaceExpr>,
        span: ByteSpan,
    },
    EffectFn {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        parameters: Vec<SurfaceParameter>,
        #[serde(default, skip_serializing_if = "is_false")]
        inferred_contract: bool,
        return_type: Option<TypeRef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        requirements: Vec<SurfaceRequirement>,
        #[serde(skip_serializing_if = "Option::is_none")]
        failure: Option<TypeRef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<SurfaceExpr>,
        span: ByteSpan,
    },
    Fn {
        visibility: Visibility,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        parameters: Vec<SurfaceParameter>,
        return_type: TypeRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<SurfaceExpr>,
        span: ByteSpan,
    },
    Newtype {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "is_false")]
        opaque: bool,
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
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
        type_parameters: Vec<TypeParameter>,
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
        type_parameters: Vec<TypeParameter>,
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
        type_parameters: Vec<TypeParameter>,
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
        type_parameters: Vec<TypeParameter>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        methods: Vec<SurfaceMethod>,
        span: ByteSpan,
    },
    Operator {
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        fixity: String,
        precedence: u32,
        spelling: String,
        parameters: Vec<SurfaceParameter>,
        return_type: TypeRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        span: ByteSpan,
    },
    Impl {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        target: TypeRef,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        members: Vec<SurfaceImplMember>,
        span: ByteSpan,
    },
    Instance {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_parameters: Vec<TypeParameter>,
        trait_name: String,
        trait_name_span: ByteSpan,
        arguments: Vec<TypeRef>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        constraints: Vec<SurfaceConstraint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        methods: Vec<SurfaceMethod>,
        span: ByteSpan,
    },
}

impl SurfaceDecl {
    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Let { span, .. }
            | Self::EffectFn { span, .. }
            | Self::Fn { span, .. }
            | Self::Newtype { span, .. }
            | Self::Alias { span, .. }
            | Self::Type { span, .. }
            | Self::Struct { span, .. }
            | Self::Trait { span, .. }
            | Self::Operator { span, .. }
            | Self::Impl { span, .. }
            | Self::Instance { span, .. } => *span,
        }
    }
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
pub struct SurfaceLambdaParameter {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_ref: Option<TypeRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceMethod {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<SurfaceParameter>,
    pub return_type: TypeRef,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<SurfaceConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<SurfaceExpr>,
    pub span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceImplMember {
    Method {
        visibility: Visibility,
        #[serde(flatten)]
        method: SurfaceMethod,
    },
    Operator {
        visibility: Visibility,
        spelling: String,
        spelling_span: ByteSpan,
        self_span: ByteSpan,
        parameters: Vec<SurfaceParameter>,
        return_type: TypeRef,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<SurfaceExpr>,
        span: ByteSpan,
    },
}

impl SurfaceImplMember {
    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Method { method, .. } => method.span,
            Self::Operator { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceConstraint {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<TypeRef>,
    pub span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceRequirement {
    Shorthand {
        name: String,
        span: ByteSpan,
    },
    Field {
        name: String,
        name_span: ByteSpan,
        #[serde(rename = "type")]
        type_ref: TypeRef,
        span: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceExpr {
    Unit {
        span: ByteSpan,
    },
    Integer {
        raw: String,
        span: ByteSpan,
    },
    String {
        raw: String,
        span: ByteSpan,
    },
    Template {
        parts: Vec<SurfaceTemplatePart>,
        span: ByteSpan,
    },
    Boolean {
        value: bool,
        span: ByteSpan,
    },
    Name {
        name: String,
        span: ByteSpan,
    },
    Member {
        receiver: Box<SurfaceExpr>,
        field: String,
        field_span: ByteSpan,
        span: ByteSpan,
    },
    Application {
        function: Box<SurfaceExpr>,
        argument: Box<SurfaceExpr>,
        span: ByteSpan,
    },
    Lambda {
        parameter: SurfaceLambdaParameter,
        body: Box<SurfaceExpr>,
        span: ByteSpan,
    },
    Tuple {
        elements: Vec<SurfaceExpr>,
        span: ByteSpan,
    },
    Array {
        elements: Vec<SurfaceExpr>,
        span: ByteSpan,
    },
    List {
        elements: Vec<SurfaceExpr>,
        span: ByteSpan,
    },
    Record {
        items: Vec<SurfaceRecordItem>,
        span: ByteSpan,
    },
    Struct {
        name: String,
        name_span: ByteSpan,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        type_arguments: Option<Vec<TypeRef>>,
        items: Vec<SurfaceRecordItem>,
        span: ByteSpan,
    },
    ArrayComprehension {
        element: Box<SurfaceExpr>,
        clauses: Vec<SurfaceComprehensionClause>,
        span: ByteSpan,
    },
    ListComprehension {
        element: Box<SurfaceExpr>,
        clauses: Vec<SurfaceComprehensionClause>,
        span: ByteSpan,
    },
    Binary {
        operator: String,
        operator_span: ByteSpan,
        left: Box<SurfaceExpr>,
        right: Box<SurfaceExpr>,
        span: ByteSpan,
    },
    If {
        condition: Box<SurfaceExpr>,
        then_branch: Box<SurfaceExpr>,
        else_branch: Box<SurfaceExpr>,
        span: ByteSpan,
    },
    Match {
        scrutinee: Box<SurfaceExpr>,
        arms: Vec<SurfaceMatchArm>,
        span: ByteSpan,
    },
    Do {
        items: Vec<SurfaceDoItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Box<SurfaceExpr>>,
        span: ByteSpan,
    },
    Grouped {
        value: Box<SurfaceExpr>,
        span: ByteSpan,
    },
    Error {
        span: ByteSpan,
    },
}

impl SurfaceExpr {
    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Unit { span }
            | Self::Integer { span, .. }
            | Self::String { span, .. }
            | Self::Template { span, .. }
            | Self::Boolean { span, .. }
            | Self::Name { span, .. }
            | Self::Member { span, .. }
            | Self::Application { span, .. }
            | Self::Lambda { span, .. }
            | Self::Tuple { span, .. }
            | Self::Array { span, .. }
            | Self::List { span, .. }
            | Self::Record { span, .. }
            | Self::Struct { span, .. }
            | Self::ArrayComprehension { span, .. }
            | Self::ListComprehension { span, .. }
            | Self::Binary { span, .. }
            | Self::If { span, .. }
            | Self::Match { span, .. }
            | Self::Do { span, .. }
            | Self::Grouped { span, .. }
            | Self::Error { span } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceRecordItem {
    Field {
        name: String,
        name_span: ByteSpan,
        value: SurfaceExpr,
        span: ByteSpan,
    },
    Spread {
        value: SurfaceExpr,
        span: ByteSpan,
    },
}

impl SurfaceRecordItem {
    pub fn value(&self) -> &SurfaceExpr {
        match self {
            Self::Field { value, .. } | Self::Spread { value, .. } => value,
        }
    }

    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Field { span, .. } | Self::Spread { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceTemplatePart {
    Text {
        value: String,
        span: ByteSpan,
    },
    Interpolation {
        value: Box<SurfaceExpr>,
        span: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceComprehensionClause {
    Generator {
        pattern: SurfacePattern,
        source: SurfaceExpr,
        span: ByteSpan,
    },
    Guard {
        condition: SurfaceExpr,
        span: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceMatchArm {
    pub pattern: SurfacePattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<SurfaceExpr>,
    pub body: SurfaceExpr,
    pub span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceRecordPatternField {
    pub name: String,
    pub name_span: ByteSpan,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    pub pattern: SurfacePattern,
    pub span: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfacePattern {
    Integer {
        raw: String,
        span: ByteSpan,
    },
    String {
        raw: String,
        span: ByteSpan,
    },
    Boolean {
        value: bool,
        span: ByteSpan,
    },
    Name {
        name: String,
        name_span: ByteSpan,
        span: ByteSpan,
    },
    Wildcard {
        span: ByteSpan,
    },
    Constructor {
        name: String,
        name_span: ByteSpan,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument: Option<Box<SurfacePattern>>,
        span: ByteSpan,
    },
    Tuple {
        elements: Vec<SurfacePattern>,
        span: ByteSpan,
    },
    Record {
        fields: Vec<SurfaceRecordPatternField>,
        span: ByteSpan,
    },
    Struct {
        name: String,
        name_span: ByteSpan,
        fields: Vec<SurfaceRecordPatternField>,
        span: ByteSpan,
    },
    Error {
        span: ByteSpan,
    },
}

impl SurfacePattern {
    pub fn span(&self) -> ByteSpan {
        match self {
            Self::Integer { span, .. }
            | Self::String { span, .. }
            | Self::Boolean { span, .. }
            | Self::Name { span, .. }
            | Self::Wildcard { span }
            | Self::Constructor { span, .. }
            | Self::Tuple { span, .. }
            | Self::Record { span, .. }
            | Self::Struct { span, .. }
            | Self::Error { span } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SurfaceDoItem {
    Bind {
        pattern: SurfacePattern,
        value: SurfaceExpr,
        span: ByteSpan,
    },
    Let {
        pattern: SurfacePattern,
        value: SurfaceExpr,
        span: ByteSpan,
    },
    Expression {
        value: SurfaceExpr,
        span: ByteSpan,
    },
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
    pub span: ByteSpan,
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
    Hole {
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
