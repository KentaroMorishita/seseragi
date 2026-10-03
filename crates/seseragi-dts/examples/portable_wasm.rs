//! Test adapter: execute the same declaration-text converter in a browser WASM module.
//! This example is not a second parser or a Playground workspace protocol.

use serde::Deserialize;
use seseragi_dts::{Converter, DeclarationInput, HostModuleIdentity};
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    settings: String,
    entry: String,
    declaration: String,
    host_module: HostModuleIdentity,
    #[serde(default)]
    previous_metadata: Option<String>,
}

#[wasm_bindgen]
pub fn convert_declaration(request: &str) -> String {
    fn convert(request: &str) -> Result<seseragi_dts::ConversionResult, String> {
        let request: Request = serde_json::from_str(request).map_err(|error| error.to_string())?;
        Converter::new(&request.settings)
            .and_then(|converter| {
                converter.convert(DeclarationInput {
                    entry: &request.entry,
                    source: &request.declaration,
                    host_module: request.host_module,
                    previous_metadata: request.previous_metadata.as_deref(),
                })
            })
            .map_err(|error| error.to_string())
    }
    match convert(request) {
        Ok(result) => serde_json::json!({"status": "success", "result": result}),
        Err(message) => serde_json::json!({"status": "error", "message": message}),
    }
    .to_string()
}
