use crate::model::{
    Declaration, Function, Interface, Namespace, OpaqueDeclaration, Parameter, Scope, Span,
    TypeAlias, TypeKind, TypeRef,
};
use std::collections::BTreeMap;
use tree_sitter::{Node, Parser};

pub fn parse_declarations(source: &str) -> Result<Scope, ParseError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .map_err(|error| ParseError {
            message: format!("failed to load the TypeScript grammar: {error}"),
            span: Span { start: 0, end: 0 },
        })?;
    let tree = parser.parse(source, None).ok_or_else(|| ParseError {
        message: "TypeScript parser was cancelled".to_owned(),
        span: Span { start: 0, end: 0 },
    })?;
    if tree.root_node().has_error() {
        let error = first_error(tree.root_node()).unwrap_or(tree.root_node());
        return Err(ParseError {
            message: "invalid TypeScript declaration syntax".to_owned(),
            span: Span::from_node(error),
        });
    }
    parse_scope(tree.root_node(), source)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

fn first_error(node: Node<'_>) -> Option<Node<'_>> {
    if node.is_error() || node.is_missing() {
        return Some(node);
    }
    let mut cursor = node.walk();
    let error = node.named_children(&mut cursor).find_map(first_error);
    error
}

fn parse_scope(container: Node<'_>, source: &str) -> Result<Scope, ParseError> {
    let mut local = BTreeMap::<String, Vec<Declaration>>::new();
    let mut exported = Vec::new();
    let mut aliases = Vec::<(String, String, Span)>::new();
    let mut cursor = container.walk();
    for child in container.named_children(&mut cursor) {
        if child.kind() == "export_statement" {
            if let Some(declaration) = child.child_by_field_name("declaration") {
                for declaration in parse_declaration(declaration, source)? {
                    local
                        .entry(declaration.original_name().to_owned())
                        .or_default()
                        .push(declaration.clone());
                    exported.push(declaration);
                }
            } else {
                collect_export_aliases(child, source, &mut aliases);
            }
            continue;
        }
        for declaration in parse_declaration(child, source)? {
            local
                .entry(declaration.original_name().to_owned())
                .or_default()
                .push(declaration);
        }
    }
    for (target, public, span) in aliases {
        let declarations = local.get(&target).ok_or_else(|| ParseError {
            message: format!("export alias `{public}` refers to missing symbol `{target}`"),
            span,
        })?;
        exported.extend(
            declarations
                .iter()
                .map(|declaration| declaration.with_export_alias(public.clone(), span)),
        );
    }
    Ok(Scope {
        declarations: exported,
    })
}

fn collect_export_aliases(
    export: Node<'_>,
    source: &str,
    output: &mut Vec<(String, String, Span)>,
) {
    let Some(clause) = export.named_child(0) else {
        return;
    };
    let mut cursor = clause.walk();
    for specifier in clause.named_children(&mut cursor) {
        if specifier.kind() != "export_specifier" {
            continue;
        }
        let Some(name) = specifier.child_by_field_name("name") else {
            continue;
        };
        let alias = specifier.child_by_field_name("alias").unwrap_or(name);
        output.push((
            node_text(name, source).to_owned(),
            unquote(node_text(alias, source)),
            Span::from_node(alias),
        ));
    }
}

fn parse_declaration(node: Node<'_>, source: &str) -> Result<Vec<Declaration>, ParseError> {
    let node = if node.kind() == "ambient_declaration" {
        node.named_child(0).unwrap_or(node)
    } else {
        node
    };
    let declaration = match node.kind() {
        "function_signature" => Some(Declaration::Function(parse_function(node, source)?)),
        "interface_declaration" => Some(Declaration::Interface(parse_interface(node, source)?)),
        "internal_module" => Some(Declaration::Namespace(parse_namespace(node, source)?)),
        "class_declaration" | "abstract_class_declaration" => {
            Some(Declaration::Class(parse_opaque(node, source)?))
        }
        "enum_declaration" => Some(Declaration::Enum(parse_opaque(node, source)?)),
        "type_alias_declaration" => Some(Declaration::TypeAlias(parse_type_alias(node, source)?)),
        _ => None,
    };
    Ok(declaration.into_iter().collect())
}

fn parse_function(node: Node<'_>, source: &str) -> Result<Function, ParseError> {
    let name = required_field(node, "name")?;
    let parameters = required_field(node, "parameters")?;
    let result = required_field(node, "return_type")?;
    Ok(Function {
        original_name: unquote(node_text(name, source)),
        public_name: unquote(node_text(name, source)),
        name_span: Span::from_node(name),
        span: Span::from_node(node),
        type_parameters: node
            .child_by_field_name("type_parameters")
            .map(|parameters| parse_type_parameters(parameters, source))
            .unwrap_or_default(),
        parameters: parse_parameters(parameters, source)?,
        result: parse_type(result, source)?,
    })
}

fn parse_interface(node: Node<'_>, source: &str) -> Result<Interface, ParseError> {
    let name = required_field(node, "name")?;
    let name_text = node_text(name, source).to_owned();
    Ok(Interface {
        name: name_text.clone(),
        public_name: name_text,
        name_span: Span::from_node(name),
        span: Span::from_node(node),
    })
}

fn parse_opaque(node: Node<'_>, source: &str) -> Result<OpaqueDeclaration, ParseError> {
    let name = required_field(node, "name")?;
    let name_text = node_text(name, source).to_owned();
    Ok(OpaqueDeclaration {
        name: name_text.clone(),
        public_name: name_text,
        name_span: Span::from_node(name),
        span: Span::from_node(node),
        type_parameters: node
            .child_by_field_name("type_parameters")
            .map(|parameters| parse_type_parameters(parameters, source))
            .unwrap_or_default(),
    })
}

fn parse_type_alias(node: Node<'_>, source: &str) -> Result<TypeAlias, ParseError> {
    let name = required_field(node, "name")?;
    let value = required_field(node, "value")?;
    let name_text = node_text(name, source).to_owned();
    Ok(TypeAlias {
        name: name_text.clone(),
        public_name: name_text,
        name_span: Span::from_node(name),
        span: Span::from_node(node),
        type_parameters: node
            .child_by_field_name("type_parameters")
            .map(|parameters| parse_type_parameters(parameters, source))
            .unwrap_or_default(),
        type_ref: parse_type(value, source)?,
    })
}

fn parse_namespace(node: Node<'_>, source: &str) -> Result<Namespace, ParseError> {
    let name = required_field(node, "name")?;
    let body = required_field(node, "body")?;
    let name_text = unquote(node_text(name, source));
    Ok(Namespace {
        original_name: name_text.clone(),
        public_name: name_text,
        name_span: Span::from_node(name),
        span: Span::from_node(node),
        scope: parse_scope(body, source)?,
    })
}

fn parse_type_parameters(node: Node<'_>, source: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut cursor = node.walk();
    for parameter in node.named_children(&mut cursor) {
        if let Some(name) = parameter.child_by_field_name("name") {
            output.push(node_text(name, source).to_owned());
        }
    }
    output
}

fn parse_parameters(node: Node<'_>, source: &str) -> Result<Vec<Parameter>, ParseError> {
    let mut output = Vec::new();
    let mut cursor = node.walk();
    for parameter in node.named_children(&mut cursor) {
        if !matches!(
            parameter.kind(),
            "required_parameter" | "optional_parameter" | "rest_pattern"
        ) {
            continue;
        }
        let pattern = parameter
            .child_by_field_name("pattern")
            .or_else(|| parameter.child_by_field_name("name"))
            .ok_or_else(|| ParseError {
                message: "function parameter must have a stable name".to_owned(),
                span: Span::from_node(parameter),
            })?;
        let type_node = parameter
            .child_by_field_name("type")
            .ok_or_else(|| ParseError {
                message: "function parameter must have an explicit type".to_owned(),
                span: Span::from_node(parameter),
            })?;
        output.push(Parameter {
            name: node_text(pattern, source).to_owned(),
            name_span: Span::from_node(pattern),
            optional: parameter.kind() == "optional_parameter",
            rest: parameter.kind() == "rest_pattern",
            type_ref: parse_type(type_node, source)?,
        });
    }
    Ok(output)
}

fn parse_type(node: Node<'_>, source: &str) -> Result<TypeRef, ParseError> {
    let node = if node.kind() == "type_annotation" {
        node.named_child(0).unwrap_or(node)
    } else {
        node
    };
    let span = Span::from_node(node);
    let kind = match node.kind() {
        "predefined_type" => TypeKind::Primitive(node_text(node, source).to_owned()),
        "type_identifier" | "nested_type_identifier" | "identifier" => {
            TypeKind::Named(node_text(node, source).to_owned())
        }
        "generic_type" => {
            let name = node
                .child_by_field_name("name")
                .map(|name| node_text(name, source).to_owned())
                .unwrap_or_else(|| node_text(node, source).to_owned());
            let mut arguments = Vec::new();
            if let Some(argument_list) = node.child_by_field_name("type_arguments") {
                let mut cursor = argument_list.walk();
                for argument in argument_list.named_children(&mut cursor) {
                    arguments.push(parse_type(argument, source)?);
                }
            }
            TypeKind::Generic { name, arguments }
        }
        "readonly_type" => {
            let child = node.named_child(0).unwrap_or(node);
            let parsed = parse_type(child, source)?;
            match parsed.kind {
                TypeKind::MutableArray(element) => TypeKind::ReadonlyArray(element),
                _ => parsed.kind,
            }
        }
        "array_type" => {
            let child = node.named_child(0).ok_or_else(|| ParseError {
                message: "array type is missing its element type".to_owned(),
                span,
            })?;
            TypeKind::MutableArray(Box::new(parse_type(child, source)?))
        }
        "union_type" => {
            let mut members = Vec::new();
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                members.push(parse_type(child, source)?);
            }
            TypeKind::Union(members)
        }
        "literal_type" => TypeKind::Literal(node_text(node, source).to_owned()),
        "function_type" => {
            let parameters = required_field(node, "parameters")?;
            let result = required_field(node, "return_type")?;
            TypeKind::Function {
                parameters: parse_parameters(parameters, source)?,
                result: Box::new(parse_type(result, source)?),
            }
        }
        "tuple_type" => {
            let mut members = Vec::new();
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                members.push(parse_type(child, source)?);
            }
            TypeKind::Tuple(members)
        }
        "parenthesized_type" => {
            return parse_type(node.named_child(0).unwrap_or(node), source);
        }
        _ => TypeKind::Unsupported(node_text(node, source).to_owned()),
    };
    Ok(TypeRef { kind, span })
}

fn required_field<'a>(node: Node<'a>, field: &str) -> Result<Node<'a>, ParseError> {
    node.child_by_field_name(field).ok_or_else(|| ParseError {
        message: format!("TypeScript declaration is missing `{field}`"),
        span: Span::from_node(node),
    })
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

fn unquote(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_owned()
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_export_aliases_and_nested_namespaces() {
        let scope = parse_declarations(
            r#"
declare function internal(): string;
export { internal as "type" };
export namespace Metrics {
  export function count(values: readonly number[]): number;
}
"#,
        )
        .unwrap();
        assert_eq!(scope.declarations.len(), 2);
        let function = scope
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Function(function) => Some(function),
                _ => None,
            })
            .unwrap();
        assert_eq!(function.original_name, "internal");
        assert_eq!(function.public_name, "type");
        let namespace = scope
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Namespace(namespace) => Some(namespace),
                _ => None,
            })
            .unwrap();
        assert_eq!(namespace.scope.declarations.len(), 1);
    }
}
