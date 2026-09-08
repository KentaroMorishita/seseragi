use serde::{Deserialize, Serialize};

/// Compiler profile selection is independent of execution target and providers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildProfile {
    #[default]
    Development,
    Release,
}

impl BuildProfile {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "development" => Ok(Self::Development),
            "release" => Ok(Self::Release),
            _ => Err(format!(
                "unknown profile `{value}`; expected `development` or `release`"
            )),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Release => "release",
        }
    }
}
