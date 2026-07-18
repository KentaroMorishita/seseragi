use super::*;
use seseragi_syntax::parse_unlinked_module_interface;

fn target(module: &str, source: &str) -> ModuleLinkTarget {
    let unlinked = parse_unlinked_module_interface(format!("{module}.ssrg"), module, source);
    ModuleLinkTarget::same_package(unlinked.header, unlinked.interface).unwrap()
}

#[test]
fn links_named_type_constructor_and_function_exports() {
    let domain = target(
        "fixture/game::domain",
        "pub type Hand =\n  | Rock\n  | Paper\n\npub fn decide first: Hand -> second: Hand -> Hand = first\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { Hand, Rock, decide as choose } from \"./domain\"\npub fn play hand: Hand -> Hand = choose hand Rock\n",
    );
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    let linked = link_module(main, &targets).unwrap();
    assert_eq!(linked.interface.dependencies.len(), 1);
    assert_eq!(
        linked.interface.dependencies[0].module,
        "fixture/game::domain"
    );
    assert_eq!(linked.interface.dependencies[0].imports.len(), 3);
    assert_eq!(linked.dependencies[0].imports.len(), 3);
    assert!(linked.dependencies[0].imports.iter().any(|import| matches!(
        import,
        LinkedImport::Symbol { local_name, export, .. }
            if local_name == "Hand" && export.namespace == "type"
    )));
    assert!(linked.dependencies[0].imports.iter().any(|import| matches!(
        import,
        LinkedImport::Symbol { local_name, export, .. }
            if local_name == "Rock" && export.declaration_kind.as_deref() == Some("constructor")
    )));
    assert!(linked.dependencies[0].imports.iter().any(|import| matches!(
        import,
        LinkedImport::Symbol { local_name, export, .. }
            if local_name == "choose" && export.declaration_kind.as_deref() == Some("function")
    )));
}

#[test]
fn one_named_newtype_import_introduces_type_and_constructor_namespaces() {
    let domain = target("fixture/game::domain", "pub newtype UserId = Int\n");
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { UserId } from \"./domain\"\npub let answer: Int = 42\n",
    );
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    let linked = link_module(main, &targets).unwrap();
    let imports = &linked.dependencies[0].imports;
    assert_eq!(imports.len(), 2);
    assert!(imports.iter().any(|import| matches!(
        import,
        LinkedImport::Symbol { local_name, export, .. }
            if local_name == "UserId" && export.namespace == "type"
    )));
    assert!(imports.iter().any(|import| matches!(
        import,
        LinkedImport::Symbol { local_name, export, .. }
            if local_name == "UserId"
                && export.namespace == "value"
                && export.constructor_of.as_deref() == Some("fixture/game::domain::UserId")
    )));
}

#[test]
fn does_not_expose_inherent_methods_as_named_value_imports() {
    let domain = target(
        "fixture/game::domain",
        "pub struct Box { value: Int }\nimpl Box { pub fn get self: Box -> Int = self.value }\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { get } from \"./domain\"\npub let answer: Int = 42\n",
    );
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    assert!(matches!(
        link_module(main, &targets).unwrap_err().as_slice(),
        [LinkError::MissingExport { name, .. }] if name == "get"
    ));
}

#[test]
fn links_namespace_and_operator_metadata_without_reading_dependency_bodies() {
    let support = target(
        "fixture/game::support",
        "pub operator infixl 6 <+>\n  left: Int -> right: Int -> Int =\n  left\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import * as support from \"./support\"\nimport { operator <+> } from \"./support\"\npub let answer: Int = 42\n",
    );
    let targets = BTreeMap::from([("./support".to_owned(), support)]);

    let linked = link_module(main, &targets).unwrap();
    assert!(linked.dependencies[0].header.is_some());
    assert!(matches!(
        &linked.dependencies[0].imports[0],
        LinkedImport::Namespace { local_name, module, .. }
            if local_name == "support" && module == "fixture/game::support"
    ));
    assert!(matches!(
        &linked.dependencies[1].imports[0],
        LinkedImport::Operator { spelling, operator, .. }
            if spelling == "<+>" && operator.precedence == 6
    ));
}

