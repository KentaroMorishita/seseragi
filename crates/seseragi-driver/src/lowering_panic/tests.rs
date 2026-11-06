use super::{catch_lowering_panic, fault};
use crate::{
    compile_core_module, compile_core_project, compile_linked_module_with_output_paths,
    compile_module, compile_project, CompileInput, LinkedCompileError, ProjectCompileError,
    ProjectModuleInput,
};
use seseragi_lowering::{GeneratedOutputPaths, TypeScriptOutputPlan};
use seseragi_project::ModuleGraph;
use seseragi_syntax::{parse_diagnostics, DiagnosticArtifact, DiagnosticSeverity};

const SOURCE: &str = "pub fn answer -> Int = 42\n";
const STAGES: [fault::Stage; 3] = [
    fault::Stage::Core,
    fault::Stage::TypeScript,
    fault::Stage::Emission,
];

fn assert_internal_error(diagnostics: &DiagnosticArtifact, source: &str) {
    assert_eq!(diagnostics.source, source);
    assert_eq!(diagnostics.schema, 1);
    assert_eq!(diagnostics.position_encoding, "utf-8");
    assert_eq!(diagnostics.diagnostics.len(), 1);
    let diagnostic = &diagnostics.diagnostics[0];
    assert_eq!(diagnostic.id, "d1");
    assert_eq!(diagnostic.code, "SES-I0001");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.message(), "Internal compiler error");
    let note = "This is a bug in the compiler, not an error in your source. Please report it with the source that triggers it.";
    assert_eq!(diagnostic.notes(), [note]);
    assert_eq!(diagnostic.primary.start, 0);
    assert_eq!(diagnostic.primary.end, 0);
    assert!(diagnostic.fixes.is_empty());
    let json = serde_json::to_value(diagnostics).unwrap();
    assert_eq!(json["diagnostics"][0]["message"], "Internal compiler error");
    assert_eq!(json["diagnostics"][0]["notes"], serde_json::json!([note]));
    let terminal = crate::render_terminal_diagnostics(diagnostics, SOURCE);
    assert!(terminal.contains(source));
    assert!(terminal.contains("error[SES-I0001]: Internal compiler error"));
    assert!(terminal.contains(&format!("  = note: {note}\n")));
}

fn project() -> (ModuleGraph<String>, Vec<ProjectModuleInput>) {
    let mut graph = ModuleGraph::new();
    graph.add_module("app/lib".to_owned(), []).unwrap();
    graph
        .add_module(
            "app/main".to_owned(),
            [("./lib".to_owned(), "app/lib".to_owned())],
        )
        .unwrap();
    let inputs = vec![
        ProjectModuleInput::new("lib.ssrg", "app/lib", SOURCE, "lib.js"),
        ProjectModuleInput::new(
            "main.ssrg",
            "app/main",
            "import { answer } from \"./lib\"\npub fn result -> Int = answer ()\n",
            "main.js",
        ),
    ];
    (graph, inputs)
}

#[test]
fn single_module_discards_output_and_recovers_after_each_lowering_stage_panics() {
    for profile in [
        crate::BuildProfile::Development,
        crate::BuildProfile::Release,
    ] {
        let input = CompileInput::new("single.ssrg", "app/single", SOURCE).with_profile(profile);
        let expected = compile_module(input).unwrap();
        assert!(!expected.generated.typescript.is_empty());
        for stage in STAGES {
            let diagnostics = fault::with_panic_at(stage, 0, || compile_module(input))
                .expect_err("panicking compilation must return diagnostics without artifacts");
            assert_internal_error(&diagnostics, "single.ssrg");
            assert_eq!(compile_module(input).unwrap(), expected);
        }
    }
}

#[test]
fn project_discards_earlier_modules_and_recovers_after_each_lowering_stage_panics() {
    let (graph, inputs) = project();
    let expected = compile_project(graph.clone(), inputs.clone()).unwrap();
    assert_eq!(expected.order, ["app/lib", "app/main"]);
    for stage in STAGES {
        let error =
            fault::with_panic_at(stage, 1, || compile_project(graph.clone(), inputs.clone()))
                .expect_err("a project must not return partial module artifacts");
        let ProjectCompileError::Compile {
            module,
            error: LinkedCompileError::Diagnostics(diagnostics),
        } = error
        else {
            panic!("expected internal-error diagnostics, got {error:?}");
        };
        assert_eq!(module, "app/main");
        assert_internal_error(&diagnostics, "main.ssrg");
        assert_eq!(
            compile_project(graph.clone(), inputs.clone()).unwrap(),
            expected
        );
    }
}

