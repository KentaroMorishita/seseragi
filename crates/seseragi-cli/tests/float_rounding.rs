use std::path::Path;
use std::process::Command;

#[test]
fn runs_canonical_float_rounding_boundaries() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/compile/float-rounding-boundaries.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&fixture)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(fixture.with_extension("stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
