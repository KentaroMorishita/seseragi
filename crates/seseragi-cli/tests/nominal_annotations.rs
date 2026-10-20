use std::{fs, path::PathBuf, process::Command};
#[path = "../../test-support/temporary.rs"]
mod temporary;

struct Project(PathBuf);
impl Project {
    fn new(main: &str, domain: Option<&str>) -> Self {
        let root = temporary::temporary_directory("seseragi-nominal-annotations");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("seseragi.toml"), "[package]\nname = \"fixture/nominal-annotations\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[layout]\nsource = \"src\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n").unwrap();
        fs::write(root.join("src/main.ssrg"), main).unwrap();
        if let Some(domain) = domain {
            fs::write(root.join("src/domain.ssrg"), domain).unwrap();
        }
        let project = Self(root);
        project.command(&["lock", "update", "."]);
        project
    }
    fn command(&self, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
    fn check_profiles(&self, expected: &str) {
        for profile in ["development", "release"] {
            assert_eq!(
                self.command(&["run", ".", "--profile", profile]),
                expected,
                "{profile}"
            );
        }
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn nominal_and_genuine_primitive_values_execute_locally_and_across_a_project_import() {
    for (name, payload, nominal, primitive, expected) in [
        ("Unit", "Int", "42", "()", "()"),
        ("Bool", "Int", "42", "True", "True"),
        ("Int", "Bool", "True", "7", "7"),
        ("Float", "Int", "42", "1.5", "1.5"),
        ("Char", "Int", "42", "'x'", "x"),
        ("String", "Int", "42", "\"hello\"", "hello"),
    ] {
        let declarations = format!("pub struct {name} {{value: {payload}}}\npub fn read value: {name} -> {payload} = value.value\npub let nominal: {name} = {name} {{value: {nominal}}}\npub effect fn genuine = succeed {primitive}\n");
        let body = "pub effect fn main = do {println (show (read nominal)); primitive <- genuine (); println (show primitive)}\n";
        let output = format!(
            "{}\n{expected}\n",
            if name == "Int" { "True" } else { "42" }
        );
        Project::new(&format!("{declarations}{body}"), None).check_profiles(&output);
        Project::new(
            &format!("import {{{name}, read, nominal, genuine}} from \"./domain\"\n{body}"),
            Some(&declarations),
        )
        .check_profiles(&output);
    }
}

#[test]
fn hidden_imported_unit_does_not_shadow_an_ordinary_unit_parameter() {
    let domain = "pub struct Unit {value: Int}\npub effect fn genuine = succeed ()\n";
    let main = "import { genuine } from \"./domain\"\nfn consume value: Unit -> Bool = True\npub effect fn main = do {let argument = (); let checked = consume argument; value <- genuine argument; println (show value)}\n";
    Project::new(main, Some(domain)).check_profiles("()\n");
}

#[test]
fn a_closed_unit_alias_keeps_the_intrinsic_unit_identity() {
    let domain = "pub alias MyUnit = Unit\n";
    let main = "import { MyUnit as Unit } from \"./domain\"\nfn consume value: Unit -> Bool = True\npub effect fn main = {let argument = (); println (show (consume argument))}\n";
    Project::new(main, Some(domain)).check_profiles("True\n");
}

#[test]
fn nominal_array_and_list_annotations_coexist_with_builtin_collection_values() {
    for (name, literal, prefix) in [("Array", "[1]", ""), ("List", "`[1]", "`")] {
        let declarations = format!("pub struct {name}<A> {{value: A}}\npub fn read<A> value: {name}<A> -> A = value.value\npub let nominal: {name}<Int> = {name} {{value: 42}}\npub effect fn genuine = succeed {literal}\n");
        let body = format!("pub effect fn main = do {{println (show (read nominal)); values <- genuine (); println (show (match values {{{prefix}[head, ...tail] -> head; {prefix}[] -> 0}}))}}\n");
        Project::new(&format!("{declarations}{body}"), None).check_profiles("42\n1\n");
        Project::new(
            &format!("import {{{name}, read, nominal, genuine}} from \"./domain\"\n{body}"),
            Some(&declarations),
        )
        .check_profiles("42\n1\n");
    }
}
