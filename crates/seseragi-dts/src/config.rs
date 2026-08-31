use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BindingsConfig {
    pub schema: u32,
    pub entries: BTreeMap<String, EntryConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EntryConfig {
    pub declaration: String,
    pub specifier: String,
    pub output: String,
    #[serde(default)]
    pub evaluation: Evaluation,
    #[serde(default)]
    pub symbols: BTreeMap<String, SymbolConfig>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Evaluation {
    Pure,
    #[default]
    Task,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SymbolConfig {
    #[serde(default)]
    pub local: Option<String>,
    #[serde(default)]
    pub mode: Option<Evaluation>,
    #[serde(default)]
    pub overloads: BTreeMap<String, OverloadConfig>,
    #[serde(default)]
    pub callbacks: BTreeMap<String, CallbackConfig>,
    #[serde(default)]
    pub parameters: BTreeMap<String, TypeOverride>,
    #[serde(default)]
    pub result: Option<TypeOverride>,
    #[serde(default)]
    pub union: Option<UnionConfig>,
    #[serde(default)]
    pub unsafe_any: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OverloadConfig {
    pub signature: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TypeOverride {
    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CallbackConfig {
    pub lifetime: CallbackLifetime,
    pub invocation: CallbackInvocation,
    pub concurrency: CallbackConcurrency,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub reentrant: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CallbackLifetime {
    DuringCall,
    UntilSettled,
    Retained,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CallbackInvocation {
    Sync,
    Promise,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CallbackConcurrency {
    Serialized,
    Parallel,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UnionConfig {
    pub discriminator: String,
    #[serde(default)]
    pub variants: BTreeMap<String, String>,
}
