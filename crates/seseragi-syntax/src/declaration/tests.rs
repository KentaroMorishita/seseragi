use crate::{
    parse_cst, parse_diagnostics, parse_module_interface, parse_surface_ast, DiagnosticSeverity,
    SurfaceDecl, Visibility,
};

#[test]
fn modifiers_attach_to_their_declaration_across_trivia_and_keep_metadata() {
    for public in ["", "pub ", "pub\n// visibility\n"] {
        for separator in [" ", "\n", "\n// keep me\n"] {
            let source = format!("fn previous -> Int = 0\n{public}deprecated \"Use 代替\\nnext\" since \"0.4.1-rc.1+build.2\"{separator}fn historical value: Int -> Int = value\npub fn next -> Int = 42\n");
            let diagnostics = parse_diagnostics("λ.ssrg", &source);
            assert!(
                !diagnostics
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == DiagnosticSeverity::Error),
                "{source}: {diagnostics:?}"
            );
            let surface = parse_surface_ast("λ.ssrg", &source);
            assert_eq!(surface.declarations.len(), 3, "{source}");
            let SurfaceDecl::Fn {
                name,
                visibility,
                span,
                ..
            } = &surface.declarations[1]
            else {
                panic!()
            };
            assert_eq!(name, "historical");
            assert_eq!(
                *visibility,
                if public.is_empty() {
                    Visibility::Private
                } else {
                    Visibility::Public
                }
            );
            assert_eq!(
                span.start,
                source
                    .find(if public.is_empty() {
                        "deprecated"
                    } else {
                        "pub"
                    })
                    .unwrap()
            );
            let metadata = &surface.deprecations[0];
            assert_eq!(metadata.declaration, *span);
            assert_eq!(metadata.deprecation.message, "Use 代替\nnext");
            assert_eq!(
                metadata.deprecation.since.as_deref(),
                Some("0.4.1-rc.1+build.2")
            );
            assert_eq!(
                &source[metadata.deprecation.origin.start..metadata.deprecation.origin.end],
                "deprecated \"Use 代替\\nnext\" since \"0.4.1-rc.1+build.2\""
            );
            let cst = parse_cst("λ.ssrg", &source);
            assert!(cst.errors.is_empty());
            assert_eq!(cst.root.children.len(), 3);
            assert!(cst.root.children[1]
                .children
                .iter()
                .any(|n| n.kind == "fn-decl"));
            assert!(cst.root.children[1]
                .children
                .iter()
                .any(|n| n.kind == "deprecation-clause"));
            let interface = parse_module_interface("λ.ssrg", &source);
            let export = interface.exports.iter().find(|e| e.name == "historical");
            if public.is_empty() {
                assert!(export.is_none());
            } else {
                assert_eq!(
                    export.unwrap().deprecation.as_ref(),
                    Some(&metadata.deprecation)
                );
            }
        }
    }
}

#[test]
fn named_top_level_shapes_keep_visibility_and_declaration_kind() {
    for declaration in [
        "let answer: Int = 41",
        "fn answer -> Int = 41",
        "effect fn answer = println \"41\"",
        "type Answer = | Answer",
        "newtype Answer = Int",
        "alias Answer = Int",
        "struct Answer { value: Int }",
        "trait Answer<A> { fn answer value: A -> A }",
        "opaque type Answer = | Answer",
        "opaque newtype Answer = Int",
        "opaque struct Answer { value: Int }",
        "operator infixl 6 <+> left: Int -> right: Int -> Int = left + right",
    ] {
        let source = format!("pub deprecated \"Use replacement\"\n{declaration}\n");
        assert!(
            !parse_diagnostics("model.ssrg", &source)
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error),
            "{source}"
        );
        let surface = parse_surface_ast("model.ssrg", &source);
        assert_eq!(surface.declarations.len(), 1, "{source}");
        assert_eq!(surface.deprecations.len(), 1, "{source}");
        let interface = parse_module_interface("model.ssrg", &source);
        // A compact Effect's public scheme is elaborated by semantics.
        if !declaration.starts_with("effect") && !declaration.starts_with("operator") {
            assert!(
                interface.exports.iter().any(|e| e.deprecation.is_some()),
                "{source}: {interface:?}"
            );
        }
    }
}

#[test]
fn malformed_or_disallowed_prefixes_never_succeed_as_missing_declarations() {
    for prefix in [
        "pub deprecated",
        "pub deprecated 41",
        "deprecated \"\"",
        "deprecated \"old\" since",
        "deprecated \"old\" since 1",
        "deprecated \"old\" since \"01.2.3\"",
        "deprecated \"old\" since \"v1.2.3\"",
        "deprecated \"old\" since \"1.2\"",
        "deprecated \"old\";",
        "deprecated \"old\" pub",
        "pub deprecated \"old\" deprecated \"again\"",
    ] {
        let source =
            format!("// λ\n{prefix}\nfn historical -> Int = 41\npub fn probe -> Int = 42\n");
        let diagnostics = parse_diagnostics("λ.ssrg", &source);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error),
            "{source}"
        );
        for d in diagnostics.diagnostics {
            assert!(source.get(d.primary.start..d.primary.end).is_some());
        }
    }
    for declaration in [
        "let (a, b): (Int, Int) = (1, 2)",
        "import { x } from \"./model\"",
        "impl Int {}",
        "instance Show<Int> {}",
        "rec { fn historical -> Int = 41 }",
        "foreign \"js\" from \"node:os\" {}",
    ] {
        let source = format!("deprecated \"old\" {declaration}\npub fn probe -> Int = 42\n");
        assert!(
            parse_diagnostics("main.ssrg", &source)
                .diagnostics
                .iter()
                .any(|d| d.severity == DiagnosticSeverity::Error),
            "{source}"
        );
    }
}

