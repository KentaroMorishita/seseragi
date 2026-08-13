use crate::TargetId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectTarget {
    Process,
    Web,
}

impl ProjectTarget {
    pub const ALL: [Self; 2] = [Self::Process, Self::Web];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::Web => "web",
        }
    }

    pub fn parse(value: &str) -> Result<Self, TargetSelectionError> {
        match value {
            "process" => Ok(Self::Process),
            "web" => Ok(Self::Web),
            _ => Err(TargetSelectionError::UnknownTarget(value.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectCommand {
    Run,
    Build,
    Dev,
}

impl ProjectCommand {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Build => "build",
            Self::Dev => "dev",
        }
    }

    const fn default_target(self) -> ProjectTarget {
        match self {
            Self::Run | Self::Build => ProjectTarget::Process,
            Self::Dev => ProjectTarget::Web,
        }
    }

    const fn supports(self, target: ProjectTarget) -> bool {
        match self {
            Self::Run => matches!(target, ProjectTarget::Process),
            Self::Build => true,
            Self::Dev => matches!(target, ProjectTarget::Web),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetSelectionSource {
    Invocation,
    Manifest,
    Capabilities,
    CommandDefault,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetSelection {
    pub target: ProjectTarget,
    pub source: TargetSelectionSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetSelectionError {
    UnknownTarget(String),
    UnsupportedCommandTarget {
        command: ProjectCommand,
        target: ProjectTarget,
    },
    NoCompatibleTarget,
    AmbiguousCapabilities(Vec<ProjectTarget>),
}

impl std::fmt::Display for TargetSelectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTarget(target) => write!(
                formatter,
                "unknown project target `{target}`; expected `process` or `web`"
            ),
            Self::UnsupportedCommandTarget { command, target } => write!(
                formatter,
                "{} does not support the `{}` target",
                command.as_str(),
                target.as_str()
            ),
            Self::NoCompatibleTarget => {
                formatter.write_str("compiled program has no compatible project target")
            }
            Self::AmbiguousCapabilities(targets) => write!(
                formatter,
                "compiled program target is ambiguous: {}",
                target_names(targets)
            ),
        }
    }
}

impl std::error::Error for TargetSelectionError {}

/// Selects one logical project target before provider resolution.
///
/// `compatible_targets` is `None` when compiled capability metadata is not
/// available. An empty slice means metadata was available but no registered
/// target can satisfy it.
pub fn select_project_target(
    command: ProjectCommand,
    invocation: Option<ProjectTarget>,
    manifest: Option<&TargetId>,
    compatible_targets: Option<&[ProjectTarget]>,
) -> Result<TargetSelection, TargetSelectionError> {
    let manifest = manifest
        .map(|target| ProjectTarget::parse(target.as_str()))
        .transpose()?;
    let (target, source) = if let Some(target) = invocation {
        (target, TargetSelectionSource::Invocation)
    } else if let Some(target) = manifest {
        (target, TargetSelectionSource::Manifest)
    } else if let Some([target]) = compatible_targets {
        (*target, TargetSelectionSource::Capabilities)
    } else {
        let default = command.default_target();
        match compatible_targets {
            Some([]) => return Err(TargetSelectionError::NoCompatibleTarget),
            Some(targets) if !targets.contains(&default) => {
                return Err(TargetSelectionError::AmbiguousCapabilities(
                    targets.to_vec(),
                ));
            }
            _ => (default, TargetSelectionSource::CommandDefault),
        }
    };
    if !command.supports(target) {
        return Err(TargetSelectionError::UnsupportedCommandTarget { command, target });
    }
    Ok(TargetSelection { target, source })
}

fn target_names(targets: &[ProjectTarget]) -> String {
    if targets.is_empty() {
        "none".to_owned()
    } else {
        targets
            .iter()
            .map(|target| target.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(value: &str) -> TargetId {
        TargetId::new(value.to_owned())
    }

    #[test]
    fn applies_override_manifest_capability_and_command_precedence() {
        assert_eq!(
            select_project_target(
                ProjectCommand::Build,
                Some(ProjectTarget::Process),
                Some(&target("web")),
                Some(&ProjectTarget::ALL),
            )
            .unwrap(),
            TargetSelection {
                target: ProjectTarget::Process,
                source: TargetSelectionSource::Invocation,
            }
        );
        assert_eq!(
            select_project_target(
                ProjectCommand::Build,
                None,
                Some(&target("web")),
                Some(&ProjectTarget::ALL),
            )
            .unwrap()
            .source,
            TargetSelectionSource::Manifest
        );
        assert_eq!(
            select_project_target(
                ProjectCommand::Build,
                None,
                None,
                Some(&[ProjectTarget::Web]),
            )
            .unwrap(),
            TargetSelection {
                target: ProjectTarget::Web,
                source: TargetSelectionSource::Capabilities,
            }
        );
        assert_eq!(
            select_project_target(ProjectCommand::Build, None, None, Some(&ProjectTarget::ALL),)
                .unwrap(),
            TargetSelection {
                target: ProjectTarget::Process,
                source: TargetSelectionSource::CommandDefault,
            }
        );
    }

    #[test]
    fn rejects_adapter_ids_and_command_gaps() {
        assert!(matches!(
            select_project_target(
                ProjectCommand::Build,
                None,
                Some(&target("bun-process")),
                None,
            ),
            Err(TargetSelectionError::UnknownTarget(value)) if value == "bun-process"
        ));
        assert!(matches!(
            select_project_target(
                ProjectCommand::Run,
                Some(ProjectTarget::Web),
                None,
                Some(&[ProjectTarget::Web]),
            ),
            Err(TargetSelectionError::UnsupportedCommandTarget { .. })
        ));
        assert_eq!(
            select_project_target(
                ProjectCommand::Build,
                Some(ProjectTarget::Process),
                None,
                Some(&[ProjectTarget::Web]),
            )
            .unwrap()
            .target,
            ProjectTarget::Process
        );
    }

    #[test]
    fn keeps_the_phase_one_dev_policy_in_the_shared_resolver() {
        assert_eq!(
            select_project_target(ProjectCommand::Dev, None, None, None).unwrap(),
            TargetSelection {
                target: ProjectTarget::Web,
                source: TargetSelectionSource::CommandDefault,
            }
        );
        assert!(matches!(
            select_project_target(
                ProjectCommand::Dev,
                Some(ProjectTarget::Process),
                None,
                None,
            ),
            Err(TargetSelectionError::UnsupportedCommandTarget { .. })
        ));
    }
}