#[test]
fn core_only_module_returns_diagnostics_and_recovers_after_lowering_panics() {
    let input = CompileInput::new("native.ssrg", "app/native", SOURCE);
    let expected = compile_core_module(input).unwrap();
    let diagnostics = fault::with_panic_at(fault::Stage::Core, 0, || compile_core_module(input))
        .expect_err("native compilation must not return partial Core IR");
    assert_internal_error(&diagnostics, "native.ssrg");
    assert_eq!(compile_core_module(input).unwrap(), expected);
}

#[test]
fn core_only_project_discards_earlier_modules_and_recovers_after_lowering_panics() {
    let (graph, inputs) = project();
    let expected = compile_core_project(graph.clone(), inputs.clone()).unwrap();
    let error = fault::with_panic_at(fault::Stage::Core, 1, || {
        compile_core_project(graph.clone(), inputs.clone())
    })
    .expect_err("native project compilation must not return partial Core IR");
    let ProjectCompileError::Diagnostics { modules } = error else {
        panic!("expected internal-error diagnostics, got {error:?}");
    };
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].module, "app/main");
    assert_internal_error(&modules[0].diagnostics, "main.ssrg");
    assert_eq!(compile_core_project(graph, inputs).unwrap(), expected);
}

#[test]
fn linked_module_propagates_internal_diagnostics_and_recovers() {
    let linked = seseragi_project::link_module(
        seseragi_syntax::parse_unlinked_module_interface("linked.ssrg", "app/linked", SOURCE),
        &Default::default(),
    )
    .unwrap();
    let plan = TypeScriptOutputPlan::default();
    let compile = || {
        compile_linked_module_with_output_paths(
            linked.clone(),
            SOURCE,
            &plan,
            GeneratedOutputPaths::default(),
        )
    };
    let expected = compile().unwrap();
    for stage in STAGES {
        let error = fault::with_panic_at(stage, 0, compile).unwrap_err();
        let LinkedCompileError::Diagnostics(diagnostics) = error else {
            panic!("expected internal-error diagnostics, got {error:?}");
        };
        assert_internal_error(&diagnostics, "linked.ssrg");
        assert_eq!(compile().unwrap(), expected);
    }
}

#[test]
fn non_string_panic_payload_also_returns_an_internal_error() {
    let diagnostics = catch_lowering_panic(parse_diagnostics("payload.ssrg", SOURCE), || {
        std::panic::panic_any(42_u32)
    })
    .unwrap_err();
    assert_internal_error(&diagnostics, "payload.ssrg");
    assert!(compile_module(CompileInput::new("payload.ssrg", "app/payload", SOURCE)).is_ok());
}

const TEMPLATE_SOURCE: &str = "pub fn label value: Int -> String = `value=${value}`\n";

fn strip_template_evidence(core: &mut seseragi_lowering::CoreModule) {
    for function in &mut core.functions {
        if let seseragi_lowering::CoreExpr::Template { parts, .. } = &mut function.body {
            for part in parts {
                if let seseragi_lowering::CoreTemplatePart::Interpolation { evidence, .. } = part {
                    *evidence = None;
                }
            }
        }
    }
}

fn assert_located_lowering_error(diagnostics: &DiagnosticArtifact, source: &str) {
    assert_eq!(diagnostics.source, source);
    assert_eq!(diagnostics.diagnostics.len(), 1);
    let diagnostic = &diagnostics.diagnostics[0];
    assert_eq!(diagnostic.code, "SES-I0001");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.message(), "Internal compiler error");
    let start = TEMPLATE_SOURCE.find('`').unwrap();
    let end = TEMPLATE_SOURCE.rfind('`').unwrap() + 1;
    assert_eq!(
        (diagnostic.primary.start, diagnostic.primary.end),
        (start, end)
    );
    let [label] = diagnostic.related.as_slice() else {
        panic!("expected one lowering label, got {:?}", diagnostic.related);
    };
    assert_eq!(label.source, None);
    assert_eq!(label.primary, diagnostic.primary);
    assert!(
        label.message.starts_with(
            "missing selected trait evidence during TypeScript lowering of an expression: template interpolation"
        ),
        "{}",
        label.message
    );
    let terminal = crate::render_terminal_diagnostics(diagnostics, TEMPLATE_SOURCE);
    assert!(
        terminal.contains(&format!("{source}:1:{}: error[SES-I0001]", start + 1)),
        "{terminal}"
    );
}

