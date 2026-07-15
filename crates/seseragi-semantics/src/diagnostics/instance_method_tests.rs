use super::semantic_diagnostics;

#[test]
fn reports_instance_method_body_type_mismatch() {
    let source = "\
trait Render<A> {
  fn render value: A -> String
}

instance Render<Int> {
  fn render value: Int -> String = 42
}
";

    let diagnostics = semantic_diagnostics("artifact/instance-body-mismatch/main.ssrg", source);

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "function.return-type-mismatch"
    );
    assert_eq!(
        &source[diagnostics.diagnostics[0].primary.start..diagnostics.diagnostics[0].primary.end],
        "42"
    );
}

#[test]
fn accepts_a_well_typed_instance_method_body() {
    let source = "\
trait Render<A> {
  fn render value: A -> String
}

instance Render<Int> {
  fn render value: Int -> String = \"number\"
}
";

    let diagnostics = semantic_diagnostics("artifact/instance-body-valid/main.ssrg", source);

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn resolves_instance_method_parameter_references_for_body_analysis() {
    let source = "\
trait Echo<A> {
  fn echo value: A -> A
}

instance Echo<Int> {
  fn echo value: Int -> Int = value
}
";

    let resolved = crate::resolve_module("artifact/instance-body-parameter/main.ssrg", source);
    let reference = resolved
        .references
        .iter()
        .find(|reference| {
            reference.namespace == crate::SymbolNamespace::Value && reference.spelling == "value"
        })
        .expect("instance method body retains its parameter reference");

    assert!(reference.target.is_some());
}
