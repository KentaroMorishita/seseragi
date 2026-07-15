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

#[test]
fn accepts_a_trait_call_backed_by_the_instance_constraint_scope() {
    let source = "\
trait Ready<A> {
  fn ready value: A -> String
}

trait Inspect<A> {
  fn inspect value: A -> String
}

instance<T> Inspect<Maybe<T>>
where Ready<T> {
  fn inspect value: Maybe<T> -> String =
    match value {
      Nothing -> \"empty\"
      Just item -> ready item
    }
}
";

    let diagnostics = semantic_diagnostics("artifact/scoped-instance-evidence/main.ssrg", source);

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn accepts_a_trait_method_with_a_satisfied_method_constraint() {
    let source = "\
type Badge = | Active
trait Labeled<A> { fn label value: A -> String }
trait Render<A> {
  fn render value: A -> String where Labeled<A>
}
instance Labeled<Badge> { fn label value: Badge -> String = \"active\" }
instance Render<Badge> {
  fn render value: Badge -> String where Labeled<Badge> = label value
}
fn status value: Badge -> String = render value
";

    let diagnostics = semantic_diagnostics("artifact/method-constraint-valid/main.ssrg", source);

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn reports_missing_method_constraint_evidence_at_the_call_site() {
    let source = "\
type Badge = | Active
trait Labeled<A> { fn label value: A -> String }
trait Render<A> {
  fn render value: A -> String where Labeled<A>
}
instance Render<Badge> {
  fn render value: Badge -> String where Labeled<Badge> = \"active\"
}
fn status value: Badge -> String = render value
";

    let diagnostics = semantic_diagnostics("artifact/method-constraint-missing/main.ssrg", source);

    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        diagnostics.diagnostics[0].code, "SES-T0101",
        "{diagnostics:#?}"
    );
    assert_eq!(diagnostics.diagnostics[0].message_key, "instance.missing");
}
