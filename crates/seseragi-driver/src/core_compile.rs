use crate::{analyze::analyze_module_frontend, CompileInput};
use seseragi_lowering::{lower_typed_module, CoreModule};
use seseragi_semantics::{AnalyzedModule, TypedModule, TypedModuleInterface};
use seseragi_syntax::DiagnosticArtifact;

/// Checked, backend-independent input. No target lowering or emission runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreCompilation {
    pub diagnostics: DiagnosticArtifact,
    pub typed_hir: TypedModule,
    pub typed_interface: TypedModuleInterface,
    pub core_ir: CoreModule,
}

pub fn compile_core_module(input: CompileInput<'_>) -> Result<CoreCompilation, DiagnosticArtifact> {
    analyze_module_frontend(input).map(lower_analyzed_module)
}

/// Project adapters may supply a checked module after resolving imports.
pub fn lower_analyzed_module(analyzed: AnalyzedModule) -> CoreCompilation {
    CoreCompilation {
        core_ir: lower_typed_module(analyzed.typed_hir.clone()),
        diagnostics: analyzed.diagnostics,
        typed_hir: analyzed.typed_hir,
        typed_interface: analyzed.typed_interface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_checked_function_value_type_without_merging_source_and_returned_stages() {
        use seseragi_lowering::CoreType;
        let compiled = compile_core_module(CompileInput::new("stages.ssrg", "stages/main", "pub fn two left: Int -> right: Int -> Bool = True\npub fn returned left: Int -> (Int -> Bool) = \\right -> True\npub fn omitted<Unit> -> (Unit -> Bool) = \\value -> True\n")).unwrap();
        let functions = &compiled.core_ir.functions;
        assert_eq!(functions[0].parameters.len(), 2);
        assert_eq!(functions[1].parameters.len(), 1);
        assert_eq!(functions[0].value_type, functions[1].value_type);
        let CoreType::Function { parameter, result } = functions[2].value_type.as_ref().unwrap()
        else {
            panic!()
        };
        assert!(
            matches!(parameter.as_ref(), CoreType::ExternalNamed { canonical, arguments, .. } if canonical=="std/prelude::Unit" && arguments.is_empty())
        );
        assert!(
            matches!(result.as_ref(), CoreType::Function { parameter, .. } if matches!(parameter.as_ref(),CoreType::Named{name,arguments}if name=="Unit" && arguments.is_empty()))
        );
        assert_eq!(functions[2].type_parameters[0].name, "Unit");
        let json = serde_json::to_string(&functions[0]).unwrap();
        let restored: seseragi_lowering::CoreFunction = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.value_type, functions[0].value_type);
        let mut older: serde_json::Value = serde_json::from_str(&json).unwrap();
        older.as_object_mut().unwrap().remove("valueType");
        let unproven: seseragi_lowering::CoreFunction = serde_json::from_value(older).unwrap();
        assert!(unproven.value_type.is_none());
    }

    #[test]
    fn recursive_ts_values_use_the_retained_closed_contract_without_filling_holes() {
        use seseragi_lowering::{TypeScriptFunction, TypeScriptType};
        let input = CompileInput::new(
            "recursive.ssrg",
            "stages/recursive",
            "pub fn deep value: Int -> Int = if value <= 0 then 0 else 1 + deep (value - 1)\n",
        );
        let compiled = crate::compile_module(input).unwrap();
        assert!(
            compiled
                .generated
                .typescript
                .contains("export const deep: (argument: number) => number ="),
            "{}",
            compiled.generated.typescript
        );
        assert!(matches!(
            &compiled.typescript_ir.functions[0],
            TypeScriptFunction::ConstFunction {
                value_type: Some(TypeScriptType::Function { .. }),
                ..
            }
        ));
        let mut missing = compiled.core_ir.clone();
        missing.functions[0].value_type = None;
        let lowered = seseragi_lowering::lower_core_module_to_typescript_ir(missing);
        assert!(matches!(
            &lowered.functions[0],
            TypeScriptFunction::ConstFunction {
                value_type: None,
                ..
            }
        ));
        for source in [
            "type Int = | LocalInt\npub fn repeat value: Int -> Int = repeat value\n",
            "type Int = | LocalInt\npub fn repeat value: Bool -> Int = if value then LocalInt else repeat True\n",
            "pub fn repeat<A> value: A -> A = repeat value\n",
            "pub fn repeat<A> value: A -> A where Show<A> = repeat value\n",
        ] {
            let other =
                crate::compile_module(CompileInput::new("other.ssrg", "stages/other", source))
                    .unwrap();
            assert!(
                matches!(
                    &other.typescript_ir.functions[0],
                    TypeScriptFunction::ConstFunction {
                        value_type: None,
                        ..
                    }
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn project_core_matches_ts_input_and_does_not_require_ts_output_paths() {
        let mut graph = seseragi_project::ModuleGraph::new();
        graph.add_module("graph/lib".to_owned(), []).unwrap();
        graph
            .add_module(
                "graph/main".to_owned(),
                [("./lib".to_owned(), "graph/lib".to_owned())],
            )
            .unwrap();
        let inputs = vec![
            crate::ProjectModuleInput::new(
                "lib.ssrg",
                "graph/lib",
                "pub fn answer -> Int = 42\n",
                "lib.js",
            ),
            crate::ProjectModuleInput::new(
                "main.ssrg",
                "graph/main",
                "import { answer } from \"./lib\"\npub fn probe -> Int = answer ()\n",
                "main.js",
            ),
        ];
        let core = crate::compile_core_project(graph.clone(), inputs.clone()).unwrap();
        let ts = crate::compile_project(graph.clone(), inputs.clone()).unwrap();
        assert_eq!(core.order, ts.order);
        for id in &core.order {
            assert_eq!(core.modules[id].core_ir, ts.modules[id].core_ir);
            assert_eq!(
                core.modules[id].typed_interface,
                ts.modules[id].typed_interface
            );
        }
        let targetless = inputs
            .into_iter()
            .map(|mut input| {
                input.output_path = format!("/not-a-ts-plan/{}", input.module_id);
                input
            })
            .collect::<Vec<_>>();
        assert!(crate::compile_project(graph.clone(), targetless.clone()).is_err());
        assert_eq!(
            crate::compile_core_project(graph, targetless).unwrap(),
            core
        );
    }

    #[test]
    fn both_backends_receive_identical_checked_core() {
        let input = CompileInput::new(
            "example.ssrg",
            "logical/example",
            "pub fn answer -> Int = 42\n",
        );
        let core = compile_core_module(input).unwrap();
        let ts = crate::compile_module(input).unwrap();
        assert_eq!(core.core_ir, ts.core_ir);
        assert_eq!(core.typed_interface, ts.typed_interface);
        assert!(compile_core_module(CompileInput::new(
            "bad.ssrg",
            "logical/bad",
            "pub fn answer -> Int = \"bad\"\n"
        ))
        .is_err());
    }
    #[test]
    fn imported_monomorphic_callable_values_keep_canonical_identity() {
        for (import, call) in [
            ("import * as remote from \"./lib\"", "remote.callback 20 22"),
            (
                "import { callback as renamed } from \"./lib\"",
                "renamed 20 22",
            ),
        ] {
            let mut graph = seseragi_project::ModuleGraph::new();
            graph.add_module("graph/lib".to_owned(), []).unwrap();
            graph
                .add_module(
                    "graph/main".to_owned(),
                    [("./lib".to_owned(), "graph/lib".to_owned())],
                )
                .unwrap();
            let inputs = vec![
                crate::ProjectModuleInput::new("lib.ssrg", "graph/lib", "fn returned n: Int -> (Int -> Int) = \\m -> n + m\npub let callback: Int -> (Int -> Int) = returned\n", "lib.js"),
                crate::ProjectModuleInput::new("main.ssrg", "graph/main", format!("{import}\npub fn probe -> Int = {call}\n"), "main.js"),
            ];
            let core = crate::compile_core_project(graph.clone(), inputs.clone()).unwrap();
            let ts = crate::compile_project(graph, inputs).unwrap();
            let main = &core.modules["graph/main"].core_ir;
            let seseragi_lowering::CoreExpr::Call {
                callee, arguments, ..
            } = &main.functions[0].body
            else {
                panic!("expected imported value call");
            };
            assert_eq!(callee, "graph/lib::callback");
            assert_eq!(arguments.len(), 2);
            assert_eq!(main, &ts.modules["graph/main"].core_ir);
            // A function-valued binding has no declaration arity proof. Both
            // targets must retain ordinary staged application of its value.
            assert!(seseragi_lowering::known_pure_call_shape(main, callee, 2).is_none());
        }
    }
}
