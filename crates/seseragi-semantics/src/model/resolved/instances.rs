use serde::{Deserialize, Serialize};
use seseragi_syntax::ByteSpan;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependencyInstance {
    pub identity: String,
    pub provider_module: String,
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub type_identity: String,
    pub origin: ByteSpan,
}
