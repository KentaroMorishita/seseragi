use std::path::Path;
use std::process::Command;

#[test]
fn executes_spec_let_polymorphism_and_preserves_dictionaries() {
    for name in [
        "let-polymorphism",
        "let-polymorphism-constraints",
        "constrained-callable-alias",
    ] {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../examples/spec/fixtures/compile/{name}.ssrg"));
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&fixture)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            std::fs::read_to_string(fixture.with_extension("stdout")).unwrap()
        );
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }
}
