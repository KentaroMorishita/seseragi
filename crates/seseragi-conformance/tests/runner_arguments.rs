use std::process::Command;

#[test]
fn rejects_multiple_roots_instead_of_silently_selecting_the_last() {
    let result = Command::new(env!("CARGO_BIN_EXE_seseragi-conformance"))
        .args([".", ".", "--list"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("only one repository ROOT"));
}

#[test]
fn rejects_zero_discovered_cases_in_json_mode() {
    let result = Command::new(env!("CARGO_BIN_EXE_seseragi-conformance"))
        .args([env!("CARGO_MANIFEST_DIR"), "--json"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["passed"], false);
    assert_eq!(report["failures"][0]["kind"], "discovery");
}

#[test]
fn discovers_real_cases_from_repository_root() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let result = Command::new(env!("CARGO_BIN_EXE_seseragi-conformance"))
        .arg(root)
        .args(["--list", "--json"])
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(!report["cases"]["moduleInterface"]
        .as_array()
        .unwrap()
        .is_empty());
}
