use super::super::model::load_project_execution_cases;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn preserves_the_root_execution_as_one_default_case() {
    let project = temp_project("root");
    write_pure_execution(&project, "root.stdout", "root.stderr");

    let cases = load_project_execution_cases(&project).unwrap();

    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, "default");
    assert_eq!(cases[0].directory, project);
    assert_eq!(cases[0].case.stdout, "root.stdout");
    fs::remove_dir_all(cases[0].directory.clone()).unwrap();
}

#[test]
fn loads_nested_cases_in_id_order_with_case_relative_snapshots() {
    let project = temp_project("nested");
    let zeta = project.join("executions/zeta");
    let alpha = project.join("executions/alpha");
    fs::create_dir_all(&zeta).unwrap();
    fs::create_dir_all(&alpha).unwrap();
    write_pure_execution(&zeta, "zeta.stdout", "zeta.stderr");
    write_pure_execution(&alpha, "alpha.stdout", "alpha.stderr");

    let cases = load_project_execution_cases(&project).unwrap();

    assert_eq!(
        cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "zeta"]
    );
    assert_eq!(cases[0].directory, alpha);
    assert_eq!(cases[0].case.stdout, "alpha.stdout");
    assert_eq!(cases[1].directory, zeta);
    assert_eq!(cases[1].case.stderr, "zeta.stderr");
    fs::remove_dir_all(project).unwrap();
}

#[test]
fn rejects_mixed_empty_and_malformed_nested_layouts() {
    let mixed = temp_project("mixed");
    write_pure_execution(&mixed, "stdout", "stderr");
    fs::create_dir_all(mixed.join("executions/case")).unwrap();
    assert!(load_project_execution_cases(&mixed)
        .unwrap_err()
        .contains("must not mix"));
    fs::remove_dir_all(mixed).unwrap();

    let empty = temp_project("empty");
    fs::create_dir_all(empty.join("executions")).unwrap();
    assert!(load_project_execution_cases(&empty)
        .unwrap_err()
        .contains("must not be empty"));
    fs::remove_dir_all(empty).unwrap();

    let malformed = temp_project("malformed");
    fs::create_dir_all(malformed.join("executions")).unwrap();
    fs::write(malformed.join("executions/not-a-case"), "invalid").unwrap();
    assert!(load_project_execution_cases(&malformed)
        .unwrap_err()
        .contains("must be case directories"));
    fs::remove_dir_all(malformed).unwrap();
}

fn write_pure_execution(directory: &Path, stdout: &str, stderr: &str) {
    fs::write(
        directory.join("execution.json"),
        format!(
            r#"{{
  "schema": 1,
  "kind": "pure-project-execution",
  "target": "node-process",
  "entry": {{ "module": "fixture/main", "export": "main" }},
  "invocation": {{
    "argument": "Unit",
    "pure": {{ "result": "json" }}
  }},
  "expected": {{
    "runtimeRequirements": [],
    "process": {{ "exitCode": 0 }},
    "stdout": "{stdout}",
    "stderr": "{stderr}"
  }}
}}"#
        ),
    )
    .unwrap();
}

fn temp_project(name: &str) -> PathBuf {
    let project = std::env::temp_dir().join(format!(
        "seseragi-project-execution-discovery-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&project);
    fs::create_dir_all(&project).unwrap();
    project
}
