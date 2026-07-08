use crate::cst::parse_cst;
pub use crate::interface_model::{
    InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceInstance,
    InterfaceOperator, InterfaceScheme, InterfaceType, ModuleInterface,
};
use crate::surface::{parse_surface_ast, SurfaceDecl, SurfaceImport, TypeRef, Visibility};

pub fn parse_module_interface(source_name: impl Into<String>, source: &str) -> ModuleInterface {
    let source_name = source_name.into();
    let module_name = module_name_from_source_name(&source_name);
    let source_file = source_file_from_source_name(&source_name);
    let cst = parse_cst(source_file.clone(), source);
    if !cst.errors.is_empty() {
        return ModuleInterface {
            schema: 1,
            module: module_name,
            source: cst.source,
            dependencies: Vec::new(),
            exports: Vec::new(),
            operators: Vec::new(),
            instances: Vec::new(),
        };
    }

    let surface_module = parse_surface_ast(source_file.clone(), source);

    let interface = ModuleInterface {
        schema: 1,
        module: module_name.clone(),
        source: surface_module.source.clone(),
        dependencies: surface_module
            .imports
            .into_iter()
            .map(|import| dependency_from_surface_import(&module_name, &source_name, import))
            .collect(),
        exports: surface_module
            .declarations
            .iter()
            .filter_map(|declaration| export_from_surface_decl(&module_name, declaration))
            .collect(),
        operators: surface_module
            .declarations
            .iter()
            .filter_map(|declaration| operator_from_surface_decl(&module_name, declaration))
            .collect(),
        instances: surface_module
            .declarations
            .into_iter()
            .filter_map(instance_from_surface_decl)
            .collect(),
    };
    interface
}

fn export_from_surface_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Option<InterfaceExport> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            type_ref,
            span,
            ..
        }
        | SurfaceDecl::EffectFn {
            visibility,
            name,
            return_type: type_ref,
            span,
            ..
        } if *visibility == Visibility::Public => {
            let type_ref = type_ref.as_ref().map(interface_type_from_type_ref)?;
            Some(InterfaceExport {
                symbol: format!("{module_name}::{name}"),
                namespace: "value".to_owned(),
                name: name.clone(),
                visibility: *visibility,
                declaration_kind: None,
                declaration: *span,
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref,
                },
                representation: None,
            })
        }
        SurfaceDecl::Newtype {
            visibility,
            name,
            representation,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::{name}"),
            namespace: "type".to_owned(),
            name: name.clone(),
            visibility: *visibility,
            declaration_kind: Some("newtype".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.clone(),
                    arity: 0,
                },
            },
            representation: Some(interface_type_from_type_ref(representation)),
        }),
        SurfaceDecl::Operator {
            visibility,
            type_parameters,
            spelling,
            parameters,
            return_type,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::operator({spelling})"),
            namespace: "operator".to_owned(),
            name: spelling.clone(),
            visibility: *visibility,
            declaration_kind: Some("custom-operator".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: Vec::new(),
                type_ref: operator_interface_type(parameters, return_type),
            },
            representation: None,
        }),
        _ => None,
    }
}

fn dependency_from_surface_import(
    module_name: &str,
    source_name: &str,
    import: SurfaceImport,
) -> InterfaceDependency {
    let module = dependency_module_name(module_name, source_name, &import.specifier);
    InterfaceDependency {
        specifier: import.specifier,
        module: module.clone(),
        origin: import.span,
        imports: import
            .items
            .into_iter()
            .map(|item| InterfaceImport {
                symbol: match item.namespace.as_str() {
                    "operator" => format!("{module}::operator({})", item.name),
                    _ => format!("{module}::{}", item.name),
                },
                namespace: item.namespace,
                name: item.name,
                local_name: item.alias,
            })
            .collect(),
    }
}

fn operator_from_surface_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Option<InterfaceOperator> {
    match declaration {
        SurfaceDecl::Operator {
            visibility,
            fixity,
            precedence,
            spelling,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceOperator {
            symbol: format!("{module_name}::operator({spelling})"),
            spelling: spelling.clone(),
            fixity: fixity.clone(),
            precedence: *precedence,
            origin: *span,
        }),
        _ => None,
    }
}

fn instance_from_surface_decl(declaration: SurfaceDecl) -> Option<InterfaceInstance> {
    match declaration {
        SurfaceDecl::Instance {
            trait_name,
            arguments,
            span,
        } => Some(InterfaceInstance {
            trait_name: trait_name.clone(),
            head: InterfaceType::Apply {
                constructor: trait_name,
                arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
            },
            constraints: Vec::new(),
            origin: span,
        }),
        _ => None,
    }
}

fn operator_interface_type(
    parameters: &[crate::surface::SurfaceParameter],
    return_type: &TypeRef,
) -> InterfaceType {
    parameters.iter().rev().fold(
        interface_type_from_type_ref(return_type),
        |result, parameter| InterfaceType::Function {
            parameter: Box::new(interface_type_from_type_ref(&parameter.type_ref)),
            result: Box::new(result),
        },
    )
}

fn interface_type_from_type_ref(type_ref: &TypeRef) -> InterfaceType {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => InterfaceType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
        },
    }
}

fn module_name_from_source_name(source_name: &str) -> String {
    let normalized = source_name.replace('\\', "/");
    let source_file = source_file_from_source_name(&normalized);
    let parent = normalized.rsplit_once('/').map(|(parent, _)| parent);

    match parent {
        Some(parent) if !parent.is_empty() && parent != "." => parent.to_owned(),
        _ => source_file
            .rsplit_once('.')
            .map(|(stem, _)| stem.to_owned())
            .unwrap_or(source_file),
    }
}

