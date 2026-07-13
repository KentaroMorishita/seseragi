use serde::{Deserialize, Serialize};
use seseragi_syntax::{ByteSpan, InterfaceConstraint, InterfaceType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependencyInstance {
    pub identity: String,
    pub provider_module: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub type_identity: String,
    pub type_parameters: Vec<String>,
    pub head: InterfaceType,
    pub constraints: Vec<InterfaceConstraint>,
    pub origin: ByteSpan,
}
