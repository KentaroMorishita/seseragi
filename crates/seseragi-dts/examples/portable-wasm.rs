//! Import-free WASM smoke for the converter's real parser, renderer and reports.
use seseragi_dts::{convert_entry, ConversionInput, HostMetadata};

#[no_mangle]
pub extern "C" fn conversion_smoke() -> u32 {
    let input = ConversionInput {
        settings: include_str!(
            "../../../examples/spec/fixtures/projects/dts-basic-conversion/seseragi.bindings.toml"
        ),
        entry: "api",
        declaration: include_str!(
            "../../../examples/spec/fixtures/projects/dts-basic-conversion/host/index.d.ts"
        ),
        host: HostMetadata {
            manifest: include_str!(
                "../../../examples/spec/fixtures/projects/dts-basic-conversion/host/package.json"
            ),
            resolved_package: None,
        },
        previous_metadata: None,
    };
    let first = convert_entry(&input).unwrap();
    assert!(first.diagnostics.is_empty());
    let generated = first.generated.unwrap();
    assert_eq!(generated.source, include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/expected/fixture-api.ssrg"));
    let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
    assert_eq!(metadata["hostModule"]["exactIdentity"], "fixture-api@1.0.0");
    let next = convert_entry(&ConversionInput {
        previous_metadata: Some(&generated.metadata),
        ..input
    })
    .unwrap()
    .generated
    .unwrap();
    assert_eq!(next.metadata, generated.metadata);
    let report: serde_json::Value = serde_json::from_str(&next.report).unwrap();
    assert_eq!(report["added"], serde_json::json!([]));
    let error = convert_entry(&ConversionInput {
        declaration: "export declare function unsafe(value: any): any;",
        ..input
    })
    .unwrap();
    assert!(error.generated.is_none());
    assert_eq!(error.diagnostics[0].code, "SES-F0101");
    assert_eq!(error.diagnostics[0].file, "host/index.d.ts");
    1
}
