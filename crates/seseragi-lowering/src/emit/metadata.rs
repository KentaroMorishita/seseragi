use crate::{
    GeneratedOutputPaths, TypeScriptBinding, TypeScriptFunction, TypeScriptInstance,
    TypeScriptModule, TypeScriptType,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedModule {
    pub schema: u32,
    pub module: String,
    pub target: String,
    pub runtime: GeneratedRuntime,
    pub exports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<GeneratedInstance>,
    pub outputs: GeneratedOutputs,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedInstance {
    pub identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub head: TypeScriptType,
    pub type_identity: String,
    pub dictionary_export: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedRuntime {
    pub identity: String,
    pub abi_major: u32,
    pub requirements: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOutputs {
    pub typescript: String,
    pub source_map: String,
}

pub(super) fn generated_module_for(
    module: TypeScriptModule,
    output_paths: GeneratedOutputPaths,
) -> GeneratedModule {
    let exports = module_exports(&module);
    let instances = module
        .instances
        .iter()
        .map(generated_instance_from_typescript)
        .collect();
    GeneratedModule {
        schema: module.schema,
        module: module.module,
        target: "typescript-es2022".to_owned(),
        runtime: GeneratedRuntime {
            identity: "@seseragi/runtime".to_owned(),
            abi_major: 1,
            requirements: module.runtime_requirements,
        },
        exports,
        instances,
        outputs: GeneratedOutputs {
            typescript: output_paths.typescript,
            source_map: output_paths.source_map,
        },
    }
}

fn generated_instance_from_typescript(instance: &TypeScriptInstance) -> GeneratedInstance {
    GeneratedInstance {
        identity: instance.identity.clone(),
        trait_name: instance.trait_name.clone(),
        head: instance.head.clone(),
        type_identity: instance.type_identity.clone(),
        dictionary_export: instance.dictionary_export.clone(),
    }
}

fn module_exports(module: &TypeScriptModule) -> Vec<String> {
    let mut exports = Vec::new();
    for adt in &module.adts {
        for variant in &adt.variants {
            if variant.exported {
                exports.push(variant.name.clone());
            }
        }
    }
    for binding in &module.bindings {
        match binding {
            TypeScriptBinding::Const { exported, name, .. } if *exported => {
                exports.push(name.clone());
            }
            _ => {}
        }
    }
    for function in &module.functions {
        match function {
            TypeScriptFunction::ConstFunction { exported, name, .. } if *exported => {
                exports.push(name.clone());
            }
            _ => {}
        }
    }
    exports
}

#[cfg(test)]
mod tests {
    use super::generated_module_for;
    use crate::{
        lower_core_module_to_typescript_ir, lower_typed_module, GeneratedOutputPaths,
        TypeScriptType,
    };
    use seseragi_semantics::type_module;

    #[test]
    fn exposes_selected_dictionary_exports_separately_from_source_exports() {
        let source = "pub type AppError deriving Show =\n  | EndOfInput\n";
        let typed = type_module("artifact/generated-show/main.ssrg", source);
        let core = lower_typed_module(typed);
        let metadata = generated_module_for(
            lower_core_module_to_typescript_ir(core),
            GeneratedOutputPaths::default(),
        );

        assert_eq!(metadata.exports, vec!["EndOfInput"]);
        assert_eq!(metadata.instances.len(), 1);
        assert_eq!(
            metadata.instances[0].identity,
            "Show<artifact/generated-show::AppError>"
        );
        assert_eq!(metadata.instances[0].trait_name, "Show");
        assert_eq!(
            metadata.instances[0].head,
            TypeScriptType::Reference {
                name: "AppError".to_owned(),
                arguments: Vec::new(),
            }
        );
        assert_eq!(
            metadata.instances[0].type_identity,
            "artifact/generated-show::AppError"
        );
        assert_eq!(
            metadata.instances[0].dictionary_export,
            "__ssrg$instance$Show$0"
        );
    }

    #[test]
    fn omits_instance_metadata_when_no_dictionary_is_selected() {
        let source = "pub let answer: Int = 42\n";
        let typed = type_module("artifact/no-generated-show/main.ssrg", source);
        let core = lower_typed_module(typed);
        let metadata = generated_module_for(
            lower_core_module_to_typescript_ir(core),
            GeneratedOutputPaths::default(),
        );
        let json = serde_json::to_value(metadata).unwrap();

        assert!(json.get("instances").is_none());
    }
}
