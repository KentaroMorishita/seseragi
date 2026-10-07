use std::path::Path;
use std::process::Command;

#[test]
fn executes_direct_nested_partial_and_lexical_callable_fields() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/compile/callable-record-fields.ssrg");
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
    assert!(output.stderr.is_empty());
}