#[test]
fn ordinary_interfaces_do_not_gain_empty_metadata() {
    let json = serde_json::to_value(parse_module_interface(
        "model.ssrg",
        "pub fn answer -> Int = 42\n",
    ))
    .unwrap();
    assert!(json["exports"][0].get("deprecation").is_none());
}

#[test]
fn member_clauses_keep_their_own_ranges_and_interface_owner() {
    let source = concat!(
        "pub trait Render<A> {\n",
        "  fn current value: A -> String\n",
        "  deprecated \"Use current\" since \"0.61.0\"\n",
        "  fn old value: A -> String\n",
        "}\n",
        "pub struct Box { value: Int }\n",
        "impl Box {\n",
        "  pub fn read self: Box -> Int = self.value\n",
        "  pub deprecated \"Use read\" fn old self: Box -> Int = self.value\n",
        "}\n",
    );
    let diagnostics = parse_diagnostics("λ/api.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let surface = parse_surface_ast("λ/api.ssrg", source);
    assert_eq!(surface.deprecations.len(), 2);
    assert_eq!(surface.deprecations[0].deprecation.message, "Use current");
    assert_eq!(surface.deprecations[1].deprecation.message, "Use read");
    for entry in &surface.deprecations {
        assert!(source[entry.declaration.start..entry.declaration.end].contains("fn old"));
        assert!(
            source[entry.deprecation.origin.start..entry.deprecation.origin.end]
                .starts_with("deprecated \"Use ")
        );
    }
    let interface = parse_module_interface("λ/api.ssrg", source);
    let owner = |name: &str| {
        interface
            .exports
            .iter()
            .find(|export| export.name == name)
            .unwrap()
    };
    for (name, message) in [("Render", "Use current"), ("Box", "Use read")] {
        let owner = owner(name);
        assert!(owner.deprecation.is_none());
        assert_eq!(owner.methods.len(), 2);
        assert!(owner.methods[0].deprecation.is_none());
        let old = &owner.methods[1];
        assert_eq!(old.name, "old");
        let metadata = old.deprecation.as_ref().unwrap();
        assert_eq!(metadata.message, message);
        assert_eq!(metadata.source, "λ/api.ssrg");
    }
}

#[test]
fn foreign_clauses_survive_nested_member_projection_without_inheritance() {
    let source = concat!(
        "pub foreign \"typescript\" from \"./host.mjs\" {\n",
        "  deprecated \"Use NewHandle\" opaque type Handle\n",
        "  deprecated \"Use current\" pure fn old value: Int -> Int\n",
        "  pure fn current value: Int -> Int\n",
        "  namespace nested {\n",
        "    deprecated \"Use modern\" since \"0.61.0\" task fn old value: Int -> Int\n",
        "    pure value count: Int\n",
        "  }\n",
        "}\n",
    );
    let diagnostics = parse_diagnostics("λ/foreign.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let surface = parse_surface_ast("λ/foreign.ssrg", source);
    assert_eq!(surface.deprecations.len(), 3);
    let interface = parse_module_interface("λ/foreign.ssrg", source);
    for (name, message) in [
        ("Handle", "Use NewHandle"),
        ("old", "Use current"),
        ("nested.old", "Use modern"),
    ] {
        let export = interface
            .exports
            .iter()
            .find(|export| export.name == name)
            .unwrap();
        assert_eq!(export.deprecation.as_ref().unwrap().message, message);
        assert_eq!(
            export.deprecation.as_ref().unwrap().source,
            "λ/foreign.ssrg"
        );
    }
    for name in ["current", "nested.count"] {
        assert!(interface
            .exports
            .iter()
            .find(|export| export.name == name)
            .unwrap()
            .deprecation
            .is_none());
    }
}

#[test]
fn malformed_member_clauses_and_instance_notices_are_positioned_errors() {
    for prefix in [
        "deprecated \"\"",
        "deprecated \"old\" since \"1.2\"",
        "deprecated 41",
    ] {
        for owner in [
            "trait Render<A>",
            "impl Int",
            "pub foreign \"typescript\" from \"./host.mjs\"",
        ] {
            let member = if owner.contains("foreign") {
                "pure fn old value: Int -> Int"
            } else {
                "fn old value: Int -> Int"
            };
            let source = format!("// 🙂\n{owner} {{\n {prefix} {member}\n}}\n");
            let diagnostics = parse_diagnostics("bad.ssrg", &source);
            assert!(
                diagnostics
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == DiagnosticSeverity::Error),
                "{source}: {diagnostics:#?}"
            );
            for diagnostic in diagnostics.diagnostics {
                assert!(source
                    .get(diagnostic.primary.start..diagnostic.primary.end)
                    .is_some());
            }
        }
    }
    let source = "type Badge = | Active\ntrait Render<A> { fn old value: A -> Int }\ninstance Render<Badge> { deprecated \"Use current\" fn old value: Badge -> Int = 41 }\n";
    let diagnostics = parse_diagnostics("bad.ssrg", source);
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.message_key == "parser.invalid-declaration-modifiers"
                    && source[diagnostic.primary.start..diagnostic.primary.end]
                        .starts_with("deprecated")
            ),
        "{diagnostics:#?}"
    );
    assert!(parse_surface_ast("bad.ssrg", source)
        .deprecations
        .is_empty());
}
