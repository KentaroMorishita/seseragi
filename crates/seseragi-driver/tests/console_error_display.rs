use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn canonical_console_error_keeps_standard_display_and_existing_web_samples() {
    for source in [
        "pub fn message error: ConsoleError -> String = show error\npub fn detail error: ConsoleError -> String = debug error\n",
        include_str!("../../../examples/samples/html-components/main.ssrg"),
        include_str!("../../../examples/tour/lessons/web-link-image/main.ssrg"),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            compile_module(CompileInput::new("console.ssrg", "fixture/console", source).with_profile(profile)).unwrap();
        }
    }
}

#[test]
fn same_named_local_console_error_does_not_borrow_standard_display() {
    for method in ["show", "debug"] {
        let source = format!("type ConsoleError = | LocalError\npub fn message error: ConsoleError -> String = {method} error\n");
        let error =
            compile_module(CompileInput::new("local.ssrg", "fixture/local", &source)).unwrap_err();
        assert!(error
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "instance.missing"));
    }
}
