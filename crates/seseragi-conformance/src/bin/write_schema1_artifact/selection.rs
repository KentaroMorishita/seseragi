use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Stage {
    Tokens,
    Cst,
    Diagnostics,
    SemanticDiagnostics,
    Analysis,
    Interface,
    SurfaceAst,
    ResolvedAst,
    TypedHir,
    TypedInterface,
    CoreIr,
    TypeScriptIr,
    GeneratedModule,
}

const STAGES: [Stage; 13] = [
    Stage::Tokens,
    Stage::Cst,
    Stage::Diagnostics,
    Stage::SemanticDiagnostics,
    Stage::Analysis,
    Stage::Interface,
    Stage::SurfaceAst,
    Stage::ResolvedAst,
    Stage::TypedHir,
    Stage::TypedInterface,
    Stage::CoreIr,
    Stage::TypeScriptIr,
    Stage::GeneratedModule,
];

impl Stage {
    fn parse(name: &str) -> Option<Self> {
        STAGES.into_iter().find(|stage| stage.name() == name)
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Tokens => "tokens",
            Self::Cst => "cst",
            Self::Diagnostics => "diagnostics",
            Self::SemanticDiagnostics => "semantic-diagnostics",
            Self::Analysis => "analysis",
            Self::Interface => "interface",
            Self::SurfaceAst => "surface-ast",
            Self::ResolvedAst => "resolved-ast",
            Self::TypedHir => "typed-hir",
            Self::TypedInterface => "typed-interface",
            Self::CoreIr => "core-ir",
            Self::TypeScriptIr => "typescript-ir",
            Self::GeneratedModule => "generated-module",
        }
    }

    fn artifact_file(self) -> &'static str {
        match self {
            Self::Tokens => "tokens.json",
            Self::Cst => "cst.json",
            Self::Diagnostics => "diagnostics.json",
            Self::SemanticDiagnostics => "semantic-diagnostics.json",
            Self::Analysis => "analysis.json",
            Self::Interface => "interface.json",
            Self::SurfaceAst => "surface-ast.json",
            Self::ResolvedAst => "resolved-ast.json",
            Self::TypedHir => "typed-hir.json",
            Self::TypedInterface => "typed-interface.json",
            Self::CoreIr => "core-ir.json",
            Self::TypeScriptIr => "typescript-ir.json",
            Self::GeneratedModule => "generated-module.json",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Selection {
    stages: BTreeSet<Stage>,
}

impl Selection {
    pub(super) fn resolve(case: &Path, argument: Option<&str>) -> Result<Self, String> {
        match argument {
            Some(argument) => Self::explicit(argument),
            None => Self::existing(case),
        }
    }

    fn explicit(argument: &str) -> Result<Self, String> {
        let Some(value) = argument.strip_prefix("--only=") else {
            return Err(format!("unknown option {argument}"));
        };
        if value.is_empty() {
            return Err("--only requires at least one stage".to_owned());
        }

        let mut stages = BTreeSet::new();
        for name in value.split(',') {
            let stage =
                Stage::parse(name).ok_or_else(|| format!("unknown artifact stage `{name}`"))?;
            if !stages.insert(stage) {
                return Err(format!(
                    "artifact stage `{name}` is selected more than once"
                ));
            }
        }
        Ok(Self { stages })
    }

    fn existing(case: &Path) -> Result<Self, String> {
        let stages = STAGES
            .into_iter()
            .filter(|stage| case.join(stage.artifact_file()).is_file())
            .collect::<BTreeSet<_>>();
        if stages.is_empty() {
            return Err("case has no existing artifacts; pass --only=STAGE[,STAGE...]".to_owned());
        }
        Ok(Self { stages })
    }

    pub(super) fn contains(&self, stage: Stage) -> bool {
        self.stages.contains(&stage)
    }

    pub(super) fn requires_driver(&self) -> bool {
        self.stages.iter().any(|stage| {
            matches!(
                stage,
                Stage::TypedHir | Stage::CoreIr | Stage::TypeScriptIr | Stage::GeneratedModule
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn validates_explicit_stage_names_and_duplicates() {
        let selection = Selection::explicit("--only=tokens,typed-hir").unwrap();
        assert!(selection.contains(Stage::Tokens));
        assert!(selection.contains(Stage::TypedHir));
        assert!(selection.requires_driver());

        assert!(Selection::explicit("--only=").is_err());
        assert!(Selection::explicit("--only=unknown").is_err());
        assert!(Selection::explicit("--only=tokens,tokens").is_err());
        assert!(Selection::explicit("tokens").is_err());
    }

    #[test]
    fn discovers_only_artifacts_that_already_exist() {
        let case =
            std::env::temp_dir().join(format!("seseragi-writer-selection-{}", std::process::id()));
        let _ = fs::remove_dir_all(&case);
        fs::create_dir_all(&case).unwrap();
        fs::write(case.join("tokens.json"), "{}\n").unwrap();
        fs::write(case.join("semantic-diagnostics.json"), "{}\n").unwrap();

        let selection = Selection::existing(&case).unwrap();
        assert!(selection.contains(Stage::Tokens));
        assert!(selection.contains(Stage::SemanticDiagnostics));
        assert!(!selection.contains(Stage::TypedHir));
        assert!(!selection.requires_driver());

        fs::remove_dir_all(case).unwrap();
    }
}
