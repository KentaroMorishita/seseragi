use crate::surface::{parse_surface_ast, ByteSpan, SurfaceDecl, TypeRef, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInterface {
    pub schema: u32,
    pub module: String,
    pub source: String,
    pub dependencies: Vec<InterfaceDependency>,
    pub exports: Vec<InterfaceExport>,
    pub operators: Vec<InterfaceOperator>,
    pub instances: Vec<InterfaceInstance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceDependency {
    pub specifier: String,
    pub module: String,
    pub origin: ByteSpan,
    pub imports: Vec<InterfaceImport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceImport {
    pub namespace: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceExport {
    pub symbol: String,
    pub namespace: String,
    pub name: String,
    pub visibility: Visibility,
    pub declaration: ByteSpan,
    pub scheme: InterfaceScheme,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceScheme {
    pub type_parameters: Vec<String>,
    pub constraints: Vec<InterfaceConstraint>,
    #[serde(rename = "type")]
    pub type_ref: InterfaceType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum InterfaceType {
    Named {
        name: String,
        arguments: Vec<InterfaceType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceOperator {
    pub symbol: String,
    pub spelling: String,
    pub fixity: String,
    pub precedence: u32,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceInstance {
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub head: InterfaceType,
    pub constraints: Vec<InterfaceConstraint>,
    pub origin: ByteSpan,
}

pub fn parse_module_interface(source_name: impl Into<String>, source: &str) -> ModuleInterface {
    let source_name = source_name.into();
    let module_name = module_name_from_source_name(&source_name);
    let source_file = source_file_from_source_name(&source_name);
    let surface_module = parse_surface_ast(source_file.clone(), source);

    ModuleInterface {
        schema: 1,
        module: module_name.clone(),
        source: surface_module.source,
        dependencies: Vec::new(),
        exports: surface_module
            .declarations
            .into_iter()
            .filter_map(|declaration| export_from_surface_decl(&module_name, declaration))
            .collect(),
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn export_from_surface_decl(
    module_name: &str,
    declaration: SurfaceDecl,
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
        } if visibility == Visibility::Public => {
            let type_ref = type_ref.as_ref().map(interface_type_from_type_ref)?;
            Some(InterfaceExport {
                symbol: format!("{module_name}::{name}"),
                namespace: "value".to_owned(),
                name,
                visibility,
                declaration: span,
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref,
                },
            })
        }
        _ => None,
    }
}

fn interface_type_from_type_ref(type_ref: &TypeRef) -> InterfaceType {
    match type_ref {
        TypeRef::Named { name, .. } => InterfaceType::Named {
            name: name.clone(),
            arguments: Vec::new(),
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
}
