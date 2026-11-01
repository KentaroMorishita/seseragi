use seseragi_project::ProjectTarget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Target {
    Project(ProjectTarget),
    Native,
}

impl Target {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "native" => Ok(Self::Native),
            "process" | "web" => ProjectTarget::parse(value)
                .map(Self::Project)
                .map_err(|error| error.to_string()),
            _ => Err(format!(
                "unknown project target `{value}`; expected `process`, `web` or `native`"
            )),
        }
    }

    pub(crate) fn project(self) -> Option<ProjectTarget> {
        match self {
            Self::Project(target) => Some(target),
            Self::Native => None,
        }
    }
}
