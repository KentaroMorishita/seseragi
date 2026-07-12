use super::{compile::compile_project_compile_case, model::load_project_compile_case};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn compiles_a_closed_project_descriptor_with_stable_source_labels() {
    let case = temp_case("valid");
    write_case(
        &case,
        r#"{
  "schema": 1,
  "kind": "closed-compile",
  "expectedOrder": ["fixture/domain", "fixture/main"],
  "modules": [
    { "id": "fixture/domain", "source": "src/domain.ssrg", "output": "dist/domain.js", "artifacts": "artifacts/domain", "imports": [] },
    { "id": "fixture/main", "source": "src/main.ssrg", "output": "dist/main.js", "artifacts": "artifacts/main", "imports": [{ "specifier": "./domain", "module": "fixture/domain" }] }
  ]
}"#,
        [
            (
                "src/domain.ssrg",
                "pub fn increment value: Int -> Int = value + 1\n",
            ),
            (
                "src/main.ssrg",
                "import { increment } from \"./domain\"\npub fn run value: Int -> Int = increment value\n",
            ),
        ],
    );

    let compiled = compile_project_compile_case(&case).unwrap();
    assert_eq!(compiled.compiled.order, compiled.descriptor.expected_order);
    let main = compiled.compiled.modules.get("fixture/main").unwrap();
    assert_eq!(main.generated.metadata.outputs.typescript, "dist/main.ts");
    assert_eq!(main.generated.source_map.file, "dist/main.ts");
    assert_eq!(main.typed_hir.source, "main.ssrg");

    fs::remove_dir_all(case).unwrap();
}

#[test]
fn rejects_ambiguous_descriptor_paths_and_imports() {
    let case = temp_case("invalid");
    write_case(
        &case,
        r#"{
  "schema": 1,
  "kind": "closed-compile",
  "expectedOrder": ["fixture/main"],
  "modules": [
    { "id": "fixture/main", "source": "src/main.ssrg", "output": "dist/main.js", "artifacts": "artifacts/main", "imports": [{ "specifier": "./domain", "module": "fixture/main" }, { "specifier": "./domain", "module": "fixture/main" }] }
  ]
}"#,
        [("src/main.ssrg", "pub let answer: Int = 42\n")],
    );

    assert!(load_project_compile_case(&case)
        .unwrap_err()
        .contains("duplicate or empty import specifier"));
    fs::remove_dir_all(case).unwrap();
}

#[test]
fn rejects_noncanonical_or_non_esm_generated_output_paths() {
    let case = temp_case("invalid-output");
    write_case(
        &case,
        r#"{
  "schema": 1,
  "kind": "closed-compile",
  "expectedOrder": ["fixture/main"],
  "modules": [
    { "id": "fixture/main", "source": "src/main.ssrg", "output": "../dist/main.js", "artifacts": "artifacts/main", "imports": [] }
  ]
}"#,
        [("src/main.ssrg", "pub let answer: Int = 42\n")],
    );

    assert!(load_project_compile_case(&case)
        .unwrap_err()
        .contains("output must be a canonical relative path"));
    fs::remove_dir_all(&case).unwrap();
}

fn temp_case(name: &str) -> PathBuf {
    let case = std::env::temp_dir().join(format!(
        "seseragi-project-compile-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&case);
    fs::create_dir_all(&case).unwrap();
    case
}

fn write_case<const N: usize>(case: &Path, descriptor: &str, sources: [(&str, &str); N]) {
    fs::write(case.join("project.json"), descriptor).unwrap();
    for (path, source) in sources {
        let path = case.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}
