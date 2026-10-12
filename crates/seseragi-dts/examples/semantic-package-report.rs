//! Inspect real declarations using the normal frontend/projection, without wrapper catalogs.
use seseragi_dts::{convert_entry, ConversionInput, HostMetadata};
use seseragi_interop::SemanticCompatibilityReport;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let [declaration_path, manifest_path, specifier] = args.as_slice() else {
        return Err(
            "usage: semantic-package-report <declaration.d.ts> <package.json> <specifier>".into(),
        );
    };
    let declaration = std::fs::read_to_string(declaration_path)?;
    let manifest = std::fs::read_to_string(manifest_path)?;
    let settings = format!("schema = 1\n[entries.api]\ndeclaration = 'declaration.d.ts'\noutput = 'api'\nspecifier = {specifier:?}\nevaluation = 'task'\n");
    let converted = convert_entry(&ConversionInput {
        settings: &settings,
        entry: "api",
        declaration: &declaration,
        host: HostMetadata {
            manifest: &manifest,
            resolved_package: Some(&manifest),
        },
        previous_metadata: None,
    })?;
    let interpretation = converted
        .semantic
        .as_ref()
        .map(SemanticCompatibilityReport::from_graph)
        .transpose()?;
    // A frontend that found no public surface is not evidence of compatibility.
    let surface_status = if interpretation
        .as_ref()
        .is_none_or(|report| report.symbols.is_empty())
    {
        "uninterpreted-public-surface"
    } else {
        "interpreted-with-explicit-boundaries"
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": 1, "kind": "seseragi-real-package-interop-report", "specifier": specifier,
            "package": serde_json::from_str::<serde_json::Value>(&manifest)?, "surfaceStatus": surface_status,
            "interpretation": interpretation, "conversion": converted
        }))?
    );
    Ok(())
}
