use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use seseragi_syntax::{
    InterfaceConstraint, InterfaceScheme, InterfaceType, TypeParameter as InterfaceTypeParameter,
};

use crate::{TypedConstraint, TypedRecordField, TypedScheme, TypedType};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeRenderLayout {
    Compact,
    Multiline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeRenderMarkup {
    Plain,
    Markdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeRenderOptions {
    pub layout: TypeRenderLayout,
    pub markup: TypeRenderMarkup,
    pub indent_width: u32,
}

impl Default for TypeRenderOptions {
    fn default() -> Self {
        Self {
            layout: TypeRenderLayout::Compact,
            markup: TypeRenderMarkup::Plain,
            indent_width: 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypeDocument {
    Named {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        canonical: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<TypeDocument>,
    },
    Variable {
        name: String,
        arity: u32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<TypeDocument>,
    },
    TypeConstructor {
        name: String,
        arity: u32,
    },
    Function {
        parameters: Vec<TypeDocument>,
        result: Box<TypeDocument>,
    },
    Tuple {
        elements: Vec<TypeDocument>,
    },
    Record {
        closed: bool,
        fields: Vec<TypeDocumentField>,
    },
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDocumentField {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_ref: TypeDocument,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeParameterDocument {
    pub name: String,
    pub arity: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeConstraintDocument {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<TypeDocument>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSchemeDocument {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<TypeParameterDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<TypeConstraintDocument>,
    #[serde(rename = "type")]
    pub type_ref: TypeDocument,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeCallableParameterDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_ref: TypeDocument,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeCallableDocument {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<TypeCallableParameterDocument>,
    pub result: TypeDocument,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<TypeParameterDocument>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<TypeConstraintDocument>,
}

impl TypeDocument {
    pub fn from_typed_type(type_ref: &TypedType) -> Self {
        typed_type_document(type_ref, &BTreeMap::new())
    }

    pub fn from_interface_type(type_ref: &InterfaceType) -> Self {
        interface_type_document(type_ref, &BTreeMap::new())
    }

    pub(crate) fn from_typed_type_with_parameters(
        type_ref: &TypedType,
        parameters: &[InterfaceTypeParameter],
    ) -> Self {
        let parameter_arities = parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.arity))
            .collect::<BTreeMap<_, _>>();
        typed_type_document(type_ref, &parameter_arities)
    }

    pub(crate) fn from_interface_type_with_parameters(
        type_ref: &InterfaceType,
        parameters: &[InterfaceTypeParameter],
    ) -> Self {
        let parameter_arities = parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.arity))
            .collect::<BTreeMap<_, _>>();
        interface_type_document(type_ref, &parameter_arities)
    }

    pub fn render(&self, options: TypeRenderOptions) -> String {
        wrap_markup(render_plain(self, options), options.markup, options.layout)
    }
}

impl TypeSchemeDocument {
    pub fn from_typed_scheme(scheme: &TypedScheme) -> Self {
        let parameter_arities = typed_parameter_arities(scheme);
        Self {
            parameters: scheme
                .type_parameters
                .iter()
                .map(|name| TypeParameterDocument {
                    name: name.clone(),
                    arity: parameter_arities.get(name).copied().unwrap_or(0),
                })
                .collect(),
            constraints: scheme
                .constraints
                .iter()
                .map(|constraint| typed_constraint_document(constraint, &parameter_arities))
                .collect(),
            type_ref: typed_type_document(&scheme.type_ref, &parameter_arities),
        }
    }

    pub fn from_interface_scheme(scheme: &InterfaceScheme) -> Self {
        let parameter_arities = scheme
            .type_parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.arity))
            .collect::<BTreeMap<_, _>>();
        Self {
            parameters: scheme
                .type_parameters
                .iter()
                .map(interface_parameter_document)
                .collect(),
            constraints: scheme
                .constraints
                .iter()
                .map(|constraint| interface_constraint_document(constraint, &parameter_arities))
                .collect(),
            type_ref: interface_type_document(&scheme.type_ref, &parameter_arities),
        }
    }

    pub fn render(&self, options: TypeRenderOptions) -> String {
        wrap_markup(
            render_plain_scheme(self, options),
            options.markup,
            options.layout,
        )
    }
}

impl TypeCallableDocument {
    pub fn from_scheme(
        name: impl Into<String>,
        parameter_names: impl IntoIterator<Item = Option<String>>,
        scheme: TypeSchemeDocument,
    ) -> Option<Self> {
        let TypeDocument::Function { parameters, result } = scheme.type_ref else {
            return None;
        };
        let mut parameter_names = parameter_names.into_iter();
        Some(Self {
            name: name.into(),
            parameters: parameters
                .into_iter()
                .map(|type_ref| TypeCallableParameterDocument {
                    name: parameter_names.next().flatten(),
                    type_ref,
                })
                .collect(),
            result: *result,
            type_parameters: scheme.parameters,
            constraints: scheme.constraints,
        })
    }

    pub fn render(&self, options: TypeRenderOptions) -> String {
        wrap_markup(
            render_plain_callable(self, options),
            options.markup,
            options.layout,
        )
    }

    pub fn as_scheme(&self) -> TypeSchemeDocument {
        TypeSchemeDocument {
            parameters: self.type_parameters.clone(),
            constraints: self.constraints.clone(),
            type_ref: TypeDocument::Function {
                parameters: self
                    .parameters
                    .iter()
                    .map(|parameter| parameter.type_ref.clone())
                    .collect(),
                result: Box::new(self.result.clone()),
            },
        }
    }
}

impl From<&TypedType> for TypeDocument {
    fn from(type_ref: &TypedType) -> Self {
        Self::from_typed_type(type_ref)
    }
}

impl From<&InterfaceType> for TypeDocument {
    fn from(type_ref: &InterfaceType) -> Self {
        Self::from_interface_type(type_ref)
    }
}

impl From<&TypedScheme> for TypeSchemeDocument {
    fn from(scheme: &TypedScheme) -> Self {
        Self::from_typed_scheme(scheme)
    }
}

impl From<&InterfaceScheme> for TypeSchemeDocument {
    fn from(scheme: &InterfaceScheme) -> Self {
        Self::from_interface_scheme(scheme)
    }
}

fn typed_type_document(
    type_ref: &TypedType,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeDocument {
    match type_ref {
        TypedType::Named { name, arguments } => named_or_variable_document(
            name,
            None,
            arguments
                .iter()
                .map(|argument| typed_type_document(argument, parameter_arities))
                .collect(),
            parameter_arities,
        ),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypeDocument::Named {
            name: name.clone(),
            canonical: Some(canonical.clone()),
            arguments: arguments
                .iter()
                .map(|argument| typed_type_document(argument, parameter_arities))
                .collect(),
        },
        TypedType::Hole => TypeDocument::Unknown,
        TypedType::Record { closed, fields } => TypeDocument::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| typed_field_document(field, parameter_arities))
                .collect(),
        },
        TypedType::Tuple { elements } => TypeDocument::Tuple {
            elements: elements
                .iter()
                .map(|element| typed_type_document(element, parameter_arities))
                .collect(),
        },
        TypedType::Function { .. } => {
            let mut parameters = Vec::new();
            let mut current = type_ref;
            while let TypedType::Function { parameter, result } = current {
                parameters.push(typed_type_document(parameter, parameter_arities));
                current = result;
            }
            TypeDocument::Function {
                parameters,
                result: Box::new(typed_type_document(current, parameter_arities)),
            }
        }
    }
}

fn interface_type_document(
    type_ref: &InterfaceType,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeDocument {
    match type_ref {
        InterfaceType::Named { name, arguments } => named_or_variable_document(
            name,
            None,
            arguments
                .iter()
                .map(|argument| interface_type_document(argument, parameter_arities))
                .collect(),
            parameter_arities,
        ),
        InterfaceType::ExternalNamed {
            name,
            canonical,
            arguments,
            ..
        } => TypeDocument::Named {
            name: name.clone(),
            canonical: Some(canonical.clone()),
            arguments: arguments
                .iter()
                .map(|argument| interface_type_document(argument, parameter_arities))
                .collect(),
        },
        InterfaceType::Hole => TypeDocument::Unknown,
        InterfaceType::TypeConstructor { name, arity } => {
            if parameter_arities.contains_key(name) {
                TypeDocument::Variable {
                    name: name.clone(),
                    arity: *arity,
                    arguments: Vec::new(),
                }
            } else {
                TypeDocument::TypeConstructor {
                    name: name.clone(),
                    arity: *arity,
                }
            }
        }
        InterfaceType::Function { .. } => {
            let mut parameters = Vec::new();
            let mut current = type_ref;
            while let InterfaceType::Function { parameter, result } = current {
                parameters.push(interface_type_document(parameter, parameter_arities));
                current = result;
            }
            TypeDocument::Function {
                parameters,
                result: Box::new(interface_type_document(current, parameter_arities)),
            }
        }
        InterfaceType::Apply {
            constructor,
            arguments,
        } => named_or_variable_document(
            constructor,
            None,
            arguments
                .iter()
                .map(|argument| interface_type_document(argument, parameter_arities))
                .collect(),
            parameter_arities,
        ),
        InterfaceType::Record { closed, fields } => TypeDocument::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| TypeDocumentField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: interface_type_document(&field.type_ref, parameter_arities),
                })
                .collect(),
        },
        InterfaceType::Tuple { elements } => TypeDocument::Tuple {
            elements: elements
                .iter()
                .map(|element| interface_type_document(element, parameter_arities))
                .collect(),
        },
    }
}

fn named_or_variable_document(
    name: &str,
    canonical: Option<String>,
    arguments: Vec<TypeDocument>,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeDocument {
    match parameter_arities.get(name) {
        Some(arity) if canonical.is_none() => TypeDocument::Variable {
            name: name.to_owned(),
            arity: *arity,
            arguments,
        },
        _ => TypeDocument::Named {
            name: name.to_owned(),
            canonical,
            arguments,
        },
    }
}

fn typed_field_document(
    field: &TypedRecordField,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeDocumentField {
    TypeDocumentField {
        name: field.name.clone(),
        optional: field.optional,
        type_ref: typed_type_document(&field.type_ref, parameter_arities),
    }
}

fn typed_constraint_document(
    constraint: &TypedConstraint,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeConstraintDocument {
    TypeConstraintDocument {
        name: constraint.name.clone(),
        canonical: None,
        arguments: constraint
            .arguments
            .iter()
            .map(|argument| typed_type_document(argument, parameter_arities))
            .collect(),
    }
}

fn interface_constraint_document(
    constraint: &InterfaceConstraint,
    parameter_arities: &BTreeMap<String, u32>,
) -> TypeConstraintDocument {
    TypeConstraintDocument {
        name: constraint.name.clone(),
        canonical: constraint.trait_identity.clone(),
        arguments: constraint
            .arguments
            .iter()
            .map(|argument| interface_type_document(argument, parameter_arities))
            .collect(),
    }
}

fn interface_parameter_document(parameter: &InterfaceTypeParameter) -> TypeParameterDocument {
    TypeParameterDocument {
        name: parameter.name.clone(),
        arity: parameter.arity,
    }
}

fn typed_parameter_arities(scheme: &TypedScheme) -> BTreeMap<String, u32> {
    let mut arities = scheme
        .type_parameters
        .iter()
        .map(|name| (name.clone(), 0))
        .collect::<BTreeMap<_, _>>();
    collect_typed_parameter_arities(&scheme.type_ref, &mut arities);
    for constraint in &scheme.constraints {
        for argument in &constraint.arguments {
            collect_typed_parameter_arities(argument, &mut arities);
        }
    }
    arities
}

fn collect_typed_parameter_arities(type_ref: &TypedType, arities: &mut BTreeMap<String, u32>) {
    match type_ref {
        TypedType::Named { name, arguments } => {
            if let Some(arity) = arities.get_mut(name) {
                *arity = (*arity).max(arguments.len() as u32);
            }
            for argument in arguments {
                collect_typed_parameter_arities(argument, arities);
            }
        }
        TypedType::ExternalNamed { arguments, .. } => {
            for argument in arguments {
                collect_typed_parameter_arities(argument, arities);
            }
        }
        TypedType::Record { fields, .. } => {
            for field in fields {
                collect_typed_parameter_arities(&field.type_ref, arities);
            }
        }
        TypedType::Tuple { elements } => {
            for element in elements {
                collect_typed_parameter_arities(element, arities);
            }
        }
        TypedType::Function { parameter, result } => {
            collect_typed_parameter_arities(parameter, arities);
            collect_typed_parameter_arities(result, arities);
        }
        TypedType::Hole => {}
    }
}

fn render_plain(document: &TypeDocument, options: TypeRenderOptions) -> String {
    match options.layout {
        TypeRenderLayout::Compact => render_compact(document, false),
        TypeRenderLayout::Multiline => {
            render_multiline(document, options.indent_width as usize, false).join("\n")
        }
    }
}

fn render_plain_scheme(document: &TypeSchemeDocument, options: TypeRenderOptions) -> String {
    let parameters = render_parameters(&document.parameters);
    let body = render_plain(&document.type_ref, options);
    let constraints = document
        .constraints
        .iter()
        .map(|constraint| render_constraint(constraint, options))
        .collect::<Vec<_>>();
    match options.layout {
        TypeRenderLayout::Compact => {
            let mut rendered = String::new();
            if !parameters.is_empty() {
                rendered.push_str("forall ");
                rendered.push_str(&parameters.join(", "));
                rendered.push_str(". ");
            }
            rendered.push_str(&body);
            if !constraints.is_empty() {
                rendered.push_str(" where ");
                rendered.push_str(&constraints.join(", "));
            }
            rendered
        }
        TypeRenderLayout::Multiline => {
            let mut lines = Vec::new();
            if !parameters.is_empty() {
                lines.push(format!("forall {}.", parameters.join(", ")));
            }
            lines.extend(body.lines().map(str::to_owned));
            if !constraints.is_empty() {
                lines.push("where".to_owned());
                let indentation = " ".repeat(options.indent_width as usize);
                for constraint in constraints {
                    let mut constraint_lines =
                        constraint.lines().map(str::to_owned).collect::<Vec<_>>();
                    if let Some(last) = constraint_lines.last_mut() {
                        last.push(',');
                    }
                    lines.extend(
                        constraint_lines
                            .into_iter()
                            .map(|line| format!("{indentation}{line}")),
                    );
                }
            }
            lines.join("\n")
        }
    }
}

fn render_plain_callable(document: &TypeCallableDocument, options: TypeRenderOptions) -> String {
    let type_parameters = render_parameters(&document.type_parameters);
    match options.layout {
        TypeRenderLayout::Compact => {
            let mut rendered = document.name.clone();
            if !type_parameters.is_empty() {
                rendered.push('<');
                rendered.push_str(&type_parameters.join(", "));
                rendered.push('>');
            }
            for (index, parameter) in document.parameters.iter().enumerate() {
                rendered.push_str(if index == 0 { " " } else { " -> " });
                if let Some(name) = &parameter.name {
                    rendered.push_str(name);
                    rendered.push_str(": ");
                }
                rendered.push_str(&render_compact(&parameter.type_ref, true));
            }
            rendered.push_str(" -> ");
            rendered.push_str(&render_compact(&document.result, false));
            let constraints = document
                .constraints
                .iter()
                .map(|constraint| render_constraint(constraint, options))
                .collect::<Vec<_>>();
            if !constraints.is_empty() {
                rendered.push_str(" where ");
                rendered.push_str(&constraints.join(", "));
            }
            rendered
        }
        TypeRenderLayout::Multiline => {
            let indentation = " ".repeat(options.indent_width as usize);
            let mut header = document.name.clone();
            if !type_parameters.is_empty() {
                header.push('<');
                header.push_str(&type_parameters.join(", "));
                header.push('>');
            }
            let mut lines = vec![header];
            for (index, parameter) in document.parameters.iter().enumerate() {
                let mut prefix = indentation.clone();
                if index > 0 {
                    prefix.push_str("-> ");
                }
                if let Some(name) = &parameter.name {
                    prefix.push_str(name);
                    prefix.push_str(": ");
                }
                let parameter_lines =
                    render_multiline(&parameter.type_ref, options.indent_width as usize, true);
                lines.extend(prefix_lines(
                    parameter_lines,
                    &prefix,
                    &" ".repeat(prefix.len()),
                ));
            }
            let result_prefix = format!("{indentation}-> ");
            lines.extend(prefix_lines(
                render_multiline(&document.result, options.indent_width as usize, false),
                &result_prefix,
                &" ".repeat(result_prefix.len()),
            ));
            if !document.constraints.is_empty() {
                lines.push("where".to_owned());
                for constraint in &document.constraints {
                    let mut constraint_lines = render_multiline(
                        &TypeDocument::Named {
                            name: constraint.name.clone(),
                            canonical: constraint.canonical.clone(),
                            arguments: constraint.arguments.clone(),
                        },
                        options.indent_width as usize,
                        false,
                    );
                    if let Some(last) = constraint_lines.last_mut() {
                        last.push(',');
                    }
                    lines.extend(indent_lines(
                        constraint_lines,
                        options.indent_width as usize,
                    ));
                }
            }
            lines.join("\n")
        }
    }
}

fn render_parameters(parameters: &[TypeParameterDocument]) -> Vec<String> {
    parameters
        .iter()
        .map(|parameter| render_constructor(&parameter.name, parameter.arity))
        .collect()
}

fn render_constraint(constraint: &TypeConstraintDocument, options: TypeRenderOptions) -> String {
    render_plain(
        &TypeDocument::Named {
            name: constraint.name.clone(),
            canonical: constraint.canonical.clone(),
            arguments: constraint.arguments.clone(),
        },
        options,
    )
}

fn render_compact(document: &TypeDocument, nested: bool) -> String {
    match document {
        TypeDocument::Named {
            name, arguments, ..
        } => render_compact_application(name, arguments),
        TypeDocument::Variable {
            name,
            arity,
            arguments,
        } => {
            if arguments.is_empty() {
                render_constructor(name, *arity)
            } else {
                render_compact_application(name, arguments)
            }
        }
        TypeDocument::TypeConstructor { name, arity } => render_constructor(name, *arity),
        TypeDocument::Function { parameters, result } => {
            let rendered = parameters
                .iter()
                .map(|parameter| render_compact(parameter, true))
                .chain(std::iter::once(render_compact(result, false)))
                .collect::<Vec<_>>()
                .join(" -> ");
            if nested {
                format!("({rendered})")
            } else {
                rendered
            }
        }
        TypeDocument::Tuple { elements } => format!(
            "({})",
            elements
                .iter()
                .map(|element| render_compact(element, true))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeDocument::Record { closed, fields } => {
            let mut items = fields
                .iter()
                .map(|field| {
                    format!(
                        "{}{}: {}",
                        field.name,
                        if field.optional { "?" } else { "" },
                        render_compact(&field.type_ref, true)
                    )
                })
                .collect::<Vec<_>>();
            if !closed {
                items.push("...".to_owned());
            }
            if items.is_empty() {
                "{}".to_owned()
            } else {
                format!("{{ {} }}", items.join(", "))
            }
        }
        TypeDocument::Unknown => "unknown".to_owned(),
    }
}

fn render_compact_application(name: &str, arguments: &[TypeDocument]) -> String {
    if arguments.is_empty() {
        return name.to_owned();
    }
    format!(
        "{name}<{}>",
        arguments
            .iter()
            .map(|argument| render_compact(argument, true))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_multiline(document: &TypeDocument, indent_width: usize, nested: bool) -> Vec<String> {
    let lines = match document {
        TypeDocument::Named {
            name, arguments, ..
        } => render_multiline_application(name, arguments, indent_width),
        TypeDocument::Variable {
            name,
            arity,
            arguments,
        } => {
            if arguments.is_empty() {
                vec![render_constructor(name, *arity)]
            } else {
                render_multiline_application(name, arguments, indent_width)
            }
        }
        TypeDocument::TypeConstructor { name, arity } => vec![render_constructor(name, *arity)],
        TypeDocument::Function { parameters, result } => {
            let mut parts = parameters.iter().collect::<Vec<_>>();
            parts.push(result);
            let mut lines = Vec::new();
            for (index, part) in parts.into_iter().enumerate() {
                let part_lines = render_multiline(part, indent_width, index < parameters.len());
                if index == 0 {
                    lines.extend(part_lines);
                } else {
                    lines.extend(prefix_lines(part_lines, "-> ", "   "));
                }
            }
            lines
        }
        TypeDocument::Tuple { elements } => {
            if elements.is_empty() {
                vec!["()".to_owned()]
            } else {
                let mut lines = vec!["(".to_owned()];
                for element in elements {
                    let mut element_lines = render_multiline(element, indent_width, true);
                    if let Some(last) = element_lines.last_mut() {
                        last.push(',');
                    }
                    lines.extend(indent_lines(element_lines, indent_width));
                }
                lines.push(")".to_owned());
                lines
            }
        }
        TypeDocument::Record { closed, fields } => {
            if fields.is_empty() && *closed {
                vec!["{}".to_owned()]
            } else {
                let mut lines = vec!["{".to_owned()];
                for field in fields {
                    let prefix =
                        format!("{}{}: ", field.name, if field.optional { "?" } else { "" });
                    let field_lines = render_multiline(&field.type_ref, indent_width, true);
                    let mut field_lines =
                        prefix_lines(field_lines, &prefix, &" ".repeat(indent_width));
                    if let Some(last) = field_lines.last_mut() {
                        last.push(',');
                    }
                    lines.extend(indent_lines(field_lines, indent_width));
                }
                if !closed {
                    lines.push(format!("{}...", " ".repeat(indent_width)));
                }
                lines.push("}".to_owned());
                lines
            }
        }
        TypeDocument::Unknown => vec!["unknown".to_owned()],
    };
    if nested && matches!(document, TypeDocument::Function { .. }) {
        let mut wrapped = vec!["(".to_owned()];
        wrapped.extend(indent_lines(lines, indent_width));
        wrapped.push(")".to_owned());
        wrapped
    } else {
        lines
    }
}

fn render_multiline_application(
    name: &str,
    arguments: &[TypeDocument],
    indent_width: usize,
) -> Vec<String> {
    if arguments.is_empty() {
        return vec![name.to_owned()];
    }
    let mut lines = vec![format!("{name}<")];
    for argument in arguments {
        let mut argument_lines = render_multiline(argument, indent_width, true);
        if let Some(last) = argument_lines.last_mut() {
            last.push(',');
        }
        lines.extend(indent_lines(argument_lines, indent_width));
    }
    lines.push(">".to_owned());
    lines
}

fn render_constructor(name: &str, arity: u32) -> String {
    if arity == 0 {
        name.to_owned()
    } else {
        format!("{name}<{}>", vec!["_"; arity as usize].join(", "))
    }
}

fn prefix_lines(lines: Vec<String>, first: &str, continuation: &str) -> Vec<String> {
    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                format!("{first}{line}")
            } else {
                format!("{continuation}{line}")
            }
        })
        .collect()
}

fn indent_lines(lines: Vec<String>, indent_width: usize) -> Vec<String> {
    let indentation = " ".repeat(indent_width);
    lines
        .into_iter()
        .map(|line| format!("{indentation}{line}"))
        .collect()
}

fn wrap_markup(plain: String, markup: TypeRenderMarkup, layout: TypeRenderLayout) -> String {
    match (markup, layout) {
        (TypeRenderMarkup::Plain, _) => plain,
        (TypeRenderMarkup::Markdown, TypeRenderLayout::Compact) => format!("`{plain}`"),
        (TypeRenderMarkup::Markdown, TypeRenderLayout::Multiline) => {
            format!("```seseragi\n{plain}\n```")
        }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::{InterfaceRecordField, TypeParameter};

    #[test]
    fn renders_nested_function_record_and_effect_from_one_document() {
        let type_ref = function(
            TypedType::Record {
                closed: true,
                fields: vec![
                    TypedRecordField {
                        name: "mapper".to_owned(),
                        optional: false,
                        type_ref: function(named("Int"), named("String")),
                    },
                    TypedRecordField {
                        name: "values".to_owned(),
                        optional: true,
                        type_ref: applied("Array", applied("Maybe", named("Int"))),
                    },
                ],
            },
            applied_many(
                "Effect",
                vec![
                    TypedType::Record {
                        closed: true,
                        fields: vec![TypedRecordField {
                            name: "console".to_owned(),
                            optional: false,
                            type_ref: named("Console"),
                        }],
                    },
                    named("String"),
                    applied("Array", named("Int")),
                ],
            ),
        );
        let document = TypeDocument::from_typed_type(&type_ref);

        assert_eq!(
            document.render(TypeRenderOptions::default()),
            "{ mapper: (Int -> String), values?: Array<Maybe<Int>> } -> \
             Effect<{ console: Console }, String, Array<Int>>"
        );
    }

    #[test]
    fn renders_higher_kinded_constraints_and_curried_partial_signatures() {
        let scheme = InterfaceScheme {
            type_parameters: vec![
                TypeParameter::constructor("F", 1),
                TypeParameter::value("A"),
                TypeParameter::value("B"),
            ],
            constraints: vec![InterfaceConstraint {
                name: "Functor".to_owned(),
                trait_identity: Some("std/prelude::Functor".to_owned()),
                arguments: vec![InterfaceType::TypeConstructor {
                    name: "F".to_owned(),
                    arity: 1,
                }],
            }],
            type_ref: interface_function(
                interface_function(interface_named("A"), interface_named("B")),
                interface_function(
                    InterfaceType::Apply {
                        constructor: "F".to_owned(),
                        arguments: vec![interface_named("A")],
                    },
                    InterfaceType::Apply {
                        constructor: "F".to_owned(),
                        arguments: vec![interface_named("B")],
                    },
                ),
            ),
        };
        let document = TypeSchemeDocument::from_interface_scheme(&scheme);

        assert_eq!(
            document.render(TypeRenderOptions::default()),
            "forall F<_>, A, B. (A -> B) -> F<A> -> F<B> where Functor<F<_>>"
        );
        assert_eq!(
            document.render(TypeRenderOptions {
                markup: TypeRenderMarkup::Markdown,
                ..TypeRenderOptions::default()
            }),
            "`forall F<_>, A, B. (A -> B) -> F<A> -> F<B> where Functor<F<_>>`"
        );
        assert_eq!(
            document.render(TypeRenderOptions {
                layout: TypeRenderLayout::Multiline,
                ..TypeRenderOptions::default()
            }),
            concat!(
                "forall F<_>, A, B.\n",
                "(\n",
                "  A\n",
                "  -> B\n",
                ")\n",
                "-> F<\n",
                "     A,\n",
                "   >\n",
                "-> F<\n",
                "     B,\n",
                "   >\n",
                "where\n",
                "  Functor<\n",
                "    F<_>,\n",
                "  >,"
            )
        );
    }

    #[test]
    fn renders_named_callable_compact_and_multiline_from_the_same_scheme() {
        let scheme = TypeSchemeDocument {
            parameters: vec![
                TypeParameterDocument {
                    name: "F".to_owned(),
                    arity: 1,
                },
                TypeParameterDocument {
                    name: "A".to_owned(),
                    arity: 0,
                },
                TypeParameterDocument {
                    name: "B".to_owned(),
                    arity: 0,
                },
            ],
            constraints: vec![TypeConstraintDocument {
                name: "Functor".to_owned(),
                canonical: Some("std/prelude::Functor".to_owned()),
                arguments: vec![TypeDocument::TypeConstructor {
                    name: "F".to_owned(),
                    arity: 1,
                }],
            }],
            type_ref: TypeDocument::Function {
                parameters: vec![
                    TypeDocument::Function {
                        parameters: vec![named_document("A")],
                        result: Box::new(named_document("B")),
                    },
                    TypeDocument::Variable {
                        name: "F".to_owned(),
                        arity: 1,
                        arguments: vec![named_document("A")],
                    },
                ],
                result: Box::new(TypeDocument::Variable {
                    name: "F".to_owned(),
                    arity: 1,
                    arguments: vec![named_document("B")],
                }),
            },
        };
        let callable = TypeCallableDocument::from_scheme(
            "map",
            [Some("mapper".to_owned()), Some("source".to_owned())],
            scheme,
        )
        .expect("function scheme becomes a callable document");

        assert_eq!(
            callable.render(TypeRenderOptions::default()),
            "map<F<_>, A, B> mapper: (A -> B) -> source: F<A> -> F<B> where Functor<F<_>>"
        );
        assert_eq!(
            callable.render(TypeRenderOptions {
                layout: TypeRenderLayout::Multiline,
                ..TypeRenderOptions::default()
            }),
            concat!(
                "map<F<_>, A, B>\n",
                "  mapper: (\n",
                "            A\n",
                "            -> B\n",
                "          )\n",
                "  -> source: F<\n",
                "               A,\n",
                "             >\n",
                "  -> F<\n",
                "       B,\n",
                "     >\n",
                "where\n",
                "  Functor<\n",
                "    F<_>,\n",
                "  >,"
            )
        );
    }

    #[test]
    fn preserves_external_identity_and_formats_unknown_recovery_types() {
        let document = TypeDocument::from_interface_type(&InterfaceType::ExternalNamed {
            name: "users.User".to_owned(),
            canonical: "package/users::User".to_owned(),
            provider_module: "package/users".to_owned(),
            provider_export: "User".to_owned(),
            arguments: vec![InterfaceType::Hole],
        });

        assert_eq!(
            document,
            TypeDocument::Named {
                name: "users.User".to_owned(),
                canonical: Some("package/users::User".to_owned()),
                arguments: vec![TypeDocument::Unknown],
            }
        );
        assert_eq!(
            document.render(TypeRenderOptions::default()),
            "users.User<unknown>"
        );
    }

    #[test]
    fn emits_plain_and_markdown_multiline_snapshots_from_the_same_tree() {
        let document = TypeDocument::Record {
            closed: false,
            fields: vec![TypeDocumentField {
                name: "callback".to_owned(),
                optional: true,
                type_ref: TypeDocument::Function {
                    parameters: vec![TypeDocument::Tuple {
                        elements: vec![named_document("Int"), named_document("String")],
                    }],
                    result: Box::new(named_document("Bool")),
                },
            }],
        };
        let options = TypeRenderOptions {
            layout: TypeRenderLayout::Multiline,
            ..TypeRenderOptions::default()
        };
        let plain = document.render(options);

        assert_eq!(
            plain,
            concat!(
                "{\n",
                "  callback?: (\n",
                "      (\n",
                "        Int,\n",
                "        String,\n",
                "      )\n",
                "      -> Bool\n",
                "    ),\n",
                "  ...\n",
                "}"
            )
        );
        assert_eq!(
            document.render(TypeRenderOptions {
                markup: TypeRenderMarkup::Markdown,
                ..options
            }),
            format!("```seseragi\n{plain}\n```")
        );
    }

    #[test]
    fn infers_typed_higher_kinded_parameter_arity_from_uses() {
        let scheme = TypedScheme {
            type_parameters: vec!["F".to_owned(), "A".to_owned()],
            constraints: vec![TypedConstraint {
                name: "Functor".to_owned(),
                arguments: vec![TypedType::Named {
                    name: "F".to_owned(),
                    arguments: Vec::new(),
                }],
            }],
            type_ref: TypedType::Named {
                name: "F".to_owned(),
                arguments: vec![TypedType::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                }],
            },
        };
        let document = TypeSchemeDocument::from_typed_scheme(&scheme);

        assert_eq!(document.parameters[0].arity, 1);
        assert_eq!(
            document.render(TypeRenderOptions::default()),
            "forall F<_>, A. F<A> where Functor<F<_>>"
        );
    }

    #[test]
    fn converts_interface_record_fields_without_losing_optional_or_open_shape() {
        let document = TypeDocument::from_interface_type(&InterfaceType::Record {
            closed: false,
            fields: vec![InterfaceRecordField {
                name: "id".to_owned(),
                optional: true,
                type_ref: interface_named("String"),
            }],
        });

        assert_eq!(
            document.render(TypeRenderOptions::default()),
            "{ id?: String, ... }"
        );
    }

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn applied(name: &str, argument: TypedType) -> TypedType {
        applied_many(name, vec![argument])
    }

    fn applied_many(name: &str, arguments: Vec<TypedType>) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments,
        }
    }

    fn function(parameter: TypedType, result: TypedType) -> TypedType {
        TypedType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        }
    }

    fn interface_named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn interface_function(parameter: InterfaceType, result: InterfaceType) -> InterfaceType {
        InterfaceType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        }
    }

    fn named_document(name: &str) -> TypeDocument {
        TypeDocument::Named {
            name: name.to_owned(),
            canonical: None,
            arguments: Vec::new(),
        }
    }
}
