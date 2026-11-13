use std::process::{Command, Output};

fn seseragi(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn the_overview_lists_every_command_with_a_summary() {
    for arguments in [&["--help"][..], &["-h"], &["help"]] {
        let output = seseragi(arguments);
        assert_eq!(output.status.code(), Some(0), "{arguments:?}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in [
            "Usage: seseragi <command> [arguments]",
            "  new           Create a project from a template",
            "  run           Compile and run a program",
            "  format        Format source files",
            "  lock update   Write the package's seseragi.lock",
            "  help          Show help for a command",
        ] {
            assert!(
                stdout.contains(line),
                "{arguments:?} missing {line:?}:\n{stdout}"
            );
        }
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn every_command_explains_its_arguments_and_options() {
    for command in [
        &["new"][..],
        &["run"],
        &["build"],
        &["dev"],
        &["test"],
        &["benchmark"],
        &["doc"],
        &["lint"],
        &["format"],
        &["lock", "update"],
        &["dts", "convert"],
        &["experimental", "build-native"],
    ] {
        for flag in ["--help", "-h"] {
            let mut arguments = command.to_vec();
            arguments.push(flag);
            let output = seseragi(&arguments);
            assert_eq!(output.status.code(), Some(0), "{arguments:?}");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("Usage:\n  seseragi "),
                "{arguments:?}:\n{stdout}"
            );
            assert!(stdout.contains("Options:\n"), "{arguments:?}:\n{stdout}");
            assert!(output.stderr.is_empty(), "{arguments:?}");
        }
    }
    let run = String::from_utf8_lossy(&seseragi(&["help", "run"]).stdout).into_owned();
    assert!(run.contains("  --signal-mode cancel|forward\n"), "{run}");
    assert!(run.contains("      Default: cancel\n"), "{run}");
    assert!(
        run.contains("Examples:\n  seseragi run hello.ssrg\n"),
        "{run}"
    );
    assert_eq!(
        String::from_utf8_lossy(&seseragi(&["run", "app.ssrg", "--help"]).stdout),
        run
    );
}

#[test]
fn an_unknown_command_names_the_closest_one() {
    let output = seseragi(&["fromat", "x.ssrg"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "seseragi: unknown command `fromat`; did you mean `format`? Run `seseragi --help` for all commands\n"
    );
    assert!(output.stdout.is_empty());

    let output = seseragi(&["frobnicate"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "seseragi: unknown command `frobnicate`; run `seseragi --help` for all commands\n"
    );

    let output = seseragi(&["help", "tset"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("did you mean `test`?"));
}
