#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvironmentPlan {
    bindings: Vec<EnvironmentBinding>,
}

impl EnvironmentPlan {
    pub(crate) fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub(super) fn new(bindings: Vec<EnvironmentBinding>) -> Self {
        Self { bindings }
    }

    pub(crate) fn bindings(&self) -> &[EnvironmentBinding] {
        &self.bindings
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvironmentBinding {
    field: String,
    adapter: HostAdapter,
}

impl EnvironmentBinding {
    pub(super) fn new(field: String, adapter: HostAdapter) -> Self {
        Self { field, adapter }
    }

    pub(crate) fn field(&self) -> &str {
        &self.field
    }

    pub(crate) fn adapter(&self) -> HostAdapter {
        self.adapter
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostAdapter {
    CaptureConsole,
    ProcessStdin,
}

impl HostAdapter {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "capture-console" => Ok(Self::CaptureConsole),
            "process-stdin" => Ok(Self::ProcessStdin),
            other => Err(format!(
                "run.json hostEnvironment adapter {other} is not supported"
            )),
        }
    }

    pub(super) fn service_type(self) -> &'static str {
        match self {
            Self::CaptureConsole => "Console",
            Self::ProcessStdin => "Stdin",
        }
    }
}
