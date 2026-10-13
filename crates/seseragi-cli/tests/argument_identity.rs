use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

#[path = "../../test-support/temporary.rs"]
mod temporary;

struct Program(PathBuf);

impl Program {
    fn new(source: &str) -> Self {
        let root = temporary::temporary_directory("seseragi-argument-identity");
        fs::write(root.join("main.ssrg"), source).unwrap();
        Self(root)
    }

    fn run(&self, profile: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args([
                "run",
                "main.ssrg",
                "--profile",
                profile,
                "--diagnostic-format",
                "json",
            ])
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn rejects_before_execution(source: &str) {
    let program = Program::new(source);
    for profile in ["development", "release"] {
        let output = program.run(profile);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{profile}: {stderr}");
        assert!(stderr.contains("SES-T0101"), "{profile}: {stderr}");
        assert!(
            output.stdout.is_empty(),
            "invalid arguments must not execute"
        );
    }
}

#[test]
fn primitive_nominal_argument_confusion_fails_before_a_runtime_defect() {
    for (name, literal, field_type, field_value, expected) in [
        ("Bool", "True", "Int", "1", "1\n"),
        ("Int", "1", "Bool", "True", "True\n"),
        ("Float", "(-1.0)", "Bool", "True", "True\n"),
        ("Char", "'a'", "Bool", "True", "True\n"),
        ("String", "\"ok\"", "Bool", "True", "True\n"),
        ("Unit", "()", "Bool", "True", "True\n"),
    ] {
        let declaration = format!(
            "struct {name} {{value: {field_type}}}\nfn read value: {name} -> {field_type} = value.value\n"
        );
        rejects_before_execution(&format!(
            "{declaration}pub effect fn main = do {{let selected = read {literal}; println \"unreachable\"}}\n"
        ));
        let valid = Program::new(&format!(
            "{declaration}pub effect fn main = do {{println (show (read ({name} {{value: {field_value}}})))}}\n"
        ));
        for profile in ["development", "release"] {
            let output = valid.run(profile);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        }
    }
}

#[test]
fn mixed_generic_arguments_fail_in_both_orders_without_a_trait_constraint() {
    for (left, right) in [("True", "Bool {value: 1}"), ("Bool {value: 1}", "True")] {
        rejects_before_execution(&format!(
            "struct Bool {{value: Int}}\nfn first<A> left: A -> right: A -> A = left\npub effect fn main = do {{let selected = first ({left}) ({right}); println \"unreachable\"}}\n"
        ));
    }
}

#[test]
fn effect_bind_keeps_the_success_value_identity_under_a_nominal_shadow() {
    let declaration = "struct Bool {value: Int}\nfn first<A> left: A -> right: A -> A = left\n";
    let valid = Program::new(&format!(
        "{declaration}pub effect fn main = do {{flag <- succeed True; if first True flag then println \"true\" else println \"false\"}}\n"
    ));
    for profile in ["development", "release"] {
        let output = valid.run(profile);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "true\n");
    }
    rejects_before_execution(&format!(
        "{declaration}pub effect fn main = do {{flag <- succeed (Bool {{value: 1}}); if flag then println \"unreachable\" else println \"unreachable\"}}\n"
    ));
    rejects_before_execution(&format!(
        "{declaration}pub effect fn main = do {{flag <- succeed (Bool {{value: 1}}); let selected = first True flag; println \"unreachable\"}}\n"
    ));
}
