#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn from_node(node: tree_sitter::Node<'_>) -> Self {
        Self {
            start: node.start_byte(),
            end: node.end_byte(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub declarations: Vec<Declaration>,
    pub private_declarations: Vec<Declaration>,
    pub deprecations: Vec<DeclarationDeprecation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclarationDeprecation {
    pub declaration: Span,
    pub origin: Span,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Declaration {
    Interface(Interface),
    Function(Function),
    Namespace(Namespace),
    Class(OpaqueDeclaration),
    Enum(OpaqueDeclaration),
    TypeAlias(TypeAlias),
}

impl Declaration {
    pub fn span(&self) -> Span {
        match self {
            Self::Interface(value) => value.span,
            Self::Function(value) => value.span,
            Self::Namespace(value) => value.span,
            Self::Class(value) | Self::Enum(value) => value.span,
            Self::TypeAlias(value) => value.span,
        }
    }

    pub fn original_name(&self) -> &str {
        match self {
            Self::Interface(value) => &value.name,
            Self::Function(value) => &value.original_name,
            Self::Namespace(value) => &value.original_name,
            Self::Class(value) | Self::Enum(value) => &value.name,
            Self::TypeAlias(value) => &value.name,
        }
    }

    pub fn with_export_alias(&self, public_name: String, span: Span) -> Self {
        let mut value = self.clone();
        match &mut value {
            Self::Function(function) => {
                function.public_name = public_name;
                function.name_span = span;
            }
            Self::Namespace(namespace) => {
                namespace.public_name = public_name;
                namespace.name_span = span;
            }
            Self::Interface(interface) => {
                interface.public_name = public_name;
                interface.name_span = span;
            }
            Self::Class(declaration) | Self::Enum(declaration) => {
                declaration.public_name = public_name;
                declaration.name_span = span;
            }
            Self::TypeAlias(alias) => {
                alias.public_name = public_name;
                alias.name_span = span;
            }
        }
        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Interface {
    pub name: String,
    pub public_name: String,
    pub name_span: Span,
    pub span: Span,
    pub type_parameter_relations: Vec<TypeParameterRelation>,
    pub body: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueDeclaration {
    pub name: String,
    pub public_name: String,
    pub name_span: Span,
    pub span: Span,
    pub type_parameters: Vec<String>,
    pub type_parameter_relations: Vec<TypeParameterRelation>,
    pub surface: Option<ClassSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSurface {
    pub constructors: Vec<TypeRef>,
    pub fields: Vec<Property>,
    pub statics: Vec<Property>,
    pub inherited: bool,
    pub constructible: bool,
    pub incomplete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub public_name: String,
    pub name_span: Span,
    pub span: Span,
    pub type_parameters: Vec<String>,
    pub type_parameter_relations: Vec<TypeParameterRelation>,
    pub type_ref: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub original_name: String,
    pub public_name: String,
    pub name_span: Span,
    pub span: Span,
    pub type_parameters: Vec<String>,
    pub type_parameter_relations: Vec<TypeParameterRelation>,
    pub parameters: Vec<Parameter>,
    pub result: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeParameterRelation {
    pub name: String,
    pub constraint: Option<TypeRef>,
    pub default: Option<TypeRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub name_span: Span,
    pub optional: bool,
    pub rest: bool,
    pub type_ref: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Namespace {
    pub original_name: String,
    pub public_name: String,
    pub name_span: Span,
    pub span: Span,
    pub scope: Scope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeRef {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeKind {
    Primitive(String),
    Named(String),
    Generic {
        name: String,
        arguments: Vec<TypeRef>,
    },
    ReadonlyArray(Box<TypeRef>),
    MutableArray(Box<TypeRef>),
    Tuple(Vec<TypeRef>),
    Union(Vec<TypeRef>),
    Intersection(Vec<TypeRef>),
    Keys(Box<TypeRef>),
    Member {
        subject: Box<TypeRef>,
        key: Box<TypeRef>,
    },
    Infer {
        name: String,
        constraint: Option<Box<TypeRef>>,
    },
    Conditional {
        subject: Box<TypeRef>,
        constraint: Box<TypeRef>,
        matched: Box<TypeRef>,
        unmatched: Box<TypeRef>,
    },
    Shape(Vec<Property>),
    Mapped {
        parameter: String,
        domain: Box<TypeRef>,
        value: Box<TypeRef>,
        readonly: Option<bool>,
        optional: Option<bool>,
    },
    Template(Vec<TemplatePart>),
    Function {
        generics: Vec<TypeParameterRelation>,
        parameters: Vec<Parameter>,
        result: Box<TypeRef>,
    },
    Literal(String),
    Unsupported(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Property {
    pub name: String,
    pub readonly: bool,
    pub optional: bool,
    pub type_ref: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Domain(TypeRef),
}