#[test]
fn single_module_reports_missing_lowering_facts_at_their_source() {
    let input = CompileInput::new("template.ssrg", "app/template", TEMPLATE_SOURCE);
    let expected = compile_module(input).unwrap();
    let diagnostics = fault::with_core_edit(strip_template_evidence, || compile_module(input))
        .expect_err("missing evidence must stop compilation");
    assert_located_lowering_error(&diagnostics, "template.ssrg");
    assert_eq!(compile_module(input).unwrap(), expected);
}

#[test]
fn linked_module_reports_missing_lowering_facts_as_diagnostics() {
    let linked = seseragi_project::link_module(
        seseragi_syntax::parse_unlinked_module_interface(
            "linked.ssrg",
            "app/linked",
            TEMPLATE_SOURCE,
        ),
        &Default::default(),
    )
    .unwrap();
    let error = fault::with_core_edit(strip_template_evidence, || {
        compile_linked_module_with_output_paths(
            linked,
            TEMPLATE_SOURCE,
            &TypeScriptOutputPlan::default(),
            GeneratedOutputPaths::default(),
        )
    })
    .unwrap_err();
    let LinkedCompileError::Diagnostics(diagnostics) = error else {
        panic!("expected located internal diagnostics, got {error:?}");
    };
    assert_located_lowering_error(&diagnostics, "linked.ssrg");
}

#[derive(Clone, Copy, Debug)]
enum MissingFact {
    GeneralizedBinding,
    RecursiveTypeImport,
    ComprehensionGenerator,
    DisplayDictionaryImport,
}

impl MissingFact {
    fn source(self) -> &'static str {
        match self {
            Self::GeneralizedBinding => "let render = \\value -> show value\n",
            Self::RecursiveTypeImport => "pub fn recurse value: Int -> Int = if value == 0 then 0 else recurse (value - 1)\n",
            Self::ComprehensionGenerator => "pub fn values items: Array<Int> -> Array<Int> = [item | item <- items]\n",
            Self::DisplayDictionaryImport => "type Box deriving Show = | Box Int\npub fn label value: Box -> String = show value\n",
        }
    }

    fn origin(self, compiled: &crate::CompiledModule) -> seseragi_lowering::SourceSpan {
        match self {
            Self::GeneralizedBinding | Self::RecursiveTypeImport => {
                compiled.core_ir.functions[0].origin.clone()
            }
            Self::ComprehensionGenerator => {
                seseragi_lowering::core_expr_origin(&compiled.core_ir.functions[0].body).clone()
            }
            Self::DisplayDictionaryImport => compiled.typescript_ir.instances[0].origin.clone(),
        }
    }

    fn label(self) -> String {
        use seseragi_lowering::{LoweringInternalErrorKind as Kind, LoweringStage as Stage};
        let (kind, stage) = match self {
            Self::GeneralizedBinding => (Kind::UnexpectedTypedHirShape, Stage::Core),
            Self::RecursiveTypeImport => (Kind::MissingTypeInformation, Stage::Type),
            Self::ComprehensionGenerator => (Kind::UnexpectedCoreShape, Stage::Expression),
            Self::DisplayDictionaryImport => (Kind::MissingDerivedEvidence, Stage::Emission),
        };
        format!("{} during {}: ", kind.description(), stage.description())
    }

    fn with_missing_fact<T>(self, action: impl FnOnce() -> T) -> T {
        match self {
            Self::GeneralizedBinding => fault::with_typed_edit(
                |module| {
                    let seseragi_semantics::TypedDecl::Let { bindings, .. } =
                        &mut module.declarations[0]
                    else {
                        panic!("generalized let");
                    };
                    bindings.clear();
                },
                action,
            ),
            Self::RecursiveTypeImport => fault::with_core_edit(
                |module| {
                    module.functions[0].value_type =
                        Some(seseragi_lowering::CoreType::ExternalNamed {
                            name: "Lost".into(),
                            canonical: "lost/module::Lost".into(),
                            arguments: vec![],
                        });
                },
                action,
            ),
            Self::ComprehensionGenerator => fault::with_core_edit(
                |module| {
                    let seseragi_lowering::CoreExpr::ArrayComprehension { clauses, .. } =
                        &mut module.functions[0].body
                    else {
                        panic!("comprehension");
                    };
                    clauses.clear();
                },
                action,
            ),
            Self::DisplayDictionaryImport => fault::with_typescript_edit(
                |module| {
                    module
                        .type_imports
                        .retain(|import| import.feature != "core.show.dictionary");
                },
                action,
            ),
        }
    }
}

