use crate::{analyze_module_interface, resolve_linked_module, SymbolKind, SymbolNamespace};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

#[test]
fn resolves_an_imported_function_to_its_canonical_dependency_symbol() {
    let domain_source = "pub fn increment value: Int -> Int = value + 1\n";
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let domain_interface = analyze_module_interface(
        seseragi_syntax::parse_diagnostics("domain.ssrg", domain_source),
        domain.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let target = ModuleLinkTarget::same_package(domain.header, domain_interface).unwrap();
    let main_source =
        "import { increment as next } from \"./domain\"\npub fn run value: Int -> Int = next value\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    let linked = link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap();

    let resolved = resolve_linked_module(linked, main_source);
    assert!(resolved.issues.is_empty());
    assert_eq!(resolved.imports.len(), 1);
    let imported = &resolved.imports[0];
    assert_eq!(imported.local_name, "next");
    assert_eq!(imported.export.symbol, "fixture/game::domain::increment");
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "next"
            && reference.namespace == SymbolNamespace::Value
            && reference.target == Some(imported.symbol)
    }));
    assert!(resolved.symbols.iter().any(|symbol| {
        symbol.id == imported.symbol
            && symbol.kind == SymbolKind::Imported
            && symbol.canonical.as_deref() == Some("fixture/game::domain::increment")
    }));
}
