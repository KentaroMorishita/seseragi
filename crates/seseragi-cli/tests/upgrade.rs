//! `seseragi upgrade` without the network: SESERAGI_VERSION names the release
//! instead of the latest one, as it does for the install scripts.

use std::process::Command;

const CURRENT: &str = env!("CARGO_PKG_VERSION");

fn upgrade(version: &str, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("upgrade")
        .args(arguments)
        .env("SESERAGI_VERSION", version)
        .output()
        .unwrap()
}

#[test]
fn the_installed_release_is_up_to_date() {
    let output = upgrade(CURRENT, &["--check"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("seseragi {CURRENT} is up to date\n")
    );
}

#[test]
fn check_reports_a_newer_release_without_installing() {
    let output = upgrade("v999.0.0", &["--check"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "seseragi 999.0.0 is available (installed: {CURRENT})\nRun `seseragi upgrade` to install it.\n"
        )
    );
}

#[test]
fn a_build_outside_the_install_layout_is_left_alone() {
    // Test binaries live in Cargo's target directory, not `<root>/bin`.
    let output = upgrade("999.0.0", &[]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("was not installed by the install script"),
        "{stderr}"
    );
    assert!(
        stderr.contains("releases/latest/download/install"),
        "{stderr}"
    );
}

#[test]
fn rejects_unknown_arguments_and_versions() {
    let output = upgrade(CURRENT, &["--force"]);
    assert_eq!(output.status.code(), Some(2));
    let output = upgrade("latest", &["--check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid release version"));
}