const MISSING_FACTS: [MissingFact; 4] = [
    MissingFact::GeneralizedBinding,
    MissingFact::RecursiveTypeImport,
    MissingFact::ComprehensionGenerator,
    MissingFact::DisplayDictionaryImport,
];

fn assert_missing_fact_diagnostic(
    diagnostics: &DiagnosticArtifact,
    fact: MissingFact,
    origin: &seseragi_lowering::SourceSpan,
) {
    assert_eq!(diagnostics.source, origin.source);
    let [diagnostic] = diagnostics.diagnostics.as_slice() else {
        panic!("one internal error: {diagnostics:?}");
    };
    assert_eq!(diagnostic.code, "SES-I0001");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(
        (diagnostic.primary.start, diagnostic.primary.end),
        (origin.start, origin.end)
    );
    let [label] = diagnostic.related.as_slice() else {
        panic!("located invariant label");
    };
    assert_eq!(label.primary, diagnostic.primary);
    assert_eq!(label.source, None);
    // The unwind safety net has no invariant label; this proves Result handling.
    assert!(
        label.message.starts_with(&fact.label()),
        "{}",
        label.message
    );
}

#[test]
fn single_module_reports_core_type_comprehension_and_emission_errors() {
    for fact in MISSING_FACTS {
        for profile in [
            crate::BuildProfile::Development,
            crate::BuildProfile::Release,
        ] {
            let input =
                CompileInput::new("fault.ssrg", "app/fault", fact.source()).with_profile(profile);
            let expected = compile_module(input).unwrap();
            let origin = fact.origin(&expected);
            let diagnostics = fact
                .with_missing_fact(|| compile_module(input))
                .unwrap_err();
            assert_missing_fact_diagnostic(&diagnostics, fact, &origin);
            assert_eq!(compile_module(input).unwrap(), expected);
        }
    }
}

#[test]
fn project_reports_core_type_comprehension_and_emission_errors() {
    for fact in MISSING_FACTS {
        let mut graph = ModuleGraph::new();
        graph.add_module("app/fault".to_owned(), []).unwrap();
        let inputs = vec![ProjectModuleInput::new(
            "fault.ssrg",
            "app/fault",
            fact.source(),
            "fault.js",
        )];
        let expected = compile_project(graph.clone(), inputs.clone()).unwrap();
        let origin = fact.origin(&expected.modules["app/fault"]);
        let error = fact
            .with_missing_fact(|| compile_project(graph.clone(), inputs.clone()))
            .unwrap_err();
        let diagnostics = match error {
            ProjectCompileError::Compile {
                module,
                error: LinkedCompileError::Diagnostics(diagnostics),
            } => {
                assert_eq!(module, "app/fault");
                diagnostics
            }
            ProjectCompileError::Diagnostics { modules } => {
                assert_eq!(modules.len(), 1);
                assert_eq!(modules[0].module, "app/fault");
                modules[0].diagnostics.clone()
            }
            other => panic!("expected internal diagnostics: {other:?}"),
        };
        assert_missing_fact_diagnostic(&diagnostics, fact, &origin);
        assert_eq!(compile_project(graph, inputs).unwrap(), expected);
    }
}

#[test]
fn core_only_module_and_project_report_missing_typed_facts() {
    let fact = MissingFact::GeneralizedBinding;
    let input = CompileInput::new("fault.ssrg", "app/fault", fact.source());
    let expected = compile_module(input).unwrap();
    let origin = fact.origin(&expected);
    let diagnostics = fact
        .with_missing_fact(|| compile_core_module(input))
        .unwrap_err();
    assert_missing_fact_diagnostic(&diagnostics, fact, &origin);
    let mut graph = ModuleGraph::new();
    graph.add_module("app/fault".to_owned(), []).unwrap();
    let inputs = vec![ProjectModuleInput::new(
        "fault.ssrg",
        "app/fault",
        fact.source(),
        "fault.js",
    )];
    let error = fact
        .with_missing_fact(|| compile_core_project(graph, inputs))
        .unwrap_err();
    let ProjectCompileError::Diagnostics { modules } = error else {
        panic!("Core project diagnostic");
    };
    assert_eq!(modules.len(), 1);
    assert_missing_fact_diagnostic(&modules[0].diagnostics, fact, &origin);
}
