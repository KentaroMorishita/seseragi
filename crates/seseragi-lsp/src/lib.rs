mod capabilities;
mod diagnostics;
mod features;
mod model;
mod protocol;
mod server;

pub use server::run;

pub const SERVER_NAME: &str = "seseragi-lsp";
pub const SERVER_VERSION: &str = seseragi_release::TOOLCHAIN_VERSION;
pub const PROTOCOL_VERSION: u32 = 1;
pub const ANALYSIS_SCHEMA_VERSION: u32 = 1;