#[test]
fn reports_missing_targets_exports_and_duplicates_at_import_item_spans() {
    let domain = target(
        "fixture/game::domain",
        "pub fn decide value: Int -> Int = value\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { missing } from \"./domain\"\nimport { decide, decide } from \"./domain\"\nimport { other } from \"./absent\"\npub let answer: Int = 42\n",
    );
    let missing_span = main.imports[0].items[0].name_span;
    let duplicate_span = main.imports[1].items[1].name_span;
    let unresolved_span = main.imports[2].span;
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    let errors = link_module(main, &targets).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        LinkError::MissingExport { name, origin, .. }
            if name == "missing" && *origin == missing_span
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        LinkError::DuplicateImport { local_name, origin, .. }
            if local_name == "decide" && *origin == duplicate_span
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        LinkError::UnresolvedSpecifier { specifier, origin }
            if specifier == "./absent" && *origin == unresolved_span
    )));
}

#[test]
fn rejects_an_import_that_collides_with_a_local_declaration_namespace() {
    let domain = target(
        "fixture/game::domain",
        "pub fn decide value: Int -> Int = value\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { decide } from \"./domain\"\nfn decide value: Int -> Int = value\n",
    );
    let import_span = main.imports[0].items[0].name_span;
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    assert!(matches!(
        link_module(main, &targets).unwrap_err().as_slice(),
        [LinkError::DuplicateImport {
            namespace,
            local_name,
            origin,
        }] if namespace == "value" && local_name == "decide" && *origin == import_span
    ));
}

#[test]
fn distinguishes_same_package_private_exports_from_missing_names() {
    let domain = target(
        "fixture/game::domain",
        "fn hidden value: Int -> Int = value\n",
    );
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { hidden, typo } from \"./domain\"\npub let answer: Int = 42\n",
    );
    let hidden_span = main.imports[0].items[0].name_span;
    let hidden_declaration = domain
        .header()
        .unwrap()
        .names
        .iter()
        .find(|entry| entry.name == "hidden")
        .unwrap()
        .declaration;
    let targets = BTreeMap::from([("./domain".to_owned(), domain)]);

    let errors = link_module(main, &targets).unwrap_err();
    assert!(errors.iter().any(|error| matches!(
        error,
        LinkError::PrivateExport {
            source,
            name,
            namespaces,
            origin,
            declarations,
            ..
        } if source == "game::domain.ssrg"
            && name == "hidden"
            && namespaces == &["value"]
            && *origin == hidden_span
            && declarations == &[hidden_declaration]
    )));
    assert!(errors.iter().any(|error| matches!(
        error,
        LinkError::MissingExport { name, .. } if name == "typo"
    )));
}

#[test]
fn external_interfaces_do_not_reveal_private_declaration_names() {
    let unlinked = parse_unlinked_module_interface(
        "dependency/domain.ssrg",
        "dependency/game::domain",
        "fn hidden value: Int -> Int = value\n",
    );
    let target = ModuleLinkTarget::external(unlinked.interface);
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import { hidden } from \"dependency/game\"\npub let answer: Int = 42\n",
    );
    let targets = BTreeMap::from([("dependency/game".to_owned(), target)]);

    assert!(matches!(
        link_module(main, &targets).unwrap_err().as_slice(),
        [LinkError::MissingExport { name, .. }] if name == "hidden"
    ));
}

#[test]
fn external_namespace_dependencies_do_not_retain_a_private_header() {
    let unlinked = parse_unlinked_module_interface(
        "dependency/domain.ssrg",
        "dependency/game::domain",
        "pub fn visible value: Int -> Int = value\n",
    );
    let target = ModuleLinkTarget::external(unlinked.interface);
    let main = parse_unlinked_module_interface(
        "src/main.ssrg",
        "fixture/game::main",
        "import * as domain from \"dependency/game\"\npub let answer: Int = 42\n",
    );
    let targets = BTreeMap::from([("dependency/game".to_owned(), target)]);

    let linked = link_module(main, &targets).unwrap();
    assert!(linked.dependencies[0].header.is_none());
}

#[test]
fn rejects_a_shallow_target_that_omits_a_public_inferred_contract() {
    let unlinked = parse_unlinked_module_interface(
        "src/support.ssrg",
        "fixture/game::support",
        "pub effect fn greet name: String =\n  println \"hello\"\n",
    );

    assert!(matches!(
        ModuleLinkTarget::same_package(unlinked.header, unlinked.interface),
        Err(LinkTargetError::MissingPublicExport {
            namespace,
            name,
            ..
        }) if namespace == "value" && name == "greet"
    ));
}
