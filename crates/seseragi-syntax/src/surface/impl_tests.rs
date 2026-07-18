use super::*;

#[test]
fn parses_generic_impl_methods_and_operator_members_in_source_order() {
    let source = "struct Box<A> { value: A }\n\
impl<A> Box<A>\n\
where Show<A> {\n\
  pub fn copied self: Box<A> -> Box<A> = Box { value: self.value }\n\
  operator + self -> other: Box<A> -> Box<A> = other\n\
  fn unwrap self: Box<A> -> A = self.value\n\
}\n";
    let module = parse_surface_ast("main.ssrg", source);

    assert_eq!(module.declarations.len(), 2);
    let SurfaceDecl::Impl {
        type_parameters,
        target,
        constraints,
        members,
        ..
    } = &module.declarations[1]
    else {
        panic!("expected impl declaration");
    };

    assert_eq!(type_parameters, &[crate::TypeParameter::value("A")]);
    assert!(matches!(
        target,
        TypeRef::Named { name, arguments, .. }
            if name == "Box"
                && matches!(arguments.as_slice(), [TypeRef::Named { name, .. }] if name == "A")
    ));
    assert!(matches!(
        constraints.as_slice(),
        [SurfaceConstraint { name, arguments, .. }]
            if name == "Show"
                && matches!(arguments.as_slice(), [TypeRef::Named { name, .. }] if name == "A")
    ));
    assert_eq!(members.len(), 3);

    let SurfaceImplMember::Method { visibility, method } = &members[0] else {
        panic!("expected first member to be a method");
    };
    assert_eq!(*visibility, Visibility::Public);
    assert_eq!(method.name, "copied");
    assert!(matches!(method.body, Some(SurfaceExpr::Struct { .. })));

    let SurfaceImplMember::Operator {
        visibility,
        spelling,
        parameters,
        body,
        ..
    } = &members[1]
    else {
        panic!("expected second member to be an operator");
    };
    assert_eq!(*visibility, Visibility::Private);
    assert_eq!(spelling, "+");
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].name, "other");
    assert!(matches!(body, Some(SurfaceExpr::Name { name, .. }) if name == "other"));

    let SurfaceImplMember::Method { visibility, method } = &members[2] else {
        panic!("expected third member to be a method");
    };
    assert_eq!(*visibility, Visibility::Private);
    assert_eq!(method.name, "unwrap");
    assert!(matches!(method.body, Some(SurfaceExpr::Member { .. })));
}
