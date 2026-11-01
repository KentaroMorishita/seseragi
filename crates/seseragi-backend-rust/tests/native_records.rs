#[path = "support/native.rs"]
mod support;

use std::process::Command;
use support::{emit, run, rust_id, Scratch};

#[test]
fn generic_struct_impl_methods_keep_impl_and_method_type_parameters() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/generic-struct-methods.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/generic-struct-methods.stdout")
            .trim_end(),
    );
}

#[test]
fn optional_field_access_preserves_presence_in_parameters_pipelines_and_matches() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/optional-field-access.ssrg"),
        "([Nothing, Just present, Nothing, Just present, Nothing, Just present, Nothing, Just present, Nothing, Just present], [absent, present])",
    );
}

#[test]
fn optional_field_access_after_spread_preserves_missing_present_and_overridden_fields() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/optional-field-spread.ssrg"),
        "[Nothing, Just present, Just fallback, Just present, Just override, Just override, Just present, Just present]",
    );
}

#[test]
fn optional_field_access_wraps_present_maybe_values_without_flattening() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/optional-field-nested-maybe.ssrg"),
        "([Nothing, Just Nothing, Just Just value, Just Nothing, Just Just value, Nothing, Just Nothing, Just Just value], [absent, present Nothing, present Just value])",
    );
}

#[test]
fn optional_field_access_evaluates_the_receiver_once_and_retains_the_selected_value() {
    let artifact = emit(
        "fn source -> { note ?: String } = { note: \"present\" }\npub fn probe -> Maybe<String> = (source ()).note\n",
    );
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    let start = generated
        .find(&format!("fn {}_direct(", rust_id("native/main::source")))
        .unwrap();
    let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
    generated.insert_str(body, " CALLS.with(|calls| calls.set(calls.get() + 1)); ");
    generated.push_str(&format!(
        r#"
thread_local! {{ static CALLS: Cell<usize> = const {{ Cell::new(0) }}; }}
fn main() {{
 let result = {}(vec![]).apply(V::Unit);
 assert!(result.tagged("std/prelude::Just"));
 assert_eq!(result.payload().render(), "present");
 CALLS.with(|calls| assert_eq!(calls.get(), 1));
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#,
        rust_id(&artifact.metadata.entry)
    ));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn record_spread_overrides_fields_in_source_order_and_keeps_other_fields() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/record-spread.ssrg"),
        "(Ren, Mio, Aoi, Engineer, Kyoto)",
    );
}

#[test]
fn record_spread_copies_optional_fields_only_when_present() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/record-spread-optional.ssrg"),
        "({name: Mio}, {name: Aoi, note: present}, {name: Mio, note: fallback}, {name: Aoi, note: present}, {name: Mio, note: present})",
    );
}

#[test]
fn record_spread_evaluates_each_source_once_in_order_with_fields() {
    let artifact = emit(
        r#"
fn before -> Int = 1
fn source -> { value: Int, other: Int } = { value: 2, other: 3 }
fn field -> Int = 4
fn later -> { value: Int } = { value: 5 }
fn after -> Int = 6
pub fn probe -> { before: Int, value: Int, other: Int, after: Int } = {
  before: before (), ...source (), value: field (), ...later (), after: after ()
}
"#,
    );
    let main = artifact.source.rfind("fn main()").unwrap();
    let mut generated = artifact.source[..main].to_owned();
    // Observe source functions without changing the generated record expression.
    for name in ["before", "source", "field", "later", "after"] {
        let start = generated
            .find(&format!(
                "fn {}_direct(",
                rust_id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + generated[start..].find("-> V {").unwrap() + "-> V {".len();
        generated.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({name:?})); "),
        );
    }
    generated.push_str(&format!(
        r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let result = {}(vec![]).apply(V::Unit);
 assert_eq!(result.render(), "{{after: 6, before: 1, other: 3, value: 5}}");
 TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["before", "source", "field", "later", "after"]));
 drop(result); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#,
        rust_id(&artifact.metadata.entry)
    ));
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&generated))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
