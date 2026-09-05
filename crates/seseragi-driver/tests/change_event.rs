use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn change_snapshot_checked_is_optional_across_standard_metadata_and_source() {
    let source = r#"
import * as html from "std/web/html"
pub fn checked event: html.ChangeEvent -> Maybe<Bool> = event.checked
pub fn value event: html.ChangeEvent -> String = event.value
pub fn readChecked event: html.ChangeEvent -> String = match event.checked {
  Just checked -> show checked
  Nothing -> event.value
}
"#;
    let analysis = analyze_module(CompileInput::new("change.ssrg", "fixture/change", source));
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let event = analysis
        .standard_library
        .iter()
        .find(|item| item.identity == "std/web/html::ChangeEvent")
        .unwrap();
    assert!(
        event.description.contains("Just for checkbox/radio"),
        "{event:?}"
    );
    assert!(
        event.description.contains("Nothing for value controls"),
        "{event:?}"
    );
    compile_module(CompileInput::new("change.ssrg", "fixture/change", source)).unwrap();
    let old = source.replace("-> Maybe<Bool> = event.checked", "-> Bool = event.checked");
    assert!(
        compile_module(CompileInput::new("old.ssrg", "fixture/old", &old)).is_err(),
        "old Bool consumers must migrate explicitly"
    );
}