fn dependency_module_name(module_name: &str, source_name: &str, specifier: &str) -> String {
    if let Some(relative) = specifier.strip_prefix("./") {
        return format!("{module_name}/{relative}");
    }
    source_name
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/{specifier}"))
        .unwrap_or_else(|| specifier.to_owned())
}

fn source_file_from_source_name(source_name: &str) -> String {
    source_name
        .replace('\\', "/")
        .rsplit_once('/')
        .map(|(_, file)| file.to_owned())
        .unwrap_or_else(|| source_name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::ByteSpan;

    #[test]
    fn parses_basic_public_let_interface() {
        let interface =
            parse_module_interface("artifact/basic/main.ssrg", "pub let answer: Int = 42\n");
        let json = serde_json::to_value(&interface).expect("interface serializes");

        assert_eq!(json["schema"], 1);
        assert_eq!(json["module"], "artifact/basic");
        assert_eq!(json["source"], "main.ssrg");
        assert_eq!(json["dependencies"], serde_json::json!([]));
        assert_eq!(json["exports"][0]["symbol"], "artifact/basic::answer");
        assert_eq!(json["exports"][0]["namespace"], "value");
        assert_eq!(json["exports"][0]["name"], "answer");
        assert_eq!(json["exports"][0]["visibility"], "public");
        assert_eq!(
            json["exports"][0]["declaration"],
            serde_json::json!({
                "start": 0,
                "end": 24,
            })
        );
        assert_eq!(
            json["exports"][0]["scheme"],
            serde_json::json!({
                "typeParameters": [],
                "constraints": [],
                "type": {
                    "kind": "named",
                    "name": "Int",
                    "arguments": [],
                },
            })
        );
        assert_eq!(json["operators"], serde_json::json!([]));
        assert_eq!(json["instances"], serde_json::json!([]));
    }

    #[test]
    fn omits_private_lets_from_interface() {
        let interface = parse_module_interface(
            "artifact/multiple-lets/main.ssrg",
            "let first = 1\npub let second: Int = 2\n",
        );

        assert_eq!(interface.exports.len(), 1);
        assert_eq!(
            interface.exports[0].symbol,
            "artifact/multiple-lets::second"
        );
        assert_eq!(interface.exports[0].name, "second");
        assert_eq!(
            interface.exports[0].declaration,
            ByteSpan { start: 14, end: 37 }
        );
    }

    #[test]
    fn omits_exports_when_module_has_parse_errors() {
        let interface =
            parse_module_interface("artifact/recovery/main.ssrg", "pub let answer: Int =");

        assert_eq!(interface.module, "artifact/recovery");
        assert_eq!(interface.source, "main.ssrg");
        assert!(interface.exports.is_empty());
    }

    #[test]
    fn parses_public_effect_fn_interface() {
        let interface = parse_module_interface(
            "artifact/effect-do/main.ssrg",
            "pub effect fn main -> Unit\nwith Console\nfails ConsoleError =\n  do {}\n",
        );
        let json = serde_json::to_value(&interface).expect("interface serializes");

        assert_eq!(json["exports"][0]["symbol"], "artifact/effect-do::main");
        assert_eq!(
            json["exports"][0]["scheme"]["type"],
            serde_json::json!({
                "kind": "named",
                "name": "Unit",
                "arguments": [],
            })
        );
    }

    #[test]
    fn parses_nested_type_arguments_in_interface() {
        let interface = parse_module_interface(
            "artifact/nested-types/main.ssrg",
            "pub let values: Array<Maybe<Int>> = []\n",
        );
        let json = serde_json::to_value(&interface).expect("interface serializes");

        assert_eq!(
            json["exports"][0]["scheme"]["type"],
            serde_json::json!({
                "kind": "named",
                "name": "Array",
                "arguments": [
                    {
                        "kind": "named",
                        "name": "Maybe",
                        "arguments": [
                            {
                                "kind": "named",
                                "name": "Int",
                                "arguments": [],
                            },
                        ],
                    },
                ],
            })
        );
    }

    #[test]
    fn parses_operator_type_parameters_in_interface() {
        let interface = parse_module_interface(
            "artifact/generic-operator/main.ssrg",
            "pub operator<A> infixr 5 <+>\n  left: A -> right: A -> A =\n  left\n",
        );
        let operator_export = interface
            .exports
            .iter()
            .find(|export| export.namespace == "operator")
            .expect("operator export exists");

        assert_eq!(operator_export.name, "<+>");
        assert_eq!(operator_export.scheme.type_parameters, vec!["A".to_owned()]);
        assert_eq!(interface.operators[0].fixity, "infixr");
        assert_eq!(interface.operators[0].precedence, 5);
    }

    #[test]
    fn parses_aliased_import_in_interface() {
        let interface = parse_module_interface(
            "artifact/import-alias/main.ssrg",
            "import { parse as parseJson } from \"json\"\n\npub let answer: Int = 42\n",
        );

        assert_eq!(interface.dependencies.len(), 1);
        assert_eq!(interface.dependencies[0].imports.len(), 1);
        assert_eq!(interface.dependencies[0].imports[0].namespace, "value");
        assert_eq!(interface.dependencies[0].imports[0].name, "parse");
        assert_eq!(
            interface.dependencies[0].imports[0].local_name,
            Some("parseJson".to_owned())
        );
    }

    #[test]
    fn parses_rich_module_interface_fixture() {
        let interface = parse_module_interface(
            "artifact/rich/main.ssrg",
            include_str!("../../../examples/spec/artifacts/interface-schema-1/rich/main.ssrg"),
        );
        let actual = serde_json::to_value(&interface).expect("interface serializes");
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../examples/spec/artifacts/interface-schema-1/rich/interface.json"
        ))
        .expect("expected interface fixture parses");

        assert_eq!(actual, expected);
    }
}
