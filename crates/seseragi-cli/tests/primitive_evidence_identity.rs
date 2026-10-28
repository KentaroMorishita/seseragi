use std::{fs, path::PathBuf, process::Command};
#[path = "../../test-support/temporary.rs"]
mod temporary;
struct Program(PathBuf);
impl Program {
    fn new(source: &str) -> Self {
        let root = temporary::temporary_directory("seseragi-primitive-evidence");
        fs::write(root.join("main.ssrg"), source).unwrap();
        fs::write(
            root.join("host.mjs"),
            "export const echo = value => value;\n",
        )
        .unwrap();
        Self(root)
    }
    fn run(&self, profile: &str) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["run", "main.ssrg", "--profile", profile])
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn primitive_show_and_interpolation_ignore_nominal_shadow_but_nominal_values_use_their_dictionary()
{
    for (name, literal, expected, field_type, nominal_value) in [
        ("Bool", "True", "True", "String", "\"nominal\""),
        ("Int", "1", "1", "Bool", "True"),
        ("Float", "(-1.0)", "-1.0", "Bool", "True"),
        ("Char", "'a'", "a", "Bool", "True"),
        ("String", "\"ok\"", "ok", "Bool", "True"),
        ("Unit", "()", "()", "Bool", "True"),
    ] {
        let source = format!("struct {name} deriving Show {{value: {field_type}}}\nforeign \"typescript\" from \"./host.mjs\" {{\n pure fn echo<A> value: A -> A\n}}\nfn keep<A> value: A -> A where Show<A> = {{let rendered = show value; value}}\npub effect fn main = do {{println (show {literal}); println (show (keep {literal})); println `${{{literal}}}`; println (show (echo {literal})); println (show ({name} {{value: {nominal_value}}}))}}\n");
        let program = Program::new(&source);
        for profile in ["development", "release"] {
            let output = program.run(profile);
            let lines = output.lines().collect::<Vec<_>>();
            assert_eq!(
                &lines[..4],
                [expected, expected, expected, expected],
                "{name}: {profile}"
            );
            assert_eq!(lines.len(), 5, "{name}: {output}");
            assert!(
                lines[4].contains(if name == "Bool" {
                    "value: nominal"
                } else {
                    "value: True"
                }),
                "nominal dictionary not selected: {name}: {output}"
            );
        }
    }
}

#[test]
fn literal_array_evidence_keeps_identity_under_derived_and_parameter_shadows() {
    let program = Program::new("struct Array<A> deriving Show {value: A}\nfn tricky<Array> value: Array -> String where Show<Array> = show [1, 2]\npub effect fn main = do {println (show [1, 2]); println (show [Array {value: True}]); println (tricky True)}\n");
    for profile in ["development", "release"] {
        assert_eq!(
            program.run(profile),
            "[1, 2]\n[Array { value: True }]\n[1, 2]\n",
            "{profile}"
        );
    }
}

#[test]
fn primitive_equality_under_derived_shadow_uses_standard_values_and_nominal_equality_uses_fields() {
    for (name, left, right, same, field_type, field_left, field_right) in [
        ("Bool", "True", "False", false, "Int", "1", "2"),
        ("Int", "1", "2", false, "Bool", "True", "False"),
        ("Char", "'a'", "'b'", false, "Bool", "True", "False"),
        ("String", "\"a\"", "\"b\"", false, "Bool", "True", "False"),
        ("Unit", "()", "()", true, "Bool", "True", "False"),
    ] {
        let source = format!("struct {name} deriving Eq {{value: {field_type}}}\npub effect fn main = do {{if {left} == {right} then println \"equal\" else println \"unequal\"; if {name} {{value: {field_left}}} == {name} {{value: {field_right}}} then println \"nominal wrong\" else println \"nominal correct\"}}\n");
        let program = Program::new(&source);
        for profile in ["development", "release"] {
            assert_eq!(
                program.run(profile),
                format!(
                    "{}\nnominal correct\n",
                    if same { "equal" } else { "unequal" }
                ),
                "{name}: {profile}"
            );
        }
    }
}

#[test]
fn primitive_dictionary_is_not_captured_by_a_same_name_generic_constraint() {
    let program = Program::new("struct Bool deriving Show {value: Int}\nfn tricky<Bool> value: Bool -> String where Show<Bool> = show True\npub effect fn main = do {println (tricky (Bool {value: 1}))}\n");
    for profile in ["development", "release"] {
        assert_eq!(program.run(profile), "True\n", "{profile}");
    }
}

#[test]
fn composite_display_keeps_primitive_and_nominal_field_dictionaries_distinct() {
    let program = Program::new("struct Float deriving Show {value: Bool}\npub effect fn main = do {println (show [(-1.0)]); println (show ((-1.0), True)); println (show {x: (-1.0)}); println (show [Float {value: True}]); println (show (Float {value: True}, True)); println (show {x: Float {value: True}})}\n");
    for profile in ["development", "release"] {
        let output = program.run(profile);
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 6, "{output}");
        for line in &lines[..3] {
            assert!(
                line.contains("-1.0") && !line.contains("value: False"),
                "{output}"
            );
        }
        for line in &lines[3..] {
            assert!(line.contains("value: True"), "{output}");
        }
    }
}

#[test]
fn imported_reexported_scalar_return_preserves_the_primitive_under_a_caller_binder() {
    let original = concat!(
        "import {again} from \"./facade\"\n",
        "fn tricky<Bool> value: Bool -> String where Show<Bool> = show (again True)\n",
        "pub effect fn main = do {println (tricky 1)}\n",
    );
    let alias = concat!(
        "import {again} from \"./facade\"\n",
        "alias Rendered = String\n",
        "fn tricky<String> value: String -> Rendered where Show<String> = show (again \"ok\")\n",
        "pub effect fn main = println (tricky 1)\n",
    );
    for (input, expected) in [(original, "True\n"), (alias, "ok\n")] {
        let program = Program::new(input);
        let source = program.0.join("src");
        let host = program.0.join("host");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&host).unwrap();
        fs::rename(program.0.join("main.ssrg"), source.join("main.ssrg")).unwrap();
        fs::rename(program.0.join("host.mjs"), host.join("host.mjs")).unwrap();
        fs::write(program.0.join("seseragi.toml"), "[package]\nname = \"fixture/primitive-evidence-import\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[layout]\nsource = \"src\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[foreign.typescript]\nresolver = \"node\"\nmanifest = \"host/package.json\"\nlockfile = \"host/bun.lock\"\n").unwrap();
        fs::write(
            host.join("package.json"),
            "{\"name\":\"primitive-evidence-host\",\"version\":\"1.0.0\",\"type\":\"module\"}\n",
        )
        .unwrap();
        fs::write(
            host.join("bun.lock"),
            "{\"lockfileVersion\":1,\"packages\":{}}\n",
        )
        .unwrap();
        fs::write(source.join("foreign.ssrg"), "pub foreign \"typescript\" from \"../host/host.mjs\" {\n pure fn echo<A> value: A -> A\n}\n").unwrap();
        fs::write(
            source.join("facade.ssrg"),
            "pub import {echo as again} from \"./foreign\"\n",
        )
        .unwrap();
        let lock = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update", "."])
            .current_dir(&program.0)
            .output()
            .unwrap();
        assert!(
            lock.status.success(),
            "{}",
            String::from_utf8_lossy(&lock.stderr)
        );
        for profile in ["development", "release"] {
            let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["run", ".", "--profile", profile])
                .current_dir(&program.0)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{profile}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{profile}"
            );
        }
    }
}
