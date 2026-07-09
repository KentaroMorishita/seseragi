use crate::discovery::{
    discover_artifact_cases, discover_cases, discover_interface_cases, discover_resolved_ast_cases,
};
use std::path::{Path, PathBuf};

pub(crate) struct Suite {
    pub(crate) token_cases: Vec<PathBuf>,
    pub(crate) cst_cases: Vec<PathBuf>,
    pub(crate) diagnostics_cases: Vec<PathBuf>,
    pub(crate) semantic_diagnostics_cases: Vec<PathBuf>,
    pub(crate) surface_ast_cases: Vec<PathBuf>,
    pub(crate) interface_cases: Vec<PathBuf>,
    pub(crate) resolved_ast_cases: Vec<PathBuf>,
    pub(crate) typed_hir_cases: Vec<PathBuf>,
    pub(crate) typed_interface_cases: Vec<PathBuf>,
    pub(crate) core_ir_cases: Vec<PathBuf>,
    pub(crate) typescript_ir_cases: Vec<PathBuf>,
    pub(crate) generated_module_cases: Vec<PathBuf>,
    pub(crate) execution_cases: Vec<PathBuf>,
    pub(crate) runtime_abi_cases: Vec<PathBuf>,
}

impl Suite {
    pub(crate) fn discover(artifacts: &Path) -> Self {
        let frontend_cases = discover_cases(&artifacts.join("schema-1"));
        let mut token_cases = frontend_cases.clone();
        token_cases.extend(discover_cases(&artifacts.join("token-schema-1")));

        let diagnostics_cases =
            discover_artifact_cases(&artifacts.join("schema-1"), "diagnostics.json");
        let mut surface_ast_cases =
            discover_artifact_cases(&artifacts.join("schema-1"), "surface-ast.json");
        surface_ast_cases.extend(discover_artifact_cases(
            &artifacts.join("interface-schema-1"),
            "surface-ast.json",
        ));
        surface_ast_cases.extend(discover_artifact_cases(
            &artifacts.join("surface-schema-1"),
            "surface-ast.json",
        ));
        surface_ast_cases.sort();

        let mut interface_cases = discover_interface_cases(&artifacts.join("schema-1"));
        interface_cases.extend(discover_interface_cases(
            &artifacts.join("interface-schema-1"),
        ));

        let mut resolved_ast_cases = discover_resolved_ast_cases(&artifacts.join("schema-1"));
        resolved_ast_cases.extend(discover_artifact_cases(
            &artifacts.join("interface-schema-1"),
            "resolved-ast.json",
        ));
        resolved_ast_cases.sort();

        Self {
            token_cases,
            cst_cases: frontend_cases,
            diagnostics_cases,
            semantic_diagnostics_cases: discover_artifact_cases(
                artifacts,
                "semantic-diagnostics.json",
            ),
            surface_ast_cases,
            interface_cases,
            resolved_ast_cases,
            typed_hir_cases: discover_artifact_cases(artifacts, "typed-hir.json"),
            typed_interface_cases: discover_artifact_cases(artifacts, "typed-interface.json"),
            core_ir_cases: discover_artifact_cases(artifacts, "core-ir.json"),
            typescript_ir_cases: discover_artifact_cases(artifacts, "typescript-ir.json"),
            generated_module_cases: discover_artifact_cases(artifacts, "generated-module.json"),
            execution_cases: discover_artifact_cases(artifacts, "run.json"),
            runtime_abi_cases: discover_artifact_cases(
                &artifacts.join("runtime-schema-1"),
                "abi.json",
            ),
        }
    }
}
