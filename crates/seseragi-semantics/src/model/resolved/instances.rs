use serde::{Deserialize, Serialize};
use seseragi_syntax::{ByteSpan, InterfaceConstraint, InterfaceType, TypeParameter};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependencyInstance {
    pub identity: String,
    pub provider_module: String,
    pub trait_identity: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub argument_identities: Vec<String>,
    pub type_identity: Option<String>,
    pub type_parameters: Vec<TypeParameter>,
    pub head: InterfaceType,
    pub constraints: Vec<InterfaceConstraint>,
    pub origin: ByteSpan,
}
